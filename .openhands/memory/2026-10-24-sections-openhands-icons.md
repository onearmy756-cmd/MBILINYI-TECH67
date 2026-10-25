# 2026-10-24 — own pages, real OpenHands conversation, search icon

## The four asks (Swahili) and what satisfied each
1. "Kila sehemu iwe na page yake, sio kufungikia kwenye page moja."
   The admin/client/staff tabs were `<button>`s that swapped content inside one
   document. Converted to real anchors: `<a data-atab="tools" href="#tools"
   onclick="adminTab('tools');return false">`. 29 admin + 7 client + 6 staff.
   `app.js` gained `setTabHash` (history.pushState), `hashTab`, `activePortal`,
   `applyHashTab`, wired to `popstate` + `hashchange`; startup reads the hash
   instead of hardcoding `'overview'`/`'home'`. So a section is linkable,
   bookmarkable, refreshable, and Back works.
2. "Interface halisi ya OpenHands ionekane."
   **An iframe is impossible.** `app.all-hands.dev` returns
   `content-security-policy: ... frame-ancestors 'self'` AND
   `x-frame-options: SAMEORIGIN`. Do not re-try embedding it.
   What works: a new `coding.php?action=transcript&id=JOB-...` reads the
   conversation's own event stream (`/api/v1/conversation/{id}/events/search`)
   and returns shaped rows `{kind,role,tool,text,at}`; `openTranscript()` in
   app.js renders them as a real conversation (user/agent/tool/error voices).
   The same id is also a deep-link: `{base}/canvas/conversations/{id}` opens
   OpenHands itself — that IS the real UI, just in its own tab.
3. "Icon halisi kwenye search, sio dunia."
   Head now advertises 96px and 144px icons (Google ignores <48px). Files:
   `assets/icons/icon-96.png`, `icon-144.png` (generated from icon-512.png with
   LANCZOS). New test `every_advertised_icon_exists_on_disk` fails if the head
   advertises an icon whose file is missing — that is the exact drift that
   makes Google fall back to a globe.
4. "Mpangilio mzuri, responsiveness, navigation interactive."
   `#mobileMenu` slides (`max-height/opacity`), closes on Escape, outside tap,
   or a link; `aria-expanded` kept in step. Nav marks the current page
   (`paintNavActive` + `aria-current="page"` + a CSS underline). 44px touch
   rows, `env(safe-area-inset-top)`, `overflow-wrap:anywhere` for long tokens,
   `prefers-reduced-motion` honoured.

## Traps that cost time here
- `toggleMobileMenu` used to toggle the `hidden` class; the new CSS animates on
  `.open`. Keep both: the button must still work if `.open` was never applied.
- `git push origin master` FAILED: the `origin` remote URL carries a stale
  token (`ghu_IC1n...`). Push with a one-off URL instead:
  `git push https://<PAT>@github.com/onearmy756-cmd/MBILINYI-TECH67.git master`.
  Never write the PAT into `.git/config`.
- The local 8080 server was a stale process from an earlier session; it served
  the OLD binary and hid the new head. `serve.sh` (port 12000) is the one that
  rebuilds when sources are newer.
- `scripts/smoke.sh` defaults to `BASE=http://127.0.0.1:8080`. Run the release
  binary with `MTS_PORT=8080` first, or the page step reports `000`.
- The CSRF token rotates once the session is established: read `window.__CSRF__`
  again AFTER `verify-login-pin`, or the next POST is `403`.
- `ConversationStateUpdateEvent.value` is the whole agent state as JSON. Sending
  it as text floods the conversation view; it is returned with empty text and
  dropped by the panel.
