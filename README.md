# Mbilinyi Tech Solutions — PHP + MySQL

The original single-file HTML site, now a real multi-page PHP system backed by a
real MySQL database. The markup, design, copy and EN/SW toggle are unchanged —
only the JavaScript data layer was rewired to the PHP API (and the checkbox/BD
details moved with their sections).

## Pages — kila sehemu ina page yake

Clicking a nav item **goes to a real new page** (no anchored scrolling):

| Page         | Contents |
|--------------|----------|
| `index.php`  | Landing: hero + CEO (contact strip) |
| `services.php` | 8 service lines |
| `stack.php`  | Tech stack + proficiency matrix |
| `ai.php`     | AI fine-tuning + live demo |
| `pricing.php`| Maintenance & security plans + 50/50 payment terms |
| `process.php`| How it works (5 steps) + payment rule |
| `work.php`   | Portfolio & testimonials |
| `track.php`  | Track Your Request |
| `contact.php`| Contact form & company details |
| `auth.php`   | Sign in / create account |
| `client.php` | Client portal (overview, requests, quotes, contracts, invoices, support) |
| `admin.php`  | Admin/CEO control center (dashboard, requests, quotes, contracts, accounts, invoices, tickets) |

Shared: `partials/head.php`, `partials/header.php` (real `<a href>` nav +
mobile menu), `partials/footer.php`, `assets/app.js` (frontend data layer),
`api/*.php` (JSON endpoints), `config/*.php` (session, CSRF, DB, schema, seed),
`lib/email.php` (Resend).

## Database — MySQL

Connection is read from `DATABASE_URL` (or `MYSQL_URL`) in the classic form:

```
mysql://user:password@host:3306/dbname
```

If no usable URL is present in the environment, the app falls back to the local
MariaDB instance created for development (`127.0.0.1`, database `mbilinyi`,
user `mts`). Schema (`config/schema.php`) and demo data (`config/seed.php`) are
applied automatically and idempotently on the first request.

## Demo data

| Account | Role |
|---------|------|
| `admin@mbilinyitech.co.tz` / `Admin@2026` | Admin / CEO |
| `demo@client.com` / `demo123` | Client (Amina Juma) |
| `baraka@kilimo.co.tz` / `demo123` | Client (Baraka Mwansa) |

Tracking IDs: `MTS-2026-4810` (Quoted + contract + 50% deposit invoice),
`MTS-2026-5523` (Under Review).

## CRUD (kuongeza / kuhariri / kufuta)

- **Requests** — client submits via the 5-step wizard; admin can edit every
  field, change status, add a CEO note, generate contract, delete.
- **Quotations** — admin builds line items (VAT 18% auto), edits, deletes.
- **Contracts** — generated from a quotation, editable (price/scope/duration)
  while the client has not signed, deletable.
- **Invoices** — create, edit, mark paid, delete (admin); client pays with an
  M-Pesa reference.
- **Accounts** — add, edit, suspend/activate, switch role, delete, view-as.
- **Tickets** — create (contact form or portal), reply, close/reopen, delete.

## Contract signing flow (mkataba) + Kanuni ya Malipo 50/50

1. Admin creates the contract from an approved quotation → status **Sent**,
   and a **50% deposit invoice is generated automatically** (also available
   under Contracts → *Create 50% Deposit Invoice* / Invoices → *New Invoice*).
2. Admin may still change the **price** while the client has not signed.
3. The **client signs first** → status **Client Signed**.
4. **Kanuni ya 50%:** kuanza project (CEO countersign / status In Progress)
   kunazuwa mpaka deposit ya **50%** ilipwe. The API returns HTTP 402 with an
   explicit message (“Lipa 50% kwanza kabla project kuanza”) and the signing
   panel offers *Create 50% Deposit Invoice* / *Mark Deposit Paid*.
5. Once the deposit invoice is **Paid**, the **admin/CEO countersigns second**
   → status **Signed**, request moves to **In Progress**. Both signatures +
   timestamps are stored on the `contracts` table.
6. After the client signs, the price is locked. Nusu iliyobaki (50% balance)
   inalipwa (na kuwandikwa kama invoice ya “Balance”) **baada ya kukabidhi**
   project.

## Email (Resend)

Notifications are sent through the Resend REST API using `RESEND_API_KEY`
(fail-soft: an email failure never blocks a request). Requests to send from an
unverified address return 403/422 and are logged only.

## Commands

```bash
npm start            # php -S 0.0.0.0:$PORT -t .
npm run typecheck    # node --check assets/app.js
bash scripts/smoke.sh   # end-to-end API test against the running preview (BASE=... env)
```

Production deploys on Freebuff use a Node.js-only builder, so this PHP app is
served by the sandbox preview rather than the static hosting pipeline.
