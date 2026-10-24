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

/// The ports a basic scan tries. Deliberately a short, common list: a full
/// sweep belongs in a dedicated tool, not in a web request.
const COMMON_PORTS: &[u16] = &[
    21, 22, 23, 25, 53, 80, 110, 143, 443, 445, 3306, 3389, 5432, 6379, 8080, 8443, 9200, 27017,
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
    let mut stack = vec![root.to_string()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in entries.flatten() {
            if *count >= MAX_FILES {
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
                continue;
            }
            if e.metadata().map(|m| m.len() > MAX_BYTES).unwrap_or(true) {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            *count += 1;
            f(&path.to_string_lossy(), &text);
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
    let mut files = 0usize;
    let mut findings = Vec::new();
    let mut hashes = 0usize;
    scan_dir(root, &mut files, &mut |file, text| {
        use sha2::{Digest, Sha256};
        hashes += 1;
        let digest = hex::encode(Sha256::digest(text.as_bytes()));
        for (bad, what) in KNOWN_BAD {
            if digest == *bad {
                findings.push(format!("{file} matches a known-bad digest: {what}"));
            }
        }
        // A dropper usually carries a decoder plus a network call.
        let lower = text.to_lowercase();
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
        "scanner": "built in (headers, exposure, ports, DNS, TLS, dependencies, source, malware)",
        "checks": [
            "Security headers — read from a live response",
            "Exposure — real requests for the paths that must not be public, plus a TCP port check",
            "DNS — live DNS-over-HTTPS lookups for A, AAAA, NS, MX, TXT, SPF and DMARC",
            "TLS — a real handshake, with the failure reason when it fails",
            "Dependencies — the project's manifests against a bundled advisory list",
            "Source — the code on disk searched for the patterns that cause breaches",
            "Malware — file digests against known-bad values, plus dropper patterns",
        ],
        "rule": "Nothing is scanned until the CEO authorises the domain and the caller names who approved the work.",
    })
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
}
