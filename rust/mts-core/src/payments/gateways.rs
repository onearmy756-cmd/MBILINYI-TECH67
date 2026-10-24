//! The gateway integrations.
//!
//! Each gateway exposes the same pair of calls:
//!
//! * `initiate` — push a charge to the provider and return what the payer must
//!   do next (a prompt on their phone, or a redirect to a hosted page).
//! * `verify` — ask the provider whether that charge actually settled.
//!
//! Nothing here runs until the admin has switched the gateway on and stored its
//! merchant credentials, because those credentials are what name the account the
//! money settles into. A gateway with no credentials has nowhere to send the
//! funds, so it is refused rather than half-attempted.

use crate::settings;
use serde_json::{json, Value as J};

pub struct Outcome {
    pub ok: bool,
    pub reference: String,
    pub message: String,
    pub redirect: Option<String>,
}

fn fail(msg: impl Into<String>) -> Outcome {
    Outcome {
        ok: false,
        reference: String::new(),
        message: msg.into(),
        redirect: None,
    }
}

fn settled_msg() -> String {
    "Payment confirmed.".to_string()
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

/// Providers in Tanzania take whole shillings.
fn whole(amount: f64) -> i64 {
    amount.round() as i64
}

/// The currencies Stripe treats as having no minor unit: the amount is sent
/// as-is. Everything else is sent in cents, so the figure is multiplied by 100.
/// Sending TZS as cents would charge a hundred times the invoice.
const ZERO_DECIMAL: &[&str] = &[
    "BIF", "CLP", "DJF", "GNF", "JPY", "KMF", "KRW", "MGA", "PYG", "RWF", "UGX", "VND", "VUV",
    "XAF", "XOF", "XPF", "TZS",
];

/// The amount in the smallest unit Stripe expects for this currency.
fn stripe_minor(amount: f64, currency: &str) -> i64 {
    let cur = currency.to_uppercase();
    if ZERO_DECIMAL.contains(&cur.as_str()) {
        amount.round() as i64
    } else {
        (amount * 100.0).round() as i64
    }
}

/// Stripe's API root. Overridable so the CEO can point at a Stripe sandbox, or
/// at a Stripe-compatible processor, without a rebuild.
fn stripe_api_base() -> String {
    let b = settings::get("gw_stripe_api_base", "");
    if b.trim().is_empty() {
        "https://api.stripe.com".to_string()
    } else {
        b.trim_end_matches('/').to_string()
    }
}

/// The webhook signing secret (`whsec_…`), which is what lets a callback be
/// trusted. Without it a webhook is only a claim, so verification is refused
/// rather than waved through.
fn stripe_webhook_secret() -> String {
    settings::get("gw_stripe_webhook_secret", "")
}

/// Is the Stripe integration switched on with credentials the platform holds?
pub fn stripe_configured() -> bool {
    !settings::get("gw_stripe_secret", "").trim().is_empty()
}

/// Does the platform hold a webhook secret, so callbacks can be authenticated?
pub fn stripe_webhook_ready() -> bool {
    !stripe_webhook_secret().trim().is_empty()
}

/// Verify a Stripe webhook signature using the stored secret.
///
/// Returns `Ok(())` only when the signature is genuine.
pub fn verify_stripe_webhook(header: &str, body: &str) -> Result<(), String> {
    let secret = stripe_webhook_secret();
    if secret.trim().is_empty() {
        return Err(
            "No Stripe webhook secret is stored, so the callback cannot be authenticated.".into(),
        );
    }
    verify_stripe_signature(&secret, header, body)
}

/// Verify a Stripe webhook signature against a given secret.
///
/// Stripe sends `Stripe-Signature: t=<unix>,v1=<hmac-sha256 hex>`, where the
/// HMAC is over `"<t>.<raw body>"` keyed with the endpoint's `whsec_…` secret.
/// The comparison is constant-time and the timestamp is bounded, so a captured
/// callback cannot be replayed later. The secret is a parameter so the check can
/// be exercised without a database.
pub fn verify_stripe_signature(secret: &str, header: &str, body: &str) -> Result<(), String> {
    if secret.trim().is_empty() {
        return Err(
            "No Stripe webhook secret is stored, so the callback cannot be authenticated.".into(),
        );
    }
    let (mut t, mut v1) = (String::new(), String::new());
    for part in header.split(',') {
        let part = part.trim();
        if let Some(x) = part.strip_prefix("t=") {
            t = x.trim().to_string();
        } else if let Some(x) = part.strip_prefix("v1=") {
            v1 = x.trim().to_string();
        }
    }
    if t.is_empty() || v1.is_empty() {
        return Err("The Stripe-Signature header is missing its timestamp or v1 signature.".into());
    }
    // Reject a timestamp more than five minutes from now, in either direction.
    let ts: i64 = t
        .parse()
        .map_err(|_| "The Stripe-Signature timestamp is not a number.".to_string())?;
    let now = chrono::Utc::now().timestamp();
    if (now - ts).abs() > 300 {
        return Err("The Stripe callback is too old to trust.".into());
    }
    let signed = format!("{t}.{body}");
    let expected = stripe_hmac_hex(secret, signed.as_bytes());
    // Constant-time compare: a byte-by-byte `==` would leak the correct prefix.
    if constant_time_eq(expected.as_bytes(), v1.as_bytes()) {
        Ok(())
    } else {
        Err("The Stripe signature does not match. The callback was refused.".into())
    }
}

fn stripe_hmac_hex(secret: &str, msg: &[u8]) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    let mut mac = match Hmac::<Sha256>::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return String::new(),
    };
    mac.update(msg);
    hex::encode(mac.finalize().into_bytes())
}

/// Compare two byte slices without an early exit on the first difference.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    use subtle::ConstantTimeEq;
    if a.len() != b.len() {
        return false;
    }
    a.ct_eq(b).into()
}

/// Read the first path that yields a non-empty scalar. Providers differ in how
/// deeply they nest the same value, and this avoids a per-gateway struct.
fn pick(v: &J, paths: &[&str]) -> String {
    for p in paths {
        let mut cur = v;
        let mut found = true;
        for seg in p.split('.') {
            match cur.get(seg) {
                Some(n) => cur = n,
                None => {
                    found = false;
                    break;
                }
            }
        }
        if found {
            if let Some(s) = cur.as_str() {
                if !s.is_empty() {
                    return s.to_string();
                }
            }
            if let Some(n) = cur.as_i64() {
                return n.to_string();
            }
        }
    }
    String::new()
}

fn is_ok(code: u16) -> bool {
    (200..300).contains(&code)
}

/// The set of success words the Tanzanian providers use.
fn is_settled(status: &str) -> bool {
    matches!(status, "SUCCESS" | "SETTLED" | "COMPLETED" | "PAID")
}

/* ------------------------------------------------------------------ *
 * A provider described as data. Most Tanzanian mobile-money APIs are the
 * same shape: POST JSON with a bearer key, then GET a status endpoint.
 * ------------------------------------------------------------------ */
struct Provider {
    /// Settings holding the base URL and the bearer key.
    url_key: &'static str,
    default_url: &'static str,
    key_setting: &'static str,
    /// Paths appended to the base URL.
    initiate_path: &'static str,
    verify_path: &'static str,
    /// JSON keys to look for, in order of preference.
    initiate_ref: &'static [&'static str],
    verify_status: &'static [&'static str],
    /// Status path placeholders: `{reference}` is substituted.
    verify_uses_query: bool,
}

const CLICKPESA: Provider = Provider {
    url_key: "gw_clickpesa_url",
    default_url: "https://api.clickpesa.com",
    key_setting: "gw_clickpesa_key",
    initiate_path: "/third-parties/payments/initiate",
    verify_path: "/third-parties/payments/{reference}",
    initiate_ref: &["data.orderReference", "orderReference", "data.id"],
    verify_status: &["data.status", "status"],
    verify_uses_query: false,
};

const SELCOM: Provider = Provider {
    url_key: "gw_selcom_url",
    default_url: "https://apigw.selcommobile.com/v1",
    key_setting: "gw_selcom_key",
    initiate_path: "/payments/create-order",
    verify_path: "/payments/order-status?vendor={vendor}&reference={reference}",
    initiate_ref: &["data.reference", "reference", "data.transid"],
    verify_status: &["data.result", "result", "status"],
    verify_uses_query: true,
};

const MPESA: Provider = Provider {
    url_key: "gw_mpesa_url",
    default_url: "https://openapi.m-pesa.com/openapi/ipg/v2",
    key_setting: "gw_mpesa_key",
    initiate_path: "/vodacomTZN/c2bPayment/singleStage/",
    verify_path: "/vodacomTZN/c2bPayment/queryTransactionStatus/",
    initiate_ref: &["output_TransactionID", "output_ConversationID"],
    verify_status: &["output_ResponseTransactionStatus", "output_ResponseCode"],
    verify_uses_query: false,
};

/// Tigo Pesa (Mixx by Yas). The public API is a JSON collect endpoint; the base
/// is a setting because Tigo issues each merchant its own host.
const TIGOPESA: Provider = Provider {
    url_key: "gw_tigopesa_url",
    default_url: "https://api.tigo.co.tz/v1",
    key_setting: "gw_tigopesa_key",
    initiate_path: "/payments/collect",
    verify_path: "/payments/{reference}",
    initiate_ref: &["transactionId", "data.transactionId", "reference"],
    verify_status: &["status", "data.status"],
    verify_uses_query: false,
};

/// Airtel Money. Same shape; Airtel issues a per-country host and a client id
/// and secret, which are held together in the key setting.
const AIRTEL: Provider = Provider {
    url_key: "gw_airtel_url",
    default_url: "https://openapi.airtel.africa",
    key_setting: "gw_airtel_key",
    initiate_path: "/merchant/v1/payments/",
    verify_path: "/standard/v1/payments/{reference}",
    initiate_ref: &["data.transaction.id", "transaction.id", "data.transactionId"],
    verify_status: &["data.transaction.status", "status"],
    verify_uses_query: false,
};

/// HaloPesa (Halotel). Served through the same collect shape.
const HALOPESA: Provider = Provider {
    url_key: "gw_halopesa_url",
    default_url: "https://api.halopesa.co.tz/v1",
    key_setting: "gw_halopesa_key",
    initiate_path: "/payments/collect",
    verify_path: "/payments/{reference}",
    initiate_ref: &["transactionId", "data.transactionId", "reference"],
    verify_status: &["status", "data.status"],
    verify_uses_query: false,
};

fn base_of(p: &Provider) -> String {
    settings::get(p.url_key, p.default_url)
        .trim_end_matches('/')
        .to_string()
}

async fn post_bearer(url: &str, key: &str, body: J) -> Result<(u16, J), String> {
    let res = client()
        .post(url)
        .header("Authorization", format!("Bearer {key}"))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    let code = res.status().as_u16();
    let text = res.text().await.unwrap_or_default();
    Ok((
        code,
        serde_json::from_str(&text).unwrap_or(json!({ "raw": text })),
    ))
}

async fn get_bearer(url: &str, key: &str) -> Result<(u16, J), String> {
    let res = client()
        .get(url)
        .header("Authorization", format!("Bearer {key}"))
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    let code = res.status().as_u16();
    let text = res.text().await.unwrap_or_default();
    Ok((
        code,
        serde_json::from_str(&text).unwrap_or(json!({ "raw": text })),
    ))
}

/// The shared body shape for a mobile-money collection.
fn collect_body(amount: f64, currency: &str, phone: &str, reference: &str, account: &str) -> J {
    json!({
        "amount": whole(amount),
        "currency": if currency.is_empty() { "TZS" } else { currency },
        "reference": reference,
        "orderReference": reference,
        "msisdn": phone,
        "phoneNumber": phone,
        "customerAccount": account,
    })
}

/// `initiate` — push a charge to the provider.
#[allow(clippy::too_many_arguments)]
pub async fn initiate(
    gateway: &str,
    amount: f64,
    currency: &str,
    invoice_currency: &str,
    payer_phone: &str,
    payer_email: &str,
    payer_name: &str,
    reference: &str,
) -> Outcome {
    let _ = (payer_email, payer_name);
    match gateway {
        "clickpesa" | "selcom" | "mpesa" | "tigopesa" | "airtel" | "halopesa" => {
            let p = match gateway {
                "clickpesa" => &CLICKPESA,
                "selcom" => &SELCOM,
                "tigopesa" => &TIGOPESA,
                "airtel" => &AIRTEL,
                "halopesa" => &HALOPESA,
                _ => &MPESA,
            };
            let key = settings::get(p.key_setting, "");
            if key.is_empty() {
                return fail(format!("{} is not configured.", gateway));
            }
            let account = settings::get(&format!("gw_{gateway}_account"), "");
            let body = if gateway == "mpesa" {
                json!({
                    "input_Amount": whole(amount).to_string(),
                    "input_Country": "TZN",
                    "input_Currency": "TZS",
                    "input_CustomerMSISDN": payer_phone,
                    "input_ServiceProviderCode": settings::get("gw_mpesa_shortcode", ""),
                    "input_TransactionReference": reference,
                    "input_ThirdPartyConversationID": reference,
                    "input_PurchasedItemsDesc": "Mbilinyi Tech Solutions",
                })
            } else if gateway == "airtel" {
                // Airtel wants the amount nested and the currency in its own
                // object, unlike the flat collect body the others take.
                json!({
                    "reference": reference,
                    "subscriber": { "country": "TZ", "currency": "TZS", "msisdn": payer_phone },
                    "transaction": {
                        "amount": whole(amount),
                        "country": "TZ",
                        "currency": "TZS",
                        "id": reference,
                    },
                })
            } else {
                collect_body(amount, currency, payer_phone, reference, &account)
            };
            let url = format!("{}{}", base_of(p), p.initiate_path);
            match post_bearer(&url, &key, body).await {
                Ok((code, v)) if is_ok(code) => {
                    let r = pick(&v, p.initiate_ref);
                    Outcome {
                        ok: true,
                        reference: if r.is_empty() {
                            reference.to_string()
                        } else {
                            r
                        },
                        message: "Approve the payment prompt on your phone.".into(),
                        redirect: {
                            let u = pick(&v, &["data.paymentUrl", "paymentUrl"]);
                            if u.is_empty() {
                                None
                            } else {
                                Some(u)
                            }
                        },
                    }
                }
                Ok((code, v)) => fail(format!(
                    "{gateway} refused the charge (HTTP {code}): {}",
                    pick(&v, &["message", "error", "output_ResponseDesc", "raw"])
                )),
                Err(e) => fail(e),
            }
        }
        "beem" => initiate_beem(amount, currency, payer_phone, reference).await,
        "stripe" => initiate_stripe(amount, currency, reference).await,
        "paypal" => initiate_paypal(amount, currency, reference).await,
        "bank_local" | "swift" | "wise" | "payoneer" => Outcome {
            ok: true,
            reference: reference.to_string(),
            message: bank_instructions(gateway, amount, invoice_currency, currency, reference),
            redirect: None,
        },
        other => fail(format!("Unknown gateway: {other}")),
    }
}

/// `verify` — ask the provider whether the charge settled.
pub async fn verify(gateway: &str, reference: &str) -> Outcome {
    match gateway {
        "clickpesa" | "selcom" | "tigopesa" | "airtel" | "halopesa" => {
            let p = match gateway {
                "clickpesa" => &CLICKPESA,
                "selcom" => &SELCOM,
                "tigopesa" => &TIGOPESA,
                "airtel" => &AIRTEL,
                _ => &HALOPESA,
            };
            let key = settings::get(p.key_setting, "");
            if key.is_empty() {
                return fail(format!("{} is not configured.", gateway));
            }
            let vendor = settings::get("gw_selcom_vendor", "");
            let path = p.verify_path.replace("{reference}", reference).replace("{vendor}", &vendor);
            let url = format!("{}{}", base_of(p), path);
            let _ = p.verify_uses_query;
            match get_bearer(&url, &key).await {
                Ok((_code, v)) => {
                    let status = pick(&v, p.verify_status).to_uppercase();
                    let ok = is_settled(&status);
                    Outcome {
                        ok,
                        reference: reference.to_string(),
                        message: if ok { settled_msg() } else { format!("{gateway} reports status {status}.") },
                        redirect: None,
                    }
                }
                Err(e) => fail(e),
            }
        }
        "beem" => verify_beem(reference).await,
        "stripe" => verify_stripe(reference).await,
        "paypal" => verify_paypal(reference).await,
        "bank_local" | "swift" | "wise" | "payoneer" | "mpesa" => Outcome {
            ok: false,
            reference: reference.to_string(),
            message: "This gateway is confirmed manually: the admin matches the reference against the statement.".into(),
            redirect: None,
        },
        other => fail(format!("Unknown gateway: {other}")),
    }
}

/// The account details a payer needs for a bank route.
///
/// The figure is stated in the invoice's own currency, because that is the debt.
/// When a payer chooses to send a different currency there is no exchange rate
/// configured here, so naming a converted figure would be inventing one.
fn bank_instructions(
    gateway: &str,
    amount: f64,
    invoice_currency: &str,
    send_currency: &str,
    reference: &str,
) -> String {
    let cur = if invoice_currency.is_empty() {
        "TZS"
    } else {
        invoice_currency
    };
    let (bank, account, holder, swift) = match gateway {
        "bank_local" => (
            settings::get("gw_bank_name", ""),
            settings::get("gw_bank_account", ""),
            settings::get("gw_bank_holder", ""),
            settings::get("gw_bank_swift", ""),
        ),
        "wise" => (
            "Wise".to_string(),
            settings::get("gw_wise_iban", ""),
            settings::get("gw_wise_holder", ""),
            String::new(),
        ),
        "payoneer" => (
            "Payoneer".to_string(),
            settings::get("gw_payoneer_id", ""),
            settings::get("gw_payoneer_email", ""),
            String::new(),
        ),
        _ => (
            settings::get("gw_intl_bank_name", ""),
            settings::get("gw_intl_iban", ""),
            settings::get("gw_intl_holder", ""),
            settings::get("gw_intl_swift", ""),
        ),
    };
    let mut out = format!("Transfer {cur} {amount:.2} to {bank}, account {account} ({holder}).");
    if !swift.is_empty() {
        out.push_str(&format!(" SWIFT/BIC {swift}."));
    }
    if !send_currency.is_empty() && send_currency != cur {
        out.push_str(&format!(
            " Send the equivalent in {send_currency} at the prevailing rate."
        ));
    }
    out.push_str(&format!(
        " Quote reference {reference} so the payment is matched automatically."
    ));
    out
}

/* ---------------- Beem Africa (key as Basic username) ---------------- */

async fn initiate_beem(amount: f64, currency: &str, phone: &str, reference: &str) -> Outcome {
    let key = settings::get("gw_beem_key", "");
    let secret = settings::get("gw_beem_secret", "");
    if key.is_empty() || secret.is_empty() {
        return fail("Beem is not configured.");
    }
    let base = settings::get("gw_beem_url", "https://apiotp.beem.africa/v1");
    let url = format!("{}/payments/collect", base.trim_end_matches('/'));
    let body = collect_body(
        amount,
        currency,
        phone,
        reference,
        &settings::get("gw_beem_account", ""),
    );
    match client()
        .post(&url)
        .basic_auth(&key, Some(&secret))
        .json(&body)
        .send()
        .await
    {
        Ok(r) => {
            let code = r.status().as_u16();
            let v: J = r.json().await.unwrap_or(json!({}));
            if is_ok(code) {
                let x = pick(&v, &["transaction_id", "data.transaction_id", "reference"]);
                Outcome {
                    ok: true,
                    reference: if x.is_empty() {
                        reference.to_string()
                    } else {
                        x
                    },
                    message: "Approve the payment prompt on your phone.".into(),
                    redirect: None,
                }
            } else {
                fail(format!(
                    "Beem refused the charge (HTTP {code}): {}",
                    pick(&v, &["message", "error", "raw"])
                ))
            }
        }
        Err(e) => fail(format!("Network error: {e}")),
    }
}

async fn verify_beem(reference: &str) -> Outcome {
    let key = settings::get("gw_beem_key", "");
    let secret = settings::get("gw_beem_secret", "");
    if key.is_empty() {
        return fail("Beem is not configured.");
    }
    let base = settings::get("gw_beem_url", "https://apiotp.beem.africa/v1");
    let url = format!(
        "{}/payments/status/{}",
        base.trim_end_matches('/'),
        reference
    );
    match client()
        .get(&url)
        .basic_auth(&key, Some(&secret))
        .send()
        .await
    {
        Ok(r) => {
            let v: J = r.json().await.unwrap_or(json!({}));
            let status = pick(&v, &["status", "data.status"]).to_uppercase();
            let ok = is_settled(&status);
            Outcome {
                ok,
                reference: reference.to_string(),
                message: if ok {
                    settled_msg()
                } else {
                    format!("Beem reports status {status}.")
                },
                redirect: None,
            }
        }
        Err(e) => fail(format!("Network error: {e}")),
    }
}

/* ---------------- Stripe ---------------- */

async fn initiate_stripe(amount: f64, currency: &str, reference: &str) -> Outcome {
    let secret = settings::get("gw_stripe_secret", "");
    let account = settings::get("gw_stripe_account", "");
    if secret.is_empty() {
        return fail("Stripe is not configured.");
    }
    // Stripe takes the smallest unit: cents for dollars, whole shillings for TZS.
    let minor = stripe_minor(amount, currency);
    let cur = if currency.is_empty() { "USD" } else { currency };
    let site = settings::get("platform_url", "");
    let success = format!("{site}/client?paid={reference}");
    let cancel = format!("{site}/client?cancelled={reference}");
    let minor_s = minor.to_string();
    let form = [
        ("mode", "payment"),
        ("success_url", success.as_str()),
        ("cancel_url", cancel.as_str()),
        ("line_items[0][price_data][currency]", cur),
        (
            "line_items[0][price_data][product_data][name]",
            "Mbilinyi Tech Solutions",
        ),
        ("line_items[0][price_data][unit_amount]", minor_s.as_str()),
        ("line_items[0][quantity]", "1"),
        ("client_reference_id", reference),
    ];
    let mut req = client()
        .post(format!("{}/v1/checkout/sessions", stripe_api_base()))
        .bearer_auth(&secret)
        .form(&form);
    if !account.is_empty() {
        // A destination charge puts the funds on the connected account.
        req = req.header("Stripe-Account", &account);
    }
    match req.send().await {
        Ok(r) => {
            let code = r.status().as_u16();
            let v: J = r.json().await.unwrap_or(json!({}));
            if is_ok(code) {
                let u = pick(&v, &["url"]);
                Outcome {
                    ok: true,
                    reference: pick(&v, &["id"]),
                    message: "Complete the card payment on Stripe.".into(),
                    redirect: if u.is_empty() { None } else { Some(u) },
                }
            } else {
                fail(format!(
                    "Stripe refused the charge (HTTP {code}): {}",
                    pick(&v, &["error.message", "message", "raw"])
                ))
            }
        }
        Err(e) => fail(format!("Network error: {e}")),
    }
}

async fn verify_stripe(reference: &str) -> Outcome {
    let secret = settings::get("gw_stripe_secret", "");
    if secret.is_empty() {
        return fail("Stripe is not configured.");
    }
    let url = format!("{}/v1/checkout/sessions/{reference}", stripe_api_base());
    match client().get(&url).bearer_auth(&secret).send().await {
        Ok(r) => {
            let v: J = r.json().await.unwrap_or(json!({}));
            let paid = v.get("payment_status").and_then(|s| s.as_str()) == Some("paid");
            Outcome {
                ok: paid,
                reference: reference.to_string(),
                message: if paid {
                    settled_msg()
                } else {
                    "Stripe has not recorded a payment yet.".into()
                },
                redirect: None,
            }
        }
        Err(e) => fail(format!("Network error: {e}")),
    }
}

/* ---------------- PayPal ---------------- */

async fn paypal_token() -> Result<(String, String), String> {
    let id = settings::get("gw_paypal_client", "");
    let secret = settings::get("gw_paypal_secret", "");
    let live = settings::flag("gw_paypal_live");
    let base = if live {
        "https://api-m.paypal.com"
    } else {
        "https://api-m.sandbox.paypal.com"
    };
    let res = client()
        .post(format!("{base}/v1/oauth2/token"))
        .basic_auth(&id, Some(&secret))
        .form(&[("grant_type", "client_credentials")])
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    let v: J = res.json().await.unwrap_or(json!({}));
    let token = pick(&v, &["access_token"]);
    if token.is_empty() {
        return Err("PayPal refused the credentials.".into());
    }
    Ok((token, base.to_string()))
}

async fn initiate_paypal(amount: f64, currency: &str, reference: &str) -> Outcome {
    let (token, base) = match paypal_token().await {
        Ok(t) => t,
        Err(e) => return fail(e),
    };
    let cur = if currency.is_empty() { "USD" } else { currency };
    let body = json!({
        "intent": "CAPTURE",
        "purchase_units": [{
            "reference_id": reference,
            "amount": { "currency_code": cur, "value": format!("{amount:.2}") }
        }],
        "application_context": { "brand_name": "Mbilinyi Tech Solutions", "user_action": "PAY_NOW" }
    });
    match client()
        .post(format!("{base}/v2/checkout/orders"))
        .bearer_auth(&token)
        .json(&body)
        .send()
        .await
    {
        Ok(r) => {
            let code = r.status().as_u16();
            let v: J = r.json().await.unwrap_or(json!({}));
            if is_ok(code) {
                let link = v
                    .get("links")
                    .and_then(|l| l.as_array())
                    .and_then(|a| {
                        a.iter()
                            .find(|x| x.get("rel").and_then(|r| r.as_str()) == Some("approve"))
                    })
                    .and_then(|x| x.get("href"))
                    .and_then(|h| h.as_str())
                    .map(str::to_string);
                Outcome {
                    ok: true,
                    reference: pick(&v, &["id"]),
                    message: "Complete the payment on PayPal.".into(),
                    redirect: link,
                }
            } else {
                fail(format!(
                    "PayPal refused the charge (HTTP {code}): {}",
                    pick(&v, &["message", "raw"])
                ))
            }
        }
        Err(e) => fail(format!("Network error: {e}")),
    }
}

async fn verify_paypal(reference: &str) -> Outcome {
    let (token, base) = match paypal_token().await {
        Ok(t) => t,
        Err(e) => return fail(e),
    };
    let url = format!("{base}/v2/checkout/orders/{reference}");
    match client().get(&url).bearer_auth(&token).send().await {
        Ok(r) => {
            let v: J = r.json().await.unwrap_or(json!({}));
            let status = pick(&v, &["status"]).to_uppercase();
            let ok = status == "COMPLETED" || status == "APPROVED";
            Outcome {
                ok,
                reference: reference.to_string(),
                message: if ok {
                    settled_msg()
                } else {
                    format!("PayPal reports {status}.")
                },
                redirect: None,
            }
        }
        Err(e) => fail(format!("Network error: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_dollar_charge_is_sent_in_cents() {
        assert_eq!(stripe_minor(1000.0, "USD"), 100_000);
        assert_eq!(stripe_minor(49.99, "USD"), 4_999);
        assert_eq!(stripe_minor(1.0, "EUR"), 100);
    }

    #[test]
    fn a_shilling_charge_is_sent_whole_never_times_a_hundred() {
        // Stripe has no minor unit for TZS: 2,600,000 TZS must not become
        // 260,000,000, which would be a hundredfold overcharge.
        assert_eq!(stripe_minor(2_600_000.0, "TZS"), 2_600_000);
        assert_eq!(stripe_minor(50_000.0, "TZS"), 50_000);
        assert_eq!(stripe_minor(1234.6, "TZS"), 1235);
    }

    #[test]
    fn the_currency_case_does_not_change_the_unit() {
        assert_eq!(stripe_minor(500.0, "tzs"), 500);
        assert_eq!(stripe_minor(5.0, "usd"), 500);
    }

    #[test]
    fn other_zero_decimal_currencies_are_not_multiplied() {
        assert_eq!(stripe_minor(1000.0, "JPY"), 1000);
        assert_eq!(stripe_minor(1000.0, "UGX"), 1000);
        assert_eq!(stripe_minor(1000.0, "KRW"), 1000);
    }

    /// Build the header Stripe would send, so the verifier can be exercised
    /// against a genuine signature rather than a guess.
    fn signed_header(secret: &str, body: &str, ts: i64) -> String {
        let signed = format!("{ts}.{body}");
        format!("t={ts},v1={}", stripe_hmac_hex(secret, signed.as_bytes()))
    }

    #[test]
    fn a_genuine_webhook_signature_verifies() {
        let body = r#"{"type":"checkout.session.completed","id":"evt_1"}"#;
        let now = chrono::Utc::now().timestamp();
        let header = signed_header("whsec_test_secret", body, now);
        assert!(verify_stripe_signature("whsec_test_secret", &header, body).is_ok());
    }

    #[test]
    fn a_forged_or_stale_webhook_is_refused() {
        let body = r#"{"id":"evt_2"}"#;
        let now = chrono::Utc::now().timestamp();
        // The right shape but the wrong key.
        let wrong = signed_header("whsec_attacker", body, now);
        assert!(verify_stripe_signature("whsec_test_secret", &wrong, body).is_err());
        // A genuine signature replayed an hour later.
        let old = signed_header("whsec_test_secret", body, now - 3600);
        assert!(verify_stripe_signature("whsec_test_secret", &old, body).is_err());
        // A body that does not match what was signed.
        let genuine = signed_header("whsec_test_secret", body, now);
        assert!(
            verify_stripe_signature("whsec_test_secret", &genuine, r#"{"id":"evt_tampered"}"#)
                .is_err()
        );
    }

    /// The bytes that were signed must be the bytes that are verified.
    ///
    /// This is the regression test for a real bug: the webhook handler used to
    /// hash a re-serialised JSON object, and `serde_json` writes object keys in
    /// sorted order. Stripe sends keys in its own order, so a genuine callback
    /// was rejected every time. The raw body must be kept and used verbatim.
    #[test]
    fn a_signature_must_be_checked_against_the_original_bytes() {
        let secret = "whsec_round_trip";
        // Key order here is deliberate: "type" before "id" is not sorted order.
        let raw = r#"{"type":"checkout.session.completed","id":"evt_1","data":{"amount":500}}"#;
        let header = signed_header(secret, raw, chrono::Utc::now().timestamp());
        assert!(
            verify_stripe_signature(secret, &header, raw).is_ok(),
            "a signature over the original bytes must verify"
        );

        // The same payload after a JSON round-trip is reordered, so it must not
        // verify against the original signature.
        let parsed: serde_json::Value = serde_json::from_str(raw).unwrap();
        let reserialised = parsed.to_string();
        assert_ne!(
            raw, reserialised,
            "the round-trip should have reordered keys"
        );
        assert!(
            verify_stripe_signature(secret, &header, &reserialised).is_err(),
            "a re-serialised body must not verify — that was the bug"
        );
    }

    #[test]
    fn no_secret_means_no_trust() {
        let now = chrono::Utc::now().timestamp();
        let header = signed_header("anything", "body", now);
        assert!(verify_stripe_signature("", &header, "body").is_err());
    }

    #[test]
    fn the_comparison_never_leaks_a_prefix() {
        assert!(constant_time_eq(b"abcdef", b"abcdef"));
        assert!(!constant_time_eq(b"abcdef", b"abcdeX"));
        assert!(!constant_time_eq(b"abc", b"abcdef"));
    }
}
