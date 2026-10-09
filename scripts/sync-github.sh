#!/bin/sh
# Send the current branch to GitHub using GITHUB_TOKEN from the environment.
# The token is NEVER written to disk or stored in git config/remotes.
#
# Usage: sh ./scripts/sync-github.sh [remote-url] [branch]
#   remote-url defaults to tech67 (MBILINYI-TECH67 repo)
#   branch defaults to the current branch (master)
set -eu

REMOTE_URL="${1:-tech67}"
BRANCH="${2:-$(git rev-parse --abbrev-ref HEAD)}"

if [ -z "${GITHUB_TOKEN:-}" ]; then
  echo "ERROR: GITHUB_TOKEN is not set."
  echo "Add it in the Freebuff Keys/API keys tab (Settings > Environment)."
  exit 2
fi

# Resolve a remote name to its URL; pass-through a full URL.
case "$REMOTE_URL" in
  https://*|http://*|git@*) URL="$REMOTE_URL" ;;
  *) URL="$(git remote get-url "$REMOTE_URL")" ;;
esac

# Build an authenticated URL without leaking the token into output.
case "$URL" in
  https://github.com/*)
    AUTH_URL="https://x-access-token:${GITHUB_TOKEN}@github.com${URL#https://github.com}"
    ;;
  *)
    echo "ERROR: Only https://github.com URLs are supported." >&2
    exit 1
    ;;
esac

git push "$AUTH_URL" "$BRANCH"
STATUS=$?
[ $STATUS -eq 0 ] && echo "Pushed $BRANCH to GitHub OK"
exit $STATUS
