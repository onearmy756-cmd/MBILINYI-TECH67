# MBILINYI-TECH67 — project memory

## What this repo is
Mbilinyi Tech Solutions: a PHP site (pages + `/api/*.php`) that is being
replaced by a **Rust core** at `rust/mts-core` (axum). The frontend
(`assets/app.js`, `templates/`) is shared by both and must not need changes.

## Implemented (2026-10-23; details in `2026-10-23-payroll-hiring-call.md`)
- **Payroll** (`payroll.rs`): payslip per escrow release, banded PAYE/NSSF/WCF,
  TRA return. Admin Payroll tab, worker "Pay & Contract" tab.
- **Stripe (USD)** (`payments/gateways.rs`): `gw_stripe_api_base` overridable for
  a mock; webhooks HMAC-verified. Needs `gw_stripe_publishable` + secret.
- **101 agents**: `agents.rs` seeds "Magma Commander" then 100 role seats.
- **Real video call**: `call.rs` relays SDP/ICE on `/ws/call/:room`; browser
  `assets/call.js` uses a real RTCPeerConnection (2 peers verified).

## Deploy (Fly.io) — see `2026-10-24-fly-deploy.md`
- **LIVE: https://mbilinyitech-tz.fly.dev/** (app `mbilinyitech-tz` — the plain
  name is taken globally on Fly). Region jnb, volume `mts_data` at `/data`,
  shared IPv4 `66.241.125.44`, IPv6 `2a09:8280:1::1cb:226c:0`.
  `scripts/smoke.sh` against the live app = 25 ok:true / 4 ok:false, the same
  as a clean local run.
- An app-scoped Fly token CANNOT create apps (`UNAUTHORIZED`); an app needs an
  ORG token. **Do not deploy over `mtechos`** — a different product (Agent-Canvas
  IDE at mtech.mbilinyitech.co.tz).
- DNS for `mbilinyitech.co.tz` is at `ns1/ns2.rodlinehost.com` (cPanel); apex A →
  91.204.209.30, `www` CNAME → apex. Fly cannot take the apex until the registrar
  A record changes; exact records are in DEPLOY.md.
- `MTS_OTP_PEPPER` is now a real Fly secret. Never set `OTP_SHOW_IN_RESPONSE`
  on the live site — it returns the sign-in code in the API response.
- **Deposit bug (only on Fly)**: two queries disagreed (DESC `%deposit%` vs ASC
  `%50%Deposit%`), now one `deposit_invoice_row()` helper. A fast local loop hides
  it — when a Rust port differs from PHP *only in production*, diff the query
  (ORDER BY/LIKE/LIMIT), not just the route list.
- **Security tools all work** (verified live). Scope rule: `exposure` refuses via
  `data.ok:false` (envelope stays `ok:true`) unless the domain is authorised AND
  the caller names who approved it.

## Facts worth keeping
- **MTECHOS is a DIFFERENT product, broad in labels not capability** — its
  1040 "skills" are `action:"plan"` checklists with no runtime. This platform has
  fewer labels, each backed by real code; compare counts only after checking the
  other side actually runs something. Details: `2026-10-24-fly-deploy.md`.
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
- **Security tools: 26 in the family, all real**, in `sandbox.rs`; reachable via
  `security.php?action=scan&kind=<k>` and as agent tool codes in `runtime.rs`.
  Catalogue total 100 tools (66+ agentic). A bare credential *prefix* is not a
  secret — rules need a minimum token length or the scanner flags its own table.
- **Runtimes**: `langrun.rs` advertises a language only when it can run it;
  compiled ones (Rust, C) are built then run. Children get only toolchain-location
  env, never the platform environment. All seven verified printing `42`.
- **Installable apps**: `downloads/` holds four real builds (Android APK,
  Windows NSIS setup + portable exe, Linux .deb) and `scripts/build-apps.sh`
  rebuilds them. `/download` page + "Get App" nav button + `installPwa()` in
  `app.js`. `static_file` adds installer MIME types and `Content-Disposition:
  attachment` under `downloads/`. The Dockerfile copies the dir, so the binaries
  must be committed.
- **Custom domain not cut over**: `mbilinyitech.co.tz` still points at the cPanel
  host (91.204.209.30); Fly ingress is 66.241.125.44. Change the registrar A/AAAA
  records, then the Fly cert verifies itself.
- **Hire a worker** with `wafanyakazi.php` action **`add`** and `services` as an
  **array** (singular `service` is ignored → the worker never matches a request).
- `rust/mts-core/target/` (~1.3 GB) and the runtime DB are git-ignored.

## Traps that already cost time
- **A template with mode 0600 404s while looking perfect.** The server runs as
  uid 1000 (`mts`), so a file copied into the image as root-only is unreadable →
  `render()` returns its 404 *with the real security headers*, indistinguishable
  from a missing route. If a page 404s in prod but its template is on disk,
  check the mode first (`flyctl ssh console` + `ls -la /app/templates/`). The
  Dockerfile now `chmod -R a+rX` the copied trees.
- **New page checklist**: add the template, add it to `KNOWN_PAGES`, add a
  `title_for` arm — and to the PHP shell if it must match. A test asserts every
  routable page has a template on disk.
- The Rust core keeps ONE connection (PHP opened one per request), so a mid-run
  `rm storage/mts.sqlite` needs `db.rs` to notice the new inode and re-bootstrap.
- `/page.php` may only 301 for GET/HEAD; redirecting a POST drops the body.
- A guest submitting a request gets an account created and signed in, so the
  response must set the session cookie.
- The 50/50 deposit gate reads the EARLIEST deposit invoice for the tracking id,
  not the sum of unpaid ones.
- Invoice/ticket ownership also runs through `tracking_id` and the account
  email, not just `user_id`.
- Keep ONE `templates/` dir at the repo root; the duplicate under
  `rust/mts-core/templates` drifted and was deleted.
- **The Rust core silently lags the PHP reference on routes.** Whenever a
  feature "works in PHP but not in the core", diff
  `grep -oE "case '[a-z-]+'" api/*.php | sort -u` against the Rust
  `match c.action()` arms. `login` must ALSO create the OTP, not just verify it.
- Login codes: sha256(`code|email|MTS_OTP_PEPPER`), 6 digits, 10-min expiry,
  `used_at` marks use, resend DELETEs the prior row. Return the code as
  `otp_debug_only` ONLY when `OTP_SHOW_IN_RESPONSE` is set — never live.
- Frontend/backend field drift recurs (`visitor_stats` sent `total`/`today` while
  the dashboard read `yesterday`/`this_week`/`this_month` → "undefined"). Grep
  the JS for the keys it reads and confirm the handler sends each one.
- `elixir --version` boots an Erlang VM that never exits → HANGS a startup script.
  Wrap it: `timeout 15 elixir --version | grep -m1 Elixir`.

Details: `2026-10-21-mbilinyi-rust-core.md`.
