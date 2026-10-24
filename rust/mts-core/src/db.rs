//! SQLite access. One connection behind a mutex is plenty for this workload and
//! keeps every query on a single writer, which is what SQLite wants anyway.
//! The guard never escapes `with_db`, so it can never be held across an await.
//!
//! The connection is re-opened when the database file behind it is replaced or
//! removed. The PHP backend opened a fresh connection per request, so deleting
//! `storage/mts.sqlite` mid-run gave you a clean database on the next request;
//! a long-lived connection would instead keep writing to a deleted inode. The
//! smoke test relies on that behaviour, so it is preserved here.

use rusqlite::{params_from_iter, Connection, OptionalExtension};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

/// Identifies the database file on disk, so a replaced or deleted file is
/// noticed. `None` means the file is not there.
fn file_sig(path: &str) -> Option<(u64, u64, i64)> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let m = std::fs::metadata(path).ok()?;
        Some((m.dev(), m.ino(), m.mtime()))
    }
    #[cfg(not(unix))]
    {
        let m = std::fs::metadata(path).ok()?;
        let t = m
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        Some((0, m.len(), t))
    }
}

struct Db {
    conn: Option<Connection>,
    sig: Option<(u64, u64, i64)>,
}

static DB: OnceLock<Mutex<Db>> = OnceLock::new();
static PATH: OnceLock<String> = OnceLock::new();
/// Guards against re-entrancy: reopening runs the schema and the seed, both of
/// which go through `with_db`.
static REOPENING: AtomicBool = AtomicBool::new(false);

fn connect(path: &str) -> rusqlite::Result<Connection> {
    if let Some(dir) = Path::new(path).parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    Ok(conn)
}

pub fn open(path: &str) -> rusqlite::Result<()> {
    let conn = connect(path)?;
    let _ = PATH.set(path.to_string());
    let _ = DB.set(Mutex::new(Db {
        conn: Some(conn),
        sig: file_sig(path),
    }));
    Ok(())
}

/// Re-open the database when the file underneath has changed. Called before
/// every query; the check is a single `stat`, which is cheap next to the query.
fn ensure_current() {
    // A nested call (from the schema or the seed below) must not recurse.
    if REOPENING.swap(true, Ordering::SeqCst) {
        return;
    }
    let path = PATH.get().expect("db::open must run first").clone();
    let want = file_sig(&path);
    let stale = {
        let m = DB.get().expect("db::open must run first");
        let g = m.lock().unwrap_or_else(|e| e.into_inner());
        g.sig != want
    };
    if stale {
        // The file was deleted or replaced. Rebuild the connection and put the
        // schema and demo data back, exactly as a first boot would.
        if let Ok(conn) = connect(&path) {
            {
                let m = DB.get().expect("db::open must run first");
                let mut g = m.lock().unwrap_or_else(|e| e.into_inner());
                g.conn = Some(conn);
                g.sig = file_sig(&path);
            }
            crate::bootstrap();
        }
    }
    REOPENING.store(false, Ordering::SeqCst);
}

/// Run a closure with the connection. The guard is confined to this function.
pub fn with_db<T>(f: impl FnOnce(&Connection) -> T) -> T {
    ensure_current();
    let m = DB.get().expect("db::open must run before with_db");
    let guard = m.lock().unwrap_or_else(|e| e.into_inner());
    f(guard.conn.as_ref().expect("connection must exist"))
}

pub fn exec(sql: &str) -> rusqlite::Result<()> {
    with_db(|c| c.execute_batch(sql))
}

pub fn run(sql: &str, args: &[rusqlite::types::Value]) -> rusqlite::Result<usize> {
    with_db(|c| c.execute(sql, params_from_iter(args.iter())))
}

/// Add a column to an existing table when it is missing. `CREATE TABLE IF NOT
/// EXISTS` does nothing to a table that already exists, so a column added to the
/// schema after a database was first created needs this to reach old files.
pub fn ensure_column(table: &str, column: &str, decl: &str) -> rusqlite::Result<()> {
    let exists = with_db(|c| -> rusqlite::Result<bool> {
        let mut st = c.prepare(&format!("PRAGMA table_info({table})"))?;
        let mut rows = st.query([])?;
        let mut found = false;
        while let Some(r) = rows.next()? {
            if r.get::<_, String>(1)? == column {
                found = true;
            }
        }
        Ok(found)
    })?;
    if exists {
        return Ok(());
    }
    with_db(|c| c.execute_batch(&format!("ALTER TABLE {table} ADD COLUMN {column} {decl}")))
}

/// A row as a JSON object. SQLite is dynamically typed and the frontend expects
/// numbers where numbers were stored, so integers and floats are distinguished
/// rather than flattened to strings.
pub type Row = serde_json::Map<String, serde_json::Value>;

pub fn rows(sql: &str, args: &[rusqlite::types::Value]) -> Vec<Row> {
    with_db(|c| {
        let mut st = match c.prepare(sql) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[db] prepare failed: {e} :: {sql}");
                return Vec::new();
            }
        };
        let names: Vec<String> = st.column_names().iter().map(|s| s.to_string()).collect();
        let mut out = Vec::new();
        let mapped = st.query_map(params_from_iter(args.iter()), |r| {
            let mut m = Row::new();
            for (i, n) in names.iter().enumerate() {
                let v: rusqlite::types::Value = r.get(i)?;
                m.insert(n.clone(), to_json(v));
            }
            Ok(m)
        });
        if let Ok(it) = mapped {
            for row in it.flatten() {
                out.push(row);
            }
        }
        out
    })
}

pub fn row(sql: &str, args: &[rusqlite::types::Value]) -> Option<Row> {
    rows(sql, args).into_iter().next()
}

pub fn scalar_i64(sql: &str, args: &[rusqlite::types::Value]) -> i64 {
    with_db(|c| {
        c.query_row(sql, params_from_iter(args.iter()), |r| {
            let v: rusqlite::types::Value = r.get(0)?;
            Ok(match v {
                rusqlite::types::Value::Integer(i) => i,
                rusqlite::types::Value::Real(f) => f as i64,
                rusqlite::types::Value::Text(t) => t.parse().unwrap_or(0),
                _ => 0,
            })
        })
        .optional()
        .ok()
        .flatten()
        .unwrap_or(0)
    })
}

pub fn to_json(v: rusqlite::types::Value) -> serde_json::Value {
    use rusqlite::types::Value as V;
    use serde_json::Value as J;
    match v {
        V::Null => J::Null,
        V::Integer(i) => J::from(i),
        V::Real(f) => J::from(f),
        V::Text(t) => J::from(t),
        V::Blob(b) => J::from(hex::encode(b)),
    }
}

/// Convenience: build the positional argument list from JSON values.
pub fn args(vals: Vec<serde_json::Value>) -> Vec<rusqlite::types::Value> {
    vals.into_iter().map(from_json).collect()
}

pub fn from_json(v: serde_json::Value) -> rusqlite::types::Value {
    use rusqlite::types::Value as V;
    use serde_json::Value as J;
    match v {
        J::Null => V::Null,
        J::Bool(b) => V::Integer(if b { 1 } else { 0 }),
        J::Number(n) => {
            if let Some(i) = n.as_i64() {
                V::Integer(i)
            } else {
                V::Real(n.as_f64().unwrap_or(0.0))
            }
        }
        J::String(s) => V::Text(s),
        other => V::Text(other.to_string()),
    }
}
