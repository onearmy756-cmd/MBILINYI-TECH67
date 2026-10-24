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

## Deploy (Fly.io) — see `2026-10-24-fly-deploy.md`
- **LIVE: https://mbilinyitech-tz.fly.dev/** (app `mbilinyitech-tz` — the plain
  name is taken globally on Fly). Region jnb, volume `mts_data` at `/data`,
  shared IPv4 `66.241.125.44`, IPv6 `2a09:8280:1::1cb:226c:0`.
  `scripts/smoke.sh` against the live app = 25 ok:true / 4 ok:false, the same
  as a clean local run.
- An app-scoped Fly token (like the one first supplied, scoped to `mtechos`)
  CANNOT create apps; every create path returns `UNAUTHORIZED`. Creating an app
  needs an ORG token (`fly tokens create org`). **Do not deploy over
  `mtechos`** — it is a different product (an Agent-Canvas "fundi" IDE at
  mtech.mbilinyitech.co.tz).
- DNS for `mbilinyitech.co.tz` is at `ns1/ns2.rodlinehost.com` (cPanel host);
  apex A → 91.204.209.30, `www` CNAME → apex, `mtech` CNAME → mtechos.fly.dev.
  Fly cannot take the apex until the registrar A record changes. Exact
  replacement records are in DEPLOY.md.
- `MTS_OTP_PEPPER` is now a real Fly secret. Never set `OTP_SHOW_IN_RESPONSE`
  on the live site — it returns the sign-in code in the API response.
- **Deposit bug fixed (only reproduced on Fly):** `deposit_info()` ordered DESC
  on `'%deposit%'` while the countersign gate `deposit_block()` ordered ASC
  LIMIT 1 on `'%50%Deposit%'`, so with two deposit invoices the admin marked the
  one shown and the gate judged the other. Both now call one
  `deposit_invoice_row()` helper, mirroring PHP. A fast local loop hides this
  (both rows share a `created_at` second); latency on Fly separates them. When a
  Rust port disagrees with PHP only in production, diff the *query*
  (ORDER BY / LIKE / LIMIT), not just the route list.

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
- **The Rust core silently lagged the PHP reference on routes.** `auth.rs` had
  no `verify-login-otp`/`resend-login-otp` while the login modal OPENS on the
  Emailed Code tab, so every sign-in died at "Unknown auth action." Fix pattern:
  `grep -oE "case '[a-z-]+'" api/*.php | sort -u` vs the Rust `match c.action()`
  arms — diff them whenever a feature "works in PHP but not in the core".
  `login` must ALSO create the code; having the verify route is not enough.
- Login codes: hash = sha256(`code|email|MTS_OTP_PEPPER`), 6 digits from
  `rand::thread_rng().gen_range(0..1_000_000)`, 10-min expiry, `used_at` marks
  consumption, and a resend DELETEs the prior row. Return the code as
  `otp_debug_only` ONLY when `OTP_SHOW_IN_RESPONSE` is set — never on the live
  site. `mail::send_login_code` already existed but nothing called it.
- Frontend/backend field drift is a recurring class: `visitor_stats` sent only
  `total`/`today` while the dashboard read `yesterday`/`this_week`/`this_month`
  → the card rendered literally "undefined". Grep the JS for the keys it reads
  and confirm the handler sends each one.
- `elixir --version` boots an Erlang VM that never exits → it HANGS a startup
  script. Always wrap it: `timeout 15 elixir --version | grep -m1 Elixir`.
  It also drops `erl_crash.dump` in the CWD (now git-ignored).

Details: `2026-10-21-mbilinyi-rust-core.md`.
