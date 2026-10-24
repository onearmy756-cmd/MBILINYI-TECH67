//! The HTML pages.
//!
//! The pages are thin shells: `assets/app.js` fetches everything it needs from
//! `/api/*` and fills the page in the browser. So a page here only has to emit
//! the same markup and the same globals (`window.__CSRF__`, `window.__ME__`,
//! `window.__PAGE__`, `window.__SERVICES_ON__`, `window.__PLATFORM__`) that the
//! PHP pages did, and the frontend keeps working untouched.
//!
//! The markup itself lives in `templates/*.html`, copied verbatim from the PHP
//! pages with the handful of PHP expressions replaced by the placeholders this
//! module substitutes.

use crate::db;
use crate::settings;
use crate::util::{self, h};
use axum::body::Body;
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};

/// The site base URL, for canonical links and social cards.
fn site_base() -> String {
    let s = std::env::var("SITE_URL").unwrap_or_else(|_| "https://mbilinyitech.co.tz".to_string());
    s.trim_end_matches('/').to_string()
}

fn page_description() -> String {
    "Mbilinyi Tech Solutions — BRELA registered (No. 706668), TIN 192-147-522. Custom software, system analysis, IT consultancy, maintenance, cybersecurity & AI fine-tuning. CEO Jackson Mbilinyi, Dodoma, Tanzania.".to_string()
}

fn title_for(name: &str) -> String {
    match name {
        "index" => {
            "Mbilinyi Tech Solutions — Custom Software, AI & Security | Jackson Mbilinyi, CEO"
                .into()
        }
        "services" => "Services — Custom Software, AI, Security | Mbilinyi Tech Solutions".into(),
        "stack" => "Tech Stack — What We Build With | Mbilinyi Tech Solutions".into(),
        "ai" => "AI & Fine-Tuning — Build Your Own AI | Mbilinyi Tech Solutions".into(),
        "pricing" => {
            "Pricing — Transparent, Admin-Published Rates | Mbilinyi Tech Solutions".into()
        }
        "process" => "How It Works — From Request to Delivery | Mbilinyi Tech Solutions".into(),
        "work" => "Our Work — Portfolio | Mbilinyi Tech Solutions".into(),
        "track" => "Track Your Request | Mbilinyi Tech Solutions".into(),
        "contact" => {
            "Contact — Let's Build Something Extraordinary | Mbilinyi Tech Solutions".into()
        }
        "auth" => "Mbilinyi Tech Solutions — Client Portal | Sign In".into(),
        "client" => "Mbilinyi Tech Solutions — Client Portal".into(),
        "admin" => "Mbilinyi Tech Solutions — Admin Control Center".into(),
        "news" => "News & Updates | Mbilinyi Tech Solutions".into(),
        "team" => "Our Team | Mbilinyi Tech Solutions".into(),
        "deploy" => "Deployment Health Check | Mbilinyi Tech Solutions".into(),
        "setup" => "Setup | Mbilinyi Tech Solutions".into(),
        "terms" => "Terms of Service | Mbilinyi Tech Solutions".into(),
        "privacy" => "Privacy Policy | Mbilinyi Tech Solutions".into(),
        "refund" => "Refund Policy | Mbilinyi Tech Solutions".into(),
        "feedback" => "Feedback | Mbilinyi Tech Solutions".into(),
        "reports" => "Reports — Proof of Work | Mbilinyi Tech Solutions".into(),
        "community" => "Community — Builders, Learners & Partners | Mbilinyi Tech Solutions".into(),
        "faq" => "Frequently Asked Questions | Mbilinyi Tech Solutions".into(),
        "careers" => "Careers (Ajira) — Apply & Be Assessed | Mbilinyi Tech Solutions".into(),
        "staff" => "Our Experts — The People Who Serve You | Mbilinyi Tech Solutions".into(),
        "mtaalamu" => "Staff Portal — Your Tasks & Tools | Mbilinyi Tech Solutions".into(),
        _ => "Mbilinyi Tech Solutions".into(),
    }
}

/// Resolve which country the visitor is in, so services that only make sense in
/// Tanzania (BRELA, TRA, NIDA…) can be hidden from everyone else.
///
/// Order matters: an explicit choice by the visitor wins, because a Tanzanian
/// working abroad still needs BRELA, and a visitor on a VPN should be able to
/// correct a wrong guess. Only then do we fall back to a geo header or the
/// browser's language, and finally to "international" — never the reverse,
/// so we never show a Tanzania-only service by accident.
fn visitor_country(headers: &axum::http::HeaderMap) -> String {
    let cookie = headers
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    for part in cookie.split(';') {
        let part = part.trim();
        if let Some(v) = part.strip_prefix("mts_country=") {
            let v = v.trim().to_uppercase();
            if !v.is_empty() && v.len() <= 3 && v.chars().all(|c| c.is_ascii_alphabetic()) {
                return v;
            }
        }
    }
    for h in ["cf-ipcountry", "x-country-code", "x-vercel-ip-country"] {
        if let Some(v) = headers.get(h).and_then(|v| v.to_str().ok()) {
            let v = v.trim().to_uppercase();
            if v.len() == 2 && v != "XX" && v.chars().all(|c| c.is_ascii_alphabetic()) {
                return v;
            }
        }
    }
    // Kiswahili in Accept-Language is the strongest hint a plain request carries.
    let lang = headers
        .get(header::ACCEPT_LANGUAGE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_lowercase();
    if lang.split(',').any(|l| l.trim().starts_with("sw")) {
        return "TZ".to_string();
    }
    "INTL".to_string()
}

pub fn country_from_headers(headers: &axum::http::HeaderMap) -> String {
    visitor_country(headers)
}

/// Services the CEO has flagged as Tanzania-only, as a comma-separated list in
/// settings. Empty means none are restricted.
fn tz_only_services() -> serde_json::Value {
    let raw = settings::get("services_tz_only", "brela,tra_tax,gov_eservices");
    let keys: Vec<serde_json::Value> = raw
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| serde_json::Value::String(s.to_string()))
        .collect();
    serde_json::Value::Array(keys)
}

/// The `window.*` globals the frontend reads. Kept identical to the PHP
/// `partials/footer.php` block.
fn globals(page: &str, csrf: &str, me: &Option<db::Row>, country: &str) -> String {
    let me_json = match me {
        Some(u) => crate::api::map::user(u).to_string(),
        None => "null".to_string(),
    };
    let platform = serde_json::json!({
        "url": settings::get("platform_url", ""),
        "name": settings::get("platform_name", "MbilinyiTech Platform"),
        "logo": settings::get("platform_logo", "/assets/icons/icon-512.png"),
        "tagline": settings::get("platform_tagline", "No-one enters a section that is not theirs."),
    });
    format!(
        "<script>\nwindow.__CSRF__ = {csrf};\nwindow.__ME__   = {me};\nwindow.__PAGE__ = {page};\nwindow.__SERVICES_ON__ = {svc};\nwindow.__TZ_ONLY__ = {tzonly};\nwindow.__COUNTRY__ = {country};\nwindow.__PLATFORM__ = {plat};\nwindow.__PAY_GATEWAYS__ = {gw};\nwindow.__OAUTH__ = {oauth};\n</script>",
        csrf = serde_json::Value::String(csrf.to_string()),
        me = me_json,
        page = serde_json::Value::String(page.to_string()),
        svc = settings::service_visibility(),
        tzonly = tz_only_services(),
        country = serde_json::Value::String(country.to_string()),
        plat = platform,
        gw = serde_json::Value::Array(crate::payments::available()),
        oauth = serde_json::Value::Array(crate::api::oauth::available()),
    )
}

/// The `<head>`, matching `partials/head.php` link for link.
fn head(name: &str, title: &str) -> String {
    let base = site_base();
    let desc = page_description();
    let canonical = format!("{base}/{}", if name == "index" { "" } else { name });
    let og = format!("{base}/assets/icons/og-image.png");
    let ld = serde_json::json!({
        "@context": "https://schema.org",
        "@type": "ProfessionalService",
        "name": "Mbilinyi Tech Solutions",
        "legalName": "Mbilinyi Tech Solutions",
        "identifier": "BRELA Reg. No. 706668",
        "taxID": "192-147-522",
        "url": format!("{base}/"),
        "logo": format!("{base}/assets/icons/icon-512.png"),
        "image": og,
        "description": "BRELA-registered software house in Dodoma, Tanzania — custom software, system analysis, IT consultancy, maintenance, cybersecurity and AI fine-tuning.",
        "founder": { "@type": "Person", "name": "Jackson Mbilinyi", "jobTitle": "CEO" },
        "email": "mbilinyitech@gmail.com",
        "telephone": "+255796752645",
        "address": { "@type": "PostalAddress", "addressLocality": "Dodoma", "addressCountry": "TZ" },
        "areaServed": { "@type": "Country", "name": "Tanzania" },
        "sameAs": ["https://instagram.com/mbilinyitech"],
    });
    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{title}</title>
<meta name="description" content="{desc}">
<link rel="canonical" href="{canonical}">
<meta name="theme-color" content="#050914">
<link rel="icon" type="image/png" sizes="32x32" href="assets/icons/favicon-32.png">
<link rel="icon" type="image/png" sizes="16x16" href="assets/icons/favicon-16.png">
<link rel="shortcut icon" href="assets/icons/favicon.ico">
<link rel="apple-touch-icon" sizes="180x180" href="assets/icons/apple-touch-icon.png">
<link rel="manifest" href="manifest.json">
<meta property="og:type" content="website">
<meta property="og:site_name" content="Mbilinyi Tech Solutions">
<meta property="og:title" content="{title}">
<meta property="og:description" content="{desc}">
<meta property="og:url" content="{canonical}">
<meta property="og:image" content="{og}">
<meta property="og:image:width" content="1200">
<meta property="og:image:height" content="630">
<meta property="og:locale" content="en_TZ">
<meta property="og:locale:alternate" content="sw_TZ">
<meta name="twitter:card" content="summary_large_image">
<meta name="twitter:title" content="{title}">
<meta name="twitter:description" content="{desc}">
<meta name="twitter:image" content="{og}">
<script type="application/ld+json">
{ld}
</script>
<meta name="apple-mobile-web-app-capable" content="yes">
<meta name="apple-mobile-web-app-status-bar-style" content="black-translucent">
<meta name="apple-mobile-web-app-title" content="Mbilinyi Tech">
<script src="https://cdn.tailwindcss.com"></script>
<script>
tailwind.config = {{
  theme: {{
    extend: {{
      colors: {{
        ink: '#050914', ink2: '#0A1226', card: '#0E1933',
        line: 'rgba(255,255,255,0.08)', cyanx: '#22d3ee', bluex: '#3b82f6',
        violetx: '#8b5cf6', gold: '#F5B301',
      }},
      fontFamily: {{
        display: ['Space Grotesk','sans-serif'],
        body: ['Inter','sans-serif'],
        mono: ['JetBrains Mono','monospace'],
      }}
    }}
  }}
}}
</script>
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=Space+Grotesk:wght@500;600;700&family=JetBrains+Mono:wght@400;600&display=swap" rel="stylesheet">
<link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.2/css/all.min.css">
<script src="https://cdn.jsdelivr.net/npm/chart.js@4.4.1/dist/chart.umd.min.js"></script>
<script src="https://cdnjs.cloudflare.com/ajax/libs/gsap/3.12.5/gsap.min.js"></script>
<script src="https://cdnjs.cloudflare.com/ajax/libs/gsap/3.12.5/ScrollTrigger.min.js"></script>
{style}
</head>"##,
        title = h(title),
        desc = h(&desc),
        canonical = h(&canonical),
        og = h(&og),
        ld = ld,
        style = STYLE,
    )
}

/// The stylesheet, verbatim from `partials/head.php` so the look is identical.
const STYLE: &str = r#"<style>
  *{scroll-behavior:smooth}
  body{background:#050914;color:#E6ECF5;font-family:Inter,sans-serif;overflow-x:hidden}
  ::-webkit-scrollbar{width:10px;height:10px}
  ::-webkit-scrollbar-track{background:#070D1F}
  ::-webkit-scrollbar-thumb{background:linear-gradient(#22d3ee,#8b5cf6);border-radius:99px}
  .glass{background:rgba(255,255,255,.04);backdrop-filter:blur(18px);-webkit-backdrop-filter:blur(18px);border:1px solid rgba(255,255,255,.08)}
  .glass-strong{background:rgba(14,25,51,.85);backdrop-filter:blur(22px);border:1px solid rgba(255,255,255,.1)}
  .grad-text{background:linear-gradient(90deg,#22d3ee,#60a5fa,#a78bfa,#f0abfc);-webkit-background-clip:text;background-clip:text;color:transparent}
  .grad-btn{background:linear-gradient(135deg,#06b6d4,#3b82f6 50%,#8b5cf6);box-shadow:0 10px 30px -8px rgba(59,130,246,.6)}
  .grad-btn:hover{transform:translateY(-2px);box-shadow:0 18px 40px -8px rgba(59,130,246,.7)}
  .gold-btn{background:linear-gradient(135deg,#F5B301,#ff7847);box-shadow:0 10px 30px -8px rgba(245,179,1,.5)}
  /* 3D cards: a raised surface that lifts and tilts on hover. The tilt is a
     transform only, so it costs no layout and stays smooth on a phone. */
  .card3d{background:linear-gradient(160deg,rgba(255,255,255,.075),rgba(255,255,255,.02));border:1px solid rgba(255,255,255,.12);border-radius:20px;transform-style:preserve-3d;transition:transform .35s cubic-bezier(.2,.8,.2,1),box-shadow .35s ease,border-color .35s ease;box-shadow:0 18px 40px -22px rgba(0,0,0,.9),inset 0 1px 0 rgba(255,255,255,.09)}
  .card3d:hover{transform:translateY(-6px) rotateX(4deg) rotateY(-4deg) scale(1.015);box-shadow:0 34px 60px -26px rgba(0,0,0,.95),0 0 0 1px rgba(34,211,238,.28),inset 0 1px 0 rgba(255,255,255,.16);border-color:rgba(34,211,238,.35)}
  .card3d.pop-cyan{background-image:linear-gradient(160deg,rgba(34,211,238,.16),rgba(255,255,255,.02))}
  .card3d.pop-violet{background-image:linear-gradient(160deg,rgba(139,92,246,.18),rgba(255,255,255,.02))}
  .card3d.pop-emerald{background-image:linear-gradient(160deg,rgba(16,185,129,.16),rgba(255,255,255,.02))}
  .card3d.pop-amber{background-image:linear-gradient(160deg,rgba(245,179,1,.16),rgba(255,255,255,.02))}
  .card3d.pop-rose{background-image:linear-gradient(160deg,rgba(244,63,94,.16),rgba(255,255,255,.02))}
  .chip{display:inline-flex;align-items:center;gap:5px;padding:2px 9px;border-radius:999px;font-size:10.5px;font-weight:700;letter-spacing:.03em;border:1px solid rgba(255,255,255,.14);background:rgba(255,255,255,.05)}
  .progress-rail{height:9px;border-radius:999px;background:rgba(255,255,255,.08);overflow:hidden}
  .progress-fill{height:100%;border-radius:999px;background:linear-gradient(90deg,#06b6d4,#3b82f6,#8b5cf6);transition:width .6s cubic-bezier(.2,.8,.2,1)}
  /* The agent company grid: a dense wall of seats that reads at a glance. */
  .agent-seat{display:flex;align-items:center;gap:9px;padding:9px 11px;border-radius:14px;background:rgba(255,255,255,.035);border:1px solid rgba(255,255,255,.08);transition:border-color .25s,transform .25s,background .25s}
  .agent-seat:hover{border-color:rgba(232,121,249,.45);transform:translateY(-2px);background:rgba(232,121,249,.07)}
  /* Live calls: a video wall that fills the dialog and stays readable. */
  .call-root{display:flex;flex-direction:column;gap:6px}
  .call-grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;max-height:56vh;overflow:auto}
  .call-tile{position:relative;aspect-ratio:16/10;border-radius:14px;overflow:hidden;background:#050914;border:1px solid rgba(255,255,255,.12)}
  .call-tile video{width:100%;height:100%;object-fit:cover;display:block}
  .call-name{position:absolute;left:8px;bottom:8px;padding:2px 9px;border-radius:999px;font-size:10.5px;font-weight:700;background:rgba(5,9,20,.72);border:1px solid rgba(255,255,255,.16)}
  .call-btn{width:38px;height:38px;border-radius:12px;display:inline-flex;align-items:center;justify-content:center;background:rgba(255,255,255,.06);border:1px solid rgba(255,255,255,.14);transition:background .2s,border-color .2s}
  .call-btn:hover{background:rgba(255,255,255,.12);border-color:rgba(34,211,238,.5)}
  .call-btn.off{background:rgba(244,63,94,.18);border-color:rgba(244,63,94,.5);color:#fda4af}
  .call-btn-end{background:rgba(244,63,94,.2);border-color:rgba(244,63,94,.45);color:#fda4af}
  .grid-bg{background-image:linear-gradient(rgba(255,255,255,.04) 1px,transparent 1px),linear-gradient(90deg,rgba(255,255,255,.04) 1px,transparent 1px);background-size:44px 44px;mask-image:radial-gradient(ellipse 90% 70% at 50% 0%,black 60%,transparent 100%)}
  .orb{position:absolute;border-radius:50%;filter:blur(90px);opacity:.5;pointer-events:none}
  .marquee-track{display:flex;gap:14px;width:max-content;animation:marquee 28s linear infinite}
  @keyframes marquee{to{transform:translateX(-50%)}}
  @keyframes float{0%,100%{transform:translateY(0)}50%{transform:translateY(-14px)}}
  .float{animation:float 5s ease-in-out infinite}
  .float2{animation:float 7s ease-in-out infinite}
  .code-dots span{width:11px;height:11px;border-radius:50%;display:inline-block}
  .typing span{width:7px;height:7px;background:#22d3ee;border-radius:50%;display:inline-block;animation:blink 1s infinite}
  .typing span:nth-child(2){animation-delay:.2s}.typing span:nth-child(3){animation-delay:.4s}
  @keyframes blink{0%,100%{opacity:.2;transform:translateY(0)}50%{opacity:1;transform:translateY(-4px)}}
  .timeline-line{background:linear-gradient(180deg,#22d3ee,#8b5cf6,#22ff88)}
  .step-dot{box-shadow:0 0 0 6px rgba(34,211,238,.15)}
  .tab-active{background:linear-gradient(135deg,#06b6d4,#8b5cf6)!important;color:white!important;border-color:transparent!important}
  .sidebar-link.active{background:linear-gradient(135deg,rgba(6,182,212,.2),rgba(139,92,246,.2));border-left:3px solid #22d3ee;color:white}
  .input{background:rgba(255,255,255,.05);border:1px solid rgba(255,255,255,.1);border-radius:12px;padding:12px 14px;width:100%;color:white;outline:none;transition:.2s}
  .input:focus{border-color:#22d3ee;box-shadow:0 0 0 3px rgba(34,211,238,.15);background:rgba(255,255,255,.07)}
  .input::placeholder{color:#7C8AA5}
  select.input option{background:#0E1933}
  .input-i{padding-left:44px}
  .ifield{position:relative}
  .ifield>i{position:absolute;left:15px;top:50%;transform:translateY(-50%);color:#7C8AA5;font-size:13px;pointer-events:none}
  .ifield:focus-within>i{color:#22d3ee}
  .pw-meter{height:4px;border-radius:99px;background:rgba(255,255,255,.08);overflow:hidden}
  .pw-meter>span{display:block;height:100%;width:0;transition:.3s;border-radius:99px}
  .badge{font-size:10px;letter-spacing:.12em;text-transform:uppercase;font-weight:700;padding:5px 10px;border-radius:99px}
  .status-Pending{background:rgba(245,179,1,.15);color:#F5B301;border:1px solid rgba(245,179,1,.3)}
  .status-UnderReview{background:rgba(59,130,246,.15);color:#60a5fa;border:1px solid rgba(59,130,246,.3)}
  .status-Quoted{background:rgba(139,92,246,.15);color:#a78bfa;border:1px solid rgba(139,92,246,.35)}
  .status-Approved{background:rgba(34,211,238,.15);color:#22d3ee;border:1px solid rgba(34,211,238,.35)}
  .status-InProgress{background:rgba(34,197,94,.15);color:#4ade80;border:1px solid rgba(34,197,94,.3)}
  .status-Completed{background:rgba(16,185,129,.2);color:#6ee7b7;border:1px solid rgba(16,185,129,.4)}
  .status-Rejected{background:rgba(239,68,68,.15);color:#fca5a5;border:1px solid rgba(239,68,68,.35)}
  .chat-bubble-ai{background:rgba(255,255,255,.06);border:1px solid rgba(255,255,255,.08);border-radius:4px 16px 16px 16px}
  .chat-bubble-me{background:linear-gradient(135deg,#0891b2,#6366f1);border-radius:16px 4px 16px 16px}
  .sig-pad{cursor:crosshair;touch-action:none}
  table.data th{font-size:11px;text-transform:uppercase;letter-spacing:.1em;color:#8EA0BF;text-align:left;padding:12px;font-weight:700;background:rgba(255,255,255,.03)}
  table.data td{padding:12px;border-top:1px solid rgba(255,255,255,.06);font-size:13.5px;vertical-align:middle}
  table.data tr:hover td{background:rgba(255,255,255,.02)}
  .wizard-step{display:none}.wizard-step.active{display:block;animation:fadeUp .4s ease}
  @keyframes fadeUp{from{opacity:0;transform:translateY(14px)}to{opacity:1;transform:translateY(0)}}
  .view-section{display:none}.view-section.active{display:block;animation:fadeUp .4s ease}
  .tech-card:hover{transform:translateY(-6px) scale(1.02)}
  .service-card{transition:.35s}.service-card:hover{transform:translateY(-8px);border-color:rgba(34,211,238,.4);box-shadow:0 24px 60px -18px rgba(34,211,238,.35)}
  #chatWindow{transition:.35s cubic-bezier(.2,.9,.3,1.2);transform-origin:bottom right}
  #chatWindow.hidden-chat{transform:scale(.6) translateY(20px);opacity:0;pointer-events:none}
  .toast-in{animation:toastIn .35s cubic-bezier(.2,.9,.3,1.2)}
  @keyframes toastIn{from{transform:translateX(100%);opacity:0}to{transform:translateX(0);opacity:1}}
  .print-only{display:none}
  @media print{
    body *{visibility:hidden}
    #printArea, #printArea *{visibility:visible}
    #printArea{position:absolute;left:0;top:0;width:100%;background:white;color:black;padding:24px}
    .print-only{display:block}
    .no-print{display:none!important}
  }
  .orbit-ring{border:1px dashed rgba(34,211,238,.25);border-radius:50%;position:absolute;animation:spin 24s linear infinite}
  @keyframes spin{to{transform:rotate(360deg)}}
  .shimmer{background:linear-gradient(110deg,transparent 30%,rgba(255,255,255,.12) 50%,transparent 70%);background-size:200% 100%;animation:shimmer 2.2s infinite}
  @keyframes shimmer{to{background-position:-200% 0}}
</style>"#;

/// The footer, matching `partials/footer.php`: the legal links, the WhatsApp
/// button, the chat widget, the wizard modal and the script tag.
fn footer(globals_html: &str, country: &str) -> String {
    format!(
        r##"</main>
<footer class="border-t border-white/10 bg-[#04070F] mt-6">
  <div class="max-w-7xl mx-auto px-4 py-12 grid md:grid-cols-4 gap-8 text-[13.5px]">
    <div><div class="flex items-center gap-3"><div class="w-10 h-10 rounded-xl grad-btn flex items-center justify-center font-bold">M</div><div class="font-display font-bold">MBILINYI TECH SOLUTIONS</div></div><p class="text-white/50 mt-3 leading-relaxed" data-en="BRELA-registered software house delivering custom systems, AI, maintenance & security — locally rooted, globally competitive." data-sw="Kampuni ya software iliyosajiliwa BRELA inayotoa mifumo maalum, AI, maintenance na usalama — yenye mizizi ya nyumbani, inashindana kimataifa.">BRELA-registered software house delivering custom systems, AI, maintenance &amp; security — locally rooted, globally competitive.</p><div class="flex gap-2 mt-4"><a href="https://instagram.com/mbilinyitech" target="_blank" class="w-9 h-9 rounded-full bg-white/5 border border-white/10 flex items-center justify-center hover:border-pink-400/50"><i class="fa-brands fa-instagram text-pink-300"></i></a><a href="https://wa.me/255796752645" target="_blank" class="w-9 h-9 rounded-full bg-white/5 border border-white/10 flex items-center justify-center hover:border-green-400/50"><i class="fa-brands fa-whatsapp text-green-300"></i></a><a href="mailto:mbilinyitech@gmail.com" class="w-9 h-9 rounded-full bg-white/5 border border-white/10 flex items-center justify-center hover:border-blue-400/50"><i class="fa-solid fa-envelope text-blue-300"></i></a><a href="tel:+255796752645" class="w-9 h-9 rounded-full bg-white/5 border border-white/10 flex items-center justify-center hover:border-cyan-400/50"><i class="fa-solid fa-phone text-cyan-300"></i></a></div></div>
    <div><div class="font-bold mb-3" data-en="Services" data-sw="Huduma">Services</div><div class="grid gap-2 text-white/55">
      <span data-en="Custom Software (Mobile • Desktop • Web)" data-sw="Software Maalum (Simu • Kompyuta • Wavuti)">Custom Software (Mobile • Desktop • Web)</span>
      <span data-en="System Analysis & Design" data-sw="Uchambuzi na Ubunifu wa Mifumo">System Analysis &amp; Design</span>
      <span data-en="IT Consultancy" data-sw="Ushauri wa IT">IT Consultancy</span>
      <span data-en="Software Maintenance & SLA" data-sw="Matengenezo ya Software na SLA">Software Maintenance &amp; SLA</span>
      {cyber}
      {forensics}
      <span data-en="Build Your Own AI & Chatbot" data-sw="Jenga AI na Chatbot Yako">Build Your Own AI &amp; Chatbot</span>
    </div></div>
    <div><div class="font-bold mb-3" data-en="Company" data-sw="Kampuni">Company</div><div class="grid gap-2 text-white/55"><span>CEO: Jackson Mbilinyi</span><span>BRELA Reg: 706668</span><span class="font-mono">TIN: 192-147-522</span><span data-en="Dodoma, Tanzania • Work delivered online" data-sw="Dodoma, Tanzania • Kazi zote mtandaoni">Dodoma, Tanzania • Work delivered online</span><a href="auth" class="text-left text-cyan-300 font-bold" data-en="Client Login →" data-sw="Kuingia kwa Mteja →">Client Login →</a></div></div>
    <div><div class="font-bold mb-3" data-en="Contact" data-sw="Mawasiliano">Contact</div><div class="grid gap-2 text-white/55"><a href="tel:+255796752645" class="hover:text-white"><i class="fa-solid fa-phone mr-2 text-green-300"></i>0796 752 645</a><a href="mailto:mbilinyitech@gmail.com" class="hover:text-white"><i class="fa-solid fa-envelope mr-2 text-blue-300"></i>mbilinyitech@gmail.com</a><span><i class="fa-brands fa-instagram mr-2 text-pink-300"></i>@mbilinyitech</span><span data-en="Mon–Sat • 8:00–20:00 EAT" data-sw="Jumatatu–Jumamosi • 8:00–20:00 EAT">Mon–Sat • 8:00–20:00 EAT</span></div></div>
  </div>
  <div class="border-t border-white/10 py-4 text-center">
    <div class="text-[12px] font-semibold tracking-[.14em] text-cyan-300/70 mb-1.5" data-en="★ YOUR PROBLEM, OUR SOLUTION ★" data-sw="★ SHIDA YAKO, TATUZO LETU ★">★ YOUR PROBLEM, OUR SOLUTION ★</div>
    <div class="flex flex-wrap items-center justify-center gap-x-4 gap-y-1.5 text-[12px] text-white/55 mb-2">
      <a href="terms" class="hover:text-cyan-300 transition" data-en="Terms of Service" data-sw="Masharti ya Huduma">Terms of Service</a>
      <span class="text-white/20">•</span>
      <a href="refund" class="hover:text-cyan-300 transition" data-en="Refund Policy" data-sw="Sera ya Kurudisha Pesa">Refund Policy</a>
      <span class="text-white/20">•</span>
      <a href="privacy" class="hover:text-cyan-300 transition" data-en="Privacy Policy" data-sw="Sera ya Faragha">Privacy Policy</a>
      <span class="text-white/20">•</span>
      <a href="feedback" class="hover:text-cyan-300 transition" data-en="Feedback" data-sw="Maoni">Feedback</a>
      <span class="text-white/20">•</span>
      <a href="reports" class="hover:text-cyan-300 transition" data-en="Reports" data-sw="Ripoti">Reports</a>
      <span class="text-white/20">•</span>
      <a href="faq" class="hover:text-cyan-300 transition" data-en="FAQ" data-sw="Maswali">FAQ</a>
      <span class="text-white/20">•</span>
      <a href="community" class="hover:text-cyan-300 transition" data-en="Community" data-sw="Jamii">Community</a>
    </div>
    <div class="text-[12px] text-white/35">&copy; {year} <span data-en="Mbilinyi Tech Solutions • CEO Jackson Mbilinyi • Built to production standards • BRELA &amp; TIN Verified" data-sw="Mbilinyi Tech Solutions • CEO Jackson Mbilinyi • Imejengwa kwa viwango vya uzalishaji • Imethibitishwa BRELA na TIN">Mbilinyi Tech Solutions • CEO Jackson Mbilinyi • Built to production standards • BRELA &amp; TIN Verified</span></div>
    <div class="mt-3 flex items-center justify-center gap-2 text-[11.5px] text-white/45">
      <i class="fa-solid fa-earth-africa text-cyan-300"></i>
      <span data-en="You are browsing as" data-sw="Unatazama kama">You are browsing as</span>
      <button onclick="setVisitorCountry('TZ')" class="{tzcls}">Tanzania</button>
      <span class="text-white/20">|</span>
      <button onclick="setVisitorCountry('INTL')" class="{intlcls}">International</button>
    </div>
  </div>
</footer>
<a href="https://wa.me/255796752645?text=Hello%20Mbilinyi%20Tech%20Solutions!%20I%20need%20a%20service." target="_blank" class="fixed bottom-24 right-4 z-[200] w-14 h-14 rounded-full bg-[#25D366] flex items-center justify-center shadow-2xl hover:scale-110 transition" title="Chat on WhatsApp"><i class="fa-brands fa-whatsapp text-2xl text-white"></i><span class="absolute -top-1 -right-1 w-4 h-4 bg-red-500 rounded-full border-2 border-[#050914] animate-pulse"></span></a>
<button onclick="toggleChat()" class="fixed bottom-6 right-4 z-[300] grad-btn w-14 h-14 rounded-full flex items-center justify-center text-xl shadow-2xl hover:scale-110 transition"><i class="fa-solid fa-robot"></i></button>
<div id="chatWindow" class="hidden-chat fixed bottom-[86px] right-4 z-[300] w-[380px] max-w-[94vw] glass-strong rounded-3xl overflow-hidden shadow-2xl flex flex-col" style="height:560px;max-height:72vh">
  <div class="p-4 bg-gradient-to-r from-cyan-600 via-blue-600 to-violet-600 flex items-center gap-3">
    <div class="w-11 h-11 rounded-2xl bg-white/15 flex items-center justify-center text-xl backdrop-blur"><i class="fa-solid fa-robot"></i></div>
    <div class="flex-1"><div class="font-bold text-[15px]"><span data-en="Mbilinyi AI Assistant" data-sw="Msaidizi wa AI wa Mbilinyi">Mbilinyi AI Assistant</span></div><div class="text-[11.5px] text-white/80 flex items-center gap-1.5"><span class="w-2 h-2 rounded-full bg-emerald-300 animate-pulse"></span> <span data-en="Online • speaks & listens" data-sw="Ipo hewani • husema na husikiliza">Online • speaks &amp; listens</span></div></div>
    <button id="voiceBtn" onclick="toggleVoice()" class="w-8 h-8 rounded-full bg-white/15" title="Speak replies / Sema majibu"><i class="fa-solid fa-volume-high text-sm"></i></button>
    <button onclick="toggleChat()" class="w-8 h-8 rounded-full bg-white/15"><i class="fa-solid fa-xmark text-sm"></i></button>
  </div>
  <div id="chatBody" class="flex-1 overflow-y-auto p-4 space-y-3 text-[13.5px]"></div>
  <div class="p-3 border-t border-white/10">
    <div class="flex flex-wrap gap-1.5 mb-2.5" id="chatQuick"></div>
    <div class="flex gap-2"><input id="chatInput" onkeydown="if(event.key==='Enter')sendChat()" class="input !py-2.5 text-[13px]" data-ph-en="Ask about services, prices, tracking..." data-ph-sw="Uliza kuhusu huduma, bei, ufuatiliaji..." placeholder="Ask about services, prices, tracking..."><button id="micBtn" onclick="toggleMic()" class="w-11 h-11 rounded-xl shrink-0 bg-white/5 border border-white/15" title="Speak your question / Ongea swali lako"><i class="fa-solid fa-microphone text-sm"></i></button><button onclick="sendChat()" class="grad-btn w-11 h-11 rounded-xl shrink-0"><i class="fa-solid fa-paper-plane text-sm"></i></button></div>
  </div>
</div>
<div id="genModal" class="fixed inset-0 z-[450] hidden items-center justify-center p-4 bg-black/70 backdrop-blur-sm overflow-y-auto">
  <div class="glass-strong rounded-3xl w-full max-w-3xl my-6 overflow-hidden">
    <div class="p-5 border-b border-white/10 flex items-center justify-between bg-white/[.02]"><div id="genTitle" class="font-display font-bold text-lg">Details</div><button onclick="closeGen()" class="w-9 h-9 rounded-full bg-white/10 hover:bg-white/20"><i class="fa-solid fa-xmark"></i></button></div>
    <div id="genBody" class="p-5 md:p-6 max-h-[75vh] overflow-y-auto"></div>
  </div>
</div>
<div id="printArea"></div>
{globals}
<script src="assets/app.js"></script>
<script>
(function () {{
  if ('serviceWorker' in navigator) {{
    window.addEventListener('load', function () {{
      navigator.serviceWorker.register('sw.js').catch(function () {{ }});
    }});
  }}
}})();
</script>
</body>
</html>"##,
        year = chrono::Local::now().format("%Y"),
        cyber = if settings::service_visible("cybersecurity") {
            r#"<span data-en="Cybersecurity & Audits" data-sw="Usalama wa Mtandao na Ukaguzi">Cybersecurity &amp; Audits</span>"#
        } else {
            ""
        },
        forensics = if settings::service_visible("forensics") {
            r#"<span data-en="Digital Forensics" data-sw="Uchunguzi wa Kidijitali">Digital Forensics</span>"#
        } else {
            ""
        },
        globals = globals_html,
        tzcls = if country == "TZ" {
            "font-bold text-cyan-300"
        } else {
            "hover:text-cyan-300"
        },
        intlcls = if country == "TZ" {
            "hover:text-cyan-300"
        } else {
            "font-bold text-cyan-300"
        },
    )
}

/// Where the templates live. `MTS_TEMPLATES` wins, then the project root, then
/// the crate directory, so the binary works however it is launched.
fn template_dir() -> String {
    if let Ok(d) = std::env::var("MTS_TEMPLATES") {
        if std::path::Path::new(&d).is_dir() {
            return d;
        }
    }
    for candidate in ["templates", "rust/mts-core/templates"] {
        if std::path::Path::new(candidate).is_dir() {
            return candidate.to_string();
        }
    }
    "templates".to_string()
}

/// Serve one page.
pub async fn render(name: &str, headers: &axum::http::HeaderMap) -> Response {
    let known = [
        "index",
        "services",
        "stack",
        "ai",
        "pricing",
        "process",
        "work",
        "track",
        "contact",
        "auth",
        "client",
        "admin",
        "news",
        "team",
        "deploy",
        "setup",
        "terms",
        "privacy",
        "refund",
        "feedback",
        "reports",
        "community",
        "faq",
        "careers",
        "staff",
        "mtaalamu",
    ];
    if !known.contains(&name) {
        return (StatusCode::NOT_FOUND, "404 Not Found").into_response();
    }

    // `client` and `admin` send a signed-out visitor to the sign-in page rather
    // than showing an empty portal.
    let session = crate::session::load(headers);
    let me = crate::session::current_user(&session);
    if matches!(name, "client" | "admin" | "mtaalamu") && me.is_none() {
        return (
            StatusCode::FOUND,
            [(header::LOCATION, format!("/auth?returnTo={name}"))],
            "",
        )
            .into_response();
    }
    if name == "auth" {
        if let Some(u) = &me {
            let to = if u.get("role").and_then(|v| v.as_str()) == Some("admin") {
                "/admin"
            } else {
                "/client"
            };
            return (StatusCode::FOUND, [(header::LOCATION, to.to_string())], "").into_response();
        }
    }

    let dir = template_dir();

    // The header and the auth panel are shared partials, read alongside the page.
    let header = tokio::fs::read_to_string(format!("{dir}/_header.html"))
        .await
        .unwrap_or_default();
    let panel = tokio::fs::read_to_string(format!("{dir}/_auth-panel.html"))
        .await
        .unwrap_or_default();

    let path = format!("{dir}/{name}.html");
    let body = match tokio::fs::read_to_string(&path).await {
        Ok(t) => t,
        Err(_) => return (StatusCode::NOT_FOUND, "404 Not Found").into_response(),
    };

    let page = name.to_string();
    let body = body.replace("<!--AUTH_PANEL-->", &panel);
    let html = format!(
        "{head}\n<body class=\"antialiased\">\n{header}\n{body}\n{footer}",
        head = head(&page, &title_for(&page)),
        header = header,
        body = body,
        footer = footer(
            &globals(&page, &session.csrf, &me, &visitor_country(headers)),
            &visitor_country(headers)
        ),
    );

    let mut res = Response::new(Body::from(html));
    res.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    res.headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    if let Ok(v) = HeaderValue::from_str(&crate::session::set_cookie(&session.id)) {
        res.headers_mut().insert(header::SET_COOKIE, v);
    }
    res
}

/// Unused import guard.
#[allow(dead_code)]
fn _keep(_: fn(&str) -> String) {
    let _ = util::h;
}
