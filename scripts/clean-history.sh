#!/bin/sh
# Purge env secret files from the ENTIRE git history (GitHub Push Protection fix).
# The files stay on disk; they become untracked in every commit.
set -eu
export FILTER_BRANCH_SQUELCH_WARNING=1

FILES=".env.local .env"

# Stage removal from the current index if still tracked (ignore if not).
git rm --cached --ignore-unmatch $FILES >/dev/null 2>&1 || true
if ! git diff --cached --quiet; then
  git commit -m "Stop tracking env secret files" >/dev/null
  echo "committed index removal"
fi

# Rewrite all commits so the files never appear in any snapshot.
git filter-branch --force --index-filter "git rm --cached --ignore-unmatch $FILES" \
  --prune-empty --tag-name-filter cat -- --all >/dev/null

# Drop old refs so the pre-rewrite objects are unreachable.
rm -rf .git/refs/original
git reflog expire --expire=now --all
git gc --prune=now --aggressive

# Verify: no commit may contain the files.
LEFTOVER=$(git log --all --oneline -- $FILES || true)
if [ -n "$LEFTOVER" ]; then
  echo "FAIL: secret files still referenced:"
  echo "$LEFTOVER"
  exit 1
fi
echo "OK: env secret files removed from all history"
echo "--- new history ---"
git log --oneline
echo "--- working tree ---"
git status --short | grep -v '^??' || echo "(clean)"
