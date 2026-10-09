# Mbilinyi Tech Solutions — SPEC ya Ongezeko (Tu Vinavyokosa)

## Tarehe: 2026-10-09
## Mtumiaji: CEO Jackson Mbilinyi

---

## 1. Shida na Malengo (Problem & Goals)

Mradi wa Mbilinyi Tech Solutions tayari uko na mazingira mazuri ya: landing page, bilingual (EN/SW), auth login/register, admin + client portals, contracts na e-signature, quote wizard, portfolio/testimonials hardcoded, na pricing plans.

**Lengo la spec hii:** KUONGEZA TU vifaa vilivyobainiwa kama havipo — kubadilisha chochote kilichopo (usibadilishe design, usifute code uliopo, uongeze tu).

### Malengo Muhimu (Non-Goals)
- Turekebishe chochote cha design uliopo (colors, spacing, layout) — HAPANA
- Tuchange code zilizopo kwa uwezo wake - HAPANA (tuongeza ziada tu)
- Tufute data za demo - HAPANA (admin aweze kufuta tu kwenye UI wake)

---

## 2. Mahitaji ya Kazi (Functional Requirements) — tu vinavyokosa

### FR-1: Tagline Mpya "YOUR PROBLEM, OUR SOLUTION"
**Maelezo:** Ongezwa tagline hii kwenye sehemu za:
  - Hero (chini ya h1 au kando ya tagline ya sasa "INNOVATE • SECURE • SCALE")
  - Footer ambapo "Innovate • Secure • Scale" inatokea (kwa barua pepe ya lib/email.php pia)
  - Kwa kila lugha (EN = "YOUR PROBLEM, OUR SOLUTION" | SW = "SHIDA YAKO, TATUZO LETU")

---

### FR-2: Auth Verification Code (OTP + Forgot Password + Resend)
**Maelezo:** Kwa sasa login inatumia email + password tu. Ongezea:
  a) **Verification code kwenye login:** Baada ya mtumiaji kuingiza email na password sahihi → mfumo utumie OTP code (6 tarakimu) kwa email → mtumiaji aingize code → mtu akaruhusiwa kuingia. Hii inahitaji:
     - Jedwali mpya wa OTPs (au column mpya kwenye users + session-based)
     - API ya kupokea OTP + ya kuthibitisha
     - "Resend Code" button kwenye OTP prompt
     - Counter timeout (e.g. 60s) kabla ya resend
  b) **Forgot Password:** Link kwenye "Sign In" → Aingize email → Tuma reset link/code kwa email → Weka password mpya
  c) **Resend Verification Code:** Karibu na OTP prompt.

*Kumbuka: Usibadilise jina la doLogin() wala mabadiliko ya API ya auth.php — tuongeza steps za kati ya password na redirect.*

---

### FR-3: PWA (Progressive Web App)
**Maelezo:** Ongezea:
  a) `manifest.json` (icons, name, theme color ya app)
  b) Service Worker js file kwa offline cache ya msingi
  c) Meta tags za PWA kwenye `partials/head.php` (theme-color, manifest link, apple-touch-icon)
  d) Icon files (tumia SVG placeholder na icon ya M bila kuadd vifaa vingi — au base64 ndogo)

---

### FR-4: Admin CRUD ya Portfolio (Projects) na Testimonials
**Maelezo:** Kwa sasa PORTFOLIO na TESTIS ni arrays hardcoded kwenye `assets/app.js`.
**Badala yake (tuongeza bila kufuta hardcode kwa muda):**
  a) Database tables mpya: `portfolio_projects` (id, title, category, description, image_url, stat_label, link_url, created_at, is_active) na `testimonials` (id, name, role, text, stars, created_at, is_active)
  b) Admin nav tab mpya au sehemu kwenye admin:
     - "Projects / Portfolio": Ongeza (picha, title, category, maelezo, stat [optional], link [optional]), Futa, Badilisha
     - "Testimonials": Ongeza (jina, nafasi, kauli, stars), Futa, Badilisha
  c) Demo data: Seed tables na PORTFOLIO + TESTIS zilizopo
  d) `work.php` na `renderWork()` iweze kuchora data kutoka DB (via API) badala ya hardcoded — ILA endapo API haija data i-rudi hardcoded kama fallback (usichambue kitu chochote)
  e) Admin dashboard iwe na "Delete" kila project/testimonial — hivyo demo data iweze kufutwa na admin kama user alivyosema.

---

### FR-5: Services Mpya kwa SERVICES Array (BRELA, TRA, Computer, Windows, Email)
**Maelezo:** Kwa sasa SERVICES = 8. Ongezea SERVICES 6 (or more) mpya kwenye `assets/app.js` (tuongeza kwenye array, usibadilise zilizopo):
  1. **BRELA & Business Registration** — Usajili wa makampuni, Biashara, SME Registration na BRELA
  2. **TRA & Tax Services** — TIN registration, File Tax Returns, TRA Systems support
  3. **Computer Troubleshooting** — Kutatua matatizo ya software kwenye kompyuta, malware removal, software repair
  4. **Windows & OS Installation** — Kupiga window, install Windows/Linux/macOS, drivers + updates
  5. **Email & Communication Setup** — Kutengeneza email accounts (G Suite, ZOHO, cPanel), kutatua matatizo ya email, SMTP setup
  6. **Government Services Support** — Huduma za NIDA, NHIF, NSSF, e-Services za serikali
  7. **Software Installation** — Kuinstall apps mbali mbali (MS Office, Adobe, IDEs, Antivirus, nk.)

  Pia ongeza `Laravel` + `PHP` kwenye STACK object (kwani user alitaja stack yake ni React, Rust backend, PHP, Laravel, R, HTML/CSS/JS — Laravel na PHP zinasahaulika kwenye STACK ya sasa).

---

### FR-6: Ollama Cloud API Integration kwa Chatbot
**Maelezo:** Kwa sasa chatbot kwenye footer (`toggleChat`, `sendChat`) ina logic ndogo ya hardcoded if/else.
Ongezea:
  a) Config ya OLLAMA_CLOUD_API_KEY + OLLAMA_MODEL (kwenye env/config)
  b) API endpoint mpya `api/ollama.php` (au endpoint ya chat) inayotuma request kwa Ollama Cloud
  c) System prompt ambayo inafundisha AI juu ya:
     - Maelezo ya kila huduma ya Mbilinyi Tech (Custom Software, BRELA, TRA, Computer, Windows, Email, nk.)
     - Bei, Malipo (50/50), Mikataba
     - Tech stack (React, Rust, PHP, Laravel, R, HTML/CSS/JS)
     - CEO Jackson Mbilinyi, Contacts (0796 752 645, mbilinyitech@gmail.com)
  d) sendChat() function: kama API key ya Ollama ipo tumia kweli, kama haipo iende kwenye hardcoded logic kama fallback (kwa usalama).
  e) AI iwe na uwezo wa kujibu kwa Kiswahili na Kingereza.

---

### FR-7: Visitor Tracking (Kila Siku Wanaotembelea Website)
**Maelezo:** Ongezea mfumo wa kuweka kila visitor (page view).
  a) Jedwali mpya `visitor_logs` (id, ip_hash, user_agent, page_url, referer, session_id, visited_at)
  b) Auto-record kila page load — kwenye `config/app.php` au partials/head.php, kwenye PHP top — usijali performance (its basic)
  c) Admin Dashboard tab "Analytics / Visitors" — Onesha:
     - Visitors leo (tarehe ya leo)
     - Visitors jana
     - Wiki hii na mwezi huu (kwa takwimu rahisi)
     - Top pages (zipi zinaendelea sana)
     - Simple Chart.js chart (kuna Chart.js tayari kwenye head.php!)

---

### FR-8: Malipo 50-50 na Masharti ya Kazi Kwenye Contracts/Quotes (Kuonyeshwa Kwa Wazi)
**Maelezo:** Kwa sasa pricing.php na client contracts tab ina zinginezo. Ongezea:
  a) Kwenye `assets/app.js` — contractHTML() function, baada ya scope, ongezea section ya "PAYMENT TERMS — MASHARTI YA MALIPO" inayosema:
     > "**1. 50% Deposit:** Nusu ya jumla ya gharama (`50% × total`) inalipwa kabla ya kazi kuanza. Mradi haunaanzzi hadi deposit isainishwe na CEO.
     > **2. 50% Mid-Project:** Nusu iliyobaki (`50% × total`) inalipwa wakati mradi umekamilikia kwa nusu, kabla ya usajili wa mwisho.
     > (Kwa mradi mrefu, instalments zinaweza kukubalianwa kwa mkataba.)"
  b) Kwenye admin tab ya "New Quotation" — Ongezea checkbox "Apply standard 50/50 payment terms" (default checked)
  c) Kwenye quote view (client) — Ongezea notice ya 50/50 inayoonekana dhahiri.
  d) Kwa kila contract — inapotumwa kwa email baada ya kusign (ambapo tayari kuna notify_*) — ongezea terms hizi kwenye email ya kusign.

*Note: User alisema "nusum hela tayarimelipwa baada ya kuanza mradi" — kwa hiyo 2nd 50% ni mid-project (kati ya mradi si mwisho tu).*

---

### FR-9: Contract Email baada ya Kila Mtu Kusaini
**Maelezo:** Kwa sasa contracts zipo na zinasign. Ongezea:
  - Baada ya client kusign → Tuma email kwa CEO kuwa "Client X amesaini contract X"
  - Baada ya CEO akisaini (countersign) → Tuma email kwa client na PDF/image ya contract iliyosainiwa na mtu wote pamoja na masharti ya malipo 50/50.
  - Contract iweze kupatikana na mtu pote (kwa admin na client portal kama inavyotokea kwa sasa).

---

## 3. Mahitaji Yasiyo ya Kazi (Non-Functional)

- **Bilingual:** Kila kitu kipya kinahitaji `data-en` + `data-sw` au toleo la EN/SW kwa consistency.
- **Backward Compatible:** Usibadilise code uliopo — ongeza tu. Kwa mfano PORTFOLIO hardcoded ibaki kama fallback.
- **Secure:** CSRF tokens zinatumika kwenye API zote mpya (kama zingine). OTP zizike hash.
- **Fail-Soft:** Ollama, Email, PWA — zote zinapaswa kuwa na fallback kama API key haipo.
- **No Breaking Changes:** Kila route, button, API endpoint iliyopo ibaki ikifanya kazi kama vile ilivyokuwa.

---

## 4. Madhara na Viwango vya Kukubaliana (Acceptance Criteria)

### AC — Vya Kutokea (Rule)

| ID | Aina | Maelezo |
|---|---|---|
| AC-1 | rule | Tagline "YOUR PROBLEM, OUR SOLUTION" / "SHIDA YAKO, TATUZO LETU" inapatikana kwenye hero, footer, na email shell yenye "Innovate•Secure•Scale" — bila kubadilisha ya sasa tuongeza kando ya au chini ya. |
| AC-2 | rule | Login iko na step mpya ya OTP: (1) email+pw sahihi → (2) OTP tumwa kwa email → (3) weka OTP → ingia. Kuna "Forgot Password", "Resend Code" zinafanya kazi. |
| AC-3 | rule | PWA manifest.json + service worker + PWA meta tags zipo kwenye head.php. |
| AC-4 | rule | Admin anaweza: (a) ku-add portfolio project (picha,title,cat,desc,stat,link), (b) ku-futa project au testimonial demo (c) zi-edit. Work.php inarender data kutoka DB kwa admin alivyobadilisha — bila data inarudial hardcoded. |
| AC-5 | rule | SERVICES array ina ziada za BRELA, TRA, Computer Troubleshooting, Windows Install, Email Setup, Government Services, Software Installation — na zinaonekana services.php. STACK ina Laravel + PHP. |
| AC-6 | rule | Chatbot inaweza kuzungumzia na Ollama Cloud API (ikiwa key ipo) — bila key inatumia logic ya zamani. System prompt inajua huduma zote, 50/50, CEO, contacts. |
| AC-7 | rule | Visitors hujirekodi kila page view. Admin dashboard anaona chart ya leo/jana/wiki/top pages. |
| AC-8 | rule | Contract na quote zina section dhahiri ya PAYMENT TERMS (50% deposit kabla, 50% mid-project). Email za contract baada ya kusign zinatumika. |

### Rubric (Vya Kubadilishwa kwa Ubora)

| ID | Aina | Kiwango (0-2) | Kipimo |
|---|---|---|---|
| AC-R1 | rubric | 2 = tagline inapatikana kila mahali pa namna ya kutosha bila kuvuruga layout | Ubora wa placement wa tagline (hero, footer, email) |
| AC-R2 | rubric | 2 = OTP + forgot password + resend zina UX nzuri (toast, errors, timeout) na zinafanya kazi bila kuzorota | UX wa mipaka ya auth |
| AC-R3 | rubric | 2 = Portfolio/Testimonials CRUD inafanya kazi kikamilifu, demo data inaonekana, admin aweze kutoa all demo | CRUD completeness |
| AC-R4 | rubric | 2 = Ollama chatbot inajibu swali kuhusu BRELA/TRA/computer kwa ufasaha + lugha ya EN/SW bila hallucinations kubwa | Ubora wa majibu ya AI |
| AC-R5 | rubric | 2 = Visitor analytics chart inatokea vizuri na data za kila siku zinajitokeza | Data + Chart ya visitors |

---

## 5. Vizungumzo vya Kufungua (Open Questions) [Hakuna kwa sasa — tumefuata maagizo ya user kwa usahihi]
