#!/usr/bin/env bash
# End-to-end smoke test for the Mbilinyi Tech Solutions PHP + MySQL system.
# Usage: BASE=http://127.0.0.1:8080 bash scripts/smoke.sh
set -u
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
B="${BASE:-http://127.0.0.1:8080}"
JA=/tmp/mts_jar_admin.txt; JC=/tmp/mts_jar_client.txt; JG=/tmp/mts_jar_guest.txt
rm -f $JA $JC $JG

# Contracts are permanent legal records and are never deleted, so the contract
# this script signs on the first run is still there on the second — step 10 would
# then find nothing "Sent" and report false failures. Start from the pristine
# database a fresh clone gets. Set SMOKE_RESET=0 to test an existing database.
if [ "${SMOKE_RESET:-1}" = "1" ]; then
  rm -f "$ROOT/storage/mts.sqlite" "$ROOT/storage/mts.sqlite-journal" "$ROOT/storage/mts.sqlite-wal" 2>/dev/null
  rm -rf "$ROOT/storage/sessions" 2>/dev/null
  for CLI in mariadb mysql; do
    if command -v "$CLI" >/dev/null 2>&1; then
      "$CLI" -u mts -pmts_local_dev -h 127.0.0.1 -e 'DROP DATABASE IF EXISTS mbilinyi;' >/dev/null 2>&1 && break
    fi
  done
  echo "-- database reset (SMOKE_RESET=0 to keep it)"
fi

csrf(){ curl -sL -b "$1" -c "$1" "$B/index.php" | grep -o 'window.__CSRF__ = "[a-f0-9]*"' | grep -o '[a-f0-9]\{64\}'; }
# -L everywhere: with router.php a .php path answers 301 to its clean URL, and
# without it the request is already final. Following redirects keeps this script
# valid against both the plain server and the routed public preview.
post(){ curl -sL -b "$1" -c "$1" -X POST "$B/$2" -H 'Content-Type: application/json' -H "X-CSRF-Token: $4" -d "$3"; }
get(){ curl -sL -b "$1" -c "$1" "$B/$2"; }
SIG="data:image/png;base64,$(head -c 400 /dev/zero | tr '\0' 'A')"

echo "== 1. public track lookup =="
curl -sL "$B/api/track.php?action=get&id=MTS-2026-4810" | jq -c '{ok, status:.data.request.status, total:.data.quote.total, contract:.data.contract.status}'

echo "== 2. admin login (password, then PIN verification) =="
C=$(csrf $JA); post $JA api/auth.php '{"action":"login","email":"mbilinyitech@gmail.com","password":"Admin@2026"}' "$C" | jq -c '{ok, otp_required:.data.otp_required, pin_kind:.data.pin_kind}'
# Login is a two-step flow: the password step only asks for the PIN, the PIN step
# establishes the session. Admin's PIN is the tail of the registered phone.
post $JA api/auth.php '{"action":"verify-login-pin","email":"mbilinyitech@gmail.com","pin":"2645"}' "$C" | jq -c '{ok, role:.data.user.role}'
C=$(csrf $JA)

echo "== 3. admin dashboard =="
get $JA "api/dashboard.php?action=stats" | jq -c '{ok, pending:.data.pending, requests:.data.requestCount, users:.data.userCount, revenue:.data.revenue}'

echo "== 4. admin adds a quotation for R-2 =="
post $JA api/quotes.php '{"action":"admin_save","requestId":"R-2","items":[{"desc":"WhatsApp Bot Development","qty":1,"price":800000},{"desc":"Deployment and Training","qty":1,"price":200000}],"validUntil":"2026-11-30","status":"Sent","adminMessage":"Quotation for the Swahili WhatsApp chatbot."}' "$C" | jq -c '{ok, id:.data.quote.id, total:.data.quote.total}'

echo "== 5. admin generates a contract for R-2 (50% deposit invoice inaundwa kiotomatiki) =="
post $JA api/contracts.php '{"action":"create","requestId":"R-2"}' "$C" | jq -c '{ok, id:.data.contract.id, status:.data.contract.status, created:.data.created}'

echo "== 6. admin edits a request =="
post $JA api/requests.php '{"action":"admin_update","id":"R-2","service":"AI Solutions & Fine-Tuning","title":"Swahili WhatsApp Chatbot for Orders + Price Lists","description":"WhatsApp chatbot in Swahili to take vegetable orders, send price lists and confirm delivery zones across Dar es Salaam with M-Pesa payment links.","priority":"Urgent","budget":"TZS 500K – 1.5M ($200 – $600)","deadline":"2026-10-30","status":"Quoted","adminNote":"Quotation sent after full review."}' "$C" | jq -c '{ok, status:.data.request.status, priority:.data.request.priority}'

echo "== 7. admin creates an invoice =="
post $JA api/invoices.php '{"action":"admin_save","id":"new","title":"Chatbot — 50% Deposit","amount":500000,"trackingId":"MTS-2026-5523","dueDate":"2026-11-15","status":"Unpaid","method":"M-Pesa / Bank"}' "$C" | jq -c '{ok, n:(.data.invoices|length), created:.data.invoices[0].title}'

echo "== 8. admin must NOT be able to sign before the client =="
CID=$(get $JA "api/contracts.php?action=admin_list" | jq -r '[.data.contracts[] | select(.status=="Sent")][0].id')
post $JA api/contracts.php "{\"action\":\"sign_admin\",\"id\":\"$CID\",\"sig\":\"$SIG\"}" "$C" | jq -c '{ok, error}'

echo "== 9. client login (password, then PIN verification) =="
# R-2 belongs to baraka@kilimo.co.tz, so that is the client who signs its contract.
CC=$(csrf $JC); post $JC api/auth.php '{"action":"login","email":"baraka@kilimo.co.tz","password":"demo123"}' "$CC" | jq -c '{ok, otp_required:.data.otp_required, pin_kind:.data.pin_kind}'
post $JC api/auth.php '{"action":"verify-login-pin","email":"baraka@kilimo.co.tz","pin":"3456"}' "$CC" | jq -c '{ok, role:.data.user.role}'
CC=$(csrf $JC)
CID=$(get $JC "api/contracts.php?action=mine" | jq -r '[.data.contracts[] | select(.status=="Sent")][0].id')
TRK=$(get $JA "api/contracts.php?action=admin_list" | jq -r --arg id "$CID" '[.data.contracts[] | select(.id==$id)][0].trackingId')
echo "client contract to sign: $CID (tracking $TRK)"

echo "== 10. client signs FIRST =="
post $JC api/contracts.php "{\"action\":\"sign_client\",\"id\":\"$CID\",\"sig\":\"$SIG\"}" "$CC" | jq -c '{ok, status:.data.contract.status, clientSig:(.data.contract.clientSig!=null)}'

echo "== 11. client cannot sign twice =="
post $JC api/contracts.php "{\"action\":\"sign_client\",\"id\":\"$CID\",\"sig\":\"$SIG\"}" "$CC" | jq -c '{ok, error}'

echo "== 12. KANUNI YA 50%: admin CANNOT countersign before the deposit is paid =="
post $JA api/contracts.php "{\"action\":\"sign_admin\",\"id\":\"$CID\",\"sig\":\"$SIG\"}" "$C" | jq -c '{ok, error}'

echo "== 12b. deposit_info inaonyesha deposit haijalipwa =="
get $JA "api/contracts.php?action=deposit_info&id=$CID" | jq -c '{ok, paid:.data.deposit.paid, invoiceId:.data.deposit.invoiceId, status:.data.deposit.status}'

echo "== 12c. admin marks THIS contract's 50% deposit invoice PAID =="
# The gate judges the EARLIEST deposit invoice for the tracking id, and
# deposit_info reports that same row. Read it from there rather than from
# admin_list, whose newest-first order can name a different invoice when a
# second deposit-titled invoice exists (step 7 creates one on this tracking id).
DID=$(get $JA "api/contracts.php?action=deposit_info&id=$CID" | jq -r '.data.deposit.invoiceId')
echo "deposit invoice: $DID"
post $JA api/invoices.php "{\"action\":\"mark_paid\",\"id\":\"$DID\"}" "$C" | jq -c '{ok, status:.data.invoice.status}'

echo "== 12d. admin countersigns (second) — sasa kunaruhusiwa =="
post $JA api/contracts.php "{\"action\":\"sign_admin\",\"id\":\"$CID\",\"sig\":\"$SIG\"}" "$C" | jq -c '{ok, status:.data.contract.status, adminSig:(.data.contract.adminSig!=null), error}'

echo "== 13. price is locked after the client signed =="
post $JA api/contracts.php "{\"action\":\"update\",\"id\":\"$CID\",\"projectTitle\":\"X\",\"scope\":\"Y\",\"duration\":\"10 days\",\"total\":1}" "$C" | jq -c '{ok, error}'

echo "== 14. price can change BEFORE the client signs =="
C1=$(get $JA "api/contracts.php?action=admin_list" | jq -r '[.data.contracts[] | select(.status=="Sent")][0].id')
post $JA api/contracts.php "{\"action\":\"update\",\"id\":\"$C1\",\"projectTitle\":\"Pharmacy Stock, Sales & M-Pesa System\",\"scope\":\"Full pharmacy management system with M-Pesa.\",\"duration\":\"45 days\",\"total\":4200000}" "$C" | jq -c '{ok, total:.data.contract.total}'

echo "== 14b. KANUNI YA 50%: set_status In Progress inazuiwa mpaka deposit ilipwe (R-2) =="
post $JA api/requests.php '{"action":"set_status","id":"R-2","status":"In Progress"}' "$C" | jq -c '{ok, error}'

echo "== 14c. deposit zote za R-2 zinalipwa kisha status inaruhusiwa =="
for ID in $(get $JA "api/invoices.php?action=admin_list" | jq -r '.data.invoices[] | select(.trackingId=="MTS-2026-5523" and (.title|contains("Deposit")) and .status=="Unpaid") | .id'); do
  post $JA api/invoices.php "{\"action\":\"mark_paid\",\"id\":\"$ID\"}" "$C" | jq -c '{ok, id:.data.invoice.id, status:.data.invoice.status}'
done
post $JA api/requests.php '{"action":"set_status","id":"R-2","status":"In Progress"}' "$C" | jq -c '{ok, status:.data.request.status}'

echo "== 15. admin deletes the invoice from step 7 =="
IV=$(get $JA "api/invoices.php?action=admin_list" | jq -r '[.data.invoices[] | select(.title|contains("Chatbot"))][0].id')
post $JA api/invoices.php "{\"action\":\"admin_delete\",\"id\":\"$IV\"}" "$C" | jq -c '{ok}'

echo "== 16. guest submits a request through the wizard =="
C0=$(csrf $JG)
post $JG api/requests.php '{"action":"create","name":"Test Mteja","phone":"0711000111","email":"test.mteja@example.com","company":"Test Co","service":"Software Maintenance","title":"Maintenance plan for school portal","description":"We need monthly maintenance, backups and monitoring for our school portal used by 900 parents.","tech":["PHP","MySQL"],"platform":"Web Application","budget":"TZS 150,000 /month","deadline":"2026-11-01","priority":"Normal"}' "$C0" | jq -c '{ok, tracking:.data.trackingId}'

echo "== 17. that request is publicly trackable =="
NEWID=$(get $JG "api/requests.php?action=mine" | jq -r '.data.requests[0].trackingId')
curl -sL "$B/api/track.php?action=get&id=$NEWID" | jq -c "{ok, status:.data.request.status}"

echo "== 18. guest opens a support ticket =="
C0=$(csrf $JG)
post $JG api/tickets.php '{"action":"create","name":"Test Mteja","email":"test.mteja@example.com","subject":"Backup question","message":"Do you backup daily?"}' "$C0" | jq -c '{ok, id:.data.ticket.id}'

echo "== 19. client pays the 50% balance invoice with an M-Pesa reference =="
post $JA api/invoices.php '{"action":"admin_save","id":"new","title":"Pharmacy System — 50% Balance (Delivery)","amount":2100000,"trackingId":"MTS-2026-4810","dueDate":"2026-12-15","status":"Unpaid","method":"M-Pesa / Bank"}' "$C" | jq -c '{ok}'
# MTS-2026-4810 is demo@client.com's request, so that client pays it.
CD=$(csrf $JC); post $JC api/auth.php '{"action":"login","email":"demo@client.com","password":"demo123"}' "$CD" >/dev/null
post $JC api/auth.php '{"action":"verify-login-pin","email":"demo@client.com","pin":"5678"}' "$CD" >/dev/null
CD=$(csrf $JC)
BAL=$(get $JC "api/invoices.php?action=mine" | jq -r '[.data.invoices[] | select((.title|contains("Balance")) and .status=="Unpaid")][0].id')
post $JC api/invoices.php "{\"action\":\"pay\",\"id\":\"$BAL\",\"ref\":\"TX984120\"}" "$CD" | jq -c '{ok, status:.data.invoice.status}'

echo "== 20. register a new account (unique email so reruns stay green) =="
C0=$(csrf $JG)
NEWMAIL="mteja.$(date +%s)@example.com"
post $JG api/auth.php "{\"action\":\"register\",\"name\":\"Mteja Mpya\",\"phone\":\"0788000111\",\"email\":\"$NEWMAIL\",\"company\":\"Mpya Ltd\",\"password\":\"secret123\",\"password2\":\"secret123\"}" "$C0" | jq -c '{ok, role:.data.user.role}'

echo "== 21. pages render (kila sehemu ni page yake) =="
for p in index.php services.php stack.php ai.php pricing.php process.php work.php track.php contact.php auth.php client.php admin.php; do
  # -L so a routed host's 301 (.php -> clean URL) still reports the final status.
  code=$(curl -sL -o /dev/null -w '%{http_code}' "$B/$p")
  echo "$p -> $code"
done

echo "== 21b. every portal section is a page of its own =="
for p in admin/requests admin/quotes admin/contracts admin/portfolio admin/testimonials admin/news admin/team admin/wafanyakazi admin/ajira admin/payroll admin/agents admin/tools admin/ofisi admin/ukuaji admin/usalama admin/users admin/invoices admin/tickets admin/feedback admin/settings admin/email admin/queue admin/runs admin/coding admin/runtimes admin/ai admin/sms admin/hamisha client/requests client/invoices client/support mtaalamu/tasks mtaalamu/tools; do
  # A signed-out visitor is sent to sign in, so a section answers 200 (signed in)
  # or 302 (redirect to /auth?returnTo=...&tab=...) — never 404.
  code=$(curl -s -o /dev/null -w '%{http_code}' "$B/$p")
  echo "$p -> $code"
done

echo "== 21c. a section that does not exist is a 404, not the dashboard =="
for p in admin/nonsense client/nonsense mtaalamu/nonsense pricing/anything; do
  code=$(curl -s -o /dev/null -w '%{http_code}' "$B/$p")
  echo "$p -> $code"
done

echo "DONE"
