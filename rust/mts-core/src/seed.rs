//! Demo data.
//!
//! A fresh clone boots with these records so every part of the portal has
//! something to show and every demo button does something. Portfolio projects
//! and testimonials are deliberately absent: the company is new, and inventing
//! delivered work or client reviews would be a lie on the public site.
//!
//! Every insert is a no-op when the row already exists, so seeding an existing
//! database never overwrites real data.

use crate::db;
use rusqlite::types::Value;
use serde_json::json;

fn arg(v: serde_json::Value) -> Value {
    db::from_json(v)
}

/// Insert only when the primary key is absent, so a real record is never
/// replaced by a demo one.
fn insert_if_absent(table: &str, id: &str, sql: &str, args: &[Value]) {
    let exists = db::scalar_i64(
        &format!("SELECT count(*) FROM {table} WHERE id = ?"),
        &[arg(json!(id))],
    );
    if exists == 0 {
        let _ = db::run(sql, args);
    }
}

pub fn run() {
    if db::scalar_i64("SELECT count(*) FROM users", &[]) > 0 {
        return;
    }

    let admin_pw = bcrypt::hash("Admin@2026", 10).unwrap_or_default();
    let demo_pw = bcrypt::hash("demo123", 10).unwrap_or_default();

    /* users */
    for (id, name, email, phone, company, pw, role, status, created) in [
        (
            "U-ADMIN",
            "Jackson Mbilinyi",
            "mbilinyitech@gmail.com",
            "0796752645",
            "Mbilinyi Tech Solutions",
            admin_pw.as_str(),
            "admin",
            "Active",
            "2024-01-15 08:00:00",
        ),
        (
            "U-DEMO",
            "Amina Juma",
            "demo@client.com",
            "0712345678",
            "Amina Pharmacy Ltd",
            demo_pw.as_str(),
            "client",
            "Active",
            "2025-08-02 09:00:00",
        ),
        (
            "U-1002",
            "Baraka Mwansa",
            "baraka@kilimo.co.tz",
            "0755123456",
            "Kilimo Fresh",
            demo_pw.as_str(),
            "client",
            "Active",
            "2025-11-10 09:00:00",
        ),
        (
            "U-1003",
            "Sarah Nkya",
            "sarah@edu.go.tz",
            "0767890123",
            "Moshi Academy",
            demo_pw.as_str(),
            "client",
            "Suspended",
            "2026-01-05 09:00:00",
        ),
    ] {
        insert_if_absent(
            "users",
            id,
            "INSERT INTO users (id,name,email,phone,company,password_hash,role,status,created_at)
             VALUES (?,?,?,?,?,?,?,?,?)",
            &[
                arg(json!(id)),
                arg(json!(name)),
                arg(json!(email)),
                arg(json!(phone)),
                arg(json!(company)),
                arg(json!(pw)),
                arg(json!(role)),
                arg(json!(status)),
                arg(json!(created)),
            ],
        );
    }

    /* requests */
    let tech1 = json!(["React", "Python", "PostgreSQL"]).to_string();
    let tech2 = json!(["Python", "React"]).to_string();
    for (id, trk, uid, name, email, phone, company, service, title, desc, tech, platform, budget, deadline, priority, status, note, created) in [
        ("R-1", "MTS-2026-4810", "U-DEMO", "Amina Juma", "demo@client.com", "0712345678", "Amina Pharmacy Ltd",
         "Custom Software Development", "Pharmacy Stock, Sales & M-Pesa System",
         "Need full pharmacy system: stock alerts, expiry tracking, sales with receipt printing, M-Pesa integration, daily reports, 3 users with roles.",
         tech1.as_str(), "Web + Mobile", "TZS 1.5M – 5M ($600 – $2,000)", "2026-11-30", "Urgent", "Quoted",
         "Reviewed — quotation sent. Awaiting client approval.", "2026-09-20 10:00:00"),
        ("R-2", "MTS-2026-5523", "U-1002", "Baraka Mwansa", "baraka@kilimo.co.tz", "0755123456", "Kilimo Fresh",
         "AI Solutions & Fine-Tuning", "Swahili WhatsApp Chatbot for Orders",
         "WhatsApp chatbot in Swahili to take vegetable orders, send price lists, confirm delivery zones in Dar es Salaam.",
         tech2.as_str(), "AI / Chatbot Solution", "TZS 500K – 1.5M ($200 – $600)", "2026-10-30", "Normal", "Under Review",
         "Analyst assigned. Scoping WhatsApp API costs.", "2026-09-28 11:30:00"),
    ] {
        insert_if_absent(
            "requests",
            id,
            "INSERT INTO requests
               (id,tracking_id,user_id,name,email,phone,company,service,title,description,tech,platform,budget,deadline,priority,status,admin_note,created_at)
             VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
            &[
                arg(json!(id)), arg(json!(trk)), arg(json!(uid)), arg(json!(name)), arg(json!(email)),
                arg(json!(phone)), arg(json!(company)), arg(json!(service)), arg(json!(title)),
                arg(json!(desc)), arg(json!(tech)), arg(json!(platform)), arg(json!(budget)),
                arg(json!(deadline)), arg(json!(priority)), arg(json!(status)), arg(json!(note)),
                arg(json!(created)),
            ],
        );
    }

    /* quote */
    let items = json!([
        {"desc": "System Analysis & UI/UX Design", "qty": 1, "price": 450000},
        {"desc": "Pharmacy Core (Stock, Sales, Receipts, Roles)", "qty": 1, "price": 1800000},
        {"desc": "M-Pesa Daraja API Integration", "qty": 1, "price": 550000},
        {"desc": "Deployment, Training & 6-Mo Support", "qty": 1, "price": 400000},
    ])
    .to_string();
    insert_if_absent(
        "quotes",
        "Q-9001",
        "INSERT INTO quotes (id,request_id,tracking_id,user_id,items,subtotal,vat,total,currency,valid_until,status,admin_message,created_at)
         VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?)",
        &[
            arg(json!("Q-9001")), arg(json!("R-1")), arg(json!("MTS-2026-4810")), arg(json!("U-DEMO")),
            arg(json!(items)), arg(json!(3200000)), arg(json!(576000)), arg(json!(3776000)),
            arg(json!("TZS")), arg(json!("2026-10-20 23:59:59")), arg(json!("Sent")),
            arg(json!("Official quotation. Includes BRELA contract, source code handover & training for 3 staff.")),
            arg(json!("2026-09-22 09:00:00")),
        ],
    );

    /* contract — unsigned, so the signature flow is demonstrable */
    insert_if_absent(
        "contracts",
        "C-7001",
        "INSERT INTO contracts (id,request_id,quote_id,tracking_id,client_name,project_title,scope,duration,total,status,created_at)
         VALUES (?,?,?,?,?,?,?,?,?,?,?)",
        &[
            arg(json!("C-7001")), arg(json!("R-1")), arg(json!("Q-9001")), arg(json!("MTS-2026-4810")),
            arg(json!("Amina Juma")), arg(json!("Pharmacy Stock, Sales & M-Pesa System")),
            arg(json!("Full pharmacy management system as per approved quotation Q-9001, including deployment, training and 6 months maintenance.")),
            arg(json!("45 days")), arg(json!(3776000)), arg(json!("Sent")), arg(json!("2026-09-23 09:00:00")),
        ],
    );

    /* invoice */
    insert_if_absent(
        "invoices",
        "INV-3001",
        "INSERT INTO invoices (id,tracking_id,user_id,title,amount,status,method,due_date,created_at)
         VALUES (?,?,?,?,?,?,?,?,?)",
        &[
            arg(json!("INV-3001")), arg(json!("MTS-2026-4810")), arg(json!("U-DEMO")),
            arg(json!("Pharmacy System — 50% Deposit")), arg(json!(1888000)), arg(json!("Unpaid")),
            arg(json!("M-Pesa / Bank")), arg(json!("2026-10-25 23:59:59")), arg(json!("2026-09-23 09:30:00")),
        ],
    );

    /* ticket and its reply */
    insert_if_absent(
        "tickets",
        "T-1",
        "INSERT INTO tickets (id,user_id,name,email,subject,message,status,created_at)
         VALUES (?,?,?,?,?,?,?,?)",
        &[
            arg(json!("T-1")),
            arg(json!("U-DEMO")),
            arg(json!("Amina Juma")),
            arg(json!("demo@client.com")),
            arg(json!("Can I pay in two installments?")),
            arg(json!("Hello, is it possible to split the payment 50/50?")),
            arg(json!("Open")),
            arg(json!("2026-09-25 08:00:00")),
        ],
    );
    insert_if_absent(
        "ticket_replies",
        "TR-1",
        "INSERT INTO ticket_replies (id,ticket_id,by_name,body,created_at) VALUES (?,?,?,?,?)",
        &[
            arg(json!("TR-1")),
            arg(json!("T-1")),
            arg(json!("Jackson Mbilinyi (CEO)")),
            arg(json!(
                "Yes — 50% to start, 50% on delivery. I will reflect this on your invoice."
            )),
            arg(json!("2026-09-26 10:00:00")),
        ],
    );
}
