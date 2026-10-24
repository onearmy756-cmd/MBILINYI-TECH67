//! Payroll.
//!
//! When escrow releases to the company, the worker who did the job still has to
//! be paid. This module turns a released contract into a payslip: gross, the
//! statutory deductions (PAYE, NSSF, WCF), and the net the worker receives.
//!
//! The rates are settings, not constants, because they change with the Finance
//! Act and the CEO must be able to correct them without a redeploy. The
//! defaults follow the Tanzanian bands in force when this was written; every
//! payslip records the rates it was computed with, so an old payslip can still
//! be explained after the settings change.

use crate::db;
use crate::settings;
use crate::util;
use serde_json::{json, Value as J};

fn arg(v: J) -> rusqlite::types::Value {
    db::from_json(v)
}

/// PAYE bands, as (monthly chargeable income ceiling in TZS, rate). The last
/// entry's ceiling is ignored — it is the top band. Kept as one list so the
/// whole table can be replaced by a single setting.
fn paye_bands() -> Vec<(f64, f64)> {
    let raw = settings::get(
        "payroll_paye_bands",
        "270000:0,520000:0.08,760000:0.20,1000000:0.25,9999999999:0.30",
    );
    raw.split(',')
        .filter_map(|pair| {
            let (ceiling, rate) = pair.split_once(':')?;
            Some((ceiling.trim().parse::<f64>().ok()?, rate.trim().parse::<f64>().ok()?))
        })
        .collect()
}

/// The pure half of `paye`, so the banded arithmetic can be tested without a
/// database and reused if the bands ever come from somewhere else.
pub fn paye_at(bands: &[(f64, f64)], chargeable: f64) -> f64 {
    if bands.is_empty() || chargeable <= 0.0 {
        return 0.0;
    }
    let last_ceiling = bands.last().map(|b| b.0).unwrap_or(f64::MAX);
    let top_rate = bands.last().map(|b| b.1).unwrap_or(0.0);
    let mut tax = 0.0;
    let mut floor = 0.0;
    for (ceiling, rate) in bands {
        if chargeable <= floor {
            break;
        }
        let slice = (chargeable.min(*ceiling) - floor).max(0.0);
        tax += slice * rate;
        floor = *ceiling;
    }
    // The top band has no ceiling of its own; tax anything above the last
    // explicit ceiling at the top rate.
    if chargeable > last_ceiling {
        tax += (chargeable - last_ceiling) * top_rate;
    }
    util_round2(tax)
}

/// The whole payslip for a gross figure, at the configured rates.
pub fn compute(gross: f64) -> J {
    compute_at(
        gross,
        settings::num("payroll_nssf_rate", 0.10),
        settings::num("payroll_wcf_rate", 0.006),
        &paye_bands(),
        &settings::get("pay_currency", "TZS"),
    )
}

/// The pure half of `compute`: the arithmetic with the rates passed in, so a
/// payslip can be explained from its own inputs and tested without a database.
pub fn compute_at(gross: f64, nssf_rate: f64, wcf_rate: f64, bands: &[(f64, f64)], currency: &str) -> J {
    let gross = util_round2(gross.max(0.0));
    let nssf = util_round2(gross * nssf_rate);
    // PAYE is charged on gross less the worker's NSSF contribution.
    let chargeable = (gross - nssf).max(0.0);
    let paye = paye_at(bands, chargeable);
    let wcf = util_round2(gross * wcf_rate);
    let net = util_round2(gross - paye - nssf);
    json!({
        "gross": gross,
        "nssf": nssf,
        "paye": paye,
        "wcf": wcf,
        "net": net,
        "currency": currency,
        "rates": {
            "nssfRate": nssf_rate,
            "wcfRate": wcf_rate,
            "payeBands": bands.iter().map(|(c, r)| format!("{c}:{r}")).collect::<Vec<_>>().join(","),
        },
    })
}

fn util_round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

/// The share of a released contract that belongs to the worker who did it,
/// before tax. Default 70%; the company keeps the rest.
pub fn worker_share(released: f64) -> f64 {
    worker_share_at(released, settings::num("payroll_worker_share", 0.70))
}

/// The pure half of `worker_share`.
pub fn worker_share_at(released: f64, pct: f64) -> f64 {
    util_round2(released.max(0.0) * pct)
}

pub fn ensure_schema() {
    let _ = db::exec(
        "CREATE TABLE IF NOT EXISTS payslips (
           id            VARCHAR(64)   NOT NULL,
           employee_id   VARCHAR(64)   NOT NULL DEFAULT '',
           employee_name VARCHAR(160)  NOT NULL DEFAULT '',
           contract_id   VARCHAR(64)   NOT NULL DEFAULT '',
           gross         DECIMAL(14,2) NOT NULL DEFAULT 0,
           paye          DECIMAL(14,2) NOT NULL DEFAULT 0,
           nssf          DECIMAL(14,2) NOT NULL DEFAULT 0,
           wcf           DECIMAL(14,2) NOT NULL DEFAULT 0,
           net           DECIMAL(14,2) NOT NULL DEFAULT 0,
           currency      VARCHAR(10)   NOT NULL DEFAULT 'TZS',
           status        VARCHAR(30)   NOT NULL DEFAULT 'Pending',
           rates         TEXT          NOT NULL DEFAULT '',
           note          VARCHAR(255)  NOT NULL DEFAULT '',
           paid_at       DATETIME      NULL,
           created_at    DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
           PRIMARY KEY (id)
         );
         CREATE INDEX IF NOT EXISTS idx_payslip_emp ON payslips(employee_id);
         CREATE INDEX IF NOT EXISTS idx_payslip_contract ON payslips(contract_id);",
    );
}

/// Raise a payslip for the worker who served a contract. The gross is the
/// worker's share of what was released; the deductions are computed now and
/// frozen onto the row, so a later change to the rates cannot rewrite history.
pub fn raise_for_contract(contract_id: &str, released: f64, note: &str) -> Option<J> {
    ensure_schema();
    let assignment = db::row(
        "SELECT * FROM request_assignment ra
           JOIN requests r ON r.id = ra.request_id
           JOIN contracts c ON c.request_id = r.id
          WHERE c.id = ?
          ORDER BY ra.created_at DESC LIMIT 1",
        &[arg(json!(contract_id))],
    )?;
    let employee_id = util::s(assignment.get("employee_id").unwrap_or(&J::Null));
    let employee_name = util::s(assignment.get("employee_name").unwrap_or(&J::Null));
    if employee_id.is_empty() {
        return None;
    }
    let gross = worker_share(released);
    let slip = compute(gross);
    let id = util::new_id("PAY-");
    let _ = db::run(
        "INSERT INTO payslips (id, employee_id, employee_name, contract_id, gross, paye, nssf, wcf, net,
                               currency, status, rates, note, created_at)
         VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
        &[
            arg(json!(id)), arg(json!(employee_id)), arg(json!(employee_name)),
            arg(json!(contract_id)),
            arg(json!(slip["gross"])), arg(json!(slip["paye"])), arg(json!(slip["nssf"])),
            arg(json!(slip["wcf"])), arg(json!(slip["net"])),
            arg(json!(slip["currency"])), arg(json!("Pending")),
            arg(json!(slip["rates"].to_string())), arg(json!(note)), arg(json!(util::now())),
        ],
    );
    Some(json!({
        "id": id,
        "employeeId": employee_id,
        "employeeName": employee_name,
        "contractId": contract_id,
        "gross": slip["gross"],
        "paye": slip["paye"],
        "nssf": slip["nssf"],
        "wcf": slip["wcf"],
        "net": slip["net"],
        "currency": slip["currency"],
        "status": "Pending",
    }))
}

pub fn mark_paid(id: &str) -> bool {
    db::run(
        "UPDATE payslips SET status = 'Paid', paid_at = ? WHERE id = ?",
        &[arg(json!(util::now())), arg(json!(id))],
    )
    .unwrap_or(0)
        > 0
}

pub fn list_for(employee_id: &str) -> Vec<J> {
    ensure_schema();
    db::rows(
        "SELECT * FROM payslips WHERE (?1 = '' OR employee_id = ?1) ORDER BY created_at DESC",
        &[arg(json!(employee_id))],
    )
    .into_iter()
    .map(|r| {
        json!({
            "id": util::s(r.get("id").unwrap_or(&J::Null)),
            "employeeId": util::s(r.get("employee_id").unwrap_or(&J::Null)),
            "employeeName": util::s(r.get("employee_name").unwrap_or(&J::Null)),
            "contractId": util::s(r.get("contract_id").unwrap_or(&J::Null)),
            "gross": util::f(r.get("gross").unwrap_or(&J::Null)),
            "paye": util::f(r.get("paye").unwrap_or(&J::Null)),
            "nssf": util::f(r.get("nssf").unwrap_or(&J::Null)),
            "wcf": util::f(r.get("wcf").unwrap_or(&J::Null)),
            "net": util::f(r.get("net").unwrap_or(&J::Null)),
            "currency": util::s(r.get("currency").unwrap_or(&J::Null)),
            "status": util::s(r.get("status").unwrap_or(&J::Null)),
            "note": util::s(r.get("note").unwrap_or(&J::Null)),
            "paidAt": util::iso(r.get("paid_at").and_then(|v| v.as_str())),
            "createdAt": util::iso(r.get("created_at").and_then(|v| v.as_str())),
        })
    })
    .collect()
}

/// What the company owes workers but has not yet paid.
pub fn outstanding() -> f64 {
    ensure_schema();
    db::rows(
        "SELECT COALESCE(SUM(net),0) AS t FROM payslips WHERE status != 'Paid'",
        &[],
    )
    .first()
    .and_then(|r| r.get("t"))
    .and_then(|v| v.as_f64())
    .unwrap_or(0.0)
}

/// Total tax the company has withheld, for the TRA return.
pub fn tax_withheld() -> J {
    ensure_schema();
    let row = db::rows(
        "SELECT COALESCE(SUM(paye),0) AS paye, COALESCE(SUM(nssf),0) AS nssf, COALESCE(SUM(wcf),0) AS wcf FROM payslips",
        &[],
    );
    let r = row.first();
    json!({
        "paye": r.and_then(|x| x.get("paye")).and_then(|v| v.as_f64()).unwrap_or(0.0),
        "nssf": r.and_then(|x| x.get("nssf")).and_then(|v| v.as_f64()).unwrap_or(0.0),
        "wcf": r.and_then(|x| x.get("wcf")).and_then(|v| v.as_f64()).unwrap_or(0.0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The bands in force when this was written: the first 270,000 is free,
    /// then 8% up to 520,000, 20% to 760,000, 25% to 1,000,000, 30% above.
    fn bands() -> Vec<(f64, f64)> {
        vec![
            (270_000.0, 0.0),
            (520_000.0, 0.08),
            (760_000.0, 0.20),
            (1_000_000.0, 0.25),
            (9_999_999_999.0, 0.30),
        ]
    }

    /// PAYE is banded: the first slice is free, the next is taxed at its own
    /// rate, and only the part inside each band is charged at that band's rate.
    #[test]
    fn paye_is_charged_band_by_band() {
        // 270,000 is below the first ceiling, so nothing is due.
        assert_eq!(paye_at(&bands(), 270_000.0), 0.0);
        // 400,000: the 130,000 above 270,000 is taxed at 8%.
        assert_eq!(paye_at(&bands(), 400_000.0), 10_400.0);
        // 600,000: 250,000 at 8% + 80,000 at 20%.
        assert_eq!(paye_at(&bands(), 600_000.0), 20_000.0 + 16_000.0);
        // Nothing owed on a zero or negative figure.
        assert_eq!(paye_at(&bands(), 0.0), 0.0);
        assert_eq!(paye_at(&[], 500_000.0), 0.0);
    }

    /// A payslip keeps gross, the deductions and the net consistent.
    #[test]
    fn payslip_balances() {
        let slip = compute_at(1_000_000.0, 0.10, 0.006, &bands(), "TZS");
        let gross = slip["gross"].as_f64().unwrap();
        let net = slip["net"].as_f64().unwrap();
        let paye = slip["paye"].as_f64().unwrap();
        let nssf = slip["nssf"].as_f64().unwrap();
        assert_eq!(nssf, 100_000.0);
        assert!((gross - paye - nssf - net).abs() < 0.01);
        assert!(net < gross, "the worker must take home less than gross");
        // The worker's net plus the deductions equals the gross exactly.
        let wcf = slip["wcf"].as_f64().unwrap();
        assert!(wcf > 0.0, "the employer contribution is shown too");
    }

    /// The worker's share of a release is the configured percentage.
    #[test]
    fn worker_share_is_a_percentage_of_the_release() {
        // With the default 70% share, a 1,000,000 release pays 700,000 gross.
        assert_eq!(worker_share_at(1_000_000.0, 0.70), 700_000.0);
        assert_eq!(worker_share_at(0.0, 0.70), 0.0);
    }
}
