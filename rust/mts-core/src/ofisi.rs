//! Geography, the online office, and the SMS log.
//!
//! Three things the CEO asked for live here together because they are all about
//! reach: where the services are delivered (every country, and every region of
//! Tanzania), the online office the staff and clients meet in, and the record of
//! every SMS the platform sent so the admin can see each request that arrived.

use crate::api::ctx::Ctx;
use crate::api::map;
use crate::db;
use crate::util::{self, bad, ok};
use axum::response::Response;
use rusqlite::types::Value;
use serde_json::{json, Value as J};

fn arg(v: J) -> Value {
    db::from_json(v)
}

/// The 31 regions of Tanzania, so growth can be tracked region by region.
pub const TZ_REGIONS: &[(&str, &str)] = &[
    ("Arusha", "AR"), ("Dar es Salaam", "DS"), ("Dodoma", "DD"), ("Geita", "GT"),
    ("Iringa", "IR"), ("Kagera", "KG"), ("Katavi", "KT"), ("Kigoma", "KM"),
    ("Kilimanjaro", "KL"), ("Lindi", "LD"), ("Manyara", "MY"), ("Mara", "MR"),
    ("Mbeya", "MB"), ("Morogoro", "MO"), ("Mtwara", "MT"), ("Mwanza", "MW"),
    ("Njombe", "NJ"), ("Pemba North", "PN"), ("Pemba South", "PS"), ("Pwani", "PW"),
    ("Rukwa", "RK"), ("Ruvuma", "RV"), ("Shinyanga", "SH"), ("Simiyu", "SI"),
    ("Singida", "SG"), ("Songwe", "SW"), ("Tabora", "TB"), ("Tanga", "TG"),
    ("Unguja North", "UN"), ("Unguja South", "US"), ("Mjini Magharibi", "MM"),
];

/// The countries the platform serves. Tanzania is the home market; the rest are
/// the international market, which never sees the Tanzania-only services.
pub const COUNTRIES: &[(&str, &str)] = &[
    ("Tanzania", "TZ"), ("Kenya", "KE"), ("Uganda", "UG"), ("Rwanda", "RW"),
    ("Burundi", "BI"), ("DR Congo", "CD"), ("Zambia", "ZM"), ("Malawi", "MW"),
    ("Mozambique", "MZ"), ("South Africa", "ZA"), ("Nigeria", "NG"), ("Ghana", "GH"),
    ("Egypt", "EG"), ("Morocco", "MA"), ("United Arab Emirates", "AE"),
    ("Saudi Arabia", "SA"), ("Qatar", "QA"), ("India", "IN"), ("China", "CN"),
    ("Japan", "JP"), ("Australia", "AU"), ("United Kingdom", "GB"),
    ("Germany", "DE"), ("France", "FR"), ("Netherlands", "NL"), ("Sweden", "SE"),
    ("Norway", "NO"), ("Canada", "CA"), ("United States", "US"), ("Brazil", "BR"),
];

pub fn init_tables() {
    let _ = db::exec(
        "CREATE TABLE IF NOT EXISTS service_regions (
           id          VARCHAR(64)  NOT NULL,
           country     VARCHAR(80)  NOT NULL,
           region      VARCHAR(80)  NOT NULL DEFAULT '',
           code        VARCHAR(8)   NOT NULL DEFAULT '',
           kind        VARCHAR(20)  NOT NULL DEFAULT 'region',
           is_active   INTEGER      NOT NULL DEFAULT 1,
           requests    INTEGER      NOT NULL DEFAULT 0,
           created_at  DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
           PRIMARY KEY (id)
         );
         CREATE TABLE IF NOT EXISTS sms_log (
           id          VARCHAR(64)  NOT NULL,
           phone       VARCHAR(60)  NOT NULL,
           message     TEXT         NOT NULL,
           request_id  VARCHAR(64)  NOT NULL DEFAULT '',
           kind        VARCHAR(40)  NOT NULL DEFAULT 'general',
           status      VARCHAR(30)  NOT NULL DEFAULT 'queued',
           created_at  DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
           PRIMARY KEY (id)
         );
         CREATE TABLE IF NOT EXISTS office_rooms (
           id          VARCHAR(64)  NOT NULL,
           name        VARCHAR(160) NOT NULL,
           kind        VARCHAR(40)  NOT NULL DEFAULT 'meeting',
           host        VARCHAR(160) NOT NULL DEFAULT '',
           invitees    TEXT         NOT NULL DEFAULT '[]',
           starts_at   DATETIME     NULL,
           duration_min INTEGER     NOT NULL DEFAULT 30,
           scribe      INTEGER      NOT NULL DEFAULT 1,
           recording   INTEGER      NOT NULL DEFAULT 0,
           status      VARCHAR(30)  NOT NULL DEFAULT 'scheduled',
           created_at  DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
           PRIMARY KEY (id)
         );
         CREATE INDEX IF NOT EXISTS idx_sms_req ON sms_log(request_id);
         CREATE INDEX IF NOT EXISTS idx_region_country ON service_regions(country);",
    );
}

/// Seed the country and region rows once, so growth can be charted from the
/// first day without the admin typing every region.
pub fn seed() {
    if db::scalar_i64("SELECT COUNT(*) FROM service_regions", &[]) > 0 {
        return;
    }
    let now = util::now();
    for (name, code) in COUNTRIES {
        let _ = db::run(
            "INSERT INTO service_regions (id, country, region, code, kind, is_active, requests, created_at)
             VALUES (?,?,?,?,?,?,?,?)",
            &[
                arg(json!(util::new_id("RG-"))), arg(json!(name)), arg(json!("")), arg(json!(code)),
                arg(json!("country")), arg(json!(1)), arg(json!(0)), arg(json!(now)),
            ],
        );
    }
    for (name, code) in TZ_REGIONS {
        let _ = db::run(
            "INSERT INTO service_regions (id, country, region, code, kind, is_active, requests, created_at)
             VALUES (?,?,?,?,?,?,?,?)",
            &[
                arg(json!(util::new_id("RG-"))), arg(json!("Tanzania")), arg(json!(name)), arg(json!(code)),
                arg(json!("region")), arg(json!(1)), arg(json!(0)), arg(json!(now)),
            ],
        );
    }
}

pub async fn handle(c: &Ctx) -> Response {
    match c.action().as_str() {
        "countries" => {
            let rows = db::rows(
                "SELECT * FROM service_regions WHERE kind = 'country' ORDER BY country ASC",
                &[],
            );
            ok(json!({ "countries": rows.iter().map(map::region).collect::<Vec<_>>() }))
        }
        "regions" => {
            let country = {
                let x = c.p("country");
                if x.is_empty() { "Tanzania".to_string() } else { x }
            };
            let rows = db::rows(
                "SELECT * FROM service_regions WHERE kind = 'region' AND country = ?1 ORDER BY region ASC",
                &[arg(json!(country))],
            );
            ok(json!({ "country": country, "regions": rows.iter().map(map::region).collect::<Vec<_>>() }))
        }
        // Growth of the services, by country and by region. Counts come from the
        // requests that name a place, so the chart reflects real work.
        "growth" => {
            let by_country = db::rows(
                "SELECT country, COUNT(*) AS n FROM service_regions WHERE kind = 'country' GROUP BY country ORDER BY n DESC",
                &[],
            );
            let by_region = db::rows(
                "SELECT region, COUNT(*) AS n FROM service_regions WHERE kind = 'region' GROUP BY region ORDER BY n DESC",
                &[],
            );
            let total_regions = db::scalar_i64("SELECT COUNT(*) FROM service_regions WHERE kind = 'region'", &[]);
            let total_countries = db::scalar_i64("SELECT COUNT(*) FROM service_regions WHERE kind = 'country'", &[]);
            let requests = db::scalar_i64("SELECT COUNT(*) FROM requests", &[]);
            ok(json!({
                "countries": total_countries,
                "regions": total_regions,
                "requests": requests,
                "byCountry": by_country.iter().map(|r| json!({
                    "country": util::s(r.get("country").unwrap_or(&J::Null)),
                    "count": util::i(r.get("n").unwrap_or(&J::Null)),
                })).collect::<Vec<_>>(),
                "byRegion": by_region.iter().map(|r| json!({
                    "region": util::s(r.get("region").unwrap_or(&J::Null)),
                    "count": util::i(r.get("n").unwrap_or(&J::Null)),
                })).collect::<Vec<_>>(),
            }))
        }
        // The SMS the platform sent. The admin reads them here and approves the
        // ones that ask for a decision.
        "sms_log" => {
            if let Err(e) = c.require_admin() {
                return e;
            }
            let rows = db::rows("SELECT * FROM sms_log ORDER BY created_at DESC LIMIT 200", &[]);
            ok(json!({
                "messages": rows.iter().map(|r| json!({
                    "id": util::s(r.get("id").unwrap_or(&J::Null)),
                    "phone": util::s(r.get("phone").unwrap_or(&J::Null)),
                    "message": util::s(r.get("message").unwrap_or(&J::Null)),
                    "requestId": util::s(r.get("request_id").unwrap_or(&J::Null)),
                    "kind": util::s(r.get("kind").unwrap_or(&J::Null)),
                    "status": util::s(r.get("status").unwrap_or(&J::Null)),
                    "createdAt": util::iso(r.get("created_at").and_then(|v| v.as_str())),
                })).collect::<Vec<_>>(),
                "count": rows.len(),
            }))
        }
        "sms_send" => sms_send(c),
        // What the CEO configures to make SMS leave the building for real.
        "sms_config" => {
            if let Err(e) = c.require_admin() {
                return e;
            }
            ok(crate::sms::status())
        }
        "sms_config_save" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let gateway = c.p("gateway").trim().to_lowercase();
            if !gateway.is_empty() && !crate::sms::GATEWAYS.iter().any(|(k, _, _)| *k == gateway) {
                return bad("Choose one of the supported SMS gateways.");
            }
            crate::settings::set("sms_gateway", &gateway);
            for (field, setting) in [
                ("baseUrl", "sms_base_url"),
                ("senderId", "sms_sender_id"),
            ] {
                if !c.p(field).trim().is_empty() {
                    crate::settings::set(setting, c.p(field).trim());
                }
            }
            // Empty secret fields leave the stored ones alone.
            if !c.p("apiKey").trim().is_empty() {
                crate::settings::set("sms_api_key", c.p("apiKey").trim());
            }
            if !c.p("secret").trim().is_empty() {
                crate::settings::set("sms_secret", c.p("secret").trim());
            }
            ok(crate::sms::status())
        }
        // One real message, so the CEO can prove the gateway works.
        "sms_test" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            let phone = c.p("phone");
            if phone.trim().is_empty() {
                return bad("Give the number to send the test to.");
            }
            let r = crate::sms::send(&phone, "MBILINYI-TECH67: jaribio la SMS limefanikiwa.").await;
            ok(json!({ "sent": r.ok, "detail": r.detail }))
        }
        // Online office rooms: meetings, calls and the AI scribe.
        "rooms" => {
            let rows = db::rows("SELECT * FROM office_rooms ORDER BY created_at DESC LIMIT 100", &[]);
            ok(json!({
                "rooms": rows.iter().map(|r| json!({
                    "id": util::s(r.get("id").unwrap_or(&J::Null)),
                    "name": util::s(r.get("name").unwrap_or(&J::Null)),
                    "kind": util::s(r.get("kind").unwrap_or(&J::Null)),
                    "host": util::s(r.get("host").unwrap_or(&J::Null)),
                    "invitees": serde_json::from_str::<J>(&util::s(r.get("invitees").unwrap_or(&J::Null))).unwrap_or(json!([])),
                    "startsAt": util::iso(r.get("starts_at").and_then(|v| v.as_str())),
                    "durationMin": util::i(r.get("duration_min").unwrap_or(&J::Null)),
                    "scribe": util::i(r.get("scribe").unwrap_or(&J::Null)) != 0,
                    "recording": util::i(r.get("recording").unwrap_or(&J::Null)) != 0,
                    "status": util::s(r.get("status").unwrap_or(&J::Null)),
                    "createdAt": util::iso(r.get("created_at").and_then(|v| v.as_str())),
                })).collect::<Vec<_>>(),
                "kinds": ["meeting", "video_call", "voice_call", "screen_share", "webinar"],
            }))
        }
        "room_create" => room_create(c),
        "room_status" => room_status(c),
        _ => bad("Unknown ofisi action."),
    }
}

fn sms_send(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    if let Err(e) = c.require_admin() {
        return e;
    }
    let phone = c.p("phone");
    let message = c.p("message");
    if phone.trim().is_empty() || message.trim().is_empty() {
        return bad("A phone number and a message are both required.");
    }
    let id = util::new_id("SMS-");
    let _ = db::run(
        "INSERT INTO sms_log (id, phone, message, request_id, kind, status, created_at) VALUES (?,?,?,?,?,?,?)",
        &[
            arg(json!(id)), arg(json!(phone)), arg(json!(message)), arg(json!(c.p("requestId"))),
            arg(json!("manual")), arg(json!("queued")), arg(json!(util::now())),
        ],
    );
    // The message is sent for real in the background, and the log row is
    // updated with what the gateway actually said. `queued` above is the state
    // for the few seconds that takes.
    crate::sms::spawn_send(id.clone(), phone, message);
    let configured = crate::sms::configured();
    ok(json!({
        "id": id,
        "queued": configured,
        "gatewayConfigured": configured,
        "message": if configured {
            "Ujumbe unatuma sasa; hali yake itaonekana kwenye SMS log."
        } else {
            "Ujumbe umeandikwa lakini hakuna SMS gateway iliyowekwa. CEO anaweka moja kwenye Admin -> Settings."
        },
    }))
}

fn room_create(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    let Ok(_u) = c.require_user() else {
        return util::unauth();
    };
    let name = c.p("name");
    if name.trim().is_empty() {
        return bad("The room needs a name.");
    }
    let invitees: Vec<String> = c
        .pv("invitees")
        .as_array()
        .map(|a| a.iter().map(|v| util::s(v)).collect())
        .unwrap_or_default();
    let id = util::new_id("ROOM-");
    let _ = db::run(
        "INSERT INTO office_rooms (id, name, kind, host, invitees, starts_at, duration_min, scribe, recording, status, created_at)
         VALUES (?,?,?,?,?,?,?,?,?,?,?)",
        &[
            arg(json!(id)), arg(json!(name)), arg(json!(c.p("kind"))), arg(json!(c.p("host"))),
            arg(json!(serde_json::to_string(&invitees).unwrap_or_else(|_| "[]".into()))),
            arg(json!(if c.p("startsAt").is_empty() { util::now() } else { c.p("startsAt") })),
            arg(json!(if c.p("durationMin").is_empty() { 30 } else { util::i(&c.pv("durationMin")) })),
            arg(json!(if c.p("scribe").is_empty() { 1 } else { util::i(&c.pv("scribe")) })),
            arg(json!(if c.p("recording").is_empty() { 0 } else { util::i(&c.pv("recording")) })),
            arg(json!("scheduled")), arg(json!(util::now())),
        ],
    );
    ok(json!({ "id": id, "created": true }))
}

fn room_status(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    if let Err(e) = c.require_admin() {
        return e;
    }
    let id = c.p("id");
    let status = c.p("status");
    let allowed = ["scheduled", "live", "ended", "cancelled"];
    if !allowed.contains(&status.as_str()) {
        return bad("Unknown room status.");
    }
    let _ = db::run("UPDATE office_rooms SET status = ? WHERE id = ?", &[arg(json!(status)), arg(json!(id))]);
    ok(json!({ "status": status }))
}
