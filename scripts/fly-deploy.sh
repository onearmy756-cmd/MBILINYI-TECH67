#!/usr/bin/env bash
# Deploy the Mbilinyi Tech Solutions Rust core to Fly.io.
#
#   FLY_API_TOKEN=... bash scripts/fly-deploy.sh
#
# The token must be able to create an app in the target organisation. A token
# scoped to one existing app can deploy *that* app but cannot create a new one,
# which is the usual reason this script stops at step 1.
set -euo pipefail

APP="${FLY_APP:-mbilinyitech}"
REGION="${FLY_REGION:-jnb}"
ORG="${FLY_ORG:-personal}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if [ -z "${FLY_API_TOKEN:-}" ]; then
  echo "FLY_API_TOKEN is not set." >&2
  exit 1
fi
export FLY_API_TOKEN

FLY="${FLYCTL:-flyctl}"
command -v "$FLY" >/dev/null 2>&1 || FLY="$HOME/.fly/bin/flyctl"

echo "== 1. does the app exist? =="
if "$FLY" status -a "$APP" >/dev/null 2>&1; then
  echo "   app '$APP' already exists"
else
  echo "   creating app '$APP' in org '$ORG'"
  "$FLY" apps create "$APP" --org "$ORG" || {
    echo >&2
    echo "Could not create the app. A token scoped to one existing app can" >&2
    echo "deploy that app but cannot create a new one. Use an org token:" >&2
    echo "    fly tokens create org -o $ORG" >&2
    echo "or create the app in the dashboard and mint a token for it:" >&2
    echo "    fly tokens create deploy -a $APP" >&2
    exit 1
  }
fi

echo "== 2. deploy (build runs on Fly's builders) =="
"$FLY" deploy --remote-only --yes

echo "== 3. status =="
"$FLY" status -a "$APP"

echo "== 4. certificates for the custom domain =="
for h in "$APP.fly.dev" mbilinyitech.co.tz www.mbilinyitech.co.tz; do
  "$FLY" certs add "$h" -a "$APP" 2>&1 | tail -2 || true
done

cat <<EOF

Done. Point the domain at Fly if you have not already:
  A     mbilinyitech.co.tz      -> the IPv4 from 'fly ips list -a $APP'
  AAAA  mbilinyitech.co.tz      -> the IPv6 from 'fly ips list -a $APP'
  CNAME www.mbilinyitech.co.tz  -> $APP.fly.dev

The current A record points at 91.204.209.30 (the cPanel host), so the domain
keeps serving cPanel until that record is changed at the registrar.
EOF
