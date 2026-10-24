#!/usr/bin/env bash
# Start the site on the preview port.
#
# The backend is Rust (`rust/mts-core`). A bare sandbox can be reset between
# sessions and lose everything installed under /usr — including the Rust
# toolchain — which shows up as "502" on the public URL with no obvious cause.
# This script makes starting the site one command either way.
#
# The toolchain is kept in /workspace/.toolchain rather than ~/.cargo, because
# /workspace survives a session reset and the home directory does not. That is
# the difference between a one-minute start and a ten-minute rebuild.
#
#   bash scripts/serve.sh          # port 12000 (the exposed preview port)
#   PORT=8080 bash scripts/serve.sh
#   FORCE_BUILD=1 bash scripts/serve.sh   # rebuild even if a binary exists
set -u

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PORT="${PORT:-12000}"
LOG="${LOG:-/tmp/mts-app.log}"
DB="${MTS_DB:-$ROOT/storage/mts.sqlite}"

TOOLCHAIN="${MTS_TOOLCHAIN:-/workspace/.toolchain}"
export CARGO_HOME="${CARGO_HOME:-$TOOLCHAIN/cargo}"
export RUSTUP_HOME="${RUSTUP_HOME:-$TOOLCHAIN/rustup}"
export PATH="$CARGO_HOME/bin:$PATH"

BIN="$ROOT/rust/mts-core/target/release/mts-core"

log() { echo "[serve] $*"; }

# --- Rust toolchain -------------------------------------------------------
if ! command -v cargo >/dev/null 2>&1; then
  log "Rust is missing — installing into $TOOLCHAIN (this takes a few minutes)..."
  mkdir -p "$TOOLCHAIN"
  if ! curl -sSf https://sh.rustup.rs -o /tmp/rustup-init.sh; then
    echo "Could not download rustup. Check the network, or install Rust by hand." >&2
    exit 1
  fi
  sh /tmp/rustup-init.sh -y --no-modify-path --profile minimal --default-toolchain stable || {
    echo "rustup failed. See the output above." >&2
    exit 1
  }
  export PATH="$CARGO_HOME/bin:$PATH"
  command -v cargo >/dev/null 2>&1 || {
    echo "cargo is still not on PATH after installing. Expected it at $CARGO_HOME/bin." >&2
    exit 1
  }
fi
log "cargo: $(cargo --version 2>/dev/null || echo unknown)"

# --- R (optional) ---------------------------------------------------------
# Only the analytics reports need R, and they degrade to a CSV export without
# it, so a failure here is a warning rather than a stop.
if ! command -v Rscript >/dev/null 2>&1; then
  log "R is missing — installing for the analytics reports..."
  if command -v apt-get >/dev/null 2>&1; then
    sudo mkdir -p /var/lib/apt/lists/partial 2>/dev/null || true
    sudo apt-get update -qq >/dev/null 2>&1 || true
    sudo DEBIAN_FRONTEND=noninteractive apt-get install -y -qq r-base-core >/dev/null 2>&1 \
      || log "R could not be installed; the reports will offer a CSV export instead."
  fi
fi
command -v Rscript >/dev/null 2>&1 && log "R: $(Rscript --version 2>&1 | head -1)" \
  || log "R not available — reports will fall back to CSV."

# --- Go, Elixir, R (the runtimes the platform advertises) ------------------
# The Runtimes screen only offers a language it can actually run, so these are
# installed on the same footing as R: a missing one degrades that language, it
# does not stop the platform.
if ! command -v go >/dev/null 2>&1 || ! command -v elixir >/dev/null 2>&1; then
  log "Installing the Go and Elixir runtimes..."
  if command -v apt-get >/dev/null 2>&1; then
    sudo mkdir -p /var/lib/apt/lists/partial 2>/dev/null || true
    sudo apt-get update -qq >/dev/null 2>&1 || true
    sudo DEBIAN_FRONTEND=noninteractive apt-get install -y -qq golang-go elixir >/dev/null 2>&1 \
      || log "Go/Elixir could not be installed; those languages will be hidden."
  fi
fi
# `go --version` is not a flag — Go uses the `go version` subcommand. `elixir
# --version` boots an Erlang VM that does not exit on its own, so it is run under
# a timeout and only the version line is kept.
if command -v go >/dev/null 2>&1; then
  log "go: $(go version 2>&1 | head -1)"
else
  log "go not available."
fi
if command -v elixir >/dev/null 2>&1; then
  log "elixir: $(timeout 15 elixir --version 2>/dev/null | grep -m1 Elixir || echo installed)"
else
  log "elixir not available."
fi

# --- build ----------------------------------------------------------------
# A binary built before a session reset is stale: the toolchain that made it is
# gone, and so are the object files it linked against.
NEED_BUILD=0
[ -n "${FORCE_BUILD:-}" ] && NEED_BUILD=1
[ -x "$BIN" ] || NEED_BUILD=1
if [ "$NEED_BUILD" = "0" ]; then
  if [ -n "$(find "$ROOT/rust/mts-core/src" "$ROOT/rust/mts-core/Cargo.toml" \
        -newer "$BIN" 2>/dev/null | head -1)" ]; then
    log "Sources are newer than the binary — rebuilding."
    NEED_BUILD=1
  fi
fi

if [ "$NEED_BUILD" = "1" ]; then
  log "Building mts-core (release)..."
  ( cd "$ROOT/rust/mts-core" && cargo build --release ) || {
    echo "Build failed. See the output above." >&2
    exit 1
  }
else
  log "Binary is up to date."
fi

[ -x "$BIN" ] || { echo "No binary at $BIN after building." >&2; exit 1; }

# --- run ------------------------------------------------------------------
# Replace any previous instance so a stale process cannot hold the port.
pkill -f "mts-core" 2>/dev/null
sleep 1

mkdir -p "$(dirname "$DB")"
cd "$ROOT" || exit 1
MTS_DB="$DB" MTS_PORT="$PORT" \
  setsid nohup "$BIN" >"$LOG" 2>&1 </dev/null &

# Wait for the port rather than sleeping a fixed time, so a slow first boot
# does not look like a failure.
code=000
for _ in $(seq 1 30); do
  code=$(curl -s -m 5 -o /dev/null -w '%{http_code}' "http://127.0.0.1:${PORT}/" 2>/dev/null || echo 000)
  [ "$code" = "200" ] && break
  sleep 1
done

if [ "$code" = "200" ]; then
  log "Site is up on port ${PORT} (log: ${LOG}, db: ${DB})."
else
  echo "Site did not answer on port ${PORT} (last code ${code}). See ${LOG}." >&2
  tail -20 "$LOG" >&2 || true
  exit 1
fi
