//! Moving the platform between hosts: export, import, verify and the plan.
//!
//! A deployment is three things: the code (in git), the database (on the
//! volume), and the operator-supplied evidence files. The code moves with a
//! `git pull`; this module moves the other two. It produces one portable bundle
//! — a gzipped tar holding a consistent SQLite snapshot, the evidence tree, and
//! a manifest — so "move to another VPS" is a download and an upload, not a
//! hand-run of mysqldump and scp.
//!
//! The snapshot is taken with SQLite's own `VACUUM INTO`, which writes a
//! consistent copy while the server keeps serving. The bundle holds secrets
//! (the provider key, SMTP password), so every action here is admin-only and a
//! bundle is never placed anywhere the static file server can reach.

use crate::api::ctx::Ctx;
use crate::db;
use crate::settings;
use crate::util::{self, bad, err, ok};
use axum::body::Body;
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use serde_json::{json, Value as J};
use std::path::{Path, PathBuf};

/// Where bundles are written. Inside the data volume, which the static file
/// server does not serve, so a bundle is never downloadable by URL guess.
fn backup_dir() -> PathBuf {
    let db_path = std::env::var("MTS_DB").unwrap_or_else(|_| "storage/mts.sqlite".to_string());
    let parent = Path::new(&db_path)
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    parent.join("backups")
}

/// The evidence directory: forensics files the operator placed on the volume.
fn evidence_dir() -> PathBuf {
    let db_path = std::env::var("MTS_DB").unwrap_or_else(|_| "storage/mts.sqlite".to_string());
    let parent = Path::new(&db_path)
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    parent.join("evidence")
}

/// The platform version, so a bundle says which build produced it.
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// A filesystem mtime as ISO-8601, so a bundle listing is readable in the UI.
fn iso_from_system(t: std::time::SystemTime) -> String {
    let dt: chrono::DateTime<chrono::Local> = t.into();
    dt.format("%Y-%m-%dT%H:%M:%S").to_string()
}

/* ------------------------------ status ------------------------------ */

/// What is on this host, so an operator can see what a move would carry.
pub fn status() -> J {
    let db_path = std::env::var("MTS_DB").unwrap_or_else(|_| "storage/mts.sqlite".to_string());
    let tables = db::scalar_i64(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table'",
        &[],
    );
    let users = db::scalar_i64("SELECT COUNT(*) FROM users", &[]);
    let requests = db::scalar_i64("SELECT COUNT(*) FROM requests", &[]);
    let db_bytes = std::fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0);
    let evidence = evidence_files();
    json!({
        "version": version(),
        "dbPath": db_path,
        "dbBytes": db_bytes,
        "tables": tables,
        "users": users,
        "requests": requests,
        "evidenceFiles": evidence.len(),
        "evidenceBytes": evidence.iter().map(|(_, n)| *n).sum::<u64>(),
        "backups": list_bundles(),
        "ready": true,
    })
}

/// Every regular file under the evidence directory, with its size.
fn evidence_files() -> Vec<(String, u64)> {
    let root = evidence_dir();
    let mut out = Vec::new();
    walk(&root, &mut |p| {
        if let Ok(m) = std::fs::metadata(p) {
            if m.is_file() {
                out.push((p.to_string_lossy().to_string(), m.len()));
            }
        }
    });
    out
}

fn walk(dir: &Path, f: &mut impl FnMut(&Path)) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            walk(&p, f);
        } else {
            f(&p);
        }
    }
}

/// The bundles already on the volume, newest first.
pub fn list_bundles() -> Vec<J> {
    let dir = backup_dir();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut out: Vec<J> = entries
        .flatten()
        .filter_map(|e| {
            let p = e.path();
            if p.extension().and_then(|s| s.to_str()) != Some("gz") {
                return None;
            }
            let m = std::fs::metadata(&p).ok()?;
            Some(json!({
                "name": p.file_name()?.to_string_lossy(),
                "bytes": m.len(),
                "created": m.modified().ok().map(iso_from_system).unwrap_or_default(),
            }))
        })
        .collect();
    out.sort_by(|a, b| {
        b.get("created")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .cmp(a.get("created").and_then(|v| v.as_str()).unwrap_or(""))
    });
    out
}

/* ------------------------------ export ------------------------------ */

/// Take a consistent snapshot of the database with `VACUUM INTO`.
fn snapshot_db(dest: &Path) -> Result<(), String> {
    let dest_s = dest.to_string_lossy().to_string();
    // VACUUM INTO fails if the destination exists, so clear it first.
    let _ = std::fs::remove_file(dest);
    db::with_db(|c| {
        c.execute("VACUUM INTO ?1", rusqlite::params![dest_s])
            .map(|_| ())
    })
    .map_err(|e| format!("could not snapshot the database: {e}"))?;
    Ok(())
}

/// Build a bundle and return its name, size and digest.
pub fn export_bundle() -> Result<J, String> {
    let dir = backup_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;

    let stamp = util::now().replace([':', '-', ' '], "");
    let name = format!("mts-bundle-{stamp}.tar.gz");
    let out_path = dir.join(&name);

    // Snapshot the database next to the bundle, then fold it in and remove it.
    let snap = dir.join(format!(".snapshot-{stamp}.sqlite"));
    snapshot_db(&snap)?;

    let manifest = json!({
        "product": "Mbilinyi Tech Solutions",
        "version": version(),
        "created": util::now(),
        "dbFile": "mts.sqlite",
        "tables": db::scalar_i64("SELECT COUNT(*) FROM sqlite_master WHERE type = 'table'", &[]),
        "users": db::scalar_i64("SELECT COUNT(*) FROM users", &[]),
        "settingsKeys": settings::all().len(),
        "note": "Restore with the Migration tab, or `scripts/migrate.sh import <bundle>`.",
    });

    let file = std::fs::File::create(&out_path)
        .map_err(|e| format!("cannot write {}: {e}", out_path.display()))?;
    let mut gz = flate2::write::GzEncoder::new(file, flate2::Compression::default());
    {
        let mut tar = tar::Builder::new(&mut gz);

        // The database snapshot.
        let mut f = std::fs::File::open(&snap).map_err(|e| format!("snapshot unreadable: {e}"))?;
        let mut header = tar::Header::new_gnu();
        header.set_size(std::fs::metadata(&snap).map(|m| m.len()).unwrap_or(0));
        header.set_mode(0o600);
        header.set_cksum();
        tar.append_data(&mut header, "mts.sqlite", &mut f)
            .map_err(|e| format!("cannot add the database to the bundle: {e}"))?;

        // The manifest.
        let body = serde_json::to_vec_pretty(&manifest).unwrap_or_default();
        let mut header = tar::Header::new_gnu();
        header.set_size(body.len() as u64);
        header.set_mode(0o600);
        header.set_cksum();
        tar.append_data(&mut header, "manifest.json", body.as_slice())
            .map_err(|e| format!("cannot add the manifest: {e}"))?;

        // Every evidence file, under evidence/ so it unpacks in one place.
        for (path, size) in evidence_files() {
            let Ok(mut f) = std::fs::File::open(&path) else {
                continue;
            };
            let rel = Path::new(&path)
                .strip_prefix(evidence_dir())
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| {
                    Path::new(&path)
                        .file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default()
                });
            let mut header = tar::Header::new_gnu();
            header.set_size(size);
            header.set_mode(0o600);
            header.set_cksum();
            let _ = tar.append_data(&mut header, format!("evidence/{rel}"), &mut f);
        }
        tar.finish()
            .map_err(|e| format!("cannot finish the bundle: {e}"))?;
    }
    gz.finish()
        .map_err(|e| format!("cannot finish the compression: {e}"))?;

    let _ = std::fs::remove_file(&snap);

    let bytes = std::fs::metadata(&out_path).map(|m| m.len()).unwrap_or(0);
    let sha = sha256_file(&out_path);
    Ok(json!({ "name": name, "bytes": bytes, "sha256": sha, "manifest": manifest }))
}

/// The SHA-256 of a file, as hex. Used to verify a bundle before importing it.
pub fn sha256_file(path: &Path) -> String {
    use sha2::{Digest, Sha256};
    match std::fs::read(path) {
        Ok(b) => hex::encode(Sha256::digest(&b)),
        Err(_) => String::new(),
    }
}

/* ------------------------------ verify ------------------------------ */

/// Check that a bundle exists, is a readable gzip tar, and holds a database.
pub fn verify_bundle(name: &str) -> Result<J, String> {
    let path = safe_bundle_path(name)?;
    if !path.is_file() {
        return Err(format!("No bundle named {name}."));
    }
    let file = std::fs::File::open(&path).map_err(|e| format!("cannot open the bundle: {e}"))?;
    let gz = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(gz);
    let mut entries: Vec<String> = Vec::new();
    let mut has_db = false;
    let mut manifest = J::Null;
    let Ok(iter) = archive.entries() else {
        return Err("The bundle is not a readable tar archive.".into());
    };
    for entry in iter.flatten() {
        let Ok(p) = entry.path() else { continue };
        let s = p.to_string_lossy().to_string();
        if s == "mts.sqlite" {
            has_db = true;
        }
        if s == "manifest.json" {
            let mut text = String::new();
            let mut e = entry;
            use std::io::Read;
            if e.read_to_string(&mut text).is_ok() {
                manifest = serde_json::from_str(&text).unwrap_or(J::Null);
            }
        }
        entries.push(s);
    }
    if !has_db {
        return Err("The bundle has no database inside it.".into());
    }
    Ok(json!({
        "name": name,
        "valid": true,
        "sha256": sha256_file(&path),
        "bytes": std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0),
        "entries": entries,
        "manifest": manifest,
    }))
}

/// Resolve a bundle name to a path, refusing anything that could escape the
/// backup directory. A name with a slash or `..` is never a bundle.
fn safe_bundle_path(name: &str) -> Result<PathBuf, String> {
    let n = name.trim();
    if n.is_empty()
        || n.contains('/')
        || n.contains('\\')
        || n.contains("..")
        || !n.ends_with(".tar.gz")
    {
        return Err("That is not a bundle name.".into());
    }
    Ok(backup_dir().join(n))
}

/* ------------------------------ import ------------------------------ */

/// Restore a bundle: replace the database and the evidence tree. The running
/// server notices the replaced file on its next query (`db::ensure_current`)
/// and rebuilds its connection, so no restart is needed.
pub fn import_bundle(name: &str) -> Result<J, String> {
    let path = safe_bundle_path(name)?;
    if !path.is_file() {
        return Err(format!("No bundle named {name}."));
    }
    let file = std::fs::File::open(&path).map_err(|e| format!("cannot open the bundle: {e}"))?;
    let gz = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(gz);

    let db_path = std::env::var("MTS_DB").unwrap_or_else(|_| "storage/mts.sqlite".to_string());
    let staging = backup_dir().join(".import-staging.sqlite");
    let _ = std::fs::remove_file(&staging);

    let mut restored_db = false;
    let mut evidence: Vec<String> = Vec::new();
    let ev_root = evidence_dir();
    std::fs::create_dir_all(&ev_root)
        .map_err(|e| format!("cannot create the evidence dir: {e}"))?;

    let Ok(iter) = archive.entries() else {
        return Err("The bundle is not a readable tar archive.".into());
    };
    for entry in iter.flatten() {
        let Ok(p) = entry.path() else { continue };
        let s = p.to_string_lossy().to_string();
        // Every path is validated: a bundle from elsewhere must not be able to
        // write outside the two directories this restores into.
        if s.contains("..") || s.starts_with('/') || s.contains('\\') {
            return Err(format!("The bundle holds an unsafe path: {s}"));
        }
        let mut e = entry;
        if s == "mts.sqlite" {
            let mut out = std::fs::File::create(&staging)
                .map_err(|e| format!("cannot write the staged database: {e}"))?;
            std::io::copy(&mut e, &mut out)
                .map_err(|e| format!("cannot read the database: {e}"))?;
            restored_db = true;
        } else if let Some(rel) = s.strip_prefix("evidence/") {
            let dest = ev_root.join(rel);
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            let mut out =
                std::fs::File::create(&dest).map_err(|e| format!("cannot write {rel}: {e}"))?;
            std::io::copy(&mut e, &mut out).map_err(|e| format!("cannot read {rel}: {e}"))?;
            evidence.push(rel.to_string());
        }
        // manifest.json and anything else are ignored on purpose.
    }

    if !restored_db {
        let _ = std::fs::remove_file(&staging);
        return Err("The bundle has no database inside it.".into());
    }

    // Replace the live database. Writing over the path gives a new inode, which
    // is exactly what `ensure_current` watches for, so the server picks it up.
    std::fs::copy(&staging, &db_path).map_err(|e| format!("cannot install the database: {e}"))?;
    let _ = std::fs::remove_file(&staging);
    // A stale WAL from the old file must not be replayed onto the new one.
    let _ = std::fs::remove_file(format!("{db_path}-wal"));
    let _ = std::fs::remove_file(format!("{db_path}-shm"));

    let users = db::scalar_i64("SELECT COUNT(*) FROM users", &[]);
    Ok(json!({
        "restored": true,
        "name": name,
        "database": db_path,
        "users": users,
        "evidenceRestored": evidence.len(),
        "evidence": evidence,
        "note": "The server has already switched to the restored database. Reload the portal.",
    }))
}

/* ------------------------------ the plan ------------------------------ */

/// The concrete steps to move the platform to another host. This is the answer
/// to "if we move to another VPS, how do we update it afterwards?" — the plan
/// names the exact commands, not a vague description.
pub fn plan(target: &str) -> J {
    let t = target.trim().to_lowercase();
    let (name, steps, update) = match t.as_str() {
        "fly" | "fly.io" => (
            "Fly.io",
            vec![
                "Install flyctl and log in: `fly auth login` (or export FLY_API_TOKEN).",
                "The app already exists; the image is built from the Dockerfile.",
                "Deploy: `fly deploy --remote-only --yes`.",
                "The volume `mts_data` mounts at /data, so the database survives a deploy.",
                "Put a bundle on the volume: `fly ssh sftp put mts-bundle.tar.gz /data/backups/`.",
                "Open Admin -> Migration and Import it, or run `fly ssh console -C '/app/mts-core migrate import <name>'`.",
            ],
            "After an improvement: `git pull && fly deploy`. The volume is untouched, so no data step is needed.",
        ),
        "vps" | "docker" | "hetzner" | "contabo" | "digitalocean" | "aws" | "ec2" => (
            "A plain VPS with Docker",
            vec![
                "Install Docker on the VPS: `curl -fsSL https://get.docker.com | sh`.",
                "Clone the code: `git clone <repo> && cd MBILINYI-TECH67`.",
                "Build the image: `docker build -t mts-core .`.",
                "Run it with a volume for the data: `docker run -d --name mts -p 80:8080 -v /srv/mts:/data -e MTS_DB=/data/mts.sqlite -e MTS_OTP_PEPPER=... mts-core`.",
                "Copy a bundle onto the host and place it in /srv/mts/backups/.",
                "Open Admin -> Migration and Import it, or run `docker exec mts /app/mts-core migrate import <name>`.",
                "Point the domain's A record at the VPS IP, then add TLS with Caddy or certbot.",
            ],
            "After an improvement: `git pull && docker build -t mts-core . && docker rm -f mts && docker run ...`. The /srv/mts volume holds the data, so nothing is lost.",
        ),
        "cpanel" | "php" | "shared" => (
            "A cPanel host running the PHP shell",
            vec![
                "Clone the repository and point the document root at it.",
                "No build step: the PHP pages work on their own and use SQLite when MySQL is absent.",
                "Copy the bundle's mts.sqlite into storage/mts.sqlite.",
                "Copy the bundle's evidence/ into storage/evidence/.",
                "Open the site once; the schema is created on the first request.",
            ],
            "After an improvement: `git pull`. PHP needs no build; clear the page cache if one is on.",
        ),
        _ => (
            "Any host that runs Docker",
            vec![
                "The Dockerfile is the whole deployment contract: build it, run it, give it a volume at /data.",
                "Choose a target: `fly`, `vps` or `cpanel` for the exact commands.",
            ],
            "After an improvement: rebuild and restart, keeping the /data volume.",
        ),
    };

    json!({
        "target": if t.is_empty() { "docker".to_string() } else { t },
        "name": name,
        "steps": steps,
        "updateAfterImprovement": update,
        "targets": ["fly", "vps", "cpanel"],
        "whatMoves": [
            "The code — from git, so `git pull` is the whole step.",
            "The database — one consistent SQLite snapshot inside the bundle.",
            "The evidence files — the images and captures the forensics tools read.",
            "The settings and secrets — inside the database, which is why a bundle is admin-only.",
        ],
        "whatDoesNotMove": [
            "The Docker image — every host builds it from the same Dockerfile.",
            "TLS certificates — issued per host (Fly does it, or certbot on a VPS).",
        ],
    })
}

/* ------------------------------ the endpoints ------------------------------ */

pub async fn handle(c: &Ctx) -> Response {
    match c.action().as_str() {
        "status" => {
            if let Err(e) = c.require_admin() {
                return e;
            }
            ok(json!({ "status": status(), "plan": plan("docker") }))
        }
        "plan" => {
            if let Err(e) = c.require_admin() {
                return e;
            }
            ok(plan(&c.p("target")))
        }
        "export" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            match export_bundle() {
                Ok(b) => ok(json!({ "exported": true, "bundle": b })),
                Err(e) => bad(&e),
            }
        }
        "verify" => {
            if let Err(e) = c.require_admin() {
                return e;
            }
            match verify_bundle(&c.p("name")) {
                Ok(v) => ok(v),
                Err(e) => bad(&e),
            }
        }
        "import" => {
            if let Err(e) = c.require_csrf() {
                return e;
            }
            if let Err(e) = c.require_admin() {
                return e;
            }
            match import_bundle(&c.p("name")) {
                Ok(v) => ok(v),
                Err(e) => bad(&e),
            }
        }
        "list" => {
            if let Err(e) = c.require_admin() {
                return e;
            }
            ok(json!({ "bundles": list_bundles() }))
        }
        "download" => download(c),
        _ => bad(
            "Choose a migration action: status, plan, export, verify, import, list or download.",
        ),
    }
}

/// Serve a bundle as a download. Admin-only, and the name is validated, so a
/// path can never be walked. A bundle holds the platform's secrets, so this is
/// deliberately the only way to fetch one and it is never a static file.
fn download(c: &Ctx) -> Response {
    if let Err(e) = c.require_admin() {
        return e;
    }
    let name = c.p("name");
    let path = match safe_bundle_path(&name) {
        Ok(p) => p,
        Err(e) => return err(&e, 400),
    };
    match std::fs::read(&path) {
        Ok(bytes) => {
            let mut res = Response::new(Body::from(bytes));
            res.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/gzip"),
            );
            if let Ok(v) = HeaderValue::from_str(&format!(
                "attachment; filename=\"{}\"",
                name.replace('"', "")
            )) {
                res.headers_mut().insert(header::CONTENT_DISPOSITION, v);
            }
            res.headers_mut()
                .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
            res
        }
        Err(_) => (StatusCode::NOT_FOUND, "Bundle not found").into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bundle_name_cannot_escape_the_backup_directory() {
        for bad_name in [
            "../etc/passwd",
            "a/b.tar.gz",
            "..\\win.tar.gz",
            "",
            "plain.txt",
        ] {
            assert!(
                safe_bundle_path(bad_name).is_err(),
                "{bad_name:?} should be refused"
            );
        }
        assert!(safe_bundle_path("mts-bundle-2026.tar.gz").is_ok());
    }

    #[test]
    fn the_plan_names_the_real_commands_for_each_target() {
        let v = plan("vps");
        let steps = v["steps"].as_array().unwrap();
        assert!(steps
            .iter()
            .any(|s| s.as_str().unwrap().contains("docker run")));
        assert!(v["updateAfterImprovement"]
            .as_str()
            .unwrap()
            .contains("docker"));

        let f = plan("fly");
        assert!(f["steps"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s.as_str().unwrap().contains("fly deploy")));

        let c = plan("cpanel");
        assert!(c["steps"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s.as_str().unwrap().contains("mts.sqlite")));
    }

    #[test]
    fn an_unknown_target_still_returns_a_usable_plan() {
        let v = plan("something-else");
        assert_eq!(v["name"], "Any host that runs Docker");
        assert!(!v["steps"].as_array().unwrap().is_empty());
    }

    #[test]
    fn the_status_reports_the_real_counts() {
        crate::test_bootstrap();
        let s = status();
        assert!(s["tables"].as_i64().unwrap() > 20);
        assert!(s["users"].as_i64().unwrap() >= 0);
        assert_eq!(s["version"], version());
    }

    /// Export must produce a real gzip tar that holds the database, and verify
    /// must read it back. This is the whole migration contract in one test.
    #[test]
    fn export_then_verify_round_trips() {
        crate::test_bootstrap();
        let b = export_bundle().expect("export should succeed");
        let name = b["name"].as_str().unwrap().to_string();
        assert!(
            b["bytes"].as_u64().unwrap() > 0,
            "the bundle must not be empty"
        );
        assert_eq!(b["sha256"].as_str().unwrap().len(), 64);

        let v = verify_bundle(&name).expect("the bundle should verify");
        assert_eq!(v["valid"], true);
        let entries = v["entries"].as_array().unwrap();
        assert!(
            entries.iter().any(|e| e == "mts.sqlite"),
            "the database must be inside: {entries:?}"
        );
        assert!(entries.iter().any(|e| e == "manifest.json"));
        assert_eq!(v["manifest"]["product"], "Mbilinyi Tech Solutions");

        // Clean up the bundle the test created.
        let _ = std::fs::remove_file(safe_bundle_path(&name).unwrap());
    }

    #[test]
    fn verify_refuses_a_bundle_that_is_not_there() {
        crate::test_bootstrap();
        let e = verify_bundle("mts-bundle-does-not-exist.tar.gz").unwrap_err();
        assert!(e.contains("No bundle"));
    }
}
