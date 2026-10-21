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
