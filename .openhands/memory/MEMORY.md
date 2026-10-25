# MBILINYI-TECH67 — project memory

## What this repo is
Mbilinyi Tech Solutions: a PHP site (pages + `/api/*.php`) that is being
replaced by a **Rust core** at `rust/mts-core` (axum). The frontend
(`assets/app.js`, `templates/`) is shared by both and must not need changes.

## Implemented (details in the dated daily logs)
- **Payroll** (`payroll.rs`): payslip per escrow release, banded PAYE/NSSF/WCF,
  TRA return. **Stripe** (`payments/gateways.rs`): `gw_stripe_api_base` is
  overridable for a mock; webhooks HMAC-verified. **101 agents** (`agents.rs`).
  **Video call** (`call.rs` + `assets/call.js`): real RTCPeerConnection.
- **The coding panel shows the real OpenHands transcript**
  (`coding.php?action=transcript&id=…`). An iframe of OpenHands Cloud is
  impossible (`frame-ancestors 'self'`); the conversation id deep-links to
  `{base}/canvas/conversations/{id}` instead.
- **Search-result icon**: head advertises 96px + 144px icons (Google ignores
  <48px and draws a globe). Test `every_advertised_icon_exists_on_disk` guards
  the head/file drift.

## Deploy (Fly.io) — details in `2026-10-24-fly-deploy.md`
- **LIVE at https://mbilinyitech.co.tz** (app `mbilinyitech-tz` — the plain name
  is taken on Fly; `mbilinyitech-tz.fly.dev`). Region jnb, volume `mts_data` at
  `/data`. Deploy: `FLY_API_TOKEN=$(cat /workspace/.secrets/fly.token) flyctl
  deploy --remote-only --yes -a mbilinyitech-tz` (remote builder, no local Docker).
- DNS is CUT OVER to Fly: apex A `66.241.125.44`, AAAA `2a09:8280:1::1cb:226c:0`
  (`server: Fly/…`, `via: fly.io`). The old cPanel host (91.204.209.30,
  LiteSpeed) served stale cached 404s during the switch — confirm with
  `dig @ns1.rodlinehost.com` before believing a 404.
- An app-scoped Fly token CANNOT create apps (`UNAUTHORIZED`); that needs an ORG
  token. **Never deploy over `mtechos`** — a different product (Agent-Canvas IDE
  at mtech.mbilinyitech.co.tz).
- `MTS_OTP_PEPPER` and `OPENHANDS_API_KEY` are real Fly secrets. Never set
  `OTP_SHOW_IN_RESPONSE` live — it returns the sign-in code in the response.
- **Mbilinyi Coding** (`coding.rs`, admin tab) is a REAL OpenHands agent:
  creates an app-conversation, resolves it from the start-task, reads the answer
  from the events. Proven end to end on prod. Traps: the events search must use
  `limit<=100` (200 → HTTP 422 → every job looks stuck "running"); a dead
  conversation emits `ConversationErrorEvent` and NO agent message, so
  `read_events()` must mark it failed or the poll hangs. Hosted coding needs
  credits on the OpenHands account (or the CEO's own LLM key).
- **LLM / fine-tuning** (`api/llm.php`) is wired, needs a provider key in
  Admin → Settings; with none, `engine.php action=run_job` records `blocked`
  honestly instead of faking success.
- **Portal sections are real URLs** (`/admin/payroll`, `/client/invoices`,
  `/mtaalamu/tools`); an unknown section 404s. Deep link → sign-in →
  `/auth?returnTo=admin&section=payroll`. The param is `section`, NOT `tab`
  (`tab` already opens the sign-up form). **Keep `templates/*.html` and the
  `.php` pages in step** — `admin.php` had 14 nav rows vs the template's 29;
  `the_rust_and_php_section_lists_agree` does NOT catch markup drift.
- **API action names that cost time**: `security` = `scan` + `kind`
  (`tls dns cert exposure headers …`, no `action=headers`); `migrate` = `status
  plan export verify import list download`; `engine` = `runs run stats cancel
  queue capabilities`; `llm` = `status save clear_key test`; `runtime action=run`
  returns the program text in **`output`**, not `stdout`.
- **All tools verified live on prod**: 101 agents, runtimes 7/7 ready, tools
  109, security `tls`/`dns`/`cert` real, `coding.php action=test` →
  `connected:true` (account onearmy756@gmail.com).
- **git push**: `$GITHUB_TOKEN` shows push=true via the API yet git-over-HTTPS is
  refused (403). Only the CEO's classic PAT pushes, via a one-off URL.
- **Deposit bug (only on Fly)**: two queries disagreed (DESC `%deposit%` vs ASC
  `%50%Deposit%`), now one `deposit_invoice_row()`. When a Rust port differs from
  PHP *only in production*, diff the query (ORDER BY/LIKE/LIMIT), not the routes.

## Facts worth keeping
- **MTECHOS is a DIFFERENT product, broad in labels not capability** — its
  1040 "skills" are `action:"plan"` checklists with no runtime. Compare tool
  counts only after checking the other side actually runs something.
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
- **Security tools: 26 in the family, all real**, in `sandbox.rs`, via
  `security.php?action=scan&kind=<k>`. A bare credential *prefix* is not a
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
- **Hire a worker** with `wafanyakazi.php` action **`add`** and `services` as an
  **array** (singular `service` is ignored → the worker never matches a request).
- `rust/mts-core/target/` (~1.3 GB) and the runtime DB are git-ignored.

## The backlog lives in `.trae/specs/` (2026-10-25)
When the user's request is truncated or says "Add C..." / "Vyote ADD", read
`.trae/specs/spec.md` + `tasks.md` — 11 tasks, FR-1..FR-9. As of commit
13a3041 all are implemented; FR-7 (visitor logging + analytics fields) and FR-9
(contract signing emails) were the last two, and they were missing ONLY on the
Rust core (the PHP shell already had them). Pattern: a spec feature can exist in
PHP + the frontend + the DB yet be absent from the Rust handler that serves
production — always check the Rust core, not just `api/*.php`.

- **The landing view is duplicated**: `templates/index.html` (Rust) and
  `index.php` (PHP/cPanel) are two hand-kept copies of the same HTML. Any
  homepage edit must land in BOTH or the two engines drift. The `/company` and
  `/services` pages existed for a while with no card linking to them from the
  homepage — only the header nav (fixed 2026-10-25, Fly v26).

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

- **`services_hidden` is the ONLY key that hides a service** from the public
  grid (empty value = everything visible). The Rust admin `plan_toggle` used to
  append its argument to that key instead of flipping `pricing_plans.is_active`
  (as PHP does), so enough plan toggles hid the **whole catalogue** on live —
  the SERVICES page rendered blank. Fixed + guarded by two tests. Never write
  `services_hidden` from anywhere but the settings screen.
- Live prod admin: `mbilinyitech@gmail.com` / `Admin@2026`, then
  `verify-login-pin` `2645`. `GITHUB_TOKEN` gets 403 pushing this repo — use the
  CEO PAT from the user's message in a one-off URL.

Details: `2026-10-21-mbilinyi-rust-core.md`.
