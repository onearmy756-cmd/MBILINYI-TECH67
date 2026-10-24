# MBILINYI-TECH67 — project memory

## What this repo is
Mbilinyi Tech Solutions: a PHP site (pages + `/api/*.php`) that is being
replaced by a **Rust core** at `rust/mts-core` (axum). The frontend
(`assets/app.js`, `templates/`) is shared by both and must not need changes.

## What is now implemented (2026-10-23)
- **Payroll**: `payroll.rs` — payslip per escrow release, banded PAYE + NSSF +
  WCF, TRA return. Admin Payroll tab; worker "Pay & Contract" tab.
- **Stripe (USD)**: `payments/gateways.rs`; `gw_stripe_api_base` is overridable
  (point it at a mock or sandbox to test). Webhooks are HMAC-verified.
  Requires `gw_stripe_publishable` AND `gw_stripe_secret`.
- **100 agents + manager**: `agents.rs` seeds "Magma Commander" (dept `ceo`)
  then 100 role seats → `agents.php?action=list` returns 101.
- **Real video call**: backend `call.rs` relays SDP/ICE on `/ws/call/:room`;
  browser `assets/call.js` (real RTCPeerConnection). Verified 2 peers connect.
- Details and the traps: `2026-10-23-payroll-hiring-call.md`.

## Facts worth keeping
- Repo `onearmy756-cmd/MBILINYI-TECH67`, default branch **master** (a stale
  `main` ref also exists). **It is public, not private.** The CEO's PAT has
  push+admin; use it in a one-off URL, never in `.git/config`.
- Clone at `/workspace/MBILINYI-TECH67`; lookalike dirs (`mbilinyitech67`,
  `MBILINYITECH`, `MTECHOS`, `MBILINYI-AI`) are unrelated.
- Rust toolchain lives in `/workspace/.toolchain` (survives a session reset;
  `~/.cargo` does not). R is at `/usr/bin/Rscript`, base R only.
- `scripts/serve.sh` starts the Rust core on port 12000 and self-heals a reset
  sandbox (installs Rust/R/Go/Elixir, rebuilds when sources are newer).
- `scripts/smoke.sh` is the acceptance test; all 26 steps pass, repeatably (the
  four `ok:false` lines are intentional refusal tests).
- **Runtimes**: `langrun.rs` advertises a language only when it can run it. A
  compiled language (Rust, C) is built then executed — driven by `compiled: bool`
  — and children receive only toolchain-location env (`RUSTUP_HOME`, `CARGO_HOME`,
  `GOPATH`, `GOCACHE`, `MIX_HOME`, `ERL_LIBS`), never the platform environment.
  All seven (python/rust/go/elixir/r/node/c) verified printing `42`.
- **Hire a worker** with `wafanyakazi.php` action **`add`** and `services` as an
  **array** (singular `service` is ignored → the worker never matches a request).
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
