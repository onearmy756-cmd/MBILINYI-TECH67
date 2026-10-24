# MBILINYI-TECH67 — payroll, hiring, Stripe, WebRTC (2026-10-23/24)

## What was built and verified this session
- **Payroll** (`rust/mts-core/src/payroll.rs`): a payslip is raised when escrow
  releases to a worker. PAYE is banded, plus NSSF and WCF; the net goes to the
  worker. Endpoints: `payments.php?action=payslips|tax_return|payslip_pay`.
  `payslips` is scoped: an admin may pass `employeeId`, a worker sees only their
  own (looked up through `wafanyakazi::employee_for_account`).
- **Admin Payroll tab** (`assets/app.js`, title key `payroll`) and **worker
  "Pay & Contract" tab** (`templates/mtaalamu.html` + `staffTab('pay')`,
  `staffPay()`), where a worker reads only their own payslips and signs their own
  employment contract.
- **Stripe (USD)** works: `payments/gateways.rs` uses
  `POST {gw_stripe_api_base}/v1/checkout/sessions` (default `https://api.stripe.com`),
  so a mock base is enough to test without real Stripe. Webhooks are verified
  with a constant-time HMAC over `"<t>.<raw body>"` and a ±300s timestamp bound.
  `gw_stripe_api_base` is overridable precisely so the CEO can point at a sandbox.
- **100 agents + manager**: `agents.rs` `COMPANY_SIZE = 100`; `seed()` writes the
  manager (`MANAGER_NAME = "Magma Commander"`, department `ceo`, id stored in each
  agent's `manager_id`) first, then 100 role seats. `agents.php?action=list`
  returns **101**. Security/finance seats are `supervised`; the rest `autonomous`.
- **Real video call**: the backend `call.rs` relays SDP/ICE over
  `/ws/call/:room`; the browser side is `assets/call.js` (real
  `RTCPeerConnection` + `getUserMedia`, glare handling). Two headless peers
  connected in ~1s with `Umeunganishwa.` and two video tiles each.

## Bugs found and fixed
- **A worker hired through `ajira` could never sign in.** `link_account` created
  the portal user with an empty phone, so `otp_required:true` came back with
  `pin_available:false` and no PIN or phone tail existed to satisfy step two.
  Fix: `link_account(employee_id, email, name, phone)` now stores the phone
  (digits only, via a new `digits_only` helper) and backfills it on an existing
  account. The sign-in PIN is the phone's last four digits.
- **`employee_for_account` was non-deterministic** when one email was hired more
  than once (several employee rows share one user account). A worker could be
  refused their own contract with "This contract is not yours to sign." Fix:
  `ORDER BY created_at DESC, id DESC`.
- **The two contract signatures were identical** because the signed message did
  not name the signer. Fix: append `|signer=admin|employee` to the signed bytes.
- `employee_for_account` still assumes one employee per account; that holds now
  because an email is only hired once in practice.

## Test harness (rebuilt from scratch after each sandbox reset)
- `cargo test` → **104 passed** (2 new: `digits_only`, the both-signed rule).
- Server: `MTS_DB=/tmp/v3.sqlite MTS_PORT=12060 ./rust/mts-core/target/release/mts-core`.
- Playwright via `node` + `/tmp/uitest/node_modules/playwright-core` +
  `executablePath:'/usr/bin/chromium'`, `--no-sandbox`; for calls add
  `--use-fake-ui-for-media-stream --use-fake-device-for-media-stream` and grant
  `permissions:['camera','microphone']`.
- Scripts: `uitabs.cjs`, `worker2.cjs`, `bothsign.cjs`, `call.cjs`, `stripe.cjs`,
  `agents.cjs` in `/tmp/uitest` (tmp, not kept).

## API quirks learned (cheap to re-discover, cost time the first time)
- Sign in is two steps: `login` then `verify-login-pin` with the phone-tail PIN.
- `users.php save` **blanks any field you omit** — always resend `phone`, or the
  account loses its PIN.
- Invoice create/edit action is **`admin_save`** (not `save`); it returns the
  whole `invoices` list, so read the new id from `data.invoices[0].id`.
- Stripe needs BOTH `gw_stripe_publishable` and `gw_stripe_secret`, or
  `enabled`/`configured` stay false even when `gw_stripe_enabled=1`.
- `agents.php?action=list` → 101 agents (100 + Magma Commander).

## "Maliza kazi zote" audit (2026-10-24)

Everything the CEO listed was re-checked against the running system, not the
source. Most of it was already live; two runtime bugs were real and are fixed.

- **All seven languages now really run.** The Runtimes screen only offers a
  language it can execute, and four of the seven were broken or absent:
  - Go, Elixir and R were not installed. `scripts/serve.sh` now installs them
    (R already was; Go and Elixir are new), so a reset sandbox comes back whole.
  - **Rust was doubly broken.** A rustup `rustc` is a *shim*: with no
    `RUSTUP_HOME` it cannot choose a version and exits, so an installed Rust
    looked missing. Worse, `rustc` only *compiles* — the old one-step run built
    the program and never executed it, reporting success with empty output. C had
    the same shape and was special-cased; that is now a `compiled: bool` on the
    language, and `run_cmd` forwards only toolchain-location variables
    (`RUSTUP_HOME`, `CARGO_HOME`, `GOPATH`, `GOCACHE`, `MIX_HOME`, `ERL_LIBS`) —
    never the platform's own environment, which holds keys and the DB path.
  - Verified by running `6*7` in all seven: python, rust, go, elixir, r, node, c
    all print `42`. Tests 106 pass (two new: the allowlist, and compiled-langs).
- **Request routing is real.** Hiring a worker with `services:['ai']` and then
  submitting a guest request for `ai` auto-assigns it: the response carries
  `assignedTo: "<worker>"` and a `request_assignment` row is written
  (`assigned_by:"auto"`), carrying that worker's granted tools. The scheduler
  entry is enqueued *held* until the CEO approves.
- **SMS works both ways.** `notify::request_created` sends `admin_new_request`
  to `admin_alert_phone` and `request_received` to the client, both written to
  `sms_log` (kind, phone, message). `sms_log` returns them under `data.messages`.
- **Logo change works.** `security.php?action=logo_set` with a real PNG stores a
  data URI and it appears on the public page. **SVG is deliberately refused** —
  do not "fix" that; it is an XSS guard.
- **Catalogue breadth**: 90 tools in 9 families. iot/game/os/security/finance/
  pentest/quantum/mobile/desktop/graphics/cv all present. Education, agriculture
  and transport are not separate tools — they are the `industry_solutions`
  service (kilimo/usafiri/fedha/elimu/afya/serikali).
- **Shells**: `mobile/` is Capacitor (`tz.co.mbilinyitech.app`, android project,
  allows checkout.stripe.com); `desktop/` is Tauri v2 with a strict CSP. Both
  load the live site.

### Environment gotchas learned this session
- A sandbox reset wipes `/tmp` (playwright-core, mock servers) **and** the
  installed toolchains. `serve.sh` re-provisions; for ad-hoc runs the toolchain
  lives at `/workspace/.toolchain/cargo/bin` and must be on the server's PATH,
  with `RUSTUP_HOME=/workspace/.toolchain/rustup` and
  `CARGO_HOME=/workspace/.toolchain/cargo` exported, or the Rust runtime is
  reported missing.
- Run the server ad-hoc with:
  `MTS_DB=/tmp/v4.sqlite MTS_PORT=12060 PATH="/workspace/.toolchain/cargo/bin:$PATH" RUSTUP_HOME=… CARGO_HOME=… ./rust/mts-core/target/release/mts-core`
- `wafanyakazi.php` action to hire is **`add`** (not `hire`), and `services`
  must be an **array**; the singular `service` field is silently ignored, so the
  worker never matches a request. This is the #1 cause of "assignedTo is empty".
- `smoke.sh` is green when the only `ok:false` lines are the four intentional
  refusals (steps 8, 11, 12d, 13). Count `^== ` steps (26) rather than trusting
  a single grep.
