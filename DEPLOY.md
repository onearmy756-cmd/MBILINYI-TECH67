# Deploy — cPanel, zero configuration

Clone this repository and open it. There is nothing to edit first.

```bash
git clone https://github.com/onearmy756-cmd/MBILINYI-TECH67.git
```

Then point the domain's document root at the cloned folder. The site boots on the
first request: it creates its own tables and its own demo content, and it picks a
database by itself.

## What happens on the first request

1. `config/env.php` reads `.env` if one exists. If not, it uses defaults.
2. `config/db.php` tries these in order and uses the first one that answers:
   - `DATABASE_URL` (or `MYSQL_URL` / `DB_URL`) — only if you set one.
   - A local MySQL/MariaDB on `127.0.0.1`.
   - **SQLite** at `storage/mts.sqlite` — created automatically.
3. `ensure_schema()` creates the tables, then seeds demo accounts and content.
   It is idempotent: run it a thousand times and nothing is duplicated.

So a host with MySQL uses MySQL. A host without MySQL still runs, on a SQLite
file. No error page, no manual SQL import.

## Requirements

- PHP 8.1 or newer.
- The `pdo_sqlite` extension (nearly always on) **or** `pdo_mysql`.
- Apache with `mod_rewrite` for clean URLs (`/pricing`, not `/pricing.php`).
  The shipped `.htaccess` does this. Nginx ignores `.htaccess`; the equivalent
  server block is written at the bottom of that file.

## Optional: use cPanel MySQL instead of SQLite

SQLite is fine for a single small site, but MySQL is the better choice if you
expect many visitors. cPanel → **MySQL Databases**:

1. Create a database, e.g. `cpuser_mbilinyi`.
2. Create a user, e.g. `cpuser_mts`, and give it **All Privileges** on that database.
3. Copy `.env.example` to `.env` and fill in one line:

```ini
DATABASE_URL=mysql://cpuser_mts:YOURPASSWORD@localhost:3306/cpuser_mbilinyi
```

The four-value shorthand that cPanel shows also works:

```ini
DATABASE_URL=localhost|cpuser_mbilinyi|cpuser_mts|YOURPASSWORD
```

If the database does not exist yet, the app creates it on the first request.
Leave `DATABASE_URL` empty to go back to automatic selection.

## Optional: email

Copy `.env.example` to `.env` and fill in the `SMTP_*` values. Without them the
site still works — outgoing mail is written to the PHP error log instead of being
sent. `/setup` walks through Gmail App Password setup.

## Permissions

These folders must be writable by the web server user:

```bash
chmod 755 storage
```

`storage/` holds sessions and, on SQLite, the database file. Nothing inside
`storage/`, `config/` or `scripts/` is reachable over HTTP: `.htaccess` returns
403 for them, and `storage/.htaccess` denies access even if the root file is
lost.

## Check the deployment

Open `/deploy`. It reports PHP version, the database driver in use, whether the
schema is ready, and which extensions are present. It shows no secrets.

## Pull updates

```bash
git pull
```

New tables and columns are added on the next request; existing rows are kept.

---

# Deploy — Fly.io (the Rust core)

The Rust core in `rust/mts-core` is a complete server: it answers every page and
every `/api/*` call that `assets/app.js` makes. On Fly it replaces the PHP
runtime entirely, so `Dockerfile` builds only the Rust binary.

```bash
FLY_API_TOKEN=... bash scripts/fly-deploy.sh
```

`fly.toml` and the `Dockerfile` are checked in, so the script only creates the
app, deploys, and adds the certificates. The build runs on Fly's builders
(`--remote-only`); nothing needs to be installed locally.

## What the image contains

The runtime is Debian with the toolchains the platform advertises, because
`langrun.rs` only offers a language it can actually run. Python, Rust, Go,
Elixir, R, Node.js and C are all present and verified printing `42`.

The server runs as the unprivileged `mts` account. The entrypoint hands the
volume at `/data` to that account and then drops privileges, because the
platform executes code that visitors submit.

## The database

`MTS_DB` is `/data/mts.sqlite`, on the `mts_data` volume, so the database and
the session table survive a redeploy. A deployment with no volume would reset
the site on every release.

## The custom domain

`mbilinyitech.co.tz` currently resolves to `91.204.209.30`, the cPanel host. Fly
cannot take over the domain until that record changes at the registrar:

| Type | Name | Value |
| --- | --- | --- |
| A | `mbilinyitech.co.tz` | IPv4 from `fly ips list` |
| AAAA | `mbilinyitech.co.tz` | IPv6 from `fly ips list` |
| CNAME | `www` | `<app>.fly.dev` |

Keep `mtech.mbilinyitech.co.tz` pointing at `mtechos.fly.dev`: that is a
different application.

## A token that cannot create apps

A Fly token scoped to one existing app can deploy *that* app but returns
`UNAUTHORIZED / "Not authorized to deploy this app"` on `createApp`. If
`scripts/fly-deploy.sh` stops at step 1, the token needs to be an
organisation-level token (`fly tokens create org`) rather than an app token.

