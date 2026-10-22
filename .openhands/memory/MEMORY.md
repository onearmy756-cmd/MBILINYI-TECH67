# MBILINYI-TECH67 — project memory

## What this repo is
Mbilinyi Tech Solutions: a PHP site (pages + `/api/*.php`) that is being
replaced by a **Rust core** at `rust/mts-core` (axum). The frontend
(`assets/app.js`, `templates/`) is shared by both and must not need changes.

## Facts worth keeping
- Repo `onearmy756-cmd/MBILINYI-TECH67`, default branch **master** (a stale
  `main` ref also exists). **It is public, not private.** The CEO's PAT has
  push+admin; use it in a one-off URL, never in `.git/config`.
- Clone at `/workspace/MBILINYI-TECH67`; lookalike dirs (`mbilinyitech67`,
  `MBILINYITECH`, `MTECHOS`, `MBILINYI-AI`) are unrelated.
- Rust toolchain lives in `/workspace/.toolchain` (survives a session reset;
  `~/.cargo` does not). R is at `/usr/bin/Rscript`, base R only.
- `scripts/serve.sh` starts the Rust core on port 12000 and self-heals a reset
  sandbox (installs Rust/R, rebuilds when sources are newer).
- `scripts/smoke.sh` is the acceptance test; all 21 steps pass, repeatably.
- `rust/mts-core/target/` (~1.3 GB) and the runtime DB are git-ignored.

## Traps that already cost time
- The Rust core keeps ONE connection. PHP opened one per request, so the smoke
  test's mid-run `rm storage/mts.sqlite` needs `db.rs` to notice the new inode
  and re-bootstrap — otherwise it writes to the deleted inode.
- `/page.php` may only 301 for GET/HEAD; redirecting a POST drops the body.
- A guest submitting a request gets an account created and signed in, so the
  response must set the session cookie.
- The 50/50 deposit gate reads the EARLIEST deposit invoice for the tracking id,
  not the sum of unpaid ones.
- Invoice/ticket ownership also runs through `tracking_id` and the account
  email, not just `user_id`.
- Keep ONE `templates/` dir at the repo root; the duplicate under
  `rust/mts-core/templates` drifted and was deleted.

Details: `2026-10-21-mbilinyi-rust-core.md`.
