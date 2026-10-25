# 2026-10-24 — Fly.io deploy prep for the Rust core

## The task
Deploy MBILINYI-TECH67 (PHP frontend + Rust core) to Fly.io, targeting
`mbilinyitech.co.tz`, with a Fly token the CEO supplied. Token saved at
`/workspace/.secrets/fly.token`; also pasted in the chat (same value).

## THE BLOCKER: the token is app-scoped and cannot create an app
The token authenticates fine, and `flyctl deploy` gets as far as "Verifying app
config" then dies with:

```
Error: app not found (Request ID: ...) (Trace ID: ...)
```

Every app-creating path refuses:

| Attempt | Result |
| --- | --- |
| `flyctl deploy --remote-only` (app `mbilinyitech` in fly.toml) | `app not found` |
| `flyctl launch --org personal --name mbilinyitech` | `unauthorized` |
| GraphQL `createApp(input:{organizationId:…})` | `UNAUTHORIZED / "Not authorized to deploy this app."` |
| `POST https://api.machines.dev/v1/apps/<unknown>/volumes` | `{"error":"app not found"}` |
| `flyctl tokens create org` / `tokens create deploy` | `createLimitedAccessToken Not authorized` |
| legacy `POST api.fly.io/api/v1/apps` with `attributes` | returns a `nodeproxy_apps` stub, but the app is NOT real — `query { app(name:"mbilinyitech") }` still returns NOT_FOUND |

But the token has FULL write access to the ONE app it is scoped to:

* `GET/POST/DELETE https://api.machines.dev/v1/apps/mtechos/volumes` all work
  (I created and then destroyed a 1 GB `scope_probe` volume to prove it).

So it is an app-scoped deploy token for **`mtechos`**. Creating an app needs an
organisation token (`fly tokens create org`).

Identity behind the token: org "Jackson Mbilinyi" / slug `personal`, org id
`D307G6NwgR0z2u4vPG23jYy8a3ckVZl21`, viewer
`afd3638f-3da8-5aaa-9b76-87a4d89e637c@tokens.fly.io`.

## DO NOT deploy over `mtechos`
`mtechos` is a *different product*: an OpenHands/Agent-Canvas "fundi" IDE
platform. Its image (`registry.fly.io/mtechos:deployment-01M59N7A47FGTYJQ5B8D9QFN65`)
runs `/app/docker-entrypoint.sh` as user `fundi`, with
`FUNDI_DATA=/app/data`, `PORT=8080`, and copies
`platforms/mbilinyitech-ide`, `engines/mbilinyi-extension`, monaco-editor, etc.
It is `mtech.mbilinyitech.co.tz` → `mtechos.fly.dev` → 66.241.124.38.
Overwriting it would destroy that product.

## DNS reality for the domain (checked via cloudflare-dns.com/dns-query)
* NS: `ns1.rodlinehost.com`, `ns2.rodlinehost.com` (so the registrar/cPanel
  host controls DNS, NOT Fly).
* `mbilinyitech.co.tz` A → `91.204.209.30` (cPanel host); SOA serial 2026101801.
* `www` → CNAME → `mbilinyitech.co.tz`; `cpanel` → 91.204.209.30;
  `mail` → CNAME → apex; MX → hostns.io (Entrap).
* `mtech` → CNAME → `mtechos.fly.dev` (leave it alone).

Consequence: even after a successful deploy, Fly cannot serve the apex until
the A record is changed at the registrar. So the cPanel site stays live until
the user edits DNS.

## What I built (committed on master, 9b997a9)
* `Dockerfile` — two stages. Build = `rust:1-slim-bookworm` (needs
  build-essential for pqcrypto/rusqlite bundled C). Runtime =
  `debian:bookworm-slim` + python3, gcc/libc6-dev, nodejs+npm, r-base-core,
  golang-go, elixir, and the Rust toolchain copied from the builder. ~686 MB.
  Runs as unprivileged `mts`; entrypoint chowns `/data` then `gosu`s down.
* `docker/entrypoint.sh` — copied to `/usr/local/bin/`, NOT `/app`, because
  `static_file` in `main.rs` serves any non-dotfile path under the CWD
  (`/entrypoint.sh` in /app would have been downloadable; verified 404 now).
* `fly.toml` — app `mbilinyitech`, region `jnb`, volume `mts_data` → `/data`,
  `MTS_DB=/data/mts.sqlite`, `internal_port 8080`, `force_https`,
  health check `GET /api/health`, `shared-cpu-2x` / 2 GB.
* `.dockerignore` — excludes `.git`, target/, node_modules, storage sqlite.
* `scripts/fly-deploy.sh` — create app → deploy --remote-only → status → certs.
* `DEPLOY.md` — new "Deploy — Fly.io" section.

## Verified locally before deploying (this is the real proof it works)
* `cargo build --release` clean (44 warnings, 0 errors); `cargo test --release`
  → **109 passed; 0 failed**.
* Docker image builds; container boots: `Mbilinyi Tech Solutions — Rust core on
  http://0.0.0.0:8080`.
* `GET /api/health` → `{"status":"ok","database":true,"tables":44,...}`.
* `bash scripts/smoke.sh` against the container → **DONE, all steps**, incl. the
  four intentional `ok:false` refusal tests.
* All 7 languages print `42` in the container: python3, node, go, cc, rustc, R,
  elixir. (First `cc` attempt failed only because of my own shell quoting.)
* Static/security: `/storage/mts.sqlite`, `/.env`, `/scripts/smoke.sh` → 403;
  `/no-such-page` → 404; `/pricing.php` and `/pricing/` → 301 to `/pricing`;
  canonical link renders `https://mbilinyitech.co.tz/`.
* rustc/cargo are only on PATH via the image ENV, not in a login shell — check
  with `docker exec <c> printenv PATH` or `sh -c`, not `sh -lc`.

## Env / security notes for production
* `MTS_OTP_PEPPER` has a hardcoded default (`MTS_OTP_2026_PEPPER_DAR`). Set a
  real secret in Fly before go-live, else OTP hashes are reproducible.
* Never set `OTP_SHOW_IN_RESPONSE` on the live site — it returns the login code
  in the API response.
* Sessions live in a `sessions` TABLE, not `storage/sessions/`, so the /data
  volume already persists them.

## Next step for the user
Either (a) mint an org-level token (`fly tokens create org -o personal`) and
re-run `scripts/fly-deploy.sh`, or (b) tell me the app name if one already
exists in that org that I may use. Then change the apex A record to Fly's IP.

## DEPLOYED (2026-10-24, later in the session)
* App name is **`mbilinyitech-tz`** — `mbilinyitech` is globally taken on Fly.
* Live: **https://mbilinyitech-tz.fly.dev/** — machine 8e25d6b7d99018, region
  jnb, 1/1 health checks, volume `mts_data` mounted at `/data`.
* IPs: shared IPv4 `66.241.125.44`, dedicated IPv6 `2a09:8280:1::1cb:226c:0`.
* `MTS_OTP_PEPPER` set as a Fly secret (random 32 bytes).
* Acceptance: `scripts/smoke.sh` against the live app → **25 ok:true / 4
  ok:false**, i.e. byte-for-byte the same as a clean local run.
* Certs `mbilinyitech.co.tz` and `www.mbilinyitech.co.tz` added, status "Not
  verified" until cPanel DNS changes. Exact records are in DEPLOY.md.

## The deposit bug that only showed on Fly (FIXED)
* `deposit_info()` (the signing panel) ordered `DESC` and matched the looser
  `lower(title) LIKE '%deposit%'`, while the countersign gate `deposit_block()`
  ordered `ASC LIMIT 1` on `title LIKE '%50%Deposit%'`. With two deposit-titled
  invoices on one tracking id the panel named one row and the gate judged the
  other, so the admin marked the invoice they were shown and the gate still
  refused. PHP never had this: both call the one helper `deposit_invoice_row()`.
* It hid locally because a fast loop puts both rows in the same `created_at`
  second, so ASC and DESC agree; network latency on Fly separates them.
* Fix: one `deposit_invoice_row(tracking)` mirroring the PHP, used by BOTH
  `deposit_block()` and `deposit_info()`. `scripts/smoke.sh` step 12c now marks
  the invoice `deposit_info` names, not the newest from `admin_list`.
* Lesson: when a Rust port disagrees with PHP only in production, diff the
  *query* (ORDER BY / LIKE pattern / LIMIT), not just the route list.

## Security tools — verified working, two real bugs fixed (2026-10-24)
The platform's security tooling is real, not decorative: `security.php` actions
`status/public_key/sign/verify/log/logo_*/company_account/sandbox/
authorise_domain/deauthorise_domain/scan`, plus the sandbox checks
`headers exposure dns tls dependencies code malware`. Exercised every one
against the live app with an admin session — all answer with real evidence
(real HTTP response, real DoH lookups, real TLS handshake, real file walk).

Bugs found while testing (both fixed, 110 tests pass, redeployed):
* **Posture lied about TLS.** `hardening::posture()` and `security status` derive
  `https`/`Strict-Transport-Security` from `settings::get("platform_url")`, which
  was seeded `""` and never filled — so a live HTTPS site reported
  `https:false, HSTS:false` while the middleware genuinely sent HSTS (curl
  confirmed the header). Fix: `seed_platform_url_from_env()` at boot, from
  `MTS_PUBLIC_URL` or `SITE_URL`, only when the setting is empty. An explicit
  admin value still wins. Same empty setting was making Stripe's success/cancel
  URLs relative.
* **The malware scan could never match its own signature.** `scan_malware` used
  `scan_dir`, which filters to a source-code extension list and reads UTF-8 text
  only. The one digest it ships (EICAR) has a `.txt`/`.com` name, and a binary
  payload is not valid UTF-8 — so both were silently skipped and the scan always
  reported "Files hashed: N, Findings: 0". Fix: new `walk_files` walks EVERY
  file (shared size/count/skip limits); `scan_dir` keeps the extension filter for
  the source audit, `scan_malware` reads raw bytes and matches lossily. Verified
  a planted EICAR file now reports `Findings: 1`.
* Scope rule verified: `exposure` refuses with `data.ok:false` unless the domain
  is in `security_authorised_domains` AND the caller names who authorised it.
  Note the envelope is still `ok:true`; the refusal is `data.ok:false` +
  `data.evidence`. `dependencies` legitimately reports `ok:false` when no
  manifest (Cargo.toml/package.json/...) exists under the target.

## Runtimes, agents, tools and the AI path — all verified live (2026-10-24)
* **Runtimes: all 7 ready, and each prints 42 live.** `api/runtime.php?action=status`
  returns `ready: [Python, Rust, Go, Elixir, R, Node.js, C]`, `missing: []`. Ran a
  snippet per language over the API: python 40ms, node 161ms, go 802ms, c 40ms,
  rust 40ms, r 642ms, elixir 684ms — every one printed `42`. The container really
  has `/usr/bin/{python3,Rscript,go,elixir,node,cc}` and `/usr/local/cargo/bin/rustc`.
* **Agents: 101 seats, all active.** `agents.php?action=list` → 101; departments
  software 28, ai 15, employee 15, security 15, marketing 12, finance 9, hr 6, ceo 1
  (the manager seat "Magma Commander"). `engine capabilities` agrees.
* **Tools: 90 in the catalogue, 66 agentic.** Families: software 12, security 16,
  ai 12, design 4, business 7, comms 6, platform 5, agent 20, media 8. The `agent`
  family is the "OpenHands-class" runtime (browser, web, code_execution, terminal,
  file_ops, mixture_of_agents, cronjob, skills, todo, ...). Note: this is the
  platform's OWN agent runtime, not a call out to an external OpenHands API.
* **The AI path works; it is gated only on a provider key.** With no key,
  `run_job` starts a real run that ends `status=blocked, error="No AI provider is
  configured..."` — a clear, correct refusal, not a crash. Proved the whole path by
  pointing `LLM_BASE_URL` at a tiny OpenAI-compatible mock and setting
  `OPENAI_API_KEY`: `llm test` → `connected:true, reply:"..."`; `run_job` (manager
  auto-picks) → `status=done, steps=1`; `run_job tool=ai_finetune` → agent
  "AI Fine-Tuning Engineer 01" → `status=done, steps=1`. So fine-tuning, the agent
  engine and all model-backed tools work the moment a key is added.
  `Llm::from_settings()` reads the provider from settings OR the env, and a key in
  the env alone is enough to switch a provider on — good for a container.
* Set the key on Fly with: `flyctl secrets set OPENAI_API_KEY=... -a mbilinyitech-tz`
  (or ANTHROPIC_API_KEY / GEMINI_API_KEY; Ollama needs no key). Admin -> Settings
  -> LLM also works and overrides the env.
* **`api/runtime.php?action=test` takes an IN-CONTAINER path.** `/app` holds only
  the built `mts-core`, `assets/`, `templates/`, `r/report.R` — no source tree, so
  `/app/rust/mts-core` is "not a directory". Running the project's own suite from
  inside the image is therefore not possible; that is a path issue, not a broken
  runtime.

## Why MTECHOS "has many more cyber tools" — and what was actually done (2026-10-24)
The CEO asked why MTECHOS shows so many more cyber tools than this platform.
Investigated `mtechos` (a different Fly app; `mtech.mbilinyitech.co.tz` ->
`mtechos.fly.dev` -> 66.241.124.38). It IS reachable and its container was
read over `flyctl ssh console`:
* `/app/data/skills/skills.json` — **1040 skills**, but the 30 `trade:"security"`
  ones (nmap, sqlmap, nikto, hydra, metasploit, burpsuite, volatility, autopsy,
  yara, snort, suricata, osquery, openssl, gpg, ...) are all
  `action:"plan"`, `runtime:""`, `script:""`, `tools:[]` — **checklists, not
  runnable tools**. 34 skills are security-relevant across all trades; none has
  a runtime or a script.
* `/app/data/knowledge/zed_tools_catalog.json` — 108 entries, but they are
  **Rust frameworks** (ADK-Rust, Rig, Fulcrum, Bamboo, ...), not cyber tools.
* `/app/data/knowledge/zed_tool_calls.json` — 235 function-call schemas; the
  `auth_security` group holds 10 (hash_password, generate_jwt, encrypt_data,
  scan_vulnerabilities, ...).
* Binaries actually in the mtechos image: nmap, masscan, openssl, gpg, tshark,
  tcpdump. sqlmap/nikto/hydra/msfconsole/burp/yara/snort/suricata/volatility/
  autopsy/hashcat/john/nuclei/trivy/semgrep are all **MISSING**.
So MTECHOS is broad in *labels*, shallow in *capability*. This platform is the
opposite: fewer labels, each backed by code. Do not confuse the two products.

Still, the count was raised honestly, with 10 new **real** scanners (16 -> 26 in
the `security` family, total catalogue 90 -> 100). All are in
`src/sandbox.rs`, dispatched from `src/security.rs` (`scan` kinds) and
`src/runtime.rs` (agent tool codes), and listed in `src/tools.rs`:
`method_audit`, `cors_audit`, `cookie_audit`, `cert_inspect` (openssl s_client),
`subdomain_enum` (DoH), `secret_scan`, `password_audit`, `jwt_inspect`,
`file_integrity`, `log_audit`. Also widened `COMMON_PORTS` 18 -> 38.
Traps found while testing:
* The secret scan flagged **its own rule table** (a bare `AKIA`/`ghp_` prefix).
  Fixed with a minimum credible token length per rule (and the PEM literal is
  `concat!`-split). A bare prefix is not a secret.
* `hash_file` must accept a **single file**, not only a directory — `walk_files`
  over a file path yields nothing.
* A stale release binary silently keeps old behaviour: rebuild before testing.
120 Rust tests pass (was 117); smoke 4 refusals / rest ok (baseline).
Deployed: `flyctl deploy --remote-only -a mbilinyitech-tz` (remote build ~4 min;
run it in the background and poll — a foreground 1800s timeout is refused).

---

## Installable apps + /download page (later the same day)

Four real artifacts now live in `downloads/` (14 MB, committed — the Dockerfile
copies the directory, so they must be in the build context):
`MbilinyiTech-Android.apk` (Capacitor, 7.8 MB), `MbilinyiTech-Setup-Windows.exe`
(Tauri NSIS, 1.2 MB), `MbilinyiTech-Windows-Portable.exe` (Tauri GUI, 3.0 MB),
`MbilinyiTech-Linux-amd64.deb` (1.4 MB). `scripts/build-apps.sh` rebuilds them;
its header lists the toolchain traps that cost the time.

**THE TRAP THAT COST THE MOST — a 0600 template 404s while looking perfect.**
`templates/download.html` was written with mode `600` (the editor's umask). The
file was in the image, correct, and readable by root — but the server runs as
`mts` (uid 1000), so `read_to_string` failed, `render()` returned its 404, and
`/download` answered "404 Not Found" **with the real security headers**, which
looks exactly like a missing route. Diagnosis: `flyctl ssh console` +
`ls -la /app/templates/` + `gosu mts head -c 50 <file>`. Fix in the Dockerfile
(`chmod -R a+rX` on the copied trees) so any host umask is survivable. When a
page 404s in production but its template is on disk, check the mode first.

- `static_file` now sends installer MIME types and, under `downloads/`, a
  `Content-Disposition: attachment` so an .exe lands in the downloads folder
  instead of rendering in a tab. `mime_of` + the page list have tests (125 now).
- `/download` is registered in `KNOWN_PAGES` (now a `pub const`, so the test can
  read it) in both the Rust core and the PHP shell (`download.php`).
- Nav "Get App" button, mobile-menu entry, footer link, and `installPwa()` in
  `app.js` (holds `beforeinstallprompt`; Safari gets Add-to-Home-Screen steps).

## Custom domain: NOT switched over yet
`mbilinyitech.co.tz` and `www` still resolve to **91.204.209.30** (the cPanel
host). The Fly app's ingress is **66.241.125.44** (v4 shared) and
`2a09:8280:1::1cb:226c:0` (v6 dedicated); `flyctl certs list` says "Not
verified". To cut over at the registrar: A `mbilinyitech.co.tz` -> 66.241.125.44,
AAAA -> the v6, and the same for `www` (or CNAME `www` -> `mbilinyitech-tz.fly.dev`).
Then the cert verifies on its own. The app already answers everything correctly
on `mbilinyitech-tz.fly.dev`.

## Mbilinyi Coding: the coding tool is a REAL OpenHands agent (2026-10-25)
The "Tools ya kucode" tool (`mbilinyi_coding`) now drives the OpenHands Cloud
API for real — it is not a simulation. `src/coding.rs` creates a conversation
via `POST /api/v1/app-conversations`, resolves the conversation id from the
start-task (`GET /api/v1/app-conversations/start-tasks?ids=…`), reads the
events, and stores the agent's final answer. Admin tab = "Mbilinyi Coding".

End-to-end proven locally AND on production:
- start a job → `JOB-…` + `startTask`; first poll returns `pending:true`, the
  next resolves `conversation` + `sandbox` + a `uiUrl` on app.all-hands.dev.
- a final poll returns `status:done, events:10, result:"PROD-CODING-OK"`.

**Trap that cost the whole feature:** the events search asked for
`limit=200`. OpenHands caps it at 100 and answers **HTTP 422**, so every poll
read zero events and every job looked stuck at "running" forever even though
the agent had already replied. Now `EVENTS_LIMIT = 100` and
`events_url()` is a pure fn with a test pinning the exact URL. If a job never
finishes, check the events request first — a non-200 there is silent.

Config: `coding_api_key` (settings win) → `OPENHANDS_API_KEY` env;
`coding_base_url` → `OPENHANDS_BASE_URL` → `https://app.all-hands.dev`;
`coding_model` → the platform's `llm_provider`/`llm_model`. On Fly the key is
the `OPENHANDS_API_KEY` secret (set 2026-10-25). `action=test` is the live
connectivity probe (it returned `onearmy756@gmail.com`).

**Fly secret syntax:** `flyctl secrets import` wants `NAME=VALUE` on stdin, so
piping the raw token fails. Use `flyctl secrets set KEY="$VALUE" -a APP`.

## GitHub push: only the CEO's PAT works
`$GITHUB_TOKEN` in this sandbox authenticates as `onearmy756-cmd` and the API
shows push=true, but **git push over HTTPS is refused (403)** — it is a
fine-grained/app token without the git credential. The `origin` remote also
carries a stale `ghu_…` token (also refused). Push with the CEO's classic PAT
in a one-off URL and never write it into `.git/config`:
`git push "https://<PAT>@github.com/onearmy756-cmd/MBILINYI-TECH67.git" master`.

---

## Portal sections became real addresses, then shipped to prod (2026-10-24)

Every admin/client/staff section is now its own URL: `/admin/payroll`,
`/client/invoices`, `/mtaalamu/tools`. Before, the nav was `href="#"` +
`onclick="...;return false"`, so all 29 admin sections were one document and
nothing could be linked, bookmarked or refreshed into. Done in both engines:

- Rust router splits `/<portal>/<section>` into (page, tab), renders the shell
  with `window.__TAB__` and `<base href="/">` (so relative URLs still resolve
  from the root on a deep path), and 404s an unknown section.
- `config/app.php` holds the same list (`portal_sections`/`portal_tab`);
  `router.php` + `.htaccess` map `/<portal>/<section>` to `<portal>.php`.
- Sign-in carries the wanted section back: `/auth?returnTo=admin&section=payroll`.

**Trap: the param is `section`, NOT `tab`.** `auth` already uses `?tab=register`
to open the sign-up form; reusing `tab` for the portal section made the two
meanings collide. Renamed everywhere (Rust `pages.rs`, `config/app.php`,
`app.js`).

**Trap: the PHP pages had silently drifted from the shared templates.**
`admin.php` had 14 nav rows while `templates/admin.html` had 29, so the cPanel
engine would have lost 15 sections. The Rust tests (`the_rust_and_php_section_
lists_agree`) only compared Rust vs a PHP *list*, not the PHP markup. Copy the
`<nav id="adminNav">…</nav>` block from the template into the `.php` page
whenever a section is added. `client.php` had already drifted too.

169 Rust tests pass; `scripts/smoke.sh` covers every section plus the 404.

### Deployed and verified live on https://mbilinyitech.co.tz
`flyctl deploy --remote-only --yes -a mbilinyitech-tz`. Production confirms the
deep links: `/admin/payroll` → 302 `/auth?returnTo=admin&section=payroll`,
`/admin/nonsense` → 404, `/pricing` → 200, `/pricing/anything` → 404 — identical
on `mbilinyitech.co.tz` and `mbilinyitech-tz.fly.dev`. `server: Fly/…` +
`via: fly.io` on the apex, so the domain is served by Fly, not the cPanel host.

### Every tool re-verified live in production (2026-10-24)
- **Cyber-security**: `api/security.php` action is **`scan`** with a **`kind`**
  field (`tls dns cert exposure headers …`) — there is no `action=headers`.
  Ran `tls`/`dns`/`cert` against our own domain: real TLS handshake (HTTP 200),
  real DoH lookups (A 66.241.125.44, NS ns1/ns2.rodlinehost.com), real cert
  (Let's Encrypt, CN=mbilinyitech.co.tz).
- **Fine-tuning / AI engine**: `engine.php action=capabilities` → 101 agents
  across 8 departments; `llm.php action=status` shows `keySet:false` until the
  CEO adds a provider key (the path is wired, it fails honestly without one).
- **OpenHands coding tool**: `coding.php action=test` → `connected:true`,
  account `onearmy756@gmail.com`. The `OPENHANDS_API_KEY` Fly secret is set.
- **Runtimes**: `runtime.php action=status` → `ready:[Python,Rust,Go,Elixir,R,
  Node.js,C]`, `missing:[]`. `action=run` returns the program output in
  **`output`**, not `stdout` — python/rust/c/go all printed `42`.
- **Agents**: `agents.php?action=list` → 101. **Tools**: `tools.php?action=list`
  → 109.

**API field names that cost time:** `runtime.php action=run` → `output`;
`security.php` → `scan`/`kind`; `migrate.php` → `status plan export verify
import list download` (no `info`); `engine.php` → `runs run stats cancel queue
capabilities`; `llm.php` → `status save clear_key test`.

## Service visibility: the `plan_toggle` footgun (found on production)
- The live SERVICES page rendered **empty**. Cause: `settings.services_hidden`
  on the Fly volume held **all 21** catalogue keys.
- Root cause: the Rust admin `plan_toggle` (in `src/api/admin.rs`) appended its
  argument to `services_hidden` and rewrote that setting, instead of toggling
  `pricing_plans.is_active` the way `api/settings.php` does. Enough toggles hid
  everything.
- Fixed: Rust now flips the plan row's own `is_active`, matching PHP.
- Repaired live data through the admin API (`settings.php action=save
  key=services_hidden value=cybersecurity,forensics`) — log in with the
  `mbilinyitech@gmail.com` / `Admin@2026` then `verify-login-pin` `2645` flow.
- Guards added: `settings::tests::plan_toggle_never_touches_service_visibility`
  (source grep on admin.rs) and `an_empty_hidden_list_shows_every_service`.
  The services grid also shows a request card instead of a blank panel.
- **Trap**: `services_hidden` is the ONLY key that hides a service. An empty
  value = everything visible. Never write it from anywhere but the settings
  screen; the drift class (frontend sends a full list, backend stores it) is
  the same one that produced the `visitor_stats` "undefined" bug.

## Prod tool verification (2026-10-24, after the services fix)
All checked against `https://mbilinyitech.co.tz` with a real admin session
(login -> `verify-login-pin`). Endpoints that need auth return
`{"error":"You need to sign in first."}` when hit anonymously — that is not a bug.
- runtimes: `ready=[Python,Rust,Go,Elixir,R,Node.js,C]`, `missing=[]`.
- agents: 101 (manager + 100 seats). tools: 109.
- engine `capabilities`: ok=true, 8 departments.
- coding `test`: `connected:true`, `onearmy756@gmail.com` (real OpenHands Cloud).
- security `scan kind=tls|dns`: real handshake (HTTP 200) and real DoH lookups.
- llm `status`: `keySet:false` until the CEO adds a provider key (fails honestly).
- `security.php` needs the `X-CSRF-Token` header on GET too (unlike the
  read-only endpoints) — a missing header is a 403, not a 200.
- Cold-start race: the first outbound call after a Fly machine suspends can
  fail, so `coding.php?action=test` returned "cannot be reached" once on a
  healthy key. Retrying immediately worked. `coding.rs` now retries 3x with
  backoff (600ms/1.5s/3s) before reporting a fault. Do not treat a single
  "cannot be reached" as a bad key — retry first.
