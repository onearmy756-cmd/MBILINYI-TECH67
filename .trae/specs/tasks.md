# Mbilinyi Tech Solutions — Tasks.md
## Kuantokana na spec.md • Muhula: Implement • Kila task ina Test Requirement (TR)

---

## Task 1: Schema Update — Jedwali Mpya za OTP, Portfolio, Testimonials, Visitor Logs
**Kifaa:** Schema + Seed
**Umuhimu:** high
**Hivi karibuni:** AC-2, AC-4, AC-7
**Status:** pending

**Maelezo:**
- Badilisha [schema.php](file:///c:/Users/STEPHANO%20AMAN/Desktop/MBILINYI-TECH67-master/config/schema.php) kuongeza tables zifuatazo (idempotent, `CREATE TABLE IF NOT EXISTS`):
  1. `otp_codes` (id, user_id/email, code_hash, purpose [login/reset], expires_at, created_at, used_at)
  2. `password_resets` (email, token_hash, expires_at, created_at)
  3. `portfolio_projects` (id, title, category, description VARCHAR, image_url, stat_label, link_url, sort_order INT DEFAULT 0, is_active TINYINT DEFAULT 1, created_at, updated_at)
  4. `testimonials` (id, name, role, text TEXT, stars INT, sort_order INT DEFAULT 0, is_active TINYINT DEFAULT 1, created_at, updated_at)
  5. `visitor_logs` (id, ip_hash VARCHAR, user_agent VARCHAR, page_url VARCHAR, referer VARCHAR, session_id VARCHAR, visited_at DATETIME DEFAULT CURRENT_TIMESTAMP)
- [seed.php](file:///c:/Users/STEPHANO%20AMAN/Desktop/MBILINYI-TECH67-master/config/seed.php) iwe na seed data: PORTFOLIO + TESTIS kutoka app.js kwenye tables mpya (kwa admin aweze kufuta kila demo)

**Test Requirements (TR):**
- TR-1.1 (rule): Kila moja wa tables 5 zinaongezwa bila error wakati schema inarun.
- TR-1.2 (rule): Seed data ina 6 portfolio + 4 testimonials zinazolingana na PORTFOLIO/TESTIS arrays.

---

## Task 2: FR-1 — Ongeza Tagline "YOUR PROBLEM, OUR SOLUTION" / "SHIDA YAKO, TATUZO LETU"
**Kifaa:** UI Pages
**Umuhimu:** medium
**Hivi karibuni:** AC-1, AC-R1
**Status:** pending

**Maelezo:**
- [index.php](file:///c:/Users/STEPHANO%20AMAN/Desktop/MBILINYI-TECH67-master/index.php#L20-L49): Hero — chini ya h1 au chini ya badges, ongeza tagline line mbili (EN/SW) kwa font-display, kwa kushirikiana na layout bila kuvuruga.
- [footer.php](file:///c:/Users/STEPHANO%20AMAN/Desktop/MBILINYI-TECH67-master/partials/footer.php#L11): Copyright line au juu yake, ongeza tagline center-text ndogo (kwa HEROKO kando ya "Innovate • Secure • Scale" (usiibadilise hilo — ongeza "YOUR PROBLEM, OUR SOLUTION" kwa mstari mpya au kando kwa | separator).
- [lib/email.php](file:///c:/Users/STEPHANO%20AMAN/Desktop/MBILINYI-TECH67-master/lib/email.php#L70): Mstari wa mwisho wa email shell "Innovate • Secure • Scale" — ongeza " | YOUR PROBLEM, OUR SOLUTION" kwa uwezo wa kuendelea.
- [header.php](file:///c:/Users/STEPHANO%20AMAN/Desktop/MBILINYI-TECH67-master/partials/header.php#L29): Chini ya "MBILINYI TECH SOLUTIONS" tagline ya sasa "INNOVATE • SECURE • SCALE" ibaki kama ilivyo — tuongeza "SHIDA YAKO, TATUZO LETU" au "YOUR PROBLEM, OUR SOLUTION" kwenye sw en kwa text ndogo chini yake kwa serif.

**Test Requirements:**
- TR-2.1 (rule): Tagline inapatikana kwa kila sehemu 4 zilizotajwa bila kuvunja layout.
- TR-2.2 (rule): Kwenye header na footer, badiliko la lugha (EN↔SW) linabadilisha tagline vizuri (kwa data-en/data-sw where applicable).
- TR-2.3 (rubric): AC-R1 score ≥ 2.

---

## Task 3: FR-2 — Verification Code (OTP) + Forgot Password + Resend kwenye Auth
**Kifaa:** Auth Flow
**Umuhimu:** high
**Hivi karibuni:** AC-2, AC-R2
**Status:** pending

**Maelezo:**
- Schema: OTP + password_resets tables (Task 1).
- [assets/app.js](file:///c:/Users/STEPHANO%20AMAN/Desktop/MBILINYI-TECH67-master/assets/app.js#L297-L306): `doLogin()` — Baada ya apiPost login ikipata "otp_required" badala ya user → onesha OTP prompt modal (genModal au dedicated HTML kwenye auth.php).
- [auth.php](file:///c:/Users/STEPHANO%20AMAN/Desktop/MBILINYI-TECH67-master/auth.php#L37-L65): Ongeza tab au section 3:
  - "Forgot Password?" link chini ya Sign In button → inafungua modal/section ya "Enter email to reset".
  - OTP verification section (inaweza iko kando ya login/register kwenye right panel).
- [api/auth.php]: Ongeza actions:
  - `verify-login-otp`: Kipimo code na mtu akaruhusiwe.
  - `resend-login-otp`: Gen code mpya, tuma email.
  - `forgot-password-init`: Tuma reset token link kwa email.
  - `forgot-password-complete`: Kipimo token, weka password mpya.
- [lib/email.php]: Ongeza functions: `notify_login_otp(email, code)`, `notify_password_reset(email, token_link)`.
- OTP timeout: Resend disabled kwa 60s, count down kwenye UI.
- Backward compat: Kwa admin tests endapo otp disabled (kama API key ya email haipo) — tupa fallback ya login ya moja kwa moja wa password ili usiwe na blocker (au admin akiwa na super flag anaweza skip).

**Test Requirements:**
- TR-3.1 (rule): Mtu ambaye hajasign up na password sahihi → atumie OTP kwa email → weka OTP → ingia successfully.
- TR-3.2 (rule): Forgot password → email na reset link → reset password → login works.
- TR-3.3 (rule): Resend button inatumia OTP mpya na count-down 60s inaonekana.
- TR-3.4 (rubric): AC-R2 ≥ 2.

---

## Task 4: FR-3 — PWA (manifest + Service Worker + Meta Tags)
**Kifaa:** PWA Config
**Umuhimu:** medium
**Hivi karibuni:** AC-3
**Status:** pending

**Maelezo:**
- Unda `manifest.json` kwenye root: name="Mbilinyi Tech Solutions", short_name="Mbilinyi Tech", start_url="/index.php", display="standalone", theme_color="#050914", background_color="#050914", icons — tumia base64 ya 192x192 na 512x512 na SVG/PNG na logo ya M (au font-awesome text rendered).
- Unda `sw.js` kwenye root: Simple cache-first ya vifaa vya static (CSS CDN, icons, js, index, pages mbili) — na network-first kwa API.
- [partials/head.php](file:///c:/Users/STEPHANO%20AMAN/Desktop/MBILINYI-TECH67-master/partials/head.php#L4-L10): Ongeza meta:
  - `<link rel="manifest" href="manifest.json">`
  - `<meta name="theme-color" content="#050914">`
  - `<link rel="apple-touch-icon" href="data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'><rect width='100' height='100' rx='22' fill='%2306b6d4'/><text x='50' y='65' font-size='60' font-weight='700' text-anchor='middle' fill='white'>M</text></svg>">`
- Register SW kwenye footer.php au app.js init (kabla ya script ya app.js mwisho).

**Test Requirements:**
- TR-4.1 (rule): manifest.json inapatikana, ina fields zote muhimu, icons zipo.
- TR-4.2 (rule): Service worker inaregister successfully (no console errors on page load).
- TR-4.3 (rule): PWA meta tags zipo kwenye kila ukurasa (kwa maana yote yanaumia partials/head.php).

---

## Task 5: FR-4 — Admin CRUD Portfolio + Testimonials
**Kifaa:** Admin UI + DB Render
**Umuhimu:** high
**Hivi karibuni:** AC-4, AC-R3
**Status:** pending

**Maelezo:**
- API endpoints:
  - `api/portfolio.php`: actions `list`, `add`, `update`, `delete` (CSRF + admin_only)
  - `api/testimonials.php`: actions `list`, `add`, `update`, `delete` (CSRF + admin_only)
- [admin.php](file:///c:/Users/STEPHANO%20AMAN/Desktop/MBILINYI-TECH67-master/admin.php): Ongeza nav items mpya kwenye sidebar:
  - `<button data-atab="portfolio" onclick="adminTab('portfolio')"...>Portfolio Projects</button>`
  - `<button data-atab="testimonials" onclick="adminTab('testimonials')"...>Testimonials</button>`
- [assets/app.js]:
  - `adminTab()` ihandle portfolio + testimonials tabs (form ya create, table ya list, buttons ya edit/delete).
  - `renderWork()` function — badilisha iwe inapull data kutoka `api/portfolio.php?action=list` + `api/testimonials.php?action=list`. Ikiwa API inarudishwa empty (no portfolio) au inakata — fallback kwenye PORTFOLIO + TESTIS hardcoded (kwa backward compat).
- Admin aweze kupata Demo data — seed wa Task 1.
- Delete button: Admin akifuta project au testimonial, iwe inactive (soft delete au hard delete, usionee) — user alisema "hizo demo data na project zina zoonekana futa mpaka nifanye kwa upande wa admin".

**Test Requirements:**
- TR-5.1 (rule): Admin anaweza ku-add, edit, delete portfolio project (title, desc, picha, cat, stat, link) na kila kitu kinahifadhiwa.
- TR-5.2 (rule): Admin anaweza ku-add, edit, delete testimonial.
- TR-5.3 (rule): work.php inarender project/testimonials za admin ambazo zimebadilishwa — hakuna data, inarender demo hardcoded bila error.
- TR-5.4 (rubric): AC-R3 ≥ 2.

---

## Task 6: FR-5 — Services Mpya kwenye SERVICES na Laravel/PHP kwenye STACK
**Kifaa:** Static Data (app.js)
**Umuhimu:** low
**Hivi karibuni:** AC-5
**Status:** pending

**Maelezo:**
- [assets/app.js#L45-L54](file:///c:/Users/STEPHANO%20AMAN/Desktop/MBILINYI-TECH67-master/assets/app.js#L45-L54): SERVICES array — ongeza items 7:
  1. { icon: 'fa-building', c: 'cyan', t: 'BRELA & Business Registration', d: 'Usajili wa makampuni, Biashara, SME na vyeti vya BRELA — kila hatua kiotomatiki.', tags: ['BRELA', 'SME'] }
  2. { icon: 'fa-file-invoice-dollar', c: 'gold', t: 'TRA & Tax Services', d: 'TIN registration, File Tax Returns, VAT, PAYE na msaada wa mifumo ya TRA.', tags: ['TIN', 'TRA', 'VAT'] }
  3. { icon: 'fa-screwdriver-wrench', c: 'emerald', t: 'Computer Troubleshooting', d: 'Kutatua matatizo ya software, malware, slow PC, crashes, data recovery na updates.', tags: ['Repair', 'Malware'] }
  4. { icon: 'fa-windows', c: 'blue', t: 'Windows & OS Installation', d: 'Windows 10/11, Linux, macOS + drivers, updates, activation na backup.', tags: ['Windows', 'Drivers'] }
  5. { icon: 'fa-envelope-open-text', c: 'fuchsia', t: 'Email & Communication Setup', d: 'Kutengeneza email (G-Suite, Zoho, cPanel), SMTP, kutatua matatizo ya email na SPF/DKIM.', tags: ['SMTP', 'G-Suite'] }
  6. { icon: 'fa-landmark', c: 'violet', t: 'Government e-Services', d: 'NIDA, NHIF, NSSF, eCitizen, Huduma Number na huduma zingine za serikali.', tags: ['NIDA', 'NHIF'] }
  7. { icon: 'fa-box', c: 'teal', t: 'Software Installation', d: 'Kuinstall MS Office, Adobe, IDEs, Antivirus, Design Apps, na customize settings.', tags: ['Office', 'Apps'] }
- [assets/app.js#L55-L64](file:///c:/Users/STEPHANO%20AMAN/Desktop/MBILINYI-TECH67-master/assets/app.js#L55-L64): STACK object — ongeza:
  - 'PHP': { level: 96, desc: 'Server-side rendering, WordPress, CMS, na REST APIs kwa mifumo ya biashara.', use: 'Websites • APIs' },
  - 'Laravel': { level: 94, desc: 'Robust PHP framework — authentication, queues, billing, admin panels na ORM.', use: 'Enterprise Apps' }
- Usibadilise zilizopo (Python, React, nk. — kabla ya ziada hii).

**Test Requirements:**
- TR-6.1 (rule): Services zote 7 mpya zinaonekana services.php kwenye grid, zina style sawa na zilizopo.
- TR-6.2 (rule): Stack page (stack.php) ina Laravel + PHP kwenye tabs, na level/description zinaonekana.
- TR-6.3 (rule): Usibadilishwi kichwa chochote cha old services — zote bado zipo 8 + 7 = 15.

---

## Task 7: FR-6 — Ollama Cloud API Integration na Chatbot
**Kifaa:** AI Chatbot
**Umuhimu:** high
**Hivi karibuni:** AC-6, AC-R4
**Status:** pending

**Maelezo:**
- [config/db.php au config/app.php]: Ongeza env-based config: `OLLAMA_CLOUD_API_URL` (default https://api.ollama.com/v1), `OLLAMA_CLOUD_API_KEY`, `OLLAMA_MODEL` (default llama3.1:8b).
- Unda `api/ollama.php`: POST `chat` action — admin/client login optional, ingiza messages array, system prompt itumie:
  ```
  "Wewe ni Mbilinyi AI Assistant wa Mbilinyi Tech Solutions (BRELA Z-418822-77-TZ, TIN 192-147-522) uongozi wa CEO Jackson Mbilinyi, Dar es Salaam, Simu 0796 752 645, Email mbilinyitech@gmail.com, Instagram @mbilinyitech.
  Huduma unazotoa: (orodha SERVICES yote — software, BRELA, TRA, troubleshooting, windows, email, government, software install, AI, nk.)
  Tech stack: React, Rust backend, PHP, Laravel, R, HTML/CSS/JS, Python, Flutter, nk. (STACK data)
  Malipo: 50% DEPOSIT kabla mradi kuanza, 50% MID-PROJECT (kati ya mradi). Bei zinaongezwa kwenye quote rasmi na CEO.
  Kazi: 120+ projects, clients 8 nchi, BRELA registered.
  Kila jibu liwe na maneno ya Kiswahili (kama swali lina Kiswahili) au Kingereza (kama lina Kingereza). Usihalucinate — kama hujui sema 'Tafadhali wasiliana na CEO Jackson 0796752645 kwa taarifa zaidi.'.
  Unajibu kwa ujumbe wa kisha, na usahihi."
  ```
- [footer.php chatbot](file:///c:/Users/STEPHANO%20AMAN/Desktop/MBILINYI-TECH67-master/partials/footer.php#L18-L30): `sendChat()` kwenye app.js — iwe:
  1. Ikiwa user anasw (yaani swali lina maneno ya Kiswahili au lugha yake app ni sw) → jibu kwa sw.
  2. Kwanza tumia hardcoded replies kama zamani kama fallback, lakini kwa kila ujumbe mpya wa user:
     - Tuma POST kwa `api/ollama.php` (ikiwa OLLAMA_API_KEY ipo — tupa check).
     - Ikiwa API key haipo || API inakata → rudi hardcoded (toast "AI offline — connecting you to CEO").
- Chat history iwe saved kwa sessionStorage (kwenye app.js), na system prompt iwe included na kila request.
- Also, `ai.php` demo ya runAiDemo iweze kuwa real ikiwa ollama key ipo (optional, fallback iendelee).

**Test Requirements:**
- TR-7.1 (rule): api/ollama.php exists, has CSRF check, returns response when key set.
- TR-7.2 (rule): sendChat() — when OLLAMA_CLOUD_API_KEY is empty, uses original hardcoded logic (no errors).
- TR-7.3 (rule): When API key set, chatbot can answer question: "Je, mnatengeneza TIN na BRELA?" → mentions TRA/BRELA, contact 0796752645, 50/50 terms.
- TR-7.4 (rubric): AC-R4 ≥ 2.

---

## Task 8: FR-7 — Visitor Tracking + Admin Analytics
**Kifaa:** Analytics
**Umuhimu:** medium
**Hivi karibuni:** AC-7, AC-R5
**Status:** pending

**Maelezo:**
- [config/app.php au top of head.php]: Ongeza call function `record_visitor()` ambayo:
  - ip_hash = hash('sha256', $_SERVER['REMOTE_ADDR'] . session_id())
  - user_agent, page_url = $_SERVER['REQUEST_URI'], referer = $_SERVER['HTTP_REFERER'] ?? ''
  - Insert kwenye `visitor_logs` (schema task 1).
- API endpoint: `api/dashboard.php?action=visitor_stats` (admin only) → returns:
  - today, yesterday, this_week, this_month counts
  - top_pages array (5 pages zaidi ya view)
  - last_7_days (array ya tarehe + count kwa kila siku kwa ajili ya chart)
- [admin.php overview tab]: Badilisha dashboard overview iwe na:
  - Karts 4: Requests, Quotes, Contracts, **Visitors Today** (badala ya au on top ya karts 4 za zamani)
  - Chart.js line chart ya "Visitors Last 7 Days" (kwa sababu Chart.js tayari ina import kwenye head.php!).
- Admin nav: Kama unataka, label "Dashboard" iwe "Dashboard & Analytics".

**Test Requirements:**
- TR-8.1 (rule): Kila page view inasave kwenye visitor_logs (kwa user mmoja anaye-tembea anarecord bila error).
- TR-8.2 (rule): Admin dashboard anaona chart + stats (today/yesterday/wiki) bila errors — data inatokea.
- TR-8.3 (rubric): AC-R5 ≥ 2.

---

## Task 9: FR-8 — 50/50 Payment Terms kwenye Contracts, Quotes na Notifications
**Kifaa:** Billing/Contracts UI
**Umuhimu:** high
**Hivi karibuni:** AC-8
**Status:** pending

**Maelezo:**
- [assets/app.js contractHTML](file:///c:/Users/STEPHANO%20AMAN/Desktop/MBILINYI-TECH67-master/assets/app.js#L591+): Baada ya section ya scope, ongeza heading 2 na terms:
  - `<h3 style="color:#0891b2;margin-top:12px">PAYMENT TERMS — MASHARTI YA MALIPO (50/50)</h3>`
  - 2 points:
    1. **DEPOSIT (50%):** `{esc(50% × c.total)}` — Lipwa kabla ya kazi kuanza. Mradi haunaanzzi hadi deposit iwe "Paid" na CEO ame-weka signature yake.
    2. **MID-PROJECT (50%):** `{esc(50% × c.total)}` — Lipwa wakati mradi umekamilikia 50% (kati ya timeline), kabla ya delivery ya mwisho.
- Kwenye client tab "Quotes" (clientTab quotes): Kwenye header ya kila quote, ongeza badge "50/50 Terms" + notice (kwa HTML ndogo).
- Admin tab "Quotations": Wakati wa ku-create quote mpya, checkbox "Apply 50/50 Payment Schedule" default on. Imewekwa, quote items include: 2 items auto (Deposit 50%, Mid-project 50%) au tuongezea note kwenye footer ya quote.
- [pricing.php]: Ongeza kwa sehemu ya Payment Terms (line 19) maneno ya: "Nusu ya 50% inalipwa wakati mradi umekamilikia nusu usoni (mid-project) si mwisho tu."

**Test Requirements:**
- TR-9.1 (rule): Contracts zote zina section dhahiri ya Payment Terms 50/50 na totals zinazolingana.
- TR-9.2 (rule): Admin create quote — check 50/50 (default on) → client view ina terms notice.
- TR-9.3 (rule): Pricing.php ina statement ya mid-project.

---

## Task 10: FR-9 — Email Notifications baada ya Kila Mtu Kusaini Contract
**Kifaa:** Email Notifications
**Umuhimu:** medium
**Hivi karibuni:** AC-8, AC-9
**Status:** pending

**Maelezo:**
- [lib/email.php] Ongeza functions:
  1. `notify_client_signed_contract(contract, clientEmail)` — Atumie kwa CEO (MTS_CEO_EMAIL): "Client X amesaini Contract ID, Tracking ID. Weka countersignature kwenye admin."
  2. `notify_contract_fully_signed(contract, clientEmail)` — Atumie kwa Client + CEO: Contract imesainishwa na mtu wote, attached maelezo ya contract + 50/50 payment terms, na deposit invoice link.
- API contracts.php (admin countersign na client sign): Baada ya kila saini, iite function hii.
- Email HTML: Tumia mts_email_shell() kwa consistency.

**Test Requirements:**
- TR-10.1 (rule): Baada ya client kusaini, CEO atatumie email notification (kwa email function inaitwa, hata kama Resend key haipo — italog error bila kukataa).
- TR-10.2 (rule): Baada ya CEO akisaini (countersign), client + CEO wote wata email fully signed.

---

## Task 11: Verify All — Marudio na Kutoa Majibu
**Kifaa:** Verifier
**Umuhimu:** high
**Hivi karibuni:** AC zote
**Status:** pending

**Maelezo:**
- Endapo kila task zimekamilika, run diagnostics:
  - PHP syntax errors: Check kila file mpya limeandikwa vizuri.
  - JS console errors: Hakuna errors on load.
  - Admin + client login: OTP, CRUD portfolio, analytics, contract sign zinafanya.
- `GetDiagnostics` + project lint check (php -l kwenye files).

**Test Requirements:**
- TR-11.1 (rule): No PHP syntax errors katika files zote zinazobadilishwa.
- TR-11.2 (rule): No uncaught JS errors kwenye console load / auth / admin actions.
- TR-11.3 (rule): All AC zilizotaja katika spec.md zinakamilika (self-verified).
