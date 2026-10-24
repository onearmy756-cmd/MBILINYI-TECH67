//! Geography, the online office, and the SMS log.
//!
//! Three things the CEO asked for live here together because they are all about
//! reach: where the services are delivered (every country, and every region of
//! Tanzania), the online office the staff and clients meet in, and the record of
//! every SMS the platform sent so the admin can see each request that arrived.

use crate::api::ctx::Ctx;
use crate::api::map;
use crate::db;
use crate::util::{self, bad, err, ok};
use axum::response::Response;
use rusqlite::types::Value;
use serde_json::{json, Value as J};

fn arg(v: J) -> Value {
    db::from_json(v)
}

/// The 31 regions of Tanzania, so growth can be tracked region by region.
pub const TZ_REGIONS: &[(&str, &str)] = &[
    ("Arusha", "AR"),
    ("Dar es Salaam", "DS"),
    ("Dodoma", "DD"),
    ("Geita", "GT"),
    ("Iringa", "IR"),
    ("Kagera", "KG"),
    ("Katavi", "KT"),
    ("Kigoma", "KM"),
    ("Kilimanjaro", "KL"),
    ("Lindi", "LD"),
    ("Manyara", "MY"),
    ("Mara", "MR"),
    ("Mbeya", "MB"),
    ("Morogoro", "MO"),
    ("Mtwara", "MT"),
    ("Mwanza", "MW"),
    ("Njombe", "NJ"),
    ("Pemba North", "PN"),
    ("Pemba South", "PS"),
    ("Pwani", "PW"),
    ("Rukwa", "RK"),
    ("Ruvuma", "RV"),
    ("Shinyanga", "SH"),
    ("Simiyu", "SI"),
    ("Singida", "SG"),
    ("Songwe", "SW"),
    ("Tabora", "TB"),
    ("Tanga", "TG"),
    ("Unguja North", "UN"),
    ("Unguja South", "US"),
    ("Mjini Magharibi", "MM"),
];

/// The countries the platform serves. Tanzania is the home market; the rest are
/// the international market, which never sees the Tanzania-only services.
pub const COUNTRIES: &[(&str, &str)] = &[
    ("Tanzania", "TZ"),
    ("Kenya", "KE"),
    ("Uganda", "UG"),
    ("Rwanda", "RW"),
    ("Burundi", "BI"),
    ("DR Congo", "CD"),
    ("Zambia", "ZM"),
    ("Malawi", "MW"),
    ("Mozambique", "MZ"),
    ("South Africa", "ZA"),
    ("Nigeria", "NG"),
    ("Ghana", "GH"),
    ("Egypt", "EG"),
    ("Morocco", "MA"),
    ("United Arab Emirates", "AE"),
    ("Saudi Arabia", "SA"),
    ("Qatar", "QA"),
    ("India", "IN"),
    ("China", "CN"),
    ("Japan", "JP"),
    ("Australia", "AU"),
    ("United Kingdom", "GB"),
    ("Germany", "DE"),
    ("France", "FR"),
    ("Netherlands", "NL"),
    ("Sweden", "SE"),
    ("Norway", "NO"),
    ("Canada", "CA"),
    ("United States", "US"),
    ("Brazil", "BR"),
];

/// The display name for an ISO country code, for the codes we serve.
pub fn country_name(code: &str) -> String {
    let up = code.trim().to_uppercase();
    COUNTRIES
        .iter()
        .find(|(_, c)| *c == up)
        .map(|(n, _)| (*n).to_string())
        .unwrap_or_else(|| if up.is_empty() { "Tanzania".into() } else { up })
}

/// Record that a request came from this country and region.
///
/// The counts live on the catalogue rows, which is what makes the growth view
/// move. A place outside the seeded list is added on first use, so a client in
/// a country the platform has never seen still shows up instead of being lost.
pub fn note_request(country: &str, region: &str) {
    let country = country.trim();
    if country.is_empty() {
        return;
    }
    bump("country", country, "", &country_code(country));
    if !region.trim().is_empty() {
        bump("region", "Tanzania", region.trim(), "");
    }
}

fn country_code(country: &str) -> String {
    COUNTRIES
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(country))
        .map(|(_, c)| (*c).to_string())
        .unwrap_or_default()
}

/// Add one to the usage count for a place, inserting the row if it is new.
fn bump(kind: &str, country: &str, region: &str, code: &str) {
    let changed = db::run(
        "UPDATE service_regions SET requests = requests + 1 WHERE kind = ? AND country = ? AND region = ?",
        &[arg(json!(kind)), arg(json!(country)), arg(json!(region))],
    )
    .unwrap_or(0);
    if changed == 0 {
        let _ = db::run(
            "INSERT INTO service_regions (id, country, region, code, kind, is_active, requests, created_at)
             VALUES (?,?,?,?,?,?,?,?)",
            &[
                arg(json!(util::new_id("RG-"))), arg(json!(country)), arg(json!(region)),
                arg(json!(code)), arg(json!(kind)), arg(json!(1)), arg(json!(1)),
                arg(json!(util::now())),
            ],
        );
    }
}

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
         /* A message a client sent in, waiting for the admin to read, approve
            and turn into a request. The admin sees these in the inbox. */
         CREATE TABLE IF NOT EXISTS sms_inbox (
           id           VARCHAR(64)  NOT NULL,
           phone        VARCHAR(60)  NOT NULL,
           sender_name  VARCHAR(160) NOT NULL DEFAULT '',
           message      TEXT         NOT NULL,
           gateway      VARCHAR(40)  NOT NULL DEFAULT '',
           status       VARCHAR(30)  NOT NULL DEFAULT 'new',
           request_id   VARCHAR(64)  NOT NULL DEFAULT '',
           handled_by   VARCHAR(120) NOT NULL DEFAULT '',
           created_at   DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
           PRIMARY KEY (id)
         );
         /* Meeting transcripts. The AI scribe writes one per room: the raw text,
            the summary and the action items, so a meeting leaves a record. */
         CREATE TABLE IF NOT EXISTS transcripts (
           id          VARCHAR(64)  NOT NULL,
           room_id     VARCHAR(64)  NOT NULL DEFAULT '',
           room_name   VARCHAR(160) NOT NULL DEFAULT '',
           language    VARCHAR(20)  NOT NULL DEFAULT 'sw',
           transcript  TEXT         NOT NULL DEFAULT '',
           summary     TEXT         NOT NULL DEFAULT '',
           actions     TEXT         NOT NULL DEFAULT '[]',
           provider    VARCHAR(40)  NOT NULL DEFAULT '',
           created_at  DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
           PRIMARY KEY (id)
         );
         CREATE INDEX IF NOT EXISTS idx_sms_req ON sms_log(request_id);
         CREATE INDEX IF NOT EXISTS idx_region_country ON service_regions(country);
         CREATE INDEX IF NOT EXISTS idx_inbox_status ON sms_inbox(status);
         CREATE INDEX IF NOT EXISTS idx_transcript_room ON transcripts(room_id);",
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
                if x.is_empty() {
                    "Tanzania".to_string()
                } else {
                    x
                }
            };
            let rows = db::rows(
                "SELECT * FROM service_regions WHERE kind = 'region' AND country = ?1 ORDER BY region ASC",
                &[arg(json!(country))],
            );
            ok(
                json!({ "country": country, "regions": rows.iter().map(map::region).collect::<Vec<_>>() }),
            )
        }
        // Growth of the services, by country and by region. Counts come from the
        // requests that name a place, so the chart reflects real work.
        "growth" => {
            // Only places that have actually been used, with their real counts.
            // The seeded catalogue is what the platform *can* serve; this is
            // what it has served, which is what a growth chart must show.
            let by_country = db::rows(
                "SELECT country, requests AS n FROM service_regions
                 WHERE kind = 'country' AND requests > 0 ORDER BY n DESC, country ASC",
                &[],
            );
            let by_region = db::rows(
                "SELECT region, requests AS n FROM service_regions
                 WHERE kind = 'region' AND requests > 0 ORDER BY n DESC, region ASC",
                &[],
            );
            let total_regions = db::scalar_i64(
                "SELECT COUNT(*) FROM service_regions WHERE kind = 'region'",
                &[],
            );
            let total_countries = db::scalar_i64(
                "SELECT COUNT(*) FROM service_regions WHERE kind = 'country'",
                &[],
            );
            let requests = db::scalar_i64("SELECT COUNT(*) FROM requests", &[]);
            // Distinct places the work has actually come from, versus the full
            // catalogue the platform advertises.
            let active_countries = db::scalar_i64(
                "SELECT COUNT(*) FROM service_regions WHERE kind = 'country' AND requests > 0",
                &[],
            );
            let active_regions = db::scalar_i64(
                "SELECT COUNT(*) FROM service_regions WHERE kind = 'region' AND requests > 0",
                &[],
            );
            ok(json!({
                "countries": total_countries,
                "regions": total_regions,
                "activeCountries": active_countries,
                "activeRegions": active_regions,
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
            let rows = db::rows(
                "SELECT * FROM sms_log ORDER BY created_at DESC LIMIT 200",
                &[],
            );
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
            for (field, setting) in [("baseUrl", "sms_base_url"), ("senderId", "sms_sender_id")] {
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
            let rows = db::rows(
                "SELECT * FROM office_rooms ORDER BY created_at DESC LIMIT 100",
                &[],
            );
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
        // ---- inbound SMS: the admin sees every message a client sent in ----
        "inbox" => inbox_list(c),
        "inbox_approve" => inbox_approve(c),
        "inbox_dismiss" => inbox_dismiss(c),
        // ---- AI scribe: turn a meeting into a written record ----
        "scribe_list" => scribe_list(c),
        "scribe_save" => scribe_save(c).await,
        _ => bad("Unknown ofisi action."),
    }
}

/// The admin's inbound SMS inbox: every message a client sent in, newest first,
/// with the ones still waiting to be handled called out.
fn inbox_list(c: &Ctx) -> Response {
    if let Err(e) = c.require_admin() {
        return e;
    }
    let status = c.p("status");
    let rows = db::rows(
        "SELECT * FROM sms_inbox WHERE (?1 = '' OR status = ?1) ORDER BY created_at DESC LIMIT 200",
        &[arg(json!(status))],
    );
    let new_count = db::scalar_i64("SELECT COUNT(*) FROM sms_inbox WHERE status = 'new'", &[]);
    ok(json!({
        "messages": rows.iter().map(|r| json!({
            "id": util::s(r.get("id").unwrap_or(&J::Null)),
            "phone": util::s(r.get("phone").unwrap_or(&J::Null)),
            "senderName": util::s(r.get("sender_name").unwrap_or(&J::Null)),
            "message": util::s(r.get("message").unwrap_or(&J::Null)),
            "gateway": util::s(r.get("gateway").unwrap_or(&J::Null)),
            "status": util::s(r.get("status").unwrap_or(&J::Null)),
            "requestId": util::s(r.get("request_id").unwrap_or(&J::Null)),
            "handledBy": util::s(r.get("handled_by").unwrap_or(&J::Null)),
            "createdAt": util::iso(r.get("created_at").and_then(|v| v.as_str())),
        })).collect::<Vec<_>>(),
        "newCount": new_count,
        "count": rows.len(),
    }))
}

/// The provider's inbound webhook. A message that arrives by SMS is stored here
/// so the admin can read it, approve it and turn it into a request — which is
/// what the CEO asked for: "admin ataona request zote kwa normal sms".
///
/// This endpoint is called by the SMS provider, so it cannot carry a CSRF token.
/// It is instead gated on a shared secret the CEO sets, passed as a header or a
/// query parameter, and the message is only *recorded* here — nothing is
/// approved automatically.
pub fn inbound_webhook(c: &Ctx) -> Response {
    let expected = crate::settings::get("sms_inbound_secret", "");
    let given = {
        let h = c
            .headers
            .get("x-sms-secret")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        if h.is_empty() {
            c.p("secret")
        } else {
            h
        }
    };
    if expected.trim().is_empty() {
        return bad("No inbound SMS secret is configured, so this endpoint is closed.");
    }
    if !crate::payments::gateways_constant_time_eq(given.as_bytes(), expected.trim().as_bytes()) {
        return err("The inbound secret does not match.", 403);
    }
    // Providers name the fields differently; accept the common spellings.
    let phone = {
        let p = c.p("from");
        if p.is_empty() {
            c.p("phone")
        } else {
            p
        }
    };
    let message = {
        let m = c.p("message");
        if m.is_empty() {
            c.p("text")
        } else {
            m
        }
    };
    if phone.trim().is_empty() || message.trim().is_empty() {
        return bad("The callback must carry a sender and a message.");
    }
    let name = {
        let n = c.p("name");
        if n.is_empty() {
            c.p("senderName")
        } else {
            n
        }
    };
    let id = util::new_id("IN-");
    let _ = db::run(
        "INSERT INTO sms_inbox (id, phone, sender_name, message, gateway, status, request_id, handled_by, created_at)
         VALUES (?,?,?,?,?,?,?,?,?)",
        &[
            arg(json!(id)), arg(json!(phone)), arg(json!(name)), arg(json!(message)),
            arg(json!(c.p("gateway"))), arg(json!("new")), arg(json!("")), arg(json!("")),
            arg(json!(util::now())),
        ],
    );
    ok(json!({ "received": true, "id": id }))
}

/// Turn an inbound SMS into a request, or mark it handled. Either way the row
/// records who dealt with it, so the admin can see nothing was lost.
fn inbox_approve(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    let Ok(admin) = c.require_admin() else {
        return util::forbidden("Admin access required.");
    };
    let id = c.p("id");
    let Some(row) = db::row("SELECT * FROM sms_inbox WHERE id = ?", &[arg(json!(id))]) else {
        return err("That message is not in the inbox.", 404);
    };
    let who = util::s(admin.get("name").unwrap_or(&J::Null));
    let request_id = c.p("requestId");
    let _ = db::run(
        "UPDATE sms_inbox SET status = ?, request_id = ?, handled_by = ? WHERE id = ?",
        &[
            arg(json!("approved")),
            arg(json!(request_id)),
            arg(json!(who)),
            arg(json!(id)),
        ],
    );
    // Tell the client their message was received, in the background.
    let phone = util::s(row.get("phone").unwrap_or(&J::Null));
    if !phone.is_empty() {
        let log_id = util::new_id("SMS-");
        let text = "Ombi lako limepokelewa. Timu ya Mbilinyi Tech inaliandaa; utapata taarifa ya maendeleo.";
        let _ = db::run(
            "INSERT INTO sms_log (id, phone, message, request_id, kind, status, created_at) VALUES (?,?,?,?,?,?,?)",
            &[
                arg(json!(log_id)), arg(json!(phone)), arg(json!(text)), arg(json!(request_id)),
                arg(json!("inbox_ack")), arg(json!("queued")), arg(json!(util::now())),
            ],
        );
        crate::sms::spawn_send(log_id, phone, text.to_string());
    }
    ok(json!({ "approved": true, "handledBy": who }))
}

fn inbox_dismiss(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    let Ok(admin) = c.require_admin() else {
        return util::forbidden("Admin access required.");
    };
    let who = util::s(admin.get("name").unwrap_or(&J::Null));
    let _ = db::run(
        "UPDATE sms_inbox SET status = 'dismissed', handled_by = ? WHERE id = ?",
        &[arg(json!(who)), arg(json!(c.p("id")))],
    );
    ok(json!({ "dismissed": true }))
}

fn scribe_list(c: &Ctx) -> Response {
    if let Err(e) = c.require_admin() {
        return e;
    }
    let room = c.p("roomId");
    let rows = db::rows(
        "SELECT * FROM transcripts WHERE (?1 = '' OR room_id = ?1) ORDER BY created_at DESC LIMIT 100",
        &[arg(json!(room))],
    );
    ok(json!({
        "transcripts": rows.iter().map(|r| json!({
            "id": util::s(r.get("id").unwrap_or(&J::Null)),
            "roomId": util::s(r.get("room_id").unwrap_or(&J::Null)),
            "roomName": util::s(r.get("room_name").unwrap_or(&J::Null)),
            "language": util::s(r.get("language").unwrap_or(&J::Null)),
            "summary": util::s(r.get("summary").unwrap_or(&J::Null)),
            "actions": serde_json::from_str::<J>(&util::s(r.get("actions").unwrap_or(&J::Null))).unwrap_or(json!([])),
            "hasTranscript": !util::s(r.get("transcript").unwrap_or(&J::Null)).is_empty(),
            "createdAt": util::iso(r.get("created_at").and_then(|v| v.as_str())),
        })).collect::<Vec<_>>(),
    }))
}

/// Store a meeting's transcript, then ask the AI for a summary and the action
/// items. The transcript is saved first, so a provider failure still leaves the
/// record the meeting produced.
async fn scribe_save(c: &Ctx) -> Response {
    if let Err(e) = c.require_csrf() {
        return e;
    }
    let Ok(_u) = c.require_user() else {
        return util::unauth();
    };
    let transcript = c.p("transcript");
    if transcript.trim().is_empty() {
        return bad("There is no transcript to save.");
    }
    let room_id = c.p("roomId");
    let room_name = {
        let n = c.p("roomName");
        if n.is_empty() {
            "Meeting".to_string()
        } else {
            n
        }
    };
    let language = {
        let l = c.p("language");
        if l.is_empty() {
            "sw".to_string()
        } else {
            l
        }
    };
    let id = util::new_id("TR-");
    let _ = db::run(
        "INSERT INTO transcripts (id, room_id, room_name, language, transcript, summary, actions, provider, created_at)
         VALUES (?,?,?,?,?,?,?,?,?)",
        &[
            arg(json!(id)), arg(json!(room_id)), arg(json!(room_name)), arg(json!(language)),
            arg(json!(transcript)), arg(json!("")), arg(json!("[]")), arg(json!("")), arg(json!(util::now())),
        ],
    );
    // The summary is written by the configured AI provider. With none set the
    // transcript stands on its own, and the reply says so rather than inventing
    // a summary.
    let (summary, actions, provider) = if crate::llm::ready() {
        let prompt = format!(
            "You are the AI scribe for Mbilinyi Tech Solutions. Summarise this meeting transcript in {} \
             in at most 120 words, then list the action items as short lines starting with '- '. \
             Reply with the summary first, then a line 'ACTIONS:' and the items.\n\nTRANSCRIPT:\n{}",
            if language == "sw" { "Kiswahili" } else { "English" },
            transcript.chars().take(12000).collect::<String>(),
        );
        match crate::llm::complete(&prompt).await {
            Ok(text) => {
                let (s, a) = split_summary(&text);
                (s, a, crate::llm::provider_name())
            }
            Err(e) => (
                format!("The transcript was saved but the summary failed: {e}"),
                Vec::new(),
                String::new(),
            ),
        }
    } else {
        (
            "The transcript was saved. No AI provider is configured, so no summary was written."
                .to_string(),
            Vec::new(),
            String::new(),
        )
    };
    let _ = db::run(
        "UPDATE transcripts SET summary = ?, actions = ?, provider = ? WHERE id = ?",
        &[
            arg(json!(summary)),
            arg(json!(
                serde_json::to_string(&actions).unwrap_or_else(|_| "[]".into())
            )),
            arg(json!(provider)),
            arg(json!(id)),
        ],
    );
    ok(json!({ "id": id, "summary": summary, "actions": actions, "provider": provider }))
}

/// Split the model's reply into the summary and the action list.
fn split_summary(text: &str) -> (String, Vec<String>) {
    let mut summary = String::new();
    let mut actions = Vec::new();
    let mut in_actions = false;
    for line in text.lines() {
        let t = line.trim();
        if t.eq_ignore_ascii_case("ACTIONS:") || t.eq_ignore_ascii_case("ACTION ITEMS:") {
            in_actions = true;
            continue;
        }
        if in_actions {
            let item = t.trim_start_matches(['-', '*', '•']).trim();
            if !item.is_empty() {
                actions.push(item.to_string());
            }
        } else {
            if !summary.is_empty() {
                summary.push('\n');
            }
            summary.push_str(t);
        }
    }
    if summary.is_empty() {
        summary = text.trim().to_string();
    }
    (summary, actions)
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
    let _ = db::run(
        "UPDATE office_rooms SET status = ? WHERE id = ?",
        &[arg(json!(status)), arg(json!(id))],
    );
    ok(json!({ "status": status }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_scribe_reply_splits_into_summary_and_actions() {
        let reply = "The client asked for a website with payments.\nBudget was agreed at 4,000,000 TZS.\nACTIONS:\n- Send the contract\n- Schedule the kick-off\n* Book a designer";
        let (summary, actions) = split_summary(reply);
        assert!(summary.contains("website with payments"));
        assert!(!summary.contains("Send the contract"));
        assert_eq!(actions.len(), 3);
        assert_eq!(actions[0], "Send the contract");
        assert_eq!(actions[2], "Book a designer");
    }

    #[test]
    fn a_reply_with_no_actions_stays_a_summary() {
        let (summary, actions) = split_summary("Just a note about the meeting.");
        assert!(summary.contains("Just a note"));
        assert!(actions.is_empty());
    }

    #[test]
    fn every_tanzanian_region_is_present() {
        assert_eq!(TZ_REGIONS.len(), 31);
        assert!(TZ_REGIONS.iter().any(|(n, _)| *n == "Dar es Salaam"));
        assert!(TZ_REGIONS.iter().any(|(n, _)| *n == "Dodoma"));
    }

    /// A country code must resolve to the name the growth view shows, and an
    /// unknown code must survive as itself rather than being dropped or
    /// panicking. This is what keeps a client in a new country visible.
    #[test]
    fn a_country_code_resolves_to_its_name() {
        assert_eq!(country_name("tz"), "Tanzania");
        assert_eq!(country_name("US"), "United States");
        assert_eq!(country_name("ZZ"), "ZZ");
        assert_eq!(country_name(""), "Tanzania");
    }

    #[test]
    fn the_country_list_reaches_every_continent() {
        for code in ["US", "GB", "AE", "IN", "ZA", "BR", "AU"] {
            assert!(COUNTRIES.iter().any(|(_, c)| *c == code), "missing {code}");
        }
    }
}
