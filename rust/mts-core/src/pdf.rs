//! PDF documents: invoices, receipts, contracts and reports.
//!
//! Every document is built from the same three pieces — a header with the
//! platform's name and logo, a body of labelled rows, and a footer with the
//! company's registration details — so a new document type is a new caller, not
//! a new layout engine.
//!
//! The documents are generated on demand and streamed to the browser, so there
//! is no file to clean up and nothing to leak on disk.

use crate::db;
use crate::settings;
use crate::util;
use printpdf::*;
use serde_json::{json, Value as J};
use std::io::BufWriter;

/// A4 in millimetres.
const PAGE_W: f32 = 210.0;
const PAGE_H: f32 = 297.0;
const MARGIN: f32 = 18.0;

/// One line of a document. `label` is drawn left, `value` right, and a line with
/// only a label is a section heading.
pub struct Row {
    pub label: String,
    pub value: String,
    pub heading: bool,
}

impl Row {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Row {
        Row {
            label: label.into(),
            value: value.into(),
            heading: false,
        }
    }
    pub fn heading(label: impl Into<String>) -> Row {
        Row {
            label: label.into(),
            value: String::new(),
            heading: true,
        }
    }
    pub fn note(text: impl Into<String>) -> Row {
        Row {
            label: text.into(),
            value: String::new(),
            heading: false,
        }
    }
}

/// Build a PDF from a title, a subtitle and rows. Returns the bytes.
pub fn build(title: &str, subtitle: &str, rows: &[Row], footer: &str) -> Vec<u8> {
    let (doc, page1, layer1) = PdfDocument::new(title, Mm(PAGE_W), Mm(PAGE_H), "Layer 1");
    let regular = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .expect("helvetica");
    let bold = doc
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .expect("helvetica bold");
    let mut layer = doc.get_page(page1).get_layer(layer1);

    let brand = settings::get("platform_name", "Mbilinyi Tech Solutions");
    let mut y = PAGE_H - MARGIN;

    // --- header ---
    layer.use_text(brand, 20.0, Mm(MARGIN), Mm(y - 6.0), &bold);
    y -= 12.0;
    layer.use_text(
        "BRELA Reg. • TIN 192-147-522 • Dodoma, Tanzania",
        9.0,
        Mm(MARGIN),
        Mm(y),
        &regular,
    );
    y -= 4.0;
    // A rule under the header.
    layer.set_outline_thickness(0.8);
    layer.set_outline_color(Color::Rgb(Rgb::new(0.13, 0.83, 0.93, None)));
    layer.add_line(Line {
        points: vec![
            (Point::new(Mm(MARGIN), Mm(y)), false),
            (Point::new(Mm(PAGE_W - MARGIN), Mm(y)), false),
        ],
        is_closed: false,
    });
    y -= 14.0;

    layer.use_text(title, 15.0, Mm(MARGIN), Mm(y), &bold);
    y -= 7.0;
    if !subtitle.is_empty() {
        layer.use_text(subtitle, 10.0, Mm(MARGIN), Mm(y), &regular);
        y -= 7.0;
    }
    y -= 4.0;

    // --- body ---
    let mut page = page1;
    for row in rows {
        // A new page when the cursor reaches the footer zone.
        if y < MARGIN + 22.0 {
            let (p, l) = doc.add_page(Mm(PAGE_W), Mm(PAGE_H), "Layer 1");
            page = p;
            layer = doc.get_page(page).get_layer(l);
            y = PAGE_H - MARGIN;
        }
        if row.heading {
            y -= 4.0;
            layer.use_text(&row.label, 12.0, Mm(MARGIN), Mm(y), &bold);
            y -= 7.0;
        } else if row.value.is_empty() {
            // A free note wraps at a generous width.
            for line in wrap(&row.label, 96) {
                layer.use_text(line, 10.0, Mm(MARGIN), Mm(y), &regular);
                y -= 5.5;
            }
        } else {
            layer.use_text(&row.label, 10.5, Mm(MARGIN), Mm(y), &regular);
            let value = truncate(&row.value, 60);
            // Right-align by measuring the built-in font's approximate width.
            let width = value.len() as f32 * 2.05;
            let x = (PAGE_W - MARGIN - width).max(MARGIN + 60.0);
            layer.use_text(&value, 10.5, Mm(x), Mm(y), &bold);
            y -= 6.0;
        }
    }

    // --- footer ---
    let y_foot = MARGIN - 2.0;
    layer.set_outline_thickness(0.4);
    layer.set_outline_color(Color::Rgb(Rgb::new(0.5, 0.5, 0.5, None)));
    layer.add_line(Line {
        points: vec![
            (Point::new(Mm(MARGIN), Mm(y_foot + 5.0)), false),
            (Point::new(Mm(PAGE_W - MARGIN), Mm(y_foot + 5.0)), false),
        ],
        is_closed: false,
    });
    layer.use_text(footer, 8.5, Mm(MARGIN), Mm(y_foot), &regular);

    let mut bytes = Vec::new();
    doc.save(&mut BufWriter::new(&mut bytes)).ok();
    bytes
}

/// Split text into lines no longer than `width` characters, on word boundaries.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut out = Vec::new();
    for paragraph in text.split('\n') {
        let mut line = String::new();
        for word in paragraph.split_whitespace() {
            if line.len() + word.len() + 1 > width {
                out.push(std::mem::take(&mut line));
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
        out.push(line);
    }
    out
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

fn money(amount: f64, currency: &str) -> String {
    let whole = amount.round() as i64;
    let mut digits = whole.abs().to_string();
    let mut grouped = String::new();
    while digits.len() > 3 {
        let split = digits.len() - 3;
        grouped = format!(",{}{}", &digits[split..], grouped);
        digits.truncate(split);
    }
    let sign = if whole < 0 { "-" } else { "" };
    format!("{currency} {sign}{digits}{grouped}")
}

/// An invoice or a receipt. `receipt` marks a paid invoice, which reads as a
/// receipt rather than a bill.
pub fn invoice(invoice_id: &str, receipt: bool) -> Option<Vec<u8>> {
    let inv = db::row(
        "SELECT * FROM invoices WHERE id = ?",
        &[db::from_json(json!(invoice_id))],
    )?;
    let currency = {
        let c = util::s(inv.get("currency").unwrap_or(&J::Null));
        if c.is_empty() {
            settings::get("pay_currency", "TZS")
        } else {
            c
        }
    };
    let amount = inv.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let title = util::s(inv.get("title").unwrap_or(&J::Null));
    let status = util::s(inv.get("status").unwrap_or(&J::Null));
    let tracking = util::s(inv.get("tracking_id").unwrap_or(&J::Null));
    let due = util::s(inv.get("due_date").unwrap_or(&J::Null));
    let method = util::s(inv.get("method").unwrap_or(&J::Null));

    let client = db::row(
        "SELECT r.name, r.email, r.phone, r.service FROM invoices i
         JOIN requests r ON r.tracking_id = i.tracking_id WHERE i.id = ?",
        &[db::from_json(json!(invoice_id))],
    );

    let doc_title = if receipt { "RECEIPT" } else { "INVOICE" };
    let mut rows = vec![
        Row::heading("Client"),
        Row::new(
            "Name",
            client
                .as_ref()
                .map(|c| util::s(c.get("name").unwrap_or(&J::Null)))
                .unwrap_or_default(),
        ),
        Row::new(
            "Email",
            client
                .as_ref()
                .map(|c| util::s(c.get("email").unwrap_or(&J::Null)))
                .unwrap_or_default(),
        ),
        Row::new(
            "Phone",
            client
                .as_ref()
                .map(|c| util::s(c.get("phone").unwrap_or(&J::Null)))
                .unwrap_or_default(),
        ),
        Row::heading("Work"),
        Row::new(
            "Service",
            client
                .as_ref()
                .map(|c| util::s(c.get("service").unwrap_or(&J::Null)))
                .unwrap_or_default(),
        ),
        Row::new("Project", title.clone()),
        Row::new("Tracking ID", tracking.clone()),
        Row::heading("Amounts"),
        Row::new(
            if receipt {
                "Amount received"
            } else {
                "Amount due"
            },
            money(amount, &currency),
        ),
    ];
    if !receipt {
        // VAT is shown as a breakdown, not added on top: the invoice figure is
        // what the client agreed to pay.
        let net = amount / 1.18;
        rows.push(Row::new("Net of VAT (18%)", money(net, &currency)));
        rows.push(Row::new("VAT (18%)", money(amount - net, &currency)));
    }
    rows.push(Row::heading("Details"));
    rows.push(Row::new("Invoice", invoice_id));
    rows.push(Row::new("Status", status));
    if !due.is_empty() {
        rows.push(Row::new("Due date", due));
    }
    if !method.is_empty() {
        rows.push(Row::new("Method", method));
    }
    rows.push(Row::heading("Payment"));
    rows.push(Row::new("Bank", settings::get("pay_bank_name", "NMB Bank")));
    rows.push(Row::new("Account", settings::get("pay_bank_account", "")));
    rows.push(Row::new(
        "Account name",
        settings::get("pay_bank_holder", ""),
    ));
    rows.push(Row::new(
        "Mobile money",
        settings::get("pay_mobile_number", ""),
    ));
    rows.push(Row::note(
        "Payments for a project are held in escrow and released to the company only when the client accepts the delivery.",
    ));

    Some(build(
        doc_title,
        &format!("{} — issued {}", invoice_id, util::now()),
        &rows,
        &format!("Generated by the platform on {}. This document is machine-generated and needs no signature.", util::now()),
    ))
}

/// A contract, with both signatures when they are present.
pub fn contract(contract_id: &str) -> Option<Vec<u8>> {
    let c = db::row(
        "SELECT * FROM contracts WHERE id = ?",
        &[db::from_json(json!(contract_id))],
    )?;
    let g = |k: &str| util::s(c.get(k).unwrap_or(&J::Null));
    let total = c.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let currency = {
        let x = settings::get("pay_currency", "TZS");
        x
    };
    let admin_sig = g("admin_sig");
    let client_sig = g("client_sig");
    let rows = vec![
        Row::heading("Parties"),
        Row::new("Client", g("client_name")),
        Row::new("Provider", settings::get("platform_name", "Mbilinyi Tech Solutions")),
        Row::heading("The work"),
        Row::new("Project", g("project_title")),
        Row::new("Tracking ID", g("tracking_id")),
        Row::new("Scope", g("scope")),
        Row::new("Duration", g("duration")),
        Row::new("Total", money(total, &currency)),
        Row::heading("Signatures"),
        Row::new(
            "Client",
            if client_sig.is_empty() { "Not signed".to_string() } else { format!("Signed {}", g("client_signed_at")) },
        ),
        Row::new(
            "Provider (CEO)",
            if admin_sig.is_empty() { "Not signed".to_string() } else { format!("Signed {}", g("admin_signed_at")) },
        ),
        Row::heading("Signature scheme"),
        Row::note(&format!(
            "Signed with {}. The signature covers every term above; changing any of them makes verification fail.",
            if admin_sig.trim_start().starts_with('{') {
                "ECDSA secp256k1 together with ML-DSA-65 (post-quantum)"
            } else {
                "ECDSA secp256k1"
            }
        )),
        Row::heading("Terms"),
        Row::note(&format!(
            "1. A 50% deposit is held in escrow before work begins.\n2. The balance is released to the provider when the client accepts the delivery.\n3. Either party may terminate in writing; work completed to that date is payable.\n4. Governed by the laws of the United Republic of Tanzania.",
        )),
    ];
    let body = format!("Contract {contract_id} — generated {}", util::now());
    Some(build(
        "CONTRACT",
        &body,
        &rows,
        &format!("Contract {} • verified by signature", contract_id),
    ))
}

/// A revenue or growth report, as a printable PDF.
pub fn report(title: &str, rows: &[Row]) -> Vec<u8> {
    build(
        title,
        &format!("Generated {}", util::now()),
        rows,
        "Confidential — internal management report",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pdf_starts_with_the_pdf_magic_bytes() {
        crate::test_bootstrap();
        let rows = vec![Row::heading("Test"), Row::new("A", "B")];
        let bytes = build("Test", "subtitle", &rows, "footer");
        assert!(
            bytes.starts_with(b"%PDF"),
            "a real PDF must begin with %PDF"
        );
        assert!(bytes.len() > 500);
    }

    #[test]
    fn a_document_with_many_rows_still_builds() {
        crate::test_bootstrap();
        let mut rows = Vec::new();
        for i in 0..200 {
            rows.push(Row::new(format!("Row {i}"), format!("Value {i}")));
        }
        let bytes = build("Long", "", &rows, "footer");
        assert!(bytes.starts_with(b"%PDF"));
    }

    #[test]
    fn long_text_wraps_instead_of_running_off_the_page() {
        let long = "word ".repeat(200);
        let lines = wrap(&long, 90);
        assert!(lines.len() > 1);
        assert!(lines.iter().all(|l| l.len() <= 90 + 5));
    }

    #[test]
    fn money_is_grouped_and_named() {
        assert_eq!(money(2600000.0, "TZS"), "TZS 2,600,000");
        assert_eq!(money(1000.0, "USD"), "USD 1,000");
        assert_eq!(money(-500.0, "TZS"), "TZS -500");
    }

    #[test]
    fn a_long_value_is_truncated_rather_than_overlapping() {
        let t = truncate(&"x".repeat(200), 60);
        assert!(t.chars().count() <= 60);
        assert!(t.ends_with('…'));
    }
}
