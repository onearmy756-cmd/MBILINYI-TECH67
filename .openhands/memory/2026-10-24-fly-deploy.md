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
