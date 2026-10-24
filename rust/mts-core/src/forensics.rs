//! Real tool wrappers: the sandbox tools that shell out to an installed binary.
//!
//! `sandbox.rs` observes a system from the inside — it opens sockets, reads
//! headers and parses files. This module is the other half: it drives the real
//! command-line tools (nmap, sleuthkit, foremost, yara, binwalk, tcpdump) so a
//! security or forensics job returns what the tool actually printed, not a
//! model's guess at what it might have printed.
//!
//! Three rules keep this safe and honest.
//!
//! 1. **Every argument is passed as an argument, never through a shell.** No
//!    caller-supplied string is ever interpolated into a command line, so a
//!    target of `; rm -rf /` is just a hostname nmap cannot resolve.
//! 2. **A network tool still needs scope.** nmap runs only on a domain the CEO
//!    authorised *and* only when the caller names who authorised it, exactly as
//!    the pure-Rust scanners do. Forensics tools work on files the operator
//!    uploaded, so they need no scope check.
//! 3. **A missing binary is reported, not faked.** `toolchain()` says which
//!    tools are installed, and a wrapper whose binary is absent returns a plain
//!    "not installed" message rather than an empty success.

use crate::sandbox;
use serde_json::{json, Value as J};
use std::process::Command;
use std::time::Duration;

/// The tools this module can drive. `bin` is the executable checked for at
/// runtime; `name` and `what` are shown in the admin panel.
const TOOLS: &[(&str, &str, &str)] = &[
    ("nmap", "Nmap", "Port and service scan of an authorised host."),
    ("tcpdump", "tcpdump", "Read a packet capture file and summarise the flows."),
    ("mmls", "SleuthKit mmls", "List the partitions in a disk image."),
    ("fls", "SleuthKit fls", "List the files and deleted entries in an image."),
    ("fsstat", "SleuthKit fsstat", "Report a filesystem's metadata from an image."),
    ("foremost", "Foremost", "Carve files out of an image by their signatures."),
    ("yara", "YARA", "Match a file against a set of rules."),
    ("binwalk", "binwalk", "Find embedded files and code inside a binary."),
    ("sqlite3", "SQLite CLI", "Read a database file directly."),
    ("objdump", "objdump", "Disassemble a binary to inspect its code."),
    ("strings", "strings", "Extract readable text from a binary."),
    ("gpg", "GnuPG", "Verify a detached signature."),
    ("whois", "whois", "Read the registration record of a domain."),
    ("dig", "dig", "Query DNS records directly."),
    ("sha256sum", "sha256sum", "Compute file digests with the system tool."),
];

/// Which of the tools above are actually installed. Reported so a capability is
/// never assumed: the admin panel shows this list, and a wrapper refuses to run
/// when its binary is missing.
pub fn toolchain() -> J {
    let list: Vec<J> = TOOLS
        .iter()
        .map(|(bin, name, what)| {
            json!({ "bin": bin, "name": name, "what": what, "installed": installed(bin) })
        })
        .collect();
    let present = list
        .iter()
        .filter(|t| t.get("installed").and_then(|v| v.as_bool()).unwrap_or(false))
        .count();
    json!({
        "tools": list,
        "installed": present,
        "total": TOOLS.len(),
        "ready": present == TOOLS.len(),
    })
}

/// Is a tool on the PATH? The name comes from the fixed list above, so this is
/// not a place a caller can inject anything.
pub fn installed(bin: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {bin} >/dev/null 2>&1"))
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Run a binary with a hard timeout and no inherited environment, and return its
/// combined output. Arguments are passed positionally — never through a shell —
/// so nothing a caller sends can become a second command.
fn run(bin: &str, args: &[String], timeout_secs: u64) -> Result<String, String> {
    if !installed(bin) {
        return Err(format!(
            "{bin} is not installed on this server. The admin panel lists the tools that are."
        ));
    }
    // `timeout` is the coreutils binary: it kills the child if it overruns, so a
    // scan of a black-holed host cannot hold a request open forever.
    let out = Command::new("timeout")
        .arg(format!("{timeout_secs}"))
        .arg(bin)
        .args(args)
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap_or_else(|_| "/usr/bin:/bin".into()))
        .env("LC_ALL", "C")
        .output()
        .map_err(|e| format!("{bin} could not be started: {e}"))?;

    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    let err = String::from_utf8_lossy(&out.stderr);
    if !err.trim().is_empty() {
        if !text.is_empty() {
            text.push('\n');
        }
        text.push_str(err.trim_end());
    }
    Ok(text.trim_end().to_string())
}

/// Cap the output so a large image or a busy host cannot return megabytes into
/// a JSON response. The tail is summarised rather than silently dropped.
fn clip(text: &str, max: usize) -> String {
    if text.len() <= max {
        return text.to_string();
    }
    let head: String = text.chars().take(max).collect();
    format!("{head}\n… output truncated at {max} characters ({} in total).", text.len())
}

/* ------------------------------ network ------------------------------ */

/// A TCP connect scan with nmap, limited to the top ports and a short timeout.
///
/// `-sT` is a plain connect scan: it needs no root and no raw sockets, which is
/// what lets it run as the unprivileged server account. `-Pn` skips host
/// discovery so a host that ignores ping is still scanned. The scope rule is
/// enforced before the binary is reached.
pub async fn nmap_scan(target: &str, authorised_by: &str) -> (String, bool) {
    let host = match sandbox::scope_check(target, authorised_by) {
        Ok(h) => h,
        Err(e) => return (e, false),
    };
    if host.is_empty() || host.contains(|c: char| c.is_whitespace()) {
        return ("Give a single hostname or IP, with no spaces.".into(), false);
    }
    let args: Vec<String> = [
        "-sT",
        "-Pn",
        "-T4",
        "--top-ports",
        "100",
        "--host-timeout",
        "60s",
        "--max-retries",
        "1",
        "-oN",
        "-",
        &host,
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    match run("nmap", &args, 90) {
        Ok(text) if !text.is_empty() => (
            format!(
                "Nmap connect scan of {host} (top 100 ports)\nAuthorised by: {authorised_by}\n\n{}",
                clip(&text, 20000)
            ),
            true,
        ),
        Ok(_) => (format!("Nmap returned no output for {host}."), false),
        Err(e) => (e, false),
    }
}

/// Read a packet capture file with tcpdump and report the conversations. Local
/// file, so no scope check: the operator supplies the capture.
pub async fn pcap_summary(path: &str) -> (String, bool) {
    let file = path.trim();
    if file.is_empty() {
        return ("Give the path to a .pcap file to read.".into(), false);
    }
    if !std::path::Path::new(file).is_file() {
        return (format!("No capture file at {file}."), false);
    }
    // `-nn` keeps addresses numeric (no reverse DNS), `-r` reads the file.
    let args = vec!["-nn".to_string(), "-r".to_string(), file.to_string()];
    let flows = match run("tcpdump", &args, 30) {
        Ok(t) => t,
        Err(e) => return (e, false),
    };
    if flows.trim().is_empty() {
        return (format!("The capture {file} holds no packets."), false);
    }
    let packets = flows.lines().count();
    let body = format!(
        "Packet capture summary of {file}\nPackets: {packets}\n\n{}",
        clip(&flows, 20000)
    );
    (body, true)
}

/* ------------------------------ forensics ------------------------------ */

/// List the partitions in a disk image with sleuthkit's `mmls`. The building
/// block for every later step: you cannot read a filesystem until you know where
/// it starts.
pub async fn image_partitions(path: &str) -> (String, bool) {
    let file = match evidence(path) {
        Ok(f) => f,
        Err(e) => return (e, false),
    };
    match run("mmls", &[file.clone()], 60) {
        Ok(t) if !t.is_empty() => (
            format!(
                "Partition layout of {file}\nThe offset column is where each filesystem begins; pass it to fls/fsstat with -o.\n\n{}",
                clip(&t, 20000)
            ),
            true,
        ),
        Ok(_) => (format!("mmls found no partition table in {file}."), false),
        Err(e) => (e, false),
    }
}

/// Report a filesystem's metadata with `fsstat`. `offset` is the partition start
/// mmls printed, in sectors; empty means the image is a bare filesystem.
pub async fn image_filesystem(path: &str, offset: &str) -> (String, bool) {
    let file = match evidence(path) {
        Ok(f) => f,
        Err(e) => return (e, false),
    };
    let mut args = Vec::new();
    if !offset.trim().is_empty() {
        args.push("-o".to_string());
        args.push(offset.trim().to_string());
    }
    args.push(file.clone());
    match run("fsstat", &args, 60) {
        Ok(t) if !t.is_empty() => (
            format!("Filesystem metadata of {file}\n\n{}", clip(&t, 20000)),
            true,
        ),
        Ok(_) => (format!("fsstat could not read a filesystem in {file}."), false),
        Err(e) => (e, false),
    }
}

/// List files in an image with `fls`, including deleted entries when `-d` is
/// asked for. A deleted file is often the whole point of an investigation.
pub async fn image_list_files(path: &str, offset: &str, deleted_only: bool) -> (String, bool) {
    let file = match evidence(path) {
        Ok(f) => f,
        Err(e) => return (e, false),
    };
    let mut args = Vec::new();
    if !offset.trim().is_empty() {
        args.push("-o".to_string());
        args.push(offset.trim().to_string());
    }
    if deleted_only {
        args.push("-d".to_string());
    }
    // Recursive listing: the interesting file is rarely in the root directory.
    args.push("-r".to_string());
    args.push("-p".to_string());
    args.push(file.clone());
    match run("fls", &args, 90) {
        Ok(t) if !t.is_empty() => {
            let lines = t.lines().count();
            let deleted = t.lines().filter(|l| l.starts_with("r/r *") || l.contains("(deleted")).count();
            (
                format!(
                    "File listing of {file}\nEntries: {lines}\nDeleted entries: {deleted}\nA leading `*` or `(deleted)` marks a removed file that is still recoverable.\n\n{}",
                    clip(&t, 20000)
                ),
                true,
            )
        }
        Ok(_) => (format!("fls found no files in {file}."), false),
        Err(e) => (e, false),
    }
}

/// Carve files out of an image by signature with `foremost`. This recovers files
/// even when the filesystem metadata is gone — the technique that matters when
/// an attacker has wiped a directory table.
pub async fn carve_files(path: &str, kinds: &str) -> (String, bool) {
    let file = match evidence(path) {
        Ok(f) => f,
        Err(e) => return (e, false),
    };
    // Foremost always writes into an output directory, so give it a fresh one
    // under the volume rather than letting it drop into the working directory.
    let out_dir = std::env::temp_dir().join(format!("mts-carve-{}", crate::util::token_hex(6)));
    if let Err(e) = std::fs::create_dir_all(&out_dir) {
        return (format!("Could not create a carve directory: {e}"), false);
    }
    let types = if kinds.trim().is_empty() { "all" } else { kinds.trim() };
    let args = vec![
        "-t".to_string(),
        types.to_string(),
        "-q".to_string(),
        "-o".to_string(),
        out_dir.to_string_lossy().to_string(),
        file.clone(),
    ];
    let result = run("foremost", &args, 180);
    // Count what was recovered, then clean the scratch directory up.
    let mut recovered = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&out_dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                if let Ok(files) = std::fs::read_dir(&p) {
                    let n = files.flatten().count();
                    if n > 0 {
                        recovered.push(format!("{}: {n} file(s)", e.file_name().to_string_lossy()));
                    }
                }
            }
        }
    }
    let _ = std::fs::remove_dir_all(&out_dir);
    match result {
        Ok(log) => (
            format!(
                "File carving of {file} (types: {types})\nRecovered: {}\n\n{}",
                if recovered.is_empty() { "nothing matched the chosen signatures".to_string() } else { recovered.join(", ") },
                clip(&log, 8000)
            ),
            true,
        ),
        Err(e) => (e, false),
    }
}

/// Scan a file or directory against YARA rules. `rules` is a path to a rule file
/// the operator supplied; with no rules the tool cannot do anything useful, so
/// that is refused rather than reported as a clean result.
pub async fn yara_scan(path: &str, rules: &str) -> (String, bool) {
    let target = match evidence_or_dir(path) {
        Ok(t) => t,
        Err(e) => return (e, false),
    };
    let rule_file = rules.trim();
    if rule_file.is_empty() {
        return ("Give the path to a YARA rule file to match against.".into(), false);
    }
    if !std::path::Path::new(rule_file).is_file() {
        return (format!("No YARA rule file at {rule_file}."), false);
    }
    let args = vec![
        "-r".to_string(),
        "-f".to_string(),
        rule_file.to_string(),
        target.clone(),
    ];
    match run("yara", &args, 120) {
        Ok(t) => {
            let hits = t.lines().filter(|l| !l.trim().is_empty()).count();
            (
                format!(
                    "YARA scan of {target} against {rule_file}\nMatches: {hits}\n\n{}",
                    if hits == 0 { "No rule matched.".to_string() } else { clip(&t, 20000) }
                ),
                true,
            )
        }
        Err(e) => (e, false),
    }
}

/// Look inside a binary for embedded files and code with `binwalk`. Useful on a
/// firmware image or a document that hides a second payload.
pub async fn embedded_scan(path: &str) -> (String, bool) {
    let file = match evidence(path) {
        Ok(f) => f,
        Err(e) => return (e, false),
    };
    match run("binwalk", &[file.clone()], 120) {
        Ok(t) if !t.is_empty() => (
            format!(
                "Embedded-content scan of {file}\nEach line names a signature binwalk recognised at that offset.\n\n{}",
                clip(&t, 20000)
            ),
            true,
        ),
        Ok(_) => (format!("binwalk found no embedded content in {file}."), false),
        Err(e) => (e, false),
    }
}

/// Disassemble a binary with `objdump` so its code can be inspected. The
/// section headers come first: they say what kind of binary it is before a
/// single instruction is read.
pub async fn disassemble(path: &str) -> (String, bool) {
    let file = match evidence(path) {
        Ok(f) => f,
        Err(e) => return (e, false),
    };
    let headers = run("objdump", &["-h".to_string(), file.clone()], 60).unwrap_or_default();
    let text = run("strings", &["-n".to_string(), "6".to_string(), file.clone()], 60)
        .unwrap_or_default();
    if headers.trim().is_empty() && text.trim().is_empty() {
        return (format!("objdump and strings could not read {file}."), false);
    }
    let body = format!(
        "Binary inspection of {file}\n\n== Section headers ==\n{}\n\n== Readable strings (min length 6) ==\n{}",
        clip(&headers, 8000),
        clip(&text, 12000)
    );
    (body, true)
}

/// Read a SQLite database file with the real sqlite3 tool: its tables, then a
/// row count for each. This is the honest way to inspect a database file that
/// arrived as evidence rather than through the platform's own connection.
pub async fn inspect_database(path: &str) -> (String, bool) {
    let file = match evidence(path) {
        Ok(f) => f,
        Err(e) => return (e, false),
    };
    let tables = match run(
        "sqlite3",
        &[file.clone(), ".tables".to_string()],
        30,
    ) {
        Ok(t) => t,
        Err(e) => return (e, false),
    };
    if tables.trim().is_empty() {
        return (format!("{file} is a SQLite database with no tables."), false);
    }
    let mut report = format!("Database file {file}\nTables: {}\n\n", tables.trim());
    for name in tables.split_whitespace().take(50) {
        if name.contains(|c: char| !c.is_alphanumeric() && c != '_') {
            continue;
        }
        let q = format!("SELECT COUNT(*) FROM \"{name}\";");
        let n = run("sqlite3", &[file.clone(), q], 20).unwrap_or_default();
        report.push_str(&format!("  {name}: {} row(s)\n", n.trim()));
    }
    (clip(&report, 20000), true)
}

/// Read a domain's registration record with `whois`, and its nameservers and
/// mail records with `dig`. Registration is public, so no scope check applies;
/// the point is to read the record the registry actually holds.
pub async fn domain_record(target: &str) -> (String, bool) {
    let host = sandbox::host_of(target);
    if host.is_empty() {
        return ("Give the domain to look up.".into(), false);
    }
    let mut report = format!("Registration and DNS record of {host}\n\n");
    match run("whois", &[host.clone()], 30) {
        Ok(t) if !t.is_empty() => {
            // whois output is long; the registrar, dates and status lines are
            // what matter, so keep those and the first block of the rest.
            let keep: Vec<&str> = t
                .lines()
                .filter(|l| {
                    let low = l.to_lowercase();
                    low.contains("registrar")
                        || low.contains("creation")
                        || low.contains("expir")
                        || low.contains("updated")
                        || low.contains("status")
                        || low.contains("name server")
                        || low.contains("nameserver")
                })
                .take(40)
                .collect();
            report.push_str("== whois (registrar, dates, status, nameservers) ==\n");
            let block = if keep.is_empty() {
                clip(&t, 6000)
            } else {
                keep.join("\n")
            };
            report.push_str(&block);
            report.push('\n');
        }
        Ok(_) => report.push_str("== whois ==\nNo registration record was returned.\n"),
        Err(e) => report.push_str(&format!("== whois ==\n{e}\n")),
    }
    for kind in ["NS", "MX", "TXT"] {
        if let Ok(t) = run("dig", &["+short".to_string(), host.clone(), kind.to_string()], 20) {
            if !t.trim().is_empty() {
                report.push_str(&format!("\n== {kind} ==\n{}\n", clip(&t, 2000)));
            }
        }
    }
    (clip(&report, 20000), true)
}

/* ------------------------------ helpers ------------------------------ */

/// Resolve an evidence path: it must be an existing regular file. The tools read
/// whatever the operator placed on the volume, so the path is not otherwise
/// restricted — but it must exist, or the tool would report a confusing error.
fn evidence(path: &str) -> Result<String, String> {
    let p = path.trim();
    if p.is_empty() {
        return Err("Give the path to the image or file to examine.".into());
    }
    if !std::path::Path::new(p).is_file() {
        return Err(format!(
            "No file at {p}. Upload the evidence to the server's volume first."
        ));
    }
    Ok(p.to_string())
}

/// Like `evidence`, but also accepts a directory — for a YARA scan over a tree.
fn evidence_or_dir(path: &str) -> Result<String, String> {
    let p = path.trim();
    if p.is_empty() {
        return Err("Give the path to scan.".into());
    }
    if !std::path::Path::new(p).exists() {
        return Err(format!("Nothing at {p}."));
    }
    Ok(p.to_string())
}

/// A hard ceiling on how long any single wrapper may hold a request, exposed so
/// the API can say what it promises.
pub const MAX_RUNTIME: Duration = Duration::from_secs(180);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_toolchain_report_names_every_tool() {
        let t = toolchain();
        let list = t.get("tools").and_then(|v| v.as_array()).unwrap();
        assert_eq!(list.len(), TOOLS.len());
        assert!(list.iter().all(|x| x.get("installed").is_some()));
    }

    #[test]
    fn a_missing_binary_is_reported_not_faked() {
        // A name that is certainly not installed, so the guard is exercised.
        let err = run("definitely-not-a-real-binary-xyz", &[], 5).unwrap_err();
        assert!(err.contains("not installed"));
    }

    #[test]
    fn output_is_clipped_with_a_visible_marker() {
        let long = "x".repeat(500);
        let c = clip(&long, 100);
        assert!(c.contains("truncated"));
        assert!(c.len() < long.len());
        // Short output is returned untouched.
        assert_eq!(clip("short", 100), "short");
    }

    #[test]
    fn an_evidence_path_must_exist() {
        assert!(evidence("").unwrap_err().contains("Give the path"));
        assert!(evidence("/no/such/evidence/file.dd")
            .unwrap_err()
            .contains("No file at"));
    }

    #[test]
    fn a_network_tool_still_needs_scope() {
        // No authorised domain is configured in the test database, so nmap must
        // refuse before it ever runs a scan.
        crate::test_bootstrap();
        let (msg, ok) = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(nmap_scan("example.com", "the client"));
        assert!(!ok);
        assert!(msg.contains("not authorised") || msg.contains("authorised"));
    }

    #[test]
    fn nmap_refuses_without_an_authoriser() {
        crate::test_bootstrap();
        let (msg, ok) = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(nmap_scan("example.com", ""));
        assert!(!ok);
        assert!(msg.contains("authorised"));
    }

    /// A real sqlite file, read back by the real `sqlite3` tool. This is the
    /// test that proves "database forensics" is not a label: the tool prints the
    /// tables and row counts it read from the file. Skipped when the binary is
    /// absent, so the suite still passes on a bare machine.
    #[test]
    fn database_forensics_reads_a_real_file() {
        if !installed("sqlite3") {
            return;
        }
        let dir = std::env::temp_dir().join(format!("mts-dbf-{}", crate::util::token_hex(6)));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("evidence.sqlite");
        {
            let conn = rusqlite::Connection::open(&path).unwrap();
            conn.execute_batch(
                "CREATE TABLE users(id INTEGER PRIMARY KEY, name TEXT);
                 INSERT INTO users(name) VALUES ('alice'),('bob'),('carol');",
            )
            .unwrap();
        }
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let (out, ok) = rt.block_on(inspect_database(path.to_str().unwrap()));
        let _ = std::fs::remove_dir_all(&dir);
        assert!(ok, "sqlite3 should have read the file: {out}");
        assert!(out.contains("users"), "the table name must appear: {out}");
        assert!(out.contains("3 row"), "the real row count must appear: {out}");
    }

    /// A real packet capture, read back by the real `tcpdump`.
    #[test]
    fn packet_review_reads_a_real_capture() {
        if !installed("tcpdump") {
            return;
        }
        let dir = std::env::temp_dir().join(format!("mts-pcap-{}", crate::util::token_hex(6)));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("capture.pcap");
        // A one-packet capture: UDP from 10.0.0.5 to 10.0.0.6:53.
        let mut f = Vec::new();
        f.extend_from_slice(&0xa1b2c3d4u32.to_le_bytes());
        f.extend_from_slice(&2u16.to_le_bytes());
        f.extend_from_slice(&4u16.to_le_bytes());
        f.extend_from_slice(&0i32.to_le_bytes());
        f.extend_from_slice(&0u32.to_le_bytes());
        f.extend_from_slice(&65535u32.to_le_bytes());
        f.extend_from_slice(&1u32.to_le_bytes());
        let payload = b"testdns1";
        let mut udp = Vec::new();
        udp.extend_from_slice(&40000u16.to_be_bytes());
        udp.extend_from_slice(&53u16.to_be_bytes());
        udp.extend_from_slice(&((8 + payload.len()) as u16).to_be_bytes());
        udp.extend_from_slice(&0u16.to_be_bytes());
        udp.extend_from_slice(payload);
        let mut ip = vec![0x45, 0, 0, 0, 0, 0, 0, 0, 64, 17, 0, 0];
        ip.extend_from_slice(&[10, 0, 0, 5]);
        ip.extend_from_slice(&[10, 0, 0, 6]);
        let total = (ip.len() + udp.len()) as u16;
        ip[2] = (total >> 8) as u8;
        ip[3] = total as u8;
        let mut pkt = vec![0xff; 6];
        pkt.extend_from_slice(&[0, 0x11, 0x22, 0x33, 0x44, 0x55]);
        pkt.extend_from_slice(&[0x08, 0x00]);
        pkt.extend_from_slice(&ip);
        pkt.extend_from_slice(&udp);
        f.extend_from_slice(&1u32.to_le_bytes());
        f.extend_from_slice(&0u32.to_le_bytes());
        f.extend_from_slice(&(pkt.len() as u32).to_le_bytes());
        f.extend_from_slice(&(pkt.len() as u32).to_le_bytes());
        f.extend_from_slice(&pkt);
        std::fs::write(&path, &f).unwrap();

        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let (out, ok) = rt.block_on(pcap_summary(path.to_str().unwrap()));
        let _ = std::fs::remove_dir_all(&dir);
        assert!(ok, "tcpdump should have read the capture: {out}");
        assert!(out.contains("10.0.0.5"), "the real source address must appear: {out}");
        assert!(out.contains(".53"), "the real destination port must appear: {out}");
    }

    /// YARA matching against a rule we write, on a file we write. A match is
    /// proof the rule engine ran, not that a string was searched for in Rust.
    #[test]
    fn yara_matches_a_real_rule() {
        if !installed("yara") {
            return;
        }
        let dir = std::env::temp_dir().join(format!("mts-yara-{}", crate::util::token_hex(6)));
        std::fs::create_dir_all(&dir).unwrap();
        let rule = dir.join("r.yar");
        std::fs::write(
            &rule,
            "rule Marker\n{\n  strings:\n    $a = \"MALWARE_TEST_MARKER\" ascii\n  condition:\n    $a\n}\n",
        )
        .unwrap();
        let target = dir.join("sample.bin");
        std::fs::write(&target, b"MALWARE_TEST_MARKER inside\n").unwrap();
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let (out, ok) = rt.block_on(yara_scan(
            target.to_str().unwrap(),
            rule.to_str().unwrap(),
        ));
        let _ = std::fs::remove_dir_all(&dir);
        assert!(ok, "yara should have run: {out}");
        assert!(out.contains("Matches: 1"), "the rule must match once: {out}");
    }

    /// A real ext4 image built with mke2fs, read back by sleuthkit's mmls/fls.
    /// This is the heaviest wrapper, so it is the one most worth proving. It is
    /// skipped wherever mke2fs or the sleuthkit tools are missing.
    #[test]
    fn disk_forensics_reads_a_real_image() {
        if !installed("mke2fs") || !installed("fls") || !installed("fsstat") {
            return;
        }
        let dir = std::env::temp_dir().join(format!("mts-img-{}", crate::util::token_hex(6)));
        let src = dir.join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("secret.txt"), b"evidence file contents\n").unwrap();
        let img = dir.join("disk.img");
        {
            let f = std::fs::File::create(&img).unwrap();
            f.set_len(8 * 1024 * 1024).unwrap();
        }
        let built = Command::new("mke2fs")
            .args(["-q", "-t", "ext4", "-d"])
            .arg(&src)
            .arg("-F")
            .arg(&img)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if !built {
            let _ = std::fs::remove_dir_all(&dir);
            return;
        }
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        // A bare filesystem (no partition table) is read with offset 0.
        let (fs, fs_ok) = rt.block_on(image_filesystem(img.to_str().unwrap(), ""));
        let (files, fls_ok) = rt.block_on(image_list_files(img.to_str().unwrap(), "", false));
        let _ = std::fs::remove_dir_all(&dir);
        assert!(fs_ok, "fsstat should have read the image: {fs}");
        assert!(
            fs.to_lowercase().contains("ext"),
            "the filesystem type must be reported: {fs}"
        );
        assert!(fls_ok, "fls should have listed the image: {files}");
        assert!(files.contains("secret.txt"), "the real file must be listed: {files}");
    }

    #[test]
    fn an_empty_path_is_refused_before_a_tool_runs() {
        assert!(evidence("").unwrap_err().contains("Give the path"));
        assert!(evidence_or_dir("").unwrap_err().contains("Give the path"));
    }

    #[test]
    fn yara_needs_a_rule_file() {
        let dir = std::env::temp_dir().join(format!("mts-y2-{}", crate::util::token_hex(6)));
        std::fs::create_dir_all(&dir).unwrap();
        let target = dir.join("t.bin");
        std::fs::write(&target, b"x").unwrap();
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let (msg, ok) = rt.block_on(yara_scan(target.to_str().unwrap(), ""));
        let _ = std::fs::remove_dir_all(&dir);
        assert!(!ok);
        assert!(msg.contains("rule file"));
    }
}
