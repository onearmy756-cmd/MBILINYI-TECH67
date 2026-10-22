# MBILINYI-TECH67 — Rust core port (2026-10-21)

## Repo
- `onearmy756-cmd/MBILINYI-TECH67`, default branch **master** (there is also a
  stale `main` ref). **The repo is PUBLIC, not private** — checked via the API.
  History is clean of real secrets; the only "credentials" in it are the seeded
  demo passwords (`Admin@2026`, `demo123`) written into `scripts/smoke.sh`.
- Clone at `/workspace/MBILINYI-TECH67`. Other lookalike dirs exist
  (`mbilinyitech67` = empty repo of the same name, `MBILINYITECH`, `MTECHOS`,
  `MBILINYI-AI`).
- The CEO's PAT (`ghp_…`) has push+admin on this repo. Use it in a one-off URL,
  never in `.git/config`.

## Rust core (`rust/mts-core`)
axum binary replacing the PHP backend; frontend (`assets/app.js`, `templates/`)
untouched. Build:
`CARGO_HOME=/workspace/.toolchain/cargo RUSTUP_HOME=/workspace/.toolchain/rustup PATH=/workspace/.toolchain/cargo/bin:$PATH cargo build --release`
Toolchain lives in `/workspace/.toolchain` because `/workspace` survives a
session reset and `~/.cargo` does not.

### Things that cost time
- **One connection, not per-request.** PHP opened a connection per request, so
  deleting `storage/mts.sqlite` mid-run gave a fresh DB. A long-lived Rust
  connection keeps writing to the deleted inode. `db.rs` now stats the file
  before every query and re-opens + re-bootstraps when the inode/mtime changes.
- **`/page.php` must 301 only for GET/HEAD.** Redirecting a POST turns it into a
  GET and drops the body — every API call would break. Mirrors `router.php`.
- **Guest request submission creates an account** and signs it in
  (`api/requests.php`). The Rust version must also emit the `Set-Cookie`, or the
  guest's tracking page is not theirs.
- **The 50/50 deposit gate reads the EARLIEST deposit invoice** for the tracking
  id (`title LIKE '%50%Deposit%' ORDER BY created_at ASC LIMIT 1`), not the sum
  of unpaid ones. Getting this wrong made the gate pass when it should block.
- Invoice/ticket ownership also runs through `tracking_id` → request owner
  (by `user_id` or `email`), not just `user_id`.
- `tickets` has no `priority`; replies live in `ticket_replies(by_name, body)`.

### Layout facts
- Templates resolve from the **repo root** `templates/` (the duplicate under
  `rust/mts-core/templates` was deleted — two copies drift).
- `rust/mts-core/target/` is ~1.3 GB and git-ignored.
- `scripts/serve.sh` starts the Rust core on port 12000, installs Rust/R if
  missing, and rebuilds when sources are newer than the binary.

## Verification
- `scripts/smoke.sh` (BASE=http://127.0.0.1:8080): all 21 steps pass, repeatably.
  The four `ok:false` replies are the intended refusals (steps 8, 11, 12, 13).
- An API sweep over 43 endpoints passes; the two errors are intended 403s.
- `cargo test`: 4 pass (webauthn).
- R reports return real results (`r:true`) for revenue/requests/payments/escrow.
