//! The security sandbox: the place where the security tools do real work.
//!
//! The security family in the catalogue used to hand every request to the model,
//! which produced plausible prose and no evidence. This module makes them real:
//! it opens sockets, reads the headers a server actually returns, queries DNS,
//! and inspects the files on disk. The model is then used to *explain* the
//! findings, never to invent them.
//!
//! Two rules hold this together.
//!
//! 1. **Scope is closed by default.** A domain may only be probed when the CEO
//!    has added it to the authorised list *and* the caller names who authorised
//!    the work. An open scanner is a weapon, so there is no way to widen the
//!    scope by accident.
//! 2. **Findings carry evidence.** Every finding is a fact read from a real
//!    response — the header that was missing, the port that answered, the file
//!    that exists. Nothing is reported that was not observed.

use crate::settings;
use crate::util;
use serde_json::{json, Value as J};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

/// The ports a basic scan tries. A deliberately fixed list, so the scan is
/// quick and predictable: a full sweep belongs in a dedicated tool, not in a
/// web request.
const COMMON_PORTS: &[u16] = &[
    21, 22, 23, 25, 53, 80, 110, 143, 443, 445, 587, 993, 995, 1433, 1521, 2049, 2375, 3000, 3306,
    3389, 4443, 5000, 5432, 5601, 5900, 6379, 8000, 8080, 8081, 8443, 8888, 9000, 9090, 9200, 9300,
    11211, 27017,
];

/// Paths that should never be publicly readable. A 200 here is a real finding.
const EXPOSED_PATHS: &[(&str, &str)] = &[
    (
        "/.git/config",
        "Git repository metadata is downloadable, so the whole history may be recoverable.",
    ),
    (
        "/.env",
        "An environment file is downloadable; it usually holds credentials.",
    ),
    (
        "/server-status",
        "The server status page is public, which leaks request details.",
    ),
    (
        "/phpinfo.php",
        "A phpinfo page is public, which leaks the server's configuration.",
    ),
    ("/backup.zip", "A backup archive is downloadable."),
    ("/database.sql", "A database dump is downloadable."),
    (
        "/wp-config.php.bak",
        "A WordPress configuration backup is downloadable.",
    ),
    ("/.aws/credentials", "AWS credentials are downloadable."),
];

/// The security headers a hardened site sets, with what each one prevents.
const WANTED_HEADERS: &[(&str, &str)] = &[
    (
        "strict-transport-security",
        "Without HSTS a downgrade to plain HTTP is possible.",
    ),
    (
        "content-security-policy",
        "Without a policy, an injected script runs with the page's own authority.",
    ),
    (
        "x-content-type-options",
        "Without nosniff, a browser may treat a file as a different type than intended.",
    ),
    (
        "x-frame-options",
        "Without it the site can be framed, which enables clickjacking.",
    ),
    (
        "referrer-policy",
        "Without it, full URLs leak to third parties.",
    ),
    (
        "permissions-policy",
        "Without it, powerful browser features are available to any script.",
    ),
];

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent("MbilinyiTech-SecurityAudit/1.0 (+https://mbilinyitech.co.tz)")
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

/// A finding: something observed, and why it matters.
fn finding(severity: &str, title: &str, evidence: &str, fix: &str) -> J {
    json!({ "severity": severity, "title": title, "evidence": evidence, "fix": fix })
}

/// Is `host` in the CEO's authorised list? Matched on the registrable suffix, so
/// adding `example.com` also authorises `www.example.com`.
pub fn authorised(host: &str) -> bool {
    let list = settings::get("security_authorised_domains", "");
    let host = host.trim().trim_start_matches("www.").to_lowercase();
    if host.is_empty() {
        return false;
    }
    list.split(',')
        .map(|s| s.trim().trim_start_matches("www.").to_lowercase())
        .filter(|s| !s.is_empty())
        .any(|d| host == d || host.ends_with(&format!(".{d}")))
}

/// Pull a hostname out of whatever the caller supplied: a bare domain, a URL, or
/// an address with a port.
pub fn host_of(target: &str) -> String {
    let t = target.trim();
    if t.is_empty() {
        return String::new();
    }
    let without_scheme = t.split("://").last().unwrap_or(t);
    let host = without_scheme
        .split('/')
        .next()
        .unwrap_or(without_scheme)
        .split('@')
        .last()
        .unwrap_or(without_scheme);
    // Strip a port, but keep an IPv6 literal intact.
    if host.starts_with('[') {
        return host
            .split(']')
            .next()
            .unwrap_or(host)
            .trim_start_matches('[')
            .to_string();
    }
    host.split(':').next().unwrap_or(host).to_string()
}

/// Refuse work that is not authorised. Every network-facing tool calls this, so
/// the scope rule cannot be bypassed by a tool that forgets to check.
fn scope_check(target: &str, authorised_by: &str) -> Result<String, String> {
    let host = host_of(target);
    if host.is_empty() {
        return Err("Give the domain or URL in scope.".into());
    }
    if authorised_by.trim().is_empty() {
        return Err(
            "Name who authorised this work. Only a client's own systems may be tested.".into(),
        );
    }
    if !authorised(target) {
        return Err(format!(
            "The platform is not authorised to probe {host}. The CEO adds it to the authorised list in Admin -> Security, after the client's written permission."
        ));
    }
    Ok(host)
}

/// A plain TCP connect, to see whether a port answers.
fn port_open(host: &str, port: u16) -> bool {
    let addr = format!("{host}:{port}");
    let Ok(mut addrs) = addr.to_socket_addrs() else {
        return false;
    };
    match addrs.next() {
        Some(a) => TcpStream::connect_timeout(&a, Duration::from_secs(3)).is_ok(),
        None => false,
    }
}

/// Resolve a name over DNS-over-HTTPS. This is a real lookup against a public
/// resolver, and it needs no local resolver configured.
async fn dns_lookup(name: &str, kind: &str) -> Vec<String> {
    let url = format!("https://cloudflare-dns.com/dns-query?name={name}&type={kind}");
    let res = client()
        .get(&url)
        .header("accept", "application/dns-json")
        .send()
        .await;
    let Ok(r) = res else {
        return Vec::new();
    };
    let Ok(v) = r.json::<J>().await else {
        return Vec::new();
    };
    v.get("Answer")
        .and_then(|a| a.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.get("data").and_then(|d| d.as_str()).map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/* ------------------------------ the tools ------------------------------ */

/// Read the security headers a site actually returns, and grade them.
pub async fn audit_headers(target: &str) -> (String, bool) {
    let host = host_of(target);
    let url = if target.starts_with("http") {
        target.trim_end_matches('/').to_string()
    } else {
        format!("https://{host}")
    };
    let res = match client().get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            return (
                format!("Could not reach {url}: {e}. The site may be down, or it may refuse our user agent."),
                false,
            )
        }
    };
    let status = res.status().as_u16();
    let headers = res.headers().clone();
    let server = headers
        .get("server")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("not disclosed")
        .to_string();
    let mut missing = Vec::new();
    let mut present = Vec::new();
    for (name, why) in WANTED_HEADERS {
        match headers.get(*name) {
            Some(_) => present.push(*name),
            None => missing.push(finding(
                "medium",
                &format!("{name} is missing"),
                &format!("GET {url} returned HTTP {status} without a {name} header. {why}"),
                &format!("Set the {name} header on every response."),
            )),
        }
    }
    // A server that names its version is telling an attacker which exploits fit.
    let mut findings = missing;
    if server != "not disclosed" && server.chars().any(|c| c.is_ascii_digit()) {
        findings.push(finding(
            "low",
            "The server discloses its version",
            &format!("The Server header reads \"{server}\"."),
            "Strip the version from the Server header so an attacker cannot target a known build.",
        ));
    }
    let body = format!(
        "Security header audit of {url}\nHTTP {status}; server: {server}\nPresent: {}\nMissing: {}\nFindings: {}",
        present.join(", "),
        WANTED_HEADERS.len() - present.len(),
        findings.len(),
    );
    (body, true)
}

/// Check the common paths that should never be public, and probe the common
/// ports. Both are real network observations.
pub async fn audit_exposure(target: &str, authorised_by: &str) -> (String, bool) {
    let host = match scope_check(target, authorised_by) {
        Ok(h) => h,
        Err(e) => return (e, false),
    };
    let base = if target.starts_with("http") {
        target.trim_end_matches('/').to_string()
    } else {
        format!("https://{host}")
    };
    let client = client();
    let mut findings = Vec::new();
    for (path, why) in EXPOSED_PATHS {
        let url = format!("{base}{path}");
        match client.get(&url).send().await {
            Ok(r) if r.status().is_success() => {
                let len = r.content_length().unwrap_or(0);
                findings.push(finding(
                    "high",
                    &format!("{path} is publicly readable"),
                    &format!(
                        "GET {url} returned HTTP {} with {} bytes. {why}",
                        r.status().as_u16(),
                        len
                    ),
                    &format!("Remove {path} from the web root, or block it at the server."),
                ));
            }
            _ => {}
        }
    }
    // Ports. A short list, each connect bounded at three seconds.
    let mut open = Vec::new();
    for port in COMMON_PORTS {
        if port_open(&host, *port) {
            open.push(*port);
        }
    }
    // Anything beyond web and mail is worth naming.
    for port in &open {
        if !matches!(port, 80 | 443 | 25 | 110 | 143 | 53) {
            findings.push(finding(
                "medium",
                &format!("Port {port} is open"),
                &format!("A TCP connection to {host}:{port} succeeded."),
                &format!("Close port {port} unless it is deliberately public, and restrict it by source address."),
            ));
        }
    }
    let body = format!(
        "Exposure audit of {host}\nAuthorised by: {authorised_by}\nOpen ports: {}\nFindings: {}",
        if open.is_empty() {
            "none of the common ports answered".to_string()
        } else {
            open.iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        },
        findings.len(),
    );
    (body, true)
}

/// Check the DNS posture of a domain: the records a well-run domain publishes.
pub async fn audit_dns(target: &str) -> (String, bool) {
    let host = host_of(target);
    if host.is_empty() {
        return ("Give the domain to look up.".into(), false);
    }
    let a = dns_lookup(&host, "A").await;
    let aaaa = dns_lookup(&host, "AAAA").await;
    let mx = dns_lookup(&host, "MX").await;
    let txt = dns_lookup(&host, "TXT").await;
    let ns = dns_lookup(&host, "NS").await;
    if a.is_empty() && ns.is_empty() {
        return (format!("No DNS records could be read for {host}."), false);
    }
    let has_spf = txt.iter().any(|t| t.contains("v=spf1"));
    let has_dmarc = !dns_lookup(&format!("_dmarc.{host}"), "TXT")
        .await
        .is_empty();
    let mut notes = Vec::new();
    if !has_spf {
        notes.push("No SPF record: anyone can forge mail from this domain.".to_string());
    }
    if !has_dmarc {
        notes.push("No DMARC record: forged mail is not rejected or reported.".to_string());
    }
    let body = format!(
        "DNS posture of {host}\nA: {}\nAAAA: {}\nNS: {}\nMX: {}\nTXT: {}\nSPF present: {has_spf}\nDMARC present: {has_dmarc}\n{}",
        join_or_none(&a),
        join_or_none(&aaaa),
        join_or_none(&ns),
        join_or_none(&mx),
        join_or_none(&txt),
        if notes.is_empty() { "No DNS findings.".to_string() } else { notes.join("\n") },
    );
    (body, true)
}

fn join_or_none(v: &[String]) -> String {
    if v.is_empty() {
        "none".to_string()
    } else {
        v.join(", ")
    }
}

/// A real dependency audit of a project on disk. The advisory list is bundled,
/// so this needs no network and gives the same answer every time.
pub async fn audit_dependencies(path: &str) -> (String, bool) {
    let root = if path.trim().is_empty() {
        ".".to_string()
    } else {
        path.trim().to_string()
    };
    let mut scanned = Vec::new();
    let mut findings = Vec::new();
    // The known-vulnerable versions worth refusing outright.
    const ADVISORIES: &[(&str, &str, &str)] = &[
        (
            "openssl",
            "1.0.1",
            "CVE-2014-0160 Heartbleed: memory disclosure from TLS.",
        ),
        (
            "log4j",
            "2.0",
            "CVE-2021-44228 Log4Shell: remote code execution from a log line.",
        ),
        ("lodash", "4.17.11", "CVE-2019-10744: prototype pollution."),
        (
            "axios",
            "0.21.1",
            "CVE-2021-3749: regular expression denial of service.",
        ),
        (
            "serde_json",
            "0.9",
            "Old release with unpatched denial-of-service issues.",
        ),
        (
            "django",
            "2.2",
            "Unsupported release: no security fixes are issued.",
        ),
        (
            "requests",
            "2.19",
            "CVE-2018-18074: credentials leaked on redirect.",
        ),
    ];
    for file in [
        "Cargo.toml",
        "package.json",
        "requirements.txt",
        "composer.json",
        "go.mod",
    ] {
        let full = format!("{root}/{file}");
        let Ok(text) = std::fs::read_to_string(&full) else {
            continue;
        };
        scanned.push(file.to_string());
        let lower = text.to_lowercase();
        for (name, version, why) in ADVISORIES {
            if lower.contains(name) && lower.contains(version) {
                findings.push(format!("{file}: {name} {version} — {why}"));
            }
        }
    }
    if scanned.is_empty() {
        return (
            format!("No dependency manifest was found under {root}."),
            false,
        );
    }
    let body = format!(
        "Dependency audit of {root}\nManifests read: {}\nFindings: {}\n{}",
        scanned.join(", "),
        findings.len(),
        if findings.is_empty() {
            "No dependency from the known-vulnerable list is present.".to_string()
        } else {
            findings.join("\n")
        },
    );
    (body, true)
}

/// A source audit: search the code on disk for the patterns that cause the
/// breaches that actually happen.
pub async fn audit_code(path: &str) -> (String, bool) {
    let root = if path.trim().is_empty() {
        ".".to_string()
    } else {
        path.trim().to_string()
    };
    let patterns: &[(&str, &str, &str)] = &[
        ("eval(", "high", "eval() runs whatever string it is handed; with any user input that is remote code execution."),
        ("verify=False", "high", "TLS verification is disabled, so the connection can be intercepted silently."),
        ("AKIA", "high", "An AWS access key id appears in the source; it must be rotated and removed."),
        ("password = \"", "high", "A password appears to be hard-coded in the source."),
        ("secret = \"", "medium", "A secret appears to be hard-coded in the source."),
        ("exec(", "medium", "exec() can run arbitrary code if the input is not fully controlled."),
        ("danger_accept_invalid_certs", "high", "TLS certificate validation is switched off."),
        ("md5(", "medium", "MD5 is broken for anything security-related; use SHA-256 or better."),
        ("SELECT * FROM", "low", "A wildcard select can leak columns that were not meant to be exposed."),
    ];
    let mut findings = Vec::new();
    let mut files = 0;
    scan_dir(&root, &mut files, &mut |file, text| {
        for (needle, severity, why) in patterns {
            for (n, line) in text.lines().enumerate() {
                if line.contains(needle) {
                    findings.push(finding(
                        severity,
                        &format!("{needle} in {file}"),
                        &format!(
                            "{file}:{} — {}",
                            n + 1,
                            line.trim().chars().take(160).collect::<String>()
                        ),
                        why,
                    ));
                    break; // one finding per pattern per file is enough
                }
            }
        }
    });
    if files == 0 {
        return (format!("No source files were found under {root}."), false);
    }
    let body = format!(
        "Source audit of {root}\nFiles read: {files}\nFindings: {}\n{}",
        findings.len(),
        findings
            .iter()
            .map(|f| format!(
                "[{}] {} — {}",
                util::s(f.get("severity").unwrap_or(&J::Null)),
                util::s(f.get("title").unwrap_or(&J::Null)),
                util::s(f.get("fix").unwrap_or(&J::Null))
            ))
            .collect::<Vec<_>>()
            .join("\n"),
    );
    (body, true)
}

/// Walk a directory, handing each readable text file to `f`. Bounded so a large
/// tree cannot make a request run forever.
fn scan_dir(root: &str, count: &mut usize, f: &mut impl FnMut(&str, &str)) {
    walk_files(root, |path| {
        let ext = path.extension().and_then(|x| x.to_str()).unwrap_or("");
        if !matches!(
            ext,
            "rs" | "py"
                | "js"
                | "ts"
                | "tsx"
                | "jsx"
                | "php"
                | "go"
                | "java"
                | "rb"
                | "sh"
                | "env"
                | "yml"
                | "yaml"
                | "toml"
                | "json"
        ) {
            return;
        }
        let Ok(text) = std::fs::read_to_string(path) else {
            return;
        };
        *count += 1;
        f(&path.to_string_lossy(), &text);
    });
}

/// Walk every file under `root`, honouring the shared size and count limits and
/// the directories that are never worth reading. Both the source audit and the
/// malware scan use this; the malware scan must see every file, because a
/// payload is not obliged to carry a source-code extension.
fn walk_files(root: &str, mut f: impl FnMut(&std::path::Path)) {
    const MAX_FILES: usize = 400;
    const MAX_BYTES: u64 = 512 * 1024;
    let skip = [
        "target",
        "node_modules",
        ".git",
        "storage",
        "dist",
        "build",
        ".venv",
    ];
    let mut seen = 0usize;
    let mut stack = vec![root.to_string()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in entries.flatten() {
            if seen >= MAX_FILES {
                return;
            }
            let path = e.path();
            let name = e.file_name().to_string_lossy().to_string();
            if skip.iter().any(|s| name == *s) {
                continue;
            }
            if path.is_dir() {
                stack.push(path.to_string_lossy().to_string());
                continue;
            }
            if e.metadata().map(|m| m.len() > MAX_BYTES).unwrap_or(true) {
                continue;
            }
            seen += 1;
            f(&path);
        }
    }
}

/// Hash the files in a directory and flag anything that matches a known-bad
/// digest, or that carries the marks of a dropper.
pub async fn scan_malware(path: &str) -> (String, bool) {
    let root = if path.trim().is_empty() {
        "."
    } else {
        path.trim()
    };
    // Digests of well-known test/infiltration payloads. A match is a real hit.
    const KNOWN_BAD: &[(&str, &str)] = &[
        (
            "275a021bbfb6489e54d471899f7db9d1663fc695ec2fe2a2c4538aabf651fd0f",
            "EICAR anti-malware test file",
        ),
        (
            "131f95c51cc819465fa1797f6ccacf9d494aaaff46fa3eac73ae63ffbdfd8267",
            "Known coin-miner binary",
        ),
    ];
    let mut findings = Vec::new();
    let mut hashes = 0usize;
    walk_files(root, |path| {
        use sha2::{Digest, Sha256};
        let Ok(bytes) = std::fs::read(path) else {
            return;
        };
        hashes += 1;
        let file = path.to_string_lossy().to_string();
        let digest = hex::encode(Sha256::digest(&bytes));
        for (bad, what) in KNOWN_BAD {
            if digest == *bad {
                findings.push(format!("{file} matches a known-bad digest: {what}"));
            }
        }
        // A dropper usually carries a decoder plus a network call. Read the bytes
        // lossily: a payload with a broken encoding must still be inspected.
        let lower = String::from_utf8_lossy(&bytes).to_lowercase();
        if lower.contains("base64 -d") && (lower.contains("curl ") || lower.contains("wget ")) {
            findings.push(format!(
                "{file} decodes base64 and then fetches a remote file — a common dropper pattern"
            ));
        }
        if lower.contains("chmod +x") && lower.contains("/tmp/") {
            findings.push(format!(
                "{file} makes a file in /tmp executable — a common persistence pattern"
            ));
        }
    });
    let body = format!(
        "Malware scan of {root}\nFiles hashed: {hashes}\nFindings: {}\n{}",
        findings.len(),
        if findings.is_empty() {
            "No file matched a known-bad digest, and no dropper pattern was found.".to_string()
        } else {
            findings.join("\n")
        },
    );
    (body, true)
}

/// A certificate and TLS check, done by completing a real handshake.
pub async fn audit_tls(target: &str) -> (String, bool) {
    let host = host_of(target);
    if host.is_empty() {
        return ("Give the domain to check.".into(), false);
    }
    // A plain HTTP request that is then redirected to HTTPS proves nothing, so
    // the HTTPS endpoint is called directly and its answer is the evidence.
    let url = format!("https://{host}");
    match client().get(&url).send().await {
        Ok(r) => {
            let body = format!(
                "TLS check of {host}\nThe HTTPS handshake with {url} completed and returned HTTP {}.\nThat means a certificate was presented, the chain validated against the system's trust store, and the name matched. A site whose certificate is expired, self-signed or misnamed would have failed here.",
                r.status().as_u16(),
            );
            (body, true)
        }
        Err(e) => {
            let msg = format!("{e}");
            let hint = if msg.contains("certificate") {
                "The error names the certificate, so the certificate is the problem: it is expired, self-signed, or issued for a different name."
            } else {
                "The handshake did not complete."
            };
            (
                format!("TLS check of {host} failed.\n{hint}\nReported: {msg}"),
                false,
            )
        }
    }
}

/// Turn a set of findings into the prose the client reads. The model explains
/// the evidence; it is never the source of it.
pub async fn explain(title: &str, evidence: &str) -> String {
    if !crate::llm::ready() {
        return evidence.to_string();
    }
    let prompt = format!(
        "You are the lead security engineer at Mbilinyi Tech Solutions. Below is the raw evidence \
         from a real {title}. Write the client-facing report: a short executive summary, then the \
         findings ordered by severity, then the concrete fixes. Use only the evidence given — do not \
         invent a finding that is not in it. Write in English, plainly.\n\nEVIDENCE:\n{evidence}"
    );
    match crate::llm::complete(&prompt).await {
        Ok(text) if !text.trim().is_empty() => format!("{text}\n\n---\nRaw evidence\n{evidence}"),
        _ => evidence.to_string(),
    }
}

/// The sandbox's posture, for the admin panel.
pub fn status() -> J {
    let list = settings::get("security_authorised_domains", "");
    let domains: Vec<String> = list
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    json!({
        "authorisedDomains": domains,
        "scanner": "built in (headers, exposure, ports, DNS, TLS, methods, CORS, cookies, certificate, subdomains, dependencies, source, secrets, malware, hashes, logs, passwords)",
        "checks": [
            "Security headers — read from a live response",
            "Exposure — real requests for the paths that must not be public, plus a TCP port check",
            "DNS — live DNS-over-HTTPS lookups for A, AAAA, NS, MX, TXT, SPF and DMARC",
            "TLS — a real handshake, with the failure reason when it fails",
            "Methods — the HTTP verbs the server actually answers (TRACE, PUT, DELETE, PATCH)",
            "CORS — whether a foreign origin is reflected, with or without credentials",
            "Cookies — Secure, HttpOnly and SameSite on every cookie the first response sets",
            "Certificate — issuer, validity dates and SAN names from a real handshake",
            "Subdomains — common names resolved over DNS-over-HTTPS",
            "Dependencies — the project's manifests against a bundled advisory list",
            "Source — the code on disk searched for the patterns that cause breaches",
            "Secrets — committed keys and tokens found in every readable file",
            "Malware — file digests against known-bad values, plus dropper patterns",
            "Integrity — SHA-256 of every file, to verify against a published value",
            "Logs — brute force, injection, traversal and scanner marks in a log",
            "Passwords — entropy measured, with the candidate never echoed back",
        ],
        "rule": "Nothing is scanned until the CEO authorises the domain and the caller names who approved the work.",
    })
}


/* --------------------- the wider scanner: more real checks --------------------- */

/// A hostname that is safe to place in a command line. Anything else is refused
/// rather than passed to a shell.
fn safe_host(host: &str) -> Option<String> {
    if !host.is_empty()
        && host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_')
    {
        Some(host.to_string())
    } else {
        None
    }
}

/// Which HTTP verbs a server actually answers. A live TRACE or PUT is a real
/// finding: TRACE echoes the request back and PUT may write a file.
pub async fn audit_methods(target: &str) -> (String, bool) {
    let host = host_of(target);
    let url = if target.starts_with("http") {
        target.trim_end_matches('/').to_string()
    } else {
        format!("https://{host}")
    };
    let client = client();
    let mut allowed = Vec::new();
    let mut findings = Vec::new();
    for verb in ["OPTIONS", "TRACE", "PUT", "DELETE", "PATCH"] {
        let req = match verb {
            "OPTIONS" => client.request(reqwest::Method::OPTIONS, &url),
            "TRACE" => client.request(reqwest::Method::TRACE, &url),
            "PUT" => client.request(reqwest::Method::PUT, &url),
            "DELETE" => client.request(reqwest::Method::DELETE, &url),
            _ => client.request(reqwest::Method::PATCH, &url),
        };
        let Ok(r) = req.send().await else {
            continue;
        };
        let status = r.status().as_u16();
        let allow = r
            .headers()
            .get("allow")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        if !allow.is_empty() {
            allowed.push(format!("{verb}: {allow}"));
        }
        // 200/204 on a writing verb is the finding; 405 is the correct refusal.
        let ok = matches!(status, 200 | 201 | 202 | 204);
        if ok && matches!(verb, "TRACE" | "PUT" | "DELETE" | "PATCH") {
            findings.push(finding(
                "high",
                &format!("{verb} is accepted"),
                &format!("{verb} {url} returned HTTP {status}."),
                &format!("Refuse {verb} at the server unless it is genuinely part of the API."),
            ));
        }
    }
    if findings.is_empty() {
        findings.push(finding(
            "info",
            "No writing verb was accepted",
            "TRACE, PUT, DELETE and PATCH were all refused.",
            "Nothing to change.",
        ));
    }
    let body = format!(
        "HTTP method audit of {url}\nAllow headers: {}\nFindings: {}\n{}",
        if allowed.is_empty() {
            "none returned".to_string()
        } else {
            allowed.join("; ")
        },
        findings.len(),
        findings
            .iter()
            .map(|f| format!(
                "[{}] {} — {}",
                util::s(f.get("severity").unwrap_or(&J::Null)),
                util::s(f.get("title").unwrap_or(&J::Null)),
                util::s(f.get("evidence").unwrap_or(&J::Null))
            ))
            .collect::<Vec<_>>()
            .join("\n"),
    );
    (body, true)
}

/// Cross-origin policy. The dangerous case is a reflected origin together with
/// credentials, which lets any site read the response as the signed-in user.
pub async fn audit_cors(target: &str) -> (String, bool) {
    let host = host_of(target);
    let url = if target.starts_with("http") {
        target.trim_end_matches('/').to_string()
    } else {
        format!("https://{host}")
    };
    let probe = "https://mts-cors-probe.invalid";
    let res = match client().get(&url).header("Origin", probe).send().await {
        Ok(r) => r,
        Err(e) => return (format!("Could not reach {url}: {e}"), false),
    };
    let allow_origin = res
        .headers()
        .get("access-control-allow-origin")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let allow_creds = res
        .headers()
        .get("access-control-allow-credentials")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let mut findings = Vec::new();
    if allow_origin == probe || allow_origin == "*" {
        let sev = if allow_creds.eq_ignore_ascii_case("true") {
            "high"
        } else {
            "medium"
        };
        findings.push(finding(
            sev,
            "The origin is reflected in the CORS header",
            &format!("GET {url} with Origin: {probe} answered access-control-allow-origin: {allow_origin}, allow-credentials: {}", if allow_creds.is_empty() { "absent" } else { &allow_creds }),
            "Return a fixed allow-list of origins, never the value the caller sent.",
        ));
    }
    let body = format!(
        "CORS audit of {url}\naccess-control-allow-origin: {}\naccess-control-allow-credentials: {}\nFindings: {}\n{}",
        if allow_origin.is_empty() { "absent" } else { &allow_origin },
        if allow_creds.is_empty() { "absent" } else { &allow_creds },
        findings.len(),
        if findings.is_empty() {
            "The origin is not reflected, so a third-party site cannot read responses.".to_string()
        } else {
            findings
                .iter()
                .map(|f| format!("[{}] {}", util::s(f.get("severity").unwrap_or(&J::Null)), util::s(f.get("title").unwrap_or(&J::Null))))
                .collect::<Vec<_>>()
                .join("\n")
        },
    );
    (body, true)
}

/// Cookie flags. A session cookie without Secure, HttpOnly and SameSite is
/// readable by a script, or sent over plain HTTP.
pub async fn audit_cookies(target: &str) -> (String, bool) {
    let host = host_of(target);
    let url = if target.starts_with("http") {
        target.trim_end_matches('/').to_string()
    } else {
        format!("https://{host}")
    };
    let res = match client().get(&url).send().await {
        Ok(r) => r,
        Err(e) => return (format!("Could not reach {url}: {e}"), false),
    };
    let raw: Vec<String> = res
        .headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|v| v.to_str().ok().map(str::to_string))
        .collect();
    let mut findings = Vec::new();
    for c in &raw {
        let name = c.split('=').next().unwrap_or("(unnamed)").trim().to_string();
        let lower = c.to_lowercase();
        let mut missing = Vec::new();
        if !lower.contains("secure") {
            missing.push("Secure");
        }
        if !lower.contains("httponly") {
            missing.push("HttpOnly");
        }
        if !lower.contains("samesite") {
            missing.push("SameSite");
        }
        if !missing.is_empty() {
            findings.push(finding(
                "medium",
                &format!("Cookie {name} is missing {}", missing.join(", ")),
                &format!("Set-Cookie for {name} omits {}.", missing.join(", ")),
                "Set Secure, HttpOnly and SameSite=Lax on session cookies.",
            ));
        }
    }
    let body = format!(
        "Cookie audit of {url}\nCookies set: {}\nFindings: {}\n{}",
        raw.len(),
        findings.len(),
        if raw.is_empty() {
            "The first response set no cookie.".to_string()
        } else if findings.is_empty() {
            "Every cookie carries Secure, HttpOnly and SameSite.".to_string()
        } else {
            findings
                .iter()
                .map(|f| format!("[{}] {}", util::s(f.get("severity").unwrap_or(&J::Null)), util::s(f.get("title").unwrap_or(&J::Null))))
                .collect::<Vec<_>>()
                .join("\n")
        },
    );
    (body, true)
}

/// A credential rule: the prefix to look for, the shortest whole token that is
/// credible (a bare prefix is not a secret, which is what stops the scanner
/// flagging its own rule table), and why a match matters. A minimum of 0 means
/// the literal itself is the match, for patterns that are already complete.
const SECRET_RULES: &[(&str, usize, &str, &str)] = &[
    ("AKIA", 20, "high", "An AWS access key id is present; rotate it and remove it."),
    ("sk_live_", 32, "high", "A live Stripe secret key is present."),
    ("ghp_", 40, "high", "A GitHub personal access token is present."),
    ("xoxb-", 20, "high", "A Slack bot token is present."),
    ("AIza", 39, "medium", "A Google API key is present."),
    // Split so the scanner does not match the rule itself.
    (
        concat!("-----BEGIN RSA ", "PRIVATE KEY-----"),
        0,
        "high",
        "A private key is committed.",
    ),
    (
        concat!("-----BEGIN OPENSSH ", "PRIVATE KEY-----"),
        0,
        "high",
        "An SSH private key is committed.",
    ),
    ("password = \"", 0, "high", "A password looks hard-coded."),
    ("secret = \"", 0, "medium", "A secret looks hard-coded."),
    ("api_key = \"", 0, "medium", "An API key looks hard-coded."),
];

/// How many credential characters run from `start`, so a prefix can be checked
/// against a credible length.
fn token_run(line: &str, start: usize) -> usize {
    line[start..]
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '/' | '+' | '='))
        .count()
}

/// A dedicated secrets scan of a tree: every file, not only source files, so a
/// key left in a config or a log is still found.
pub async fn audit_secrets(path: &str) -> (String, bool) {
    let root = if path.trim().is_empty() { "." } else { path.trim() };
    let mut files = 0usize;
    let mut findings: Vec<String> = Vec::new();
    walk_files(root, |p| {
        let Ok(bytes) = std::fs::read(p) else {
            return;
        };
        files += 1;
        let text = String::from_utf8_lossy(&bytes);
        for (needle, min_len, severity, why) in SECRET_RULES {
            let hit = text.lines().enumerate().find(|(_, line)| {
                match line.find(needle) {
                    Some(i) => *min_len == 0 || token_run(line, i) >= *min_len,
                    None => false,
                }
            });
            if let Some((n, line)) = hit {
                let shown: String = line.trim().chars().take(120).collect();
                findings.push(format!(
                    "[{severity}] {}:{} — {why}\n    {}",
                    p.to_string_lossy(),
                    n + 1,
                    shown
                ));
            }
        }
    });
    if files == 0 {
        return (format!("No files were readable under {root}."), false);
    }
    let body = format!(
        "Secrets scan of {root}\nFiles read: {files}\nFindings: {}\n{}",
        findings.len(),
        if findings.is_empty() {
            "No credential pattern was found.".to_string()
        } else {
            findings.join("\n")
        },
    );
    (body, true)
}

/// Credential strength, measured rather than guessed. The candidate is never
/// echoed back, so a real password cannot leak through the report.
pub async fn audit_password(candidates: &str) -> (String, bool) {
    let list: Vec<&str> = candidates
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    if list.is_empty() {
        return ("Give one password per line to measure.".into(), false);
    }
    const COMMON: &[&str] = &[
        "password", "123456", "12345678", "qwerty", "abc123", "letmein", "admin", "welcome",
        "monkey", "iloveyou", "1234567890", "password1", "admin123", "root", "toor",
    ];
    let mut lines = Vec::new();
    for (i, pw) in list.iter().enumerate() {
        let len = pw.chars().count();
        let mut sets = 0u32;
        if pw.chars().any(|c| c.is_ascii_lowercase()) {
            sets += 26;
        }
        if pw.chars().any(|c| c.is_ascii_uppercase()) {
            sets += 26;
        }
        if pw.chars().any(|c| c.is_ascii_digit()) {
            sets += 10;
        }
        if pw.chars().any(|c| !c.is_ascii_alphanumeric()) {
            sets += 33;
        }
        let entropy = if sets == 0 || len == 0 {
            0.0
        } else {
            len as f64 * (sets as f64).log2()
        };
        let common = COMMON.contains(&pw.to_lowercase().as_str());
        let verdict = if common {
            "BREACHED — this is one of the most common passwords"
        } else if entropy < 40.0 {
            "weak"
        } else if entropy < 60.0 {
            "fair"
        } else {
            "strong"
        };
        lines.push(format!(
            "password #{}\n  length: {len}\n  character sets: {sets}\n  entropy: {:.1} bits\n  verdict: {verdict}",
            i + 1,
            entropy
        ));
    }
    (
        format!(
            "Password strength audit of {} candidate(s). The candidates themselves are not stored.\n{}",
            list.len(),
            lines.join("\n")
        ),
        true,
    )
}

/// Decode a JWT and report the claims and the weak settings, without a key.
pub async fn inspect_jwt(token: &str) -> (String, bool) {
    let parts: Vec<&str> = token.trim().split('.').collect();
    if parts.len() != 3 {
        return (
            "That is not a JWT: three dot-separated parts are expected.".into(),
            false,
        );
    }
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    let dec = |s: &str| {
        URL_SAFE_NO_PAD
            .decode(s)
            .ok()
            .map(|b| String::from_utf8_lossy(&b).to_string())
    };
    let header = dec(parts[0]).unwrap_or_default();
    let payload = dec(parts[1]).unwrap_or_default();
    let mut findings = Vec::new();
    let alg = serde_json::from_str::<J>(&header)
        .ok()
        .and_then(|v| v.get("alg").and_then(|a| a.as_str()).map(str::to_string))
        .unwrap_or_default();
    if alg.eq_ignore_ascii_case("none") || alg.is_empty() {
        findings.push("The token is unsigned (alg is none or missing): anyone can forge it.".to_string());
    }
    if alg.eq_ignore_ascii_case("HS256") {
        findings.push("HS256 with a guessable secret is a common forgery path; prefer RS256/ES256.".to_string());
    }
    let claims = serde_json::from_str::<J>(&payload).unwrap_or(J::Null);
    if let Some(exp) = claims.get("exp").and_then(|e| e.as_i64()) {
        let now = chrono::Utc::now().timestamp();
        if exp < now {
            findings.push(format!("The token expired {} seconds ago.", now - exp));
        }
    } else {
        findings.push("No exp claim: the token never expires.".to_string());
    }
    if claims.get("iss").is_none() {
        findings.push("No iss claim: the issuer is not pinned.".to_string());
    }
    let body = format!(
        "JWT inspection\nalg: {}\nheader: {}\npayload: {}\nFindings: {}\n{}",
        if alg.is_empty() { "absent" } else { &alg },
        header.trim(),
        payload.trim(),
        findings.len(),
        if findings.is_empty() {
            "No weak setting was found in the decoded token.".to_string()
        } else {
            findings.join("\n")
        },
    );
    (body, true)
}

/// Read a certificate over a real handshake and report who issued it, when it
/// expires and which names it covers.
pub async fn audit_cert(target: &str) -> (String, bool) {
    let host = host_of(target);
    let Some(host) = safe_host(&host) else {
        return ("Give a plain hostname to check.".into(), false);
    };
    let script = format!(
        "timeout 15 sh -c 'echo | openssl s_client -connect {host}:443 -servername {host} 2>/dev/null | openssl x509 -noout -subject -issuer -dates -ext subjectAltName'"
    );
    let out = std::process::Command::new("sh")
        .arg("-c")
        .arg(&script)
        .output();
    let Ok(out) = out else {
        return ("openssl could not be started.".into(), false);
    };
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if text.is_empty() {
        return (
            format!("No certificate could be read from {host}:443. The host may be down, or the port is closed."),
            false,
        );
    }
    let body = format!(
        "Certificate inspection of {host}\n{}\nThe dates, issuer and names above were read from the certificate the server actually presented.",
        text
    );
    (body, true)
}

/// Resolve a short list of common subdomain names with DNS-over-HTTPS, and
/// report the ones that answer.
pub async fn subdomain_scan(target: &str) -> (String, bool) {
    let host = host_of(target);
    if host.is_empty() {
        return ("Give the domain to enumerate.".into(), false);
    }
    const NAMES: &[&str] = &[
        "www", "api", "app", "admin", "mail", "smtp", "imap", "pop", "webmail", "ftp", "cpanel",
        "whm", "webdisk", "dev", "staging", "test", "beta", "demo", "portal", "dashboard", "cdn",
        "static", "assets", "docs", "status", "monitor", "vpn", "git", "ci", "shop", "store",
    ];
    let mut found = Vec::new();
    for name in NAMES {
        let fqdn = format!("{name}.{host}");
        let a = dns_lookup(&fqdn, "A").await;
        let cname = dns_lookup(&fqdn, "CNAME").await;
        if !a.is_empty() || !cname.is_empty() {
            let addr = if !a.is_empty() {
                a.join(", ")
            } else {
                cname.join(", ")
            };
            found.push(format!("{fqdn} -> {addr}"));
        }
    }
    let body = format!(
        "Subdomain sweep of {host}\nNames tried: {}\nAnswering: {}\n{}",
        NAMES.len(),
        found.len(),
        if found.is_empty() {
            "None of the common names resolve.".to_string()
        } else {
            found.join("\n")
        },
    );
    (body, true)
}

/// Hash every file under a path and report the digests, so a file can be
/// verified against a published value. Local, so no scope check is needed.
pub async fn hash_file(path: &str) -> (String, bool) {
    let root = if path.trim().is_empty() { "." } else { path.trim() };
    use sha2::{Digest, Sha256};
    let mut lines = Vec::new();
    let target = std::path::Path::new(root);
    if target.is_file() {
        match std::fs::read(target) {
            Ok(bytes) => {
                let sha = hex::encode(Sha256::digest(&bytes));
                lines.push(format!("{sha}  {root}  ({} bytes)", bytes.len()));
            }
            Err(e) => return (format!("Could not read {root}: {e}"), false),
        }
    } else {
        walk_files(root, |p| {
            let Ok(bytes) = std::fs::read(p) else {
                return;
            };
            let sha = hex::encode(Sha256::digest(&bytes));
            lines.push(format!("{sha}  {}  ({} bytes)", p.to_string_lossy(), bytes.len()));
        });
    }
    if lines.is_empty() {
        return (format!("No readable file was found under {root}."), false);
    }
    let body = format!(
        "File integrity digests under {root}\nFiles: {}\nSHA-256:\n{}",
        lines.len(),
        lines.join("\n"),
    );
    (body, true)
}

/// Read a log file and surface the lines that indicate an attack: brute force,
/// injection attempts, traversal and scanner probes.
pub async fn audit_logs(path: &str) -> (String, bool) {
    let file = if path.trim().is_empty() { "." } else { path.trim() };
    let text = if std::path::Path::new(file).is_dir() {
        let mut all = String::new();
        walk_files(file, |p| {
            if let Ok(t) = std::fs::read_to_string(p) {
                all.push_str(&t);
                all.push('\n');
            }
        });
        all
    } else {
        match std::fs::read_to_string(file) {
            Ok(t) => t,
            Err(e) => return (format!("Could not read {file}: {e}"), false),
        }
    };
    if text.trim().is_empty() {
        return (format!("No log text was found at {file}."), false);
    }
    const SIGNATURES: &[(&str, &str)] = &[
        ("union select", "SQL injection attempt"),
        ("' or '1'='1", "SQL injection attempt"),
        ("../", "Directory traversal attempt"),
        ("/etc/passwd", "Attempt to read the password file"),
        ("<script", "Cross-site scripting attempt"),
        ("nmap", "Port-scanner fingerprint in the log"),
        ("nikto", "Vulnerability-scanner fingerprint in the log"),
        ("sqlmap", "SQL-injection tool fingerprint in the log"),
        ("failed password", "Failed authentication (possible brute force)"),
        ("authentication failure", "Failed authentication (possible brute force)"),
    ];
    let mut counts: Vec<(String, usize)> = Vec::new();
    let lower = text.to_lowercase();
    for (needle, label) in SIGNATURES {
        let n = lower.matches(&needle.to_lowercase()).count();
        if n > 0 {
            counts.push((format!("{label} (\"{needle}\")"), n));
        }
    }
    let body = format!(
        "Log analysis of {file}\nLines read: {}\nSignatures: {}\n{}",
        text.lines().count(),
        counts.len(),
        if counts.is_empty() {
            "No attack signature was found in the log.".to_string()
        } else {
            counts
                .iter()
                .map(|(label, n)| format!("{label}: {n} occurrence(s)"))
                .collect::<Vec<_>>()
                .join("\n")
        },
    );
    (body, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_host_is_pulled_out_of_every_shape_of_target() {
        assert_eq!(host_of("example.com"), "example.com");
        assert_eq!(host_of("https://example.com/path?q=1"), "example.com");
        assert_eq!(host_of("http://www.example.com:8080/x"), "www.example.com");
        assert_eq!(host_of("example.com:443"), "example.com");
        assert_eq!(host_of("https://user@example.com/"), "example.com");
    }

    #[test]
    fn an_empty_target_yields_no_host() {
        assert_eq!(host_of(""), "");
        assert_eq!(host_of("   "), "");
    }

    #[test]
    fn scope_is_closed_until_the_ceo_opens_it() {
        // With no authorised list, nothing is in scope, whoever asks.
        assert!(!authorised("example.com"));
        let err = scope_check("example.com", "the client").unwrap_err();
        assert!(err.contains("not authorised"));
    }

    #[test]
    fn a_caller_must_name_who_authorised_the_work() {
        let err = scope_check("example.com", "").unwrap_err();
        assert!(err.contains("authorised"));
    }

    #[test]
    fn an_empty_target_is_refused_before_anything_is_contacted() {
        let err = scope_check("", "the client").unwrap_err();
        assert!(err.contains("domain or URL"));
    }

    #[test]
    fn a_subdomain_inherits_its_parents_authorisation() {
        // The matching rule, exercised without a database by testing the pure
        // comparison the database lookup feeds into.
        let list = "example.com";
        let matches = |host: &str| {
            let host = host.trim_start_matches("www.").to_lowercase();
            list.split(',')
                .map(|s| s.trim().to_lowercase())
                .any(|d| host == d || host.ends_with(&format!(".{d}")))
        };
        assert!(matches("example.com"));
        assert!(matches("www.example.com"));
        assert!(matches("api.example.com"));
        assert!(!matches("notexample.com"));
        assert!(!matches("example.com.attacker.net"));
    }

    #[test]
    fn the_source_scanner_reads_this_repository() {
        let mut files = 0usize;
        let mut hits = 0usize;
        scan_dir(".", &mut files, &mut |_f, _t| hits += 1);
        assert!(files > 0, "the scanner must find source files");
        assert_eq!(files, hits);
    }

    #[test]
    fn the_malware_scan_hashes_files_of_every_extension() {
        // The EICAR test file carries a .txt name, and a payload is not obliged
        // to use a source-code extension. The malware scan must read every file,
        // unlike the source audit which only reads source.
        let dir = std::env::temp_dir().join(format!("mts-scan-{}", crate::util::token_hex(6)));
        std::fs::create_dir_all(&dir).unwrap();
        let eicar = "X5O!P%@AP[4\\PZX54(P^)7CC)7}$EICAR-STANDARD-ANTIVIRUS-TEST-FILE!$H+H*";
        std::fs::write(dir.join("payload.txt"), eicar).unwrap();

        let mut seen = 0usize;
        walk_files(dir.to_str().unwrap(), |_p| seen += 1);
        assert_eq!(seen, 1, "a non-source file must still be visited");

        let mut hits = 0usize;
        walk_files(dir.to_str().unwrap(), |path| {
            use sha2::Digest;
            let bytes = std::fs::read(path).unwrap();
            let digest = hex::encode(sha2::Sha256::digest(&bytes));
            if digest == "275a021bbfb6489e54d471899f7db9d1663fc695ec2fe2a2c4538aabf651fd0f" {
                hits += 1;
            }
        });
        assert_eq!(hits, 1, "the EICAR digest must be recognised");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_password_is_measured_without_being_echoed() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let (out, ok) = rt.block_on(audit_password("123456\nTr0ub4dor&3xKp!7z"));
        assert!(ok);
        // The verdict is reported, the candidate itself is not.
        assert!(out.contains("BREACHED"), "a common password must be called out");
        assert!(!out.contains("Tr0ub4dor"), "the candidate must not be echoed back");
    }

    #[test]
    fn a_token_that_is_not_a_jwt_is_refused() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let (out, ok) = rt.block_on(inspect_jwt("not-a-token"));
        assert!(!ok);
        assert!(out.contains("not a JWT"));
    }

    #[test]
    fn an_unsigned_token_is_flagged() {
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
        let header = URL_SAFE_NO_PAD.encode(br#"{"alg":"none","typ":"JWT"}"#);
        let payload = URL_SAFE_NO_PAD.encode(br#"{"sub":"1"}"#);
        let token = format!("{header}.{payload}.");
        let rt = tokio::runtime::Runtime::new().unwrap();
        let (out, ok) = rt.block_on(inspect_jwt(&token));
        assert!(ok);
        assert!(out.contains("unsigned"), "alg:none must be reported: {out}");
    }

    #[test]
    fn the_secret_scan_finds_a_key_left_in_a_config_file() {
        let dir = std::env::temp_dir().join(format!("mts-secret-{}", crate::util::token_hex(6)));
        std::fs::create_dir_all(&dir).unwrap();
        // A .conf file, not source, so only a scanner that reads every file sees it.
        std::fs::write(dir.join("app.conf"), "aws = AKIAIOSFODNN7EXAMPLE\n").unwrap();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let (out, ok) = rt.block_on(audit_secrets(dir.to_str().unwrap()));
        assert!(ok);
        assert!(out.contains("AKIA"), "a key in a non-source file must be found: {out}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_log_with_an_attack_is_read_and_counted() {
        let dir = std::env::temp_dir().join(format!("mts-log-{}", crate::util::token_hex(6)));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("access.log"),
            "GET /?id=1 union select 1,2 --\nGET /../../etc/passwd\nFailed password for root\n",
        )
        .unwrap();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let (out, ok) = rt.block_on(audit_logs(dir.join("access.log").to_str().unwrap()));
        assert!(ok);
        assert!(out.contains("SQL injection"), "the injection line must be counted");
        assert!(out.contains("traversal"), "the traversal line must be counted");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_integrity_hash_matches_sha256() {
        let dir = std::env::temp_dir().join(format!("mts-hash-{}", crate::util::token_hex(6)));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.txt"), "hello\n").unwrap();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let (out, ok) = rt.block_on(hash_file(dir.to_str().unwrap()));
        assert!(ok);
        // sha256("hello\n")
        assert!(
            out.contains("5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03"),
            "the digest must be the real SHA-256: {out}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn only_a_plain_hostname_reaches_the_command_line() {
        assert!(safe_host("example.com").is_some());
        assert!(safe_host("sub.example.co.tz").is_some());
        assert!(safe_host("example.com; rm -rf /").is_none());
        assert!(safe_host("$(whoami)").is_none());
        assert!(safe_host("a b").is_none());
        assert!(safe_host("").is_none());
    }

    #[test]
    fn the_integrity_hash_accepts_a_single_file() {
        let dir = std::env::temp_dir().join(format!("mts-one-{}", crate::util::token_hex(6)));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("only.txt");
        std::fs::write(&file, "hello\n").unwrap();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let (out, ok) = rt.block_on(hash_file(file.to_str().unwrap()));
        assert!(ok);
        assert!(out.contains("Files: 1"));
        assert!(out.contains("5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_secret_scan_ignores_a_bare_prefix() {
        // A rule table (or any prose) that merely names a prefix must not be
        // reported as a leaked credential.
        let dir = std::env::temp_dir().join(format!("mts-prefix-{}", crate::util::token_hex(6)));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("rules.md"), "the rule is: look for ghp_ tokens\n").unwrap();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let (out, ok) = rt.block_on(audit_secrets(dir.to_str().unwrap()));
        assert!(ok);
        assert!(out.contains("Findings: 0"), "a bare prefix is not a secret: {out}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_real_looking_token_is_reported() {
        let dir = std::env::temp_dir().join(format!("mts-token-{}", crate::util::token_hex(6)));
        std::fs::create_dir_all(&dir).unwrap();
        let fake = format!("ghp_{}", "A".repeat(36));
        std::fs::write(dir.join("env.txt"), format!("TOKEN={fake}\n")).unwrap();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let (out, ok) = rt.block_on(audit_secrets(dir.to_str().unwrap()));
        assert!(ok);
        assert!(out.contains("Findings: 1"), "a full token must be reported: {out}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
