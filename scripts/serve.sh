#!/usr/bin/env bash
# Start the site on the preview port, installing PHP first if it is missing.
#
# A bare sandbox can be reset between sessions and lose /usr/bin/php, which
# shows up as "502" on the public URL with no obvious cause. This script makes
# starting the site one command either way.
#
#   bash scripts/serve.sh          # port 12000 (the exposed preview port)
#   PORT=8080 bash scripts/serve.sh
set -u

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PORT="${PORT:-12000}"
LOG="${LOG:-/tmp/mts-app.log}"

need_php() {
  command -v php >/dev/null 2>&1 || return 0
  php -r 'exit(class_exists("PDO") && extension_loaded("pdo_sqlite") ? 0 : 1);' || return 0
  return 1
}

if need_php; then
  echo "PHP (with PDO + SQLite) is missing — installing..."
  if command -v apt-get >/dev/null 2>&1; then
    sudo apt-get update -qq || true
    sudo DEBIAN_FRONTEND=noninteractive apt-get install -y -qq \
      php-cli php-sqlite3 php-mysql php-curl php-mbstring || {
        echo "Could not install PHP automatically. Install php-cli and php-sqlite3 by hand." >&2
        exit 1
      }
  else
    echo "No apt-get here — install php-cli and php-sqlite3 by hand." >&2
    exit 1
  fi
fi

# Replace any previous instance so a stale process cannot hold the port.
pkill -f "php -S 0.0.0.0:${PORT}" 2>/dev/null
sleep 1

cd "$ROOT" || exit 1
setsid nohup php -S "0.0.0.0:${PORT}" -t . router.php >"$LOG" 2>&1 </dev/null &
sleep 4

code=$(curl -s -m 15 -o /dev/null -w '%{http_code}' "http://127.0.0.1:${PORT}/" || echo 000)
if [ "$code" = "200" ]; then
  echo "Site is up on port ${PORT} (log: ${LOG})."
else
  echo "Site did not answer on port ${PORT} (last code ${code}). See ${LOG}." >&2
  tail -20 "$LOG" >&2 || true
  exit 1
fi
