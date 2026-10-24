//! Real code execution and test running, for every language the platform sells.
//!
//! The catalogue advertises Rust, Python, Go, Elixir, R and React. This module is
//! what makes those claims true: it detects which toolchains are installed,
//! writes the code to a scratch directory, runs it, and returns what the
//! compiler and the program actually said.
//!
//! Three rules keep this from becoming a hole in the platform:
//!
//! 1. **A scratch directory per run.** Code is written under the system temp
//!    directory, never inside the project, and the directory is removed after.
//! 2. **A hard timeout.** Every child process is killed when it overruns, so a
//!    program that loops forever cannot hold a worker thread.
//! 3. **A clean environment.** The child gets no inherited environment, so a
//!    provider key or a database URL cannot be read out of a snippet.

use crate::util;
use serde_json::{json, Value as J};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// A language the platform can actually run, if its toolchain is installed.
pub struct Lang {
    pub key: &'static str,
    pub name: &'static str,
    /// The file the snippet is written to.
    pub ext: &'static str,
    /// The program to invoke, and its fixed leading arguments.
    pub run: &'static [&'static str],
    /// The compiler/checker, run before execution when there is one.
    pub check: Option<&'static [&'static str]>,
    /// The command that runs the project's own test suite.
    pub test: Option<&'static [&'static str]>,
    /// True for a language whose `run` compiles rather than executes: the
    /// platform builds a binary and then runs it. `run` must end in the output
    /// flag (`-o`), so the build is `run… <binary> <source>`.
    pub compiled: bool,
}

pub const LANGS: &[Lang] = &[
    Lang {
        key: "python",
        name: "Python",
        ext: "py",
        run: &["python3"],
        check: Some(&["python3", "-m", "py_compile"]),
        test: Some(&["python3", "-m", "pytest", "-q"]),
        compiled: false,
    },
    Lang {
        key: "rust",
        name: "Rust",
        ext: "rs",
        run: &["rustc", "--edition", "2021", "-O", "-o"],
        check: None,
        test: Some(&["cargo", "test", "--quiet"]),
        compiled: true,
    },
    Lang {
        key: "go",
        name: "Go",
        ext: "go",
        run: &["go", "run"],
        check: Some(&["go", "vet"]),
        test: Some(&["go", "test", "./..."]),
        compiled: false,
    },
    Lang {
        key: "elixir",
        name: "Elixir",
        ext: "exs",
        run: &["elixir"],
        check: None,
        test: Some(&["mix", "test"]),
        compiled: false,
    },
    Lang {
        key: "r",
        name: "R",
        ext: "R",
        run: &["Rscript"],
        check: None,
        test: None,
        compiled: false,
    },
    Lang {
        key: "node",
        name: "Node.js",
        ext: "js",
        run: &["node"],
        check: Some(&["node", "--check"]),
        test: Some(&["npm", "test", "--silent"]),
        compiled: false,
    },
    Lang {
        key: "c",
        name: "C",
        ext: "c",
        run: &["cc", "-O2", "-o"],
        check: None,
        test: None,
        compiled: true,
    },
];

/// Find a language by its key, or by a filename extension.
pub fn find(key: &str) -> Option<&'static Lang> {
    let k = key.trim().to_lowercase();
    LANGS.iter().find(|l| l.key == k).or_else(|| {
        LANGS.iter().find(|l| {
            l.ext.eq_ignore_ascii_case(&k)
                || l.name.to_lowercase() == k
                || matches!(
                    (l.key, k.as_str()),
                    ("rust", "rs") | ("node", "js") | ("node", "javascript") | ("elixir", "ex")
                )
        })
    })
}

/// Which toolchains are actually installed. Reported to the admin so nobody has
/// to guess why a language is refused.
pub fn installed() -> Vec<J> {
    LANGS
        .iter()
        .map(|l| {
            let bin = l.run[0];
            json!({
                "key": l.key,
                "name": l.name,
                "binary": bin,
                "available": have(bin),
                "canTest": l.test.is_some() && have(l.test.unwrap()[0]),
            })
        })
        .collect()
}

/// The PATH the platform itself was started with, captured once.
///
/// This matters: `env_clear()` makes `execvp` fall back to the POSIX default
/// path, which is `/bin:/usr/bin` and does *not* include `/usr/local/bin`. A
/// python installed there would then look "missing" and a runnable language
/// would be refused. Reading the real PATH once and passing it explicitly keeps
/// detection and execution in agreement.
fn path_env() -> &'static str {
    static PATH: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    PATH.get_or_init(|| {
        let from_env = std::env::var("PATH").unwrap_or_default();
        // Belt and braces: the usual install locations are always searched, so
        // a stripped-down PATH cannot hide an installed toolchain.
        let extra = "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";
        if from_env.split(':').any(|p| p == "/usr/local/bin") {
            from_env
        } else {
            format!("{extra}:{from_env}")
        }
    })
}

/// Is a program on the path?
///
/// The program is started and then killed. A successful `spawn` means the
/// executable was found; only a `NotFound` error means it is absent. Exit status
/// is deliberately ignored, because `--version` is not universal — `sh` on
/// Debian rejects it and exits non-zero while being perfectly present.
fn have(bin: &str) -> bool {
    let child = Command::new(bin)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .env_clear()
        .env("PATH", path_env())
        .spawn();
    match child {
        Ok(mut c) => {
            let _ = c.kill();
            let _ = c.wait();
            true
        }
        Err(_) => false,
    }
}

/// What a run produced.
pub struct Ran {
    pub ok: bool,
    pub output: String,
    pub ms: u128,
}

fn scratch() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("mts-run-{}", util::new_id("").trim_matches('-')));
    let _ = std::fs::create_dir_all(&dir);
    dir
}

fn write(dir: &Path, name: &str, body: &str) -> std::io::Result<PathBuf> {
    let p = dir.join(name);
    let mut f = std::fs::File::create(&p)?;
    f.write_all(body.as_bytes())?;
    Ok(p)
}

/// Toolchain-location variables a child needs to run an installed language.
///
/// A rustup-managed `rustc` is a shim: with no `RUSTUP_HOME` it cannot choose a
/// version and refuses to run, so a language that is installed would look
/// broken. Go, Elixir and the rest use their own cache/home variables the same
/// way. Only these named locations are forwarded — never the whole environment,
/// which is what would let a snippet read a key or a database path.
const TOOLCHAIN_ENV: [&str; 6] = [
    "RUSTUP_HOME",
    "CARGO_HOME",
    "GOPATH",
    "GOCACHE",
    "MIX_HOME",
    "ERL_LIBS",
];

/// Run a command in `dir` with a clean environment and a hard deadline. The
/// child's output is captured, capped so a program that prints forever cannot
/// exhaust memory, and returned as text.
fn run_cmd(dir: &Path, argv: &[String], timeout: Duration) -> Ran {
    let start = Instant::now();
    let mut cmd = Command::new(&argv[0]);
    cmd.args(&argv[1..])
        .current_dir(dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        // No inherited environment: a snippet must not be able to read a key or
        // a database path out of the platform's own environment. The toolchain
        // locations are re-added by name below, because a shim such as rustup's
        // `rustc` cannot run without them.
        .env_clear()
        .env("PATH", path_env())
        .env("HOME", dir)
        .env("LANG", "C.UTF-8");
    for key in TOOLCHAIN_ENV {
        if let Ok(v) = std::env::var(key) {
            cmd.env(key, v);
        }
    }
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            return Ran {
                ok: false,
                output: format!("Could not start {}: {e}", argv[0]),
                ms: start.elapsed().as_millis(),
            }
        }
    };
    // Poll for exit; kill on the deadline.
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Ran {
                        ok: false,
                        output: format!(
                            "Stopped after {} seconds — the program did not finish.",
                            timeout.as_secs()
                        ),
                        ms: start.elapsed().as_millis(),
                    };
                }
                std::thread::sleep(Duration::from_millis(40));
            }
            Err(e) => {
                return Ran {
                    ok: false,
                    output: format!("The program could not be waited on: {e}"),
                    ms: start.elapsed().as_millis(),
                }
            }
        }
    }
    let out = child.wait_with_output().ok();
    let (stdout, stderr, code) = match out {
        Some(o) => (
            String::from_utf8_lossy(&o.stdout).to_string(),
            String::from_utf8_lossy(&o.stderr).to_string(),
            o.status.code().unwrap_or(-1),
        ),
        None => (String::new(), String::new(), -1),
    };
    let mut text = String::new();
    if !stdout.trim().is_empty() {
        text.push_str(&cap(&stdout, 12_000));
    }
    if !stderr.trim().is_empty() {
        if !text.is_empty() {
            text.push_str("\n--- stderr ---\n");
        }
        text.push_str(&cap(&stderr, 12_000));
    }
    if text.trim().is_empty() {
        text = "(the program produced no output)".to_string();
    }
    Ran {
        ok: code == 0,
        output: text,
        ms: start.elapsed().as_millis(),
    }
}

fn cap(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let mut end = max;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}\n… output truncated at {max} bytes", &s[..end])
}

/// Run a snippet in a named language. This is the real thing: it compiles and
/// executes, and reports exactly what happened.
pub fn run_snippet(lang_key: &str, code: &str) -> Ran {
    run_snippet_within(lang_key, code, Duration::from_secs(60))
}

/// The same, with the deadline given explicitly. The production caller uses the
/// default; tests use a short one so a runaway program is proven to be killed
/// without the suite waiting a full minute.
pub fn run_snippet_within(lang_key: &str, code: &str, limit: Duration) -> Ran {
    let Some(lang) = find(lang_key) else {
        return Ran {
            ok: false,
            output: format!(
                "No runtime for \"{lang_key}\". Available: {}.",
                LANGS.iter().map(|l| l.key).collect::<Vec<_>>().join(", ")
            ),
            ms: 0,
        };
    };
    if code.trim().is_empty() {
        return Ran {
            ok: false,
            output: "Give the code to run.".into(),
            ms: 0,
        };
    }
    if !have(lang.run[0]) {
        return Ran {
            ok: false,
            output: format!(
                "{} is not installed on this machine, so the code cannot be run.",
                lang.name
            ),
            ms: 0,
        };
    }
    let dir = scratch();
    let file = match write(&dir, &format!("main.{}", lang.ext), code) {
        Ok(p) => p,
        Err(e) => {
            let _ = std::fs::remove_dir_all(&dir);
            return Ran {
                ok: false,
                output: format!("Could not write the file: {e}"),
                ms: 0,
            };
        }
    };
    let name = file.to_string_lossy().to_string();

    // A syntax check first, when the language has one, so a compile error is
    // reported as a compile error rather than as a failed run.
    if let Some(check) = lang.check {
        let argv: Vec<String> = check
            .iter()
            .map(|s| s.to_string())
            .chain(std::iter::once(name.clone()))
            .collect();
        let r = run_cmd(&dir, &argv, Duration::from_secs(30));
        if !r.ok {
            let _ = std::fs::remove_dir_all(&dir);
            return Ran {
                ok: false,
                output: format!("{} did not accept the code:\n{}", lang.name, r.output),
                ms: r.ms,
            };
        }
    }

    // A compiled language needs its two-step build, so the compile and the run
    // are separate: build a binary, then execute it. `rustc -o` and `cc -o` are
    // the same shape, which is why the flag lives on the language.
    let argv: Vec<String> = if lang.compiled {
        let bin = dir.join("a.out").to_string_lossy().to_string();
        let build: Vec<String> = lang
            .run
            .iter()
            .map(|s| s.to_string())
            .chain(std::iter::once(bin.clone()))
            .chain(std::iter::once(name.clone()))
            .collect();
        let b = run_cmd(&dir, &build, Duration::from_secs(60));
        if !b.ok {
            let _ = std::fs::remove_dir_all(&dir);
            return Ran {
                ok: false,
                output: format!("The {} compiler rejected the code:\n{}", lang.name, b.output),
                ms: b.ms,
            };
        }
        vec![bin]
    } else {
        lang.run
            .iter()
            .map(|s| s.to_string())
            .chain(std::iter::once(name.clone()))
            .collect()
    };

    let r = run_cmd(&dir, &argv, limit);
    let _ = std::fs::remove_dir_all(&dir);
    r
}

/// Run a project's own test suite. The command comes from the language, so a
/// Rust project runs `cargo test` and a Python one runs `pytest`.
pub fn run_tests(lang_key: &str, dir: &str) -> Ran {
    let Some(lang) = find(lang_key) else {
        return Ran {
            ok: false,
            output: format!("No test runner for \"{lang_key}\"."),
            ms: 0,
        };
    };
    let Some(test) = lang.test else {
        return Ran {
            ok: false,
            output: format!("{} has no test command configured.", lang.name),
            ms: 0,
        };
    };
    if !have(test[0]) {
        return Ran {
            ok: false,
            output: format!("{} is not installed, so the tests cannot run.", test[0]),
            ms: 0,
        };
    }
    let root = PathBuf::from(if dir.trim().is_empty() {
        "."
    } else {
        dir.trim()
    });
    if !root.is_dir() {
        return Ran {
            ok: false,
            output: format!("{dir} is not a directory."),
            ms: 0,
        };
    }
    let argv: Vec<String> = test.iter().map(|s| s.to_string()).collect();
    run_cmd(&root, &argv, Duration::from_secs(600))
}

/// The languages that can run right now, and those that need installing. Shown
/// in the admin panel so the CEO can see the platform's real capability.
pub fn status() -> J {
    let all = installed();
    let ready: Vec<String> = all
        .iter()
        .filter(|l| {
            l.get("available")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
        })
        .filter_map(|l| l.get("name").and_then(|v| v.as_str()).map(str::to_string))
        .collect();
    let missing: Vec<String> = all
        .iter()
        .filter(|l| {
            !l.get("available")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
        })
        .filter_map(|l| l.get("name").and_then(|v| v.as_str()).map(str::to_string))
        .collect();
    json!({
        "ready": ready,
        "missing": missing,
        "languages": all,
        "rule": "Code runs in a scratch directory with no inherited environment and a hard timeout.",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_language_is_found_by_key_name_and_extension() {
        assert_eq!(find("python").map(|l| l.key), Some("python"));
        assert_eq!(find("PYTHON").map(|l| l.key), Some("python"));
        assert_eq!(find("py").map(|l| l.key), Some("python"));
        assert_eq!(find("rs").map(|l| l.key), Some("rust"));
        assert_eq!(find("javascript").map(|l| l.key), Some("node"));
        assert_eq!(find("Elixir").map(|l| l.key), Some("elixir"));
        assert_eq!(find("nonsense").map(|l| l.key), None);
    }

    #[test]
    fn every_advertised_language_has_a_runner() {
        for key in ["rust", "python", "go", "elixir", "r", "node"] {
            let l = find(key).unwrap_or_else(|| panic!("{key} has no runner"));
            assert!(!l.run.is_empty());
        }
    }

    #[test]
    fn a_snippet_with_no_code_is_refused() {
        let r = run_snippet("python", "   ");
        assert!(!r.ok);
        assert!(r.output.contains("Give the code"));
    }

    #[test]
    fn an_unknown_language_says_what_is_available() {
        let r = run_snippet("cobol", "DISPLAY 'hi'.");
        assert!(!r.ok);
        assert!(r.output.contains("No runtime"));
        assert!(r.output.contains("python"));
    }

    #[test]
    fn python_code_actually_runs_when_python_is_present() {
        if !have("python3") {
            return; // the environment has no python; nothing to prove here
        }
        let r = run_snippet("python", "print(6 * 7)");
        assert!(r.ok, "python should run: {}", r.output);
        assert!(r.output.contains("42"), "output was: {}", r.output);
    }

    #[test]
    fn a_python_error_is_reported_rather_than_hidden() {
        if !have("python3") {
            return;
        }
        let r = run_snippet("python", "raise SystemExit('deliberate failure')");
        assert!(!r.ok);
        assert!(
            r.output.contains("deliberate failure"),
            "output was: {}",
            r.output
        );
    }

    #[test]
    fn a_runaway_program_is_stopped() {
        if !have("python3") {
            return;
        }
        let started = Instant::now();
        // A loop that never ends must be killed, not waited on forever.
        let r = run_snippet_within("python", "while True: pass", Duration::from_secs(2));
        assert!(!r.ok);
        assert!(
            r.output.contains("did not finish"),
            "output was: {}",
            r.output
        );
        assert!(
            started.elapsed() < Duration::from_secs(30),
            "took {:?}",
            started.elapsed()
        );
    }

    #[test]
    fn output_is_capped_so_a_flood_cannot_exhaust_memory() {
        let long = "x".repeat(50_000);
        let capped = cap(&long, 1000);
        assert!(capped.len() < 2000);
        assert!(capped.contains("truncated"));
    }

    // A rustup `rustc` shim cannot pick a toolchain without RUSTUP_HOME, so the
    // child must receive it. Only toolchain locations may be forwarded — the
    // allowlist must never grow a name that carries a secret, such as a key.
    #[test]
    fn only_toolchain_locations_are_forwarded_to_a_snippet() {
        assert!(TOOLCHAIN_ENV.contains(&"RUSTUP_HOME"));
        assert!(TOOLCHAIN_ENV.contains(&"CARGO_HOME"));
        for forbidden in ["OPENAI_API_KEY", "MTS_DB", "GITHUB_TOKEN", "PATH", "HOME"] {
            assert!(
                !TOOLCHAIN_ENV.contains(&forbidden),
                "{forbidden} must not be forwarded into a snippet"
            );
        }
    }

    // Rust and C compile rather than interpret, so a one-step "run" would leave
    // the program built but never executed — it reported success with no output.
    #[test]
    fn a_compiled_language_is_built_and_then_run() {
        for key in ["rust", "c"] {
            let l = find(key).expect("known language");
            assert!(l.compiled, "{key} should be marked compiled");
            assert!(
                l.run.last().map(|s| *s == "-o").unwrap_or(false),
                "{key} run must end in -o so a binary can be named"
            );
        }
        assert!(!find("python").unwrap().compiled);
        assert!(!find("go").unwrap().compiled);
    }

    /// The PATH handed to a child must include the usual install directories.
    /// Clearing the environment without this made `/usr/local/bin` invisible,
    /// so an installed Python reported itself as missing.
    #[test]
    fn the_child_path_still_finds_a_local_install() {
        assert!(path_env().split(':').any(|p| p == "/usr/local/bin"));
        assert!(path_env().split(':').any(|p| p == "/usr/bin"));
    }

    #[test]
    fn a_program_that_exists_is_found_and_a_made_up_one_is_not() {
        // `sh` is present on every system this runs on, and it does not need
        // `--version` to be recognised — which is exactly the bug this pins.
        assert!(have("sh"), "sh should be found with the platform PATH");
        assert!(!have("definitely-not-a-real-program-xyz"));
    }

    #[test]
    fn the_status_names_the_languages_it_can_run() {
        let s = status();
        assert!(
            s.get("languages")
                .and_then(|v| v.as_array())
                .map(|a| a.len())
                .unwrap_or(0)
                >= 6
        );
    }
}
