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
