/* ================================================================
   MBILINYI TECH SOLUTIONS — FRONTEND DATA LAYER (PHP API-backed)
   Same UI, same function names — persistence now lives in Postgres
   through the PHP endpoints in /api.
================================================================ */

/* ---------- helpers ---------- */
const $ = id => document.getElementById(id);
const fmtTZS = n => 'TZS ' + Number(n || 0).toLocaleString('en-TZ');

/* Money in whichever currency an invoice was written in. */
const fmtMoney = (n, cur) => {
  const c = (cur || 'TZS').toUpperCase();
  if (c === 'TZS') return fmtTZS(n);
  const sym = c === 'USD' ? '$' : c === 'EUR' ? '€' : c === 'GBP' ? '£' : '';
  const v = Number(n || 0).toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
  return sym ? sym + v : c + ' ' + v;
};
const fmtDate = d => d ? new Date(d).toLocaleDateString('en-GB', { day: 'numeric', month: 'short', year: 'numeric' }) : '—';
const uid = p => p + Math.random().toString(36).slice(2, 8).toUpperCase();
const esc = s => String(s ?? '').replace(/[&<>"']/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));

/* ---------- API ---------- */
/* Clean URLs: the server rewrites /api/settings -> api/settings.php, so the
 * browser never has to see (or send) a file extension. One line keeps all
 * 86 call sites extension-free. */
const cleanPath = p => String(p).replace(/\.php$/, '');

async function apiGet(path, params) {
  const qs = params ? '?' + new URLSearchParams(params).toString() : '';
  const res = await fetch(cleanPath(path) + qs, { headers: { 'X-Requested-With': 'fetch' } });
  return apiHandle(res);
}

async function apiPost(path, body) {
  const res = await fetch(cleanPath(path), {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'X-CSRF-Token': window.__CSRF__ || '',
      'X-Requested-With': 'fetch',
    },
    body: JSON.stringify(body || {}),
  });
  return apiHandle(res);
}

async function apiHandle(res) {
  let data = null;
  try { data = await res.json(); } catch (e) { data = null; }
  if (!data) throw new Error('Server error (' + res.status + '). Reload the page.');
  if (!data.ok) throw new Error(data.error || 'Request failed.');
  return data.data;
}

function currentUser() { return window.__ME__ || null; }

/* Each service carries its own request form — fields differ per service.
 *
 * `kind` decides which wizard shape the service gets:
 *   'software' — a project build: title, requirements, tech stack.
 *   'service'  — a done-for-you task (registration, tax, repair, install…):
 *                its own title/subject and detail wording, and NO tech-stack,
 *                platform or "project" language, which only belongs to builds.
 */
const SERVICES = [
  { k: 'custom_software', kind: 'software', icon: 'fa-code', c: 'cyan', t: 'Custom Software Development', tSw: 'Uundaji wa Software Maalum', d: 'Build your own software with MBILINYITECH — mobile apps, desktop apps & web applications in React, Python, Rust, R & HTML — production-ready, tested & documented.', dSw: 'Jenga software yako mwenyewe na MBILINYITECH — programu za simu, kompyuta na wavuti kwa React, Python, Rust, R na HTML — tayari kutumika, imepimwa na yenye nyaraka.', tags: ['Mobile App', 'Desktop App', 'Web App'] },
  { k: 'system_analysis', kind: 'software', icon: 'fa-diagram-project', c: 'violet', t: 'System Analysis & Design', tSw: 'Uchambuzi na Ubunifu wa Mifumo', d: 'Professional SRS, database design, UML, workflows & feasibility studies before a single line of code.', dSw: 'SRS ya kitaalamu, ubunifu wa hifadhidata, UML, mtiririko wa kazi na upembuzi yakinifu kabla ya mstari mmoja wa code kuandikwa.', tags: ['SRS', 'UML', 'ERD'] },
  { k: 'it_consultancy', kind: 'service', icon: 'fa-briefcase', c: 'gold', t: 'IT Consultancy', tSw: 'Ushauri wa IT', d: 'Digital strategy, system audits, tech hiring advice & CTO-as-a-service for businesses & NGOs.', dSw: 'Mkakati wa kidijitali, ukaguzi wa mifumo, ushauri wa kuajiri wataalamu na huduma ya CTO kwa makampuni na taasisi.', tags: ['Strategy', 'Audit'] },
  { k: 'maintenance', kind: 'software', icon: 'fa-screwdriver-wrench', c: 'emerald', t: 'Software Maintenance', tSw: 'Matengenezo ya Software', d: 'Software maintenance & support with MBILINYITECH — bug fixes, updates, backups, uptime monitoring & SLA support — your system never sleeps.', dSw: 'Matengenezo na msaada wa software na MBILINYITECH — kurekebisha hitilafu, masasisho, nakala rudufu, ufuatiliaji wa muda wa kufanya kazi na msaada wa SLA — mfumo wako haukulala.', tags: ['SLA options', 'Monitoring', 'AMC'] },
  { k: 'cybersecurity', kind: 'service', icon: 'fa-shield-halved', c: 'red', t: 'Cybersecurity Services', tSw: 'Huduma za Usalama wa Mtandao', d: 'Penetration testing, OWASP audits, malware removal, SSL, firewall & staff security training.', dSw: 'Upimaji wa uvamizi, ukaguzi wa OWASP, kuondoa programu hasidi, SSL, firewall na mafunzo ya usalama kwa wafanyakazi.', tags: ['Pentest', 'OWASP'] },
  { k: 'forensics', kind: 'service', icon: 'fa-fingerprint', c: 'red', t: 'Digital Forensics', tSw: 'Uchunguzi wa Kidijitali (Forensics)', d: 'Digital forensics & cyber-crime investigation — forensic acquisition, disk/mobile/memory analysis, deleted-file recovery, timeline & chain-of-custody reports usable in legal proceedings.', dSw: 'Uchunguzi wa kidijitali na uhalifu wa mtandao — ukusanyaji wa ushahidi, uchambuzi wa diski/simu/kumbukumbu, kurejesha faili zilizofutwa, ripoti za muda na mnyororo wa ushahidi zinazoweza kutumika mahakamani.', tags: ['Forensics', 'Evidence', 'Incident Response'] },
  { k: 'graphics_design', kind: 'software', icon: 'fa-palette', c: 'pink', t: 'Graphics Design', tSw: 'Ubunifu wa Picha (Graphics)', d: 'Brand identity, logos, posters, flyers, banners, packaging, social media kits, UI mockups & print-ready artwork — designed to make your brand unmistakable.', dSw: 'Utambulisho wa chapa, nembo, mabango, vipeperushi, mabango ya wavuti, vifungashio, seti za mitandao ya kijamii, michoro ya UI na sanaa tayari kuchapishwa — ili chapa yako itambulike mara moja.', tags: ['Branding', 'Logos', 'Print'] },
  { k: 'cv_writing', kind: 'service', icon: 'fa-file-lines', c: 'amber', t: 'CV & Profile Writing', tSw: 'Kuandaa CV na Wasifu', d: 'Professional CV, cover letter, LinkedIn profile & portfolio — ATS-friendly wording that gets shortlisted, written for your target role and country.', dSw: 'CV ya kitaalamu, barua ya maombi, wasifu wa LinkedIn na portfolio — maneno yanayopita kwenye mifumo ya ATS ili uorodheshwe, yakiandikwa kwa nafasi na nchi unayolenga.', tags: ['CV', 'Cover Letter', 'LinkedIn'] },
  { k: 'mobile_desktop_app', kind: 'software', icon: 'fa-mobile-screen-button', c: 'indigo', t: 'Mobile & Desktop Apps', tSw: 'Programu za Simu na Kompyuta', d: 'Android, iOS, Windows, macOS & Linux applications — from Flutter cross-platform apps to installable desktop software with a proper setup installer.', dSw: 'Programu za Android, iOS, Windows, macOS na Linux — kutoka programu za Flutter zinazofanya kazi zote hadi software ya kompyuta inayosakinishwa na setup kamili.', tags: ['Android', 'iOS', 'Desktop Setup'] },
  { k: 'iot_gaming', kind: 'software', icon: 'fa-microchip', c: 'orange', t: 'IoT & Gaming', tSw: 'IoT na Michezo (Gaming)', d: 'IoT device software, sensor dashboards, embedded firmware, plus game and gamified-training builds for web and mobile.', dSw: 'Software ya vifaa vya IoT, dashibodi za sensor, firmware ya vifaa, pamoja na michezo na mafunzo ya kigemu kwa wavuti na simu.', tags: ['IoT', 'Sensors', 'Games'] },
  { k: 'industry_solutions', kind: 'software', icon: 'fa-industry', c: 'teal', t: 'Industry Solutions', tSw: 'Mifumo ya Sekta Mbalimbali', d: 'Ready-to-build systems for agriculture, transport, finance, education, health & government — records, payments, tracking, reporting and offline-first operation.', dSw: 'Mifumo tayari kujengwa kwa kilimo, usafiri, fedha, elimu, afya na serikali — kumbukumbu, malipo, ufuatiliaji, ripoti na kufanya kazi bila intaneti.', tags: ['Agriculture', 'Transport', 'Finance', 'Education'] },
  { k: 'ai_chatbots', kind: 'software', icon: 'fa-robot', c: 'fuchsia', t: 'AI Chatbots', tSw: 'Chatbots za AI', d: 'Build your own AI & Chatbot with MBILINYITECH — website & WhatsApp bots in English & Kiswahili that answer, sell & book — 24/7.', dSw: 'Jenga AI na Chatbot yako na MBILINYITECH — bots za wavuti na WhatsApp kwa Kiingereza na Kiswahili zinazojibu, zinazouza na zinazopanga — saa 24.', tags: ['WhatsApp', 'Web', 'Build Your Own'] },
  { k: 'ai_finetune', kind: 'software', icon: 'fa-brain', c: 'blue', t: 'AI Fine-Tuning', tSw: 'Urekebishaji wa AI', d: 'Build your own AI & Chatbot with MBILINYITECH — we fine-tune LLMs on YOUR data (price lists, policies, FAQs) for accurate, on-brand answers.', dSw: 'Jenga AI na Chatbot yako na MBILINYITECH — tunarekebisha LLM kwa data YAKO (orodha za bei, sera, maswali) kwa majibu sahihi na yenye utambulisho wako.', tags: ['LoRA', 'RAG', 'Your Own AI'] },
  { k: 'cloud_devops', kind: 'software', icon: 'fa-cloud', c: 'teal', t: 'Cloud & DevOps', tSw: 'Cloud na DevOps', d: 'Deployment on AWS/VPS, CI/CD, Docker, domains, business emails & blazing performance.', dSw: 'Usambazaji kwenye AWS/VPS, CI/CD, Docker, domain, email za biashara na utendaji wa kasi.', tags: ['AWS', 'Docker'] },
  { k: 'brela', kind: 'service', icon: 'fa-building', c: 'cyan', t: 'BRELA & Business Registration', tSw: 'Usajili wa BRELA na Biashara', d: 'Company, business & SME registration and BRELA certificates — every step handled for you.', dSw: 'Usajili wa kampuni, biashara na SME pamoja na vyeti vya BRELA — kila hatua tunaishughulikia kwa niaba yako.', tags: ['BRELA', 'SME'] },
  { k: 'tra_tax', kind: 'service', icon: 'fa-file-invoice-dollar', c: 'gold', t: 'TRA & Tax Services', tSw: 'Huduma za TRA na Kodi', d: 'TIN registration, tax returns, VAT, PAYE and full support for TRA business systems.', dSw: 'Usajili wa TIN, kuwasilisha kodi, VAT, PAYE na msaada kamili wa mifumo ya TRA kwa biashara.', tags: ['TIN', 'TRA', 'VAT'] },
  { k: 'troubleshooting', kind: 'service', icon: 'fa-screwdriver-wrench', c: 'emerald', t: 'Computer Troubleshooting', tSw: 'Utatuzi wa Matatizo ya Kompyuta', d: 'Fixing software problems, malware removal, slow PC, crashes, data recovery, drivers and updates.', dSw: 'Kutatua matatizo ya software, kuondoa programu hasidi, kompyuta ya polepole, kuganda, kurejesha data, drivers na masasisho.', tags: ['Repair', 'Malware'] },
  { k: 'windows_os', kind: 'service', icon: 'fa-windows', c: 'blue', t: 'Windows & OS Installation', tSw: 'Usakinishaji wa Windows na OS', d: 'Windows installation & OS setup with MBILINYITECH — Windows 10/11, Linux, macOS + drivers, updates, activation, backup and safe data transfer.', dSw: 'Usakinishaji wa Windows na mipangilio ya OS na MBILINYITECH — Windows 10/11, Linux, macOS + drivers, masasisho, uanzishaji, nakala rudufu na uhamishaji salama wa data.', tags: ['Windows', 'Drivers', 'Activation'] },
  { k: 'email_setup', kind: 'service', icon: 'fa-envelope-open-text', c: 'fuchsia', t: 'Email & Communication Setup', tSw: 'Usanidi wa Email na Mawasiliano', d: 'Business email setup with MBILINYITECH — G-Suite, Zoho, cPanel, SMTP, SPF/DKIM, and fixing every email problem.', dSw: 'Usanidi wa email za biashara na MBILINYITECH — G-Suite, Zoho, cPanel, SMTP, SPF/DKIM, na kutatua matatizo yote ya email.', tags: ['SMTP', 'G-Suite', 'SPF/DKIM'] },
  { k: 'gov_eservices', kind: 'service', icon: 'fa-landmark', c: 'violet', t: 'Government e-Services', tSw: 'Huduma za Serikali kwa Njia ya Kidijitali', d: 'Government application services — NIDA, NHIF, NSSF, eCitizen, Huduma Number, BRELA & TRA portals, TIN, licences and other government services — we guide you through every step.', dSw: 'Huduma za maombi ya serikali — NIDA, NHIF, NSSF, eCitizen, Huduma Number, portal za BRELA na TRA, TIN, leseni na huduma zingine za serikali — tunakupitia kila hatua.', tags: ['NIDA', 'NHIF', 'eCitizen', 'TIN'] },
  { k: 'software_install', kind: 'service', icon: 'fa-box', c: 'teal', t: 'Software Installation', tSw: 'Usakinishaji wa Software', d: 'Installing MS Office, Adobe, IDEs, antivirus, design apps, and customising settings properly.', dSw: 'Kusakinisha MS Office, Adobe, IDE, antivirus, programu za ubunifu, na kurekebisha mipangilio kwa usahihi.', tags: ['Office', 'Apps'] },
];

/* Admin can switch any service OFF in Settings → Service Visibility. Disabled
 * services disappear from the grid, the quote wizard and admin dropdowns. */
const SERVICES_ON = window.__SERVICES_ON__ || {};

/* BRELA, TRA, NIDA and the rest only exist in Tanzania, so they are hidden from
   everyone else. The CEO decides which services those are in Admin → Settings.
   A visitor can switch country themselves — a Tanzanian abroad still needs
   BRELA, and a wrong guess from an IP header must be correctable. */
const TZ_ONLY = window.__TZ_ONLY__ || [];
let VISITOR_COUNTRY = window.__COUNTRY__ || 'INTL';
function isTanzania() { return VISITOR_COUNTRY === 'TZ'; }
function setVisitorCountry(c) {
  document.cookie = `mts_country=${c};path=/;max-age=31536000;samesite=lax`;
  location.reload();
}
const VISIBLE_SERVICES = SERVICES.filter(s => SERVICES_ON[s.k] !== false && (isTanzania() || !TZ_ONLY.includes(s.k)));
const STACK = {
  'Python': { level: 98, desc: 'Backend APIs (Django/FastAPI), AI/ML, chatbots, automation & data pipelines. Our #1 backend.', use: 'APIs • AI • Automation' },
  'React': { level: 97, desc: 'Blazing dashboards, portals & mobile-friendly SPAs with modern UI/UX and reusable components.', use: 'Web Apps • Dashboards' },
  'Rust': { level: 90, desc: 'Memory-safe, ultra-fast microservices & system tools where performance and security are critical.', use: 'High-perf services' },
  'PHP': { level: 96, desc: 'Server-side rendering, portals, CMS, WordPress, payment gateways & robust REST APIs.', use: 'Websites • Portals' },
  'Laravel': { level: 94, desc: 'Enterprise-grade PHP framework: auth, queues, billing, ORM, admin panels & secure APIs.', use: 'Enterprise Apps' },
  'R': { level: 88, desc: 'Statistical engines, research dashboards, reports & data visualization for institutions & NGOs.', use: 'Analytics • Research' },
  'Go': { level: 91, desc: 'Fast, concurrent services, CLIs, network daemons and cloud infrastructure where throughput matters.', use: 'Cloud • Networks' },
  'Elixir': { level: 89, desc: 'Fault-tolerant, always-on systems on the BEAM — realtime messaging, telecoms, live chat and high-concurrency APIs that heal themselves.', use: 'Realtime • Fault-tolerant' },
  'HTML/CSS/JS': { level: 99, desc: 'Pixel-perfect, responsive, SEO-ready websites with Tailwind, animations & accessibility.', use: 'Websites • Landing' },
  'Node.js': { level: 92, desc: 'Realtime apps, M-Pesa integrations, websockets & scalable REST/GraphQL backends.', use: 'Realtime • M-Pesa' },
  'Flutter': { level: 85, desc: 'One codebase Android + iOS apps — perfect for shops, schools & delivery businesses.', use: 'Mobile Apps' },
  'SQL & Cloud': { level: 94, desc: 'PostgreSQL/MySQL design, AWS/VPS deployment, Docker, backups & monitoring.', use: 'Data • Deploy' },
};
const MAINT = [
  { n: 'Starter Care', p: 'TZS 150,000', per: '/month', f: ['Monthly health check & updates', 'Bug fixes (up to 5 hrs)', 'Weekly backups', 'Email support (48hr)', 'Uptime monitoring'], hot: false },
  { n: 'Business SLA', p: 'TZS 450,000', per: '/month', f: ['Everything in Starter', 'Priority fixes (20 hrs) + 24hr response', 'Security patches & SSL', 'Monthly performance report', 'WhatsApp priority line', 'Small feature tweaks'], hot: true },
  { n: 'Enterprise 24/7', p: 'TZS 1,200,000', per: '/month', f: ['Dedicated engineer', 'Unlimited critical fixes', 'Uptime monitoring & incident response', 'Daily backups + DR', 'On-call 24/7', 'Quarterly security audit'], hot: false },
];
const SEC = [
  { n: 'Security Audit', p: 'TZS 800,000', per: 'one-time', f: ['Full OWASP Top-10 scan', 'Vulnerability report + fixes plan', 'SSL & headers hardening', '1 staff training session'] },
  { n: 'Penetration Test', p: 'TZS 2,500,000', per: 'per system', f: ['Manual + automated pentest', 'Exploit verification', 'Executive + technical reports', 'Free re-test after fixes'] },
  { n: 'Managed Security', p: 'TZS 900,000', per: '/month', f: ['Firewall + WAF management', 'Malware monitoring & removal', 'Monthly scans & patching', 'Incident response 24/7'] },
];
/* Real delivered projects and client testimonials live in the database and are
 * managed by the admin (Portfolio / Testimonials tabs). These stay empty on a
 * fresh install so the site never claims work that has not been done yet. */
const PORTFOLIO = [];
const TESTIS = [];
let testiIdx = 0;

/* ---------- TOAST ---------- */
function toast(msg, type = 'ok') {
  const box = $('toasts');
  if (!box) return;
  const colors = { ok: ['border-emerald-400/40', 'fa-circle-check', 'text-emerald-300'], err: ['border-red-400/40', 'fa-circle-exclamation', 'text-red-300'], info: ['border-cyan-400/40', 'fa-circle-info', 'text-cyan-300'] };
  const c = colors[type] || colors.ok;
  const el = document.createElement('div');
  el.className = `toast-in glass-strong rounded-2xl p-4 border ${c[0]} flex gap-3 items-start shadow-2xl text-[13.5px]`;
  el.innerHTML = `<i class="fa-solid ${c[1]} ${c[2]} mt-0.5"></i><div class="flex-1">${msg}</div>`;
  box.appendChild(el);
  setTimeout(() => { el.style.transition = '.4s'; el.style.opacity = '0'; el.style.transform = 'translateX(100%)'; setTimeout(() => el.remove(), 400); }, 4200);
}

/* ---------- OUR PROJECTS → MbilinyiTech Platform ----------
 * The CEO pastes the platform link in Admin → Settings → Company Information.
 * Until it is set we hide the button rather than send visitors to a dead page,
 * and the "see projects on this site" link stays as the working alternative. */
/* ---------- COMPANY BRAND ----------
 * The CEO can replace the logo and the platform name in
 * Admin → Settings → Company Information. The header ships with the default
 * mark so the page renders before this runs; here we swap in whatever the
 * company actually set, on every page. */
function paintBrand() {
  const p = window.__PLATFORM__ || {};
  const logo = String(p.logo || '').trim();
  const img = $('brandLogo');
  if (img && logo && img.getAttribute('src') !== logo) {
    img.setAttribute('src', logo);
  }
  const nameEl = $('brandName');
  const name = String(p.name || '').trim();
  // Only replace the wordmark when the company set its own name, so the
  // designed default ("MBILINYI TECH SOLUTIONS") is never lost to a blank.
  if (nameEl && name && name !== 'MbilinyiTech Platform') {
    nameEl.textContent = name;
  }
}

function paintPlatformCta() {
  const a = $('platformCta');
  if (!a) return;
  const p = window.__PLATFORM__ || {};
  const url = String(p.url || '').trim();
  if (!url) { a.classList.add('hidden'); return; }
  a.href = url;
  const label = a.querySelector('[data-en]');
  if (label) {
    const name = String(p.name || '').trim() || 'MbilinyiTech Platform';
    label.dataset.en = `Visit ${name}`;
    label.dataset.sw = `Tembelea ${name}`;
    label.textContent = LANG === 'sw' ? label.dataset.sw : label.dataset.en;
  }
}

/* ---------- NAV / PAGES ---------- */
/* Kila sehemu ya landing ni ukurasa wake halisi — kubonyeza inakupeleka ukurasa mpya.
 * URLs are extensionless: the server rewrites /pricing -> pricing.php. */
const PAGES = {
  landing: '', services: 'services', stack: 'stack', ai: 'ai',
  pricing: 'pricing', process: 'process', work: 'work', track: 'track',
  contact: 'contact', auth: 'auth', client: 'client', admin: 'admin',
  mtaalamu: 'mtaalamu'
};

/* Normalise a location.pathname to a page key, accepting both the clean
 * form (/pricing) and the legacy one (/pricing.php) so old bookmarks and
 * any link we missed still highlight correctly. */
function pageKey(p) {
  const last = String(p || '/').split('/').pop();
  if (!last) return '';
  return last.replace(/\.php$/, '');
}

function goPage(v) {
  const target = PAGES[v];
  if (target === undefined) return;
  if (target === pageKey(location.pathname)) {
    window.scrollTo({ top: 0, behavior: 'smooth' });
    return;
  }
  location.href = target || '/';
}
/* showView inabaki kama jina la legacy — inaelekeza kwenye ukurasa halisi sasa. */
function showView(v) { goPage(v); }

function toggleMobileMenu() { const m = $('mobileMenu'); if (m) m.classList.toggle('hidden'); }

/* ---------- LANG ---------- */
/* Uchaguzi wa lugha unahifadhiwa (localStorage) na hurudishwa kwenye kila ukurasa. */
let LANG = 'en';
function applyLang(l) {
  LANG = l;
  document.documentElement.setAttribute('lang', l === 'sw' ? 'sw' : 'en');
  if ($('langEnBtn')) $('langEnBtn').className = l === 'en' ? 'px-2.5 py-1 rounded-full bg-white/10 font-bold text-white' : 'px-2.5 py-1 rounded-full text-white/60 font-bold hover:bg-white/10';
  if ($('langSwBtn')) $('langSwBtn').className = l === 'sw' ? 'px-2.5 py-1 rounded-full bg-white/10 font-bold text-white' : 'px-2.5 py-1 rounded-full text-white/60 font-bold hover:bg-white/10';
  /* Text content — never mix languages: each element carries BOTH strings and we
   * swap the whole node, so an English page shows English only and a Kiswahili
   * page shows Kiswahili only. */
  document.querySelectorAll('[data-en]').forEach(el => { el.textContent = l === 'sw' ? (el.dataset.sw ?? el.dataset.en) : el.dataset.en; });
  /* Placeholders get the same treatment via data-ph-en / data-ph-sw. */
  document.querySelectorAll('[data-ph-en]').forEach(el => { el.setAttribute('placeholder', l === 'sw' ? (el.dataset.phSw ?? el.dataset.phEn) : el.dataset.phEn); });
}
/* Inline bilingual helper for JS-rendered markup: T('English', 'Kiswahili'). */
function T(en, sw) { return LANG === 'sw' ? sw : en; }

function setLang(l) {
  try { localStorage.setItem('mts_lang', l); } catch (e) {}
  applyLang(l);
  /* Dynamic sections render their own strings from LANG, so re-draw them. */
  if (typeof renderServices === 'function') renderServices();
  if (typeof renderPricing === 'function') renderPricing();
  if (typeof renderNews === 'function') renderNews();
  if (typeof renderTeam === 'function') renderTeam();
  if (typeof paintWiz === 'function' && typeof $ === 'function' && $('wizLabel')) { updateWizardLabels(); paintWiz(); }
  /* The portal content is rendered from LANG, so redraw whichever tab is open. */
  if ($('clientContent') && currentUser()) clientTab(cTab);
  if ($('adminContent') && currentUser()?.role === 'admin') adminTab(aTab);
  if ($('staffContent') && currentUser()) staffTab(sTab);
  if (typeof paintVoiceBtn === 'function') paintVoiceBtn();
  if (typeof paintMicBtn === 'function') paintMicBtn();
  if (typeof stopSpeaking === 'function') stopSpeaking();
  toast(l === 'sw' ? 'Lugha imebadilishwa: Kiswahili ✓' : 'Language switched: English ✓', 'info');
}
try { LANG = localStorage.getItem('mts_lang') === 'sw' ? 'sw' : 'en'; } catch (e) {}
applyLang(LANG);

/* ---------- LANDING RENDER ---------- */
/* Kila sehemu inarenderiwa kwenye ukurasa wake — guards za kila section hapa chini. */
function renderServices() {
  if (!$('servicesGrid')) return;
  const cc = { cyan: 'text-cyan-300 bg-cyan-500/10 border-cyan-400/25', violet: 'text-violet-300 bg-violet-500/10 border-violet-400/25', gold: 'text-yellow-300 bg-yellow-500/10 border-yellow-400/25', emerald: 'text-emerald-300 bg-emerald-500/10 border-emerald-400/25', red: 'text-red-300 bg-red-500/10 border-red-400/25', fuchsia: 'text-fuchsia-300 bg-fuchsia-500/10 border-fuchsia-400/25', blue: 'text-blue-300 bg-blue-500/10 border-blue-400/25', teal: 'text-teal-300 bg-teal-500/10 border-teal-400/25' };
  $('servicesGrid').innerHTML = VISIBLE_SERVICES.map(s => {
    const sw = LANG === 'sw';
    const name = sw ? (s.tSw || s.t) : s.t;
    const desc = sw ? (s.dSw || s.d) : s.d;
    return `
    <div class="service-card glass rounded-3xl p-6 border border-white/10">
      <div class="w-12 h-12 rounded-2xl border flex items-center justify-center text-lg ${cc[s.c]}"><i class="fa-solid ${s.icon}"></i></div>
      <div class="font-bold mt-4 text-[15px]">${name}</div>
      <p class="text-white/55 text-[12.5px] mt-2 leading-relaxed">${desc}</p>
      <div class="flex flex-wrap gap-1.5 mt-3">${s.tags.map(t => `<span class="text-[10.5px] font-mono px-2 py-0.5 rounded-full bg-white/5 border border-white/10 text-white/60">${t}</span>`).join('')}</div>
      <button onclick="openQuoteWizard('${s.t}')" class="mt-4 text-[13px] font-bold text-cyan-300 hover:text-white">${sw ? 'Omba hii' : 'Request this'} →</button>
    </div>`;
  }).join('');
}
function renderStack() {
  if (!$('stackTabs')) return;
  $('stackTabs').innerHTML = Object.keys(STACK).map((k, i) => `<button onclick="showStack('${k}',this)" class="stk-tab px-4 py-2 rounded-xl border border-white/10 text-[12.5px] font-bold text-white/60 hover:border-cyan-400/40 ${i === 0 ? 'tab-active' : ''}">${k}</button>`).join('');
  showStack('Python');
  $('skillBars').innerHTML = Object.entries(STACK).map(([k, v]) => `
    <div class="rounded-2xl bg-white/[.03] border border-white/10 p-3.5">
      <div class="flex justify-between text-[12.5px] font-bold"><span class="font-mono">${k}</span><span class="text-cyan-300">${v.level}%</span></div>
      <div class="h-2 rounded-full bg-white/10 mt-2 overflow-hidden"><div class="h-full rounded-full bg-gradient-to-r from-cyan-400 to-violet-500 skill-fill" style="width:${v.level}%"></div></div>
    </div>`).join('');
}
function renderPricing() {
  if ($('maintPlans')) $('maintPlans').innerHTML = `<div class="col-span-full text-[12px] font-bold tracking-[.2em] text-emerald-300 mt-2"><i class="fa-solid fa-screwdriver-wrench mr-2"></i>SOFTWARE MAINTENANCE PLANS</div>` + MAINT.map(m => `
    <div class="glass rounded-3xl p-6 relative ${m.hot ? 'border-cyan-400/40 shadow-[0_0_50px_-12px_rgba(34,211,238,.4)]' : ''}">
      ${m.hot ? '<span class="absolute -top-3 left-1/2 -translate-x-1/2 badge grad-btn text-white">MOST POPULAR</span>' : ''}
      <div class="font-display font-bold text-lg">${m.n}</div>
      <div class="mt-2"><span class="font-display font-bold text-3xl grad-text">${m.p}</span><span class="text-white/40 text-sm">${m.per}</span></div>
      <div class="grid gap-2 mt-4 text-[13px] text-white/65">${m.f.map(f => `<div class="flex gap-2"><i class="fa-solid fa-check text-emerald-300 mt-1 text-xs"></i>${f}</div>`).join('')}</div>
      <button onclick="openQuoteWizard('Software Maintenance')" class="w-full mt-5 py-3 rounded-xl font-bold text-sm ${m.hot ? 'grad-btn' : 'border border-white/15 hover:bg-white/5'}">Subscribe →</button>
    </div>`).join('');
  /* The security block only appears when the admin has Cyber Security switched ON. */
  if ($('secPlans')) {
    const showSec = SERVICES_ON['cybersecurity'] !== false;
    $('secPlans').innerHTML = showSec
      ? `<div class="col-span-full text-[12px] font-bold tracking-[.2em] text-red-300 mt-6"><i class="fa-solid fa-shield-halved mr-2"></i>${LANG === 'sw' ? 'HUDUMA ZA USALAMA' : 'SECURITY SERVICES'}</div>` + SEC.map(m => `
    <div class="glass rounded-3xl p-6">
      <div class="font-display font-bold text-lg">${m.n}</div>
      <div class="mt-2"><span class="font-display font-bold text-3xl text-red-200">${m.p}</span><span class="text-white/40 text-sm"> ${m.per}</span></div>
      <div class="grid gap-2 mt-4 text-[13px] text-white/65">${m.f.map(f => `<div class="flex gap-2"><i class="fa-solid fa-check text-red-300 mt-1 text-xs"></i>${f}</div>`).join('')}</div>
      <button onclick="openQuoteWizard('Cybersecurity Services')" class="w-full mt-5 py-3 rounded-xl font-bold text-sm border border-red-400/30 text-red-200 hover:bg-red-500/10">${LANG === 'sw' ? 'Linda Mfumo Wangu' : 'Secure My System'} →</button>
    </div>`).join('')
      : '';
  }
}
let PORTFOLIO_DB = PORTFOLIO.slice();
let TESTIS_DB = TESTIS.slice();
let NEWS_DB = [];
let TEAM_DB = [];

async function loadNewsAndTeam(force = false) {
  try {
    const [n, tm] = await Promise.all([
      apiGet('api/news.php', { action: 'list' }).catch(() => null),
      apiGet('api/team.php', { action: 'list' }).catch(() => null),
    ]);
    if (n && Array.isArray(n.news)) NEWS_DB = n.news;
    if (tm && Array.isArray(tm.team)) TEAM_DB = tm.team;
  } catch { /* keep whatever we already have */ }
}

/* ---------- NEWS (public) ---------- */
function renderNews() {
  const box = $('newsGrid');
  if (!box) return;
  loadNewsAndTeam().then(() => {
    if (!NEWS_DB.length) {
      box.innerHTML = '<div class="md:col-span-2 lg:col-span-3 glass rounded-3xl p-8 text-center text-white/45"><span data-en="No news published yet. Check back soon." data-sw="Hakuna habari iliyochapishwa bado. Angalia tena hivi karibuni.">No news published yet. Check back soon.</span></div>';
      applyLang(LANG);
      return;
    }
    box.innerHTML = NEWS_DB.map(n => `
      <article class="glass rounded-3xl overflow-hidden border border-white/10 flex flex-col">
        ${n.imageUrl ? `<div class="h-44 overflow-hidden"><img src="${esc(n.imageUrl)}" class="w-full h-full object-cover" loading="lazy" alt="${esc(n.title)}"></div>` : ''}
        <div class="p-5 flex-1 flex flex-col">
          <div class="flex items-center gap-2"><span class="badge bg-cyan-500/10 text-cyan-300 border border-cyan-400/30">${esc(n.category)}</span><span class="text-[11.5px] text-white/40">${fmtDate(n.createdAt)}</span></div>
          <h3 class="font-bold text-[16px] mt-3 leading-snug">${esc(n.title)}</h3>
          <p class="text-white/55 text-[13px] mt-2 leading-relaxed flex-1">${esc(n.excerpt)}</p>
          <button onclick="viewNews('${n.id}')" class="mt-4 text-[13px] font-bold text-cyan-300 hover:text-white text-left"><span data-en="Read more →" data-sw="Soma zaidi →">Read more →</span></button>
        </div>
      </article>`).join('');
    applyLang(LANG);
  });
}

async function viewNews(id) {
  try {
    const d = await apiGet('api/news.php', { action: 'get', id });
    const n = d.post;
    openGen(esc(n.title), `
      <div class="flex flex-wrap items-center gap-2 mb-3"><span class="badge bg-cyan-500/10 text-cyan-300 border border-cyan-400/30">${esc(n.category)}</span><span class="text-[12px] text-white/40">${fmtDate(n.createdAt)}</span></div>
      ${n.imageUrl ? `<img src="${esc(n.imageUrl)}" class="w-full rounded-2xl mb-4" alt="${esc(n.title)}">` : ''}
      <div class="text-[14px] text-white/75 leading-relaxed whitespace-pre-line">${esc(n.body)}</div>`);
  } catch (e) { toast(esc(e.message), 'err'); }
}

/* ---------- OUR TEAM (public) ---------- */
function renderTeam() {
  const box = $('teamGrid');
  if (!box) return;
  loadNewsAndTeam().then(() => {
    if (!TEAM_DB.length) {
      box.innerHTML = '<div class="md:col-span-2 lg:col-span-3 glass rounded-3xl p-8 text-center text-white/45"><span data-en="Our team profiles will appear here." data-sw="Profaili za timu yetu zitaonekana hapa.">Our team profiles will appear here.</span></div>';
      applyLang(LANG);
      return;
    }
    box.innerHTML = TEAM_DB.map(m => {
      const initials = (m.name || 'MTS').split(' ').map(w => w[0]).join('').slice(0, 2).toUpperCase();
      return `<div class="glass rounded-3xl p-6 border border-white/10 text-center">
        ${m.imageUrl
          ? `<img src="${esc(m.imageUrl)}" class="w-24 h-24 rounded-3xl object-cover mx-auto border border-white/10" alt="${esc(m.name)}">`
          : `<div class="w-24 h-24 rounded-3xl grad-btn flex items-center justify-center font-display font-bold text-3xl mx-auto">${esc(initials)}</div>`}
        <div class="font-bold text-[16px] mt-4">${esc(m.name)}</div>
        <div class="text-[12.5px] text-cyan-300 font-semibold mt-1">${esc(LANG === 'sw' ? (m.roleSw || m.role) : m.role)}</div>
        <p class="text-white/55 text-[13px] mt-3 leading-relaxed">${esc(LANG === 'sw' ? (m.bioSw || m.bio) : m.bio)}</p>
        <div class="flex justify-center gap-2 mt-4">
          ${m.email ? `<a href="mailto:${esc(m.email)}" class="w-9 h-9 rounded-full bg-white/5 border border-white/10 flex items-center justify-center hover:border-blue-400/50" title="Email"><i class="fa-solid fa-envelope text-blue-300 text-xs"></i></a>` : ''}
          ${m.phone ? `<a href="tel:${esc(m.phone)}" class="w-9 h-9 rounded-full bg-white/5 border border-white/10 flex items-center justify-center hover:border-cyan-400/50" title="Call"><i class="fa-solid fa-phone text-cyan-300 text-xs"></i></a>` : ''}
          ${m.linkedin ? `<a href="${esc(m.linkedin)}" target="_blank" class="w-9 h-9 rounded-full bg-white/5 border border-white/10 flex items-center justify-center hover:border-cyan-400/50" title="LinkedIn"><i class="fa-brands fa-linkedin text-cyan-300 text-xs"></i></a>` : ''}
        </div>
      </div>`;
    }).join('');
  });
}

async function loadPortfolioAndTestimonials(force = false) {
  try {
    const [p, t] = await Promise.all([
      apiGet('api/portfolio.php', { action: 'list' }).catch(() => null),
      apiGet('api/testimonials.php', { action: 'list' }).catch(() => null),
    ]);
    if (p && Array.isArray(p.projects) && p.projects.length > 0) PORTFOLIO_DB = p.projects;
    else if (!force) PORTFOLIO_DB = PORTFOLIO.slice();
    if (t && Array.isArray(t.testimonials) && t.testimonials.length > 0) {
      TESTIS_DB = t.testimonials.map(x => ({
        n: x.name, r: x.role, t: x.quote, s: x.stars || 5, id: x.id, sort: x.sortOrder,
      }));
    } else if (!force) {
      TESTIS_DB = TESTIS.slice();
    }
  } catch { PORTFOLIO_DB = PORTFOLIO.slice(); TESTIS_DB = TESTIS.slice(); }
}

function renderWork() {
  if (!$('portfolioGrid')) return;
  loadPortfolioAndTestimonials().then(() => {
    $('portfolioGrid').innerHTML = PORTFOLIO_DB.map(p => {
      const img = p.img || p.imageUrl || p.image_url || '';
      const cat = p.c || p.category || '';
      const title = p.t || p.title || '';
      const desc = p.d || p.description || '';
      const stat = p.s || p.stat || '';
      const link = p.link || p.url || '';
      const card = `
        <div class="glass rounded-3xl overflow-hidden group">
          <div class="relative h-48 overflow-hidden"><img src="${img || 'data:image/svg+xml;utf8,<svg xmlns=%22http://www.w3.org/2000/svg%22 viewBox=%220 0 800 600%22><defs><linearGradient id=%22g%22 x1=%220%22 y1=%220%22 x2=%221%22 y2=%221%22><stop offset=%220%22 stop-color=%22%2306b6d4%22/><stop offset=%22.5%22 stop-color=%22%233b82f6%22/><stop offset=%221%22 stop-color=%22%238b5cf6%22/></linearGradient></defs><rect width=%22800%22 height=%22600%22 fill=%22url(%23g)%22/></svg>'}" class="w-full h-full object-cover group-hover:scale-110 transition duration-700" loading="lazy" onerror="this.onerror=null;this.src='data:image/svg+xml;utf8,<svg xmlns=%22http://www.w3.org/2000/svg%22 viewBox=%220 0 800 600%22><rect width=%22800%22 height=%22600%22 fill=%22%230a1226%22/><text x=%2250%25%22 y=%2250%25%22 font-family=%22Arial%22 font-size=%2242%22 fill=%22%2322d3ee%22 text-anchor=%22middle%22>MTS Project</text></svg>'"><div class="absolute inset-0 bg-gradient-to-t from-[#050914] via-transparent"></div>${cat ? `<span class="absolute top-3 left-3 badge bg-black/60 text-cyan-200 border border-white/20 backdrop-blur">${cat}</span>` : ''}${stat ? `<span class="absolute bottom-3 left-3 badge bg-emerald-500/80 text-white">${stat}</span>` : ''}${link ? `<a href="${link}" target="_blank" class="absolute bottom-3 right-3 w-9 h-9 rounded-full bg-black/60 border border-white/20 backdrop-blur flex items-center justify-center text-white hover:bg-cyan-500/80 transition"><i class="fa-solid fa-arrow-up-right-from-square text-[11px]"></i></a>` : ''}</div>
          <div class="p-5"><div class="font-bold">${title}</div><p class="text-white/55 text-[12.5px] mt-1.5">${desc}</p></div>
        </div>`;
      return card;
    }).join('') || `<div class="text-white/50 text-[13.5px] glass rounded-3xl p-8 col-span-full text-center leading-relaxed"><i class="fa-solid fa-hammer text-cyan-300 text-2xl mb-2 block"></i>${LANG === 'sw' ? 'Bado hatujamaliza mradi wa kwanza — na hatutadai uongo. Tunaunda mifumo halisi sasa; kuwa mteja wetu wa kwanza na kazi yako itaonekana hapa.' : 'We have not finished our first project yet — and we will not claim otherwise. We are building real systems now; become our first client and your work will appear here.'}</div>`;
    renderTesti();
    testiIdx = 0;
    if (window._testiInt) clearInterval(window._testiInt);
    if (TESTIS_DB.length > 1) {
      window._testiInt = setInterval(() => { testiIdx = (testiIdx + 1) % TESTIS_DB.length; renderTesti(); }, 7000);
    }
  });
}
function renderHero() {
  if (!$('techMarquee')) return;
  $('techMarquee').innerHTML = Array(2).fill(['PHP','LARAVEL','R','RUST','REACT','PYTHON','HTML5','CSS3','JAVASCRIPT','TYPESCRIPT','NODE.JS','FLUTTER','POSTGRESQL','DOCKER','AWS','TENSORFLOW','TAILWIND','DJANGO'].map(t => `<span class="flex items-center gap-2 px-4"><span class="w-1.5 h-1.5 rounded-full bg-cyan-400"></span>${t}</span>`).join('')).join('');
  const obs = new IntersectionObserver(es => es.forEach(e => { if (e.isIntersecting) { const el = e.target, t = +el.dataset.target; let n = 0; const iv = setInterval(() => { n += Math.max(1, Math.round(t / 40)); if (n >= t) { n = t; clearInterval(iv); } el.textContent = n; }, 30); obs.unobserve(el); } }));
  const svcCount = $('svcCount');
  if (svcCount) svcCount.dataset.target = VISIBLE_SERVICES.length;
  document.querySelectorAll('.counter').forEach(el => obs.observe(el));
  setInterval(() => {
    const stacks = ['"react", "rust", "php"', '"laravel", "node", "postgres"', '"r", "react", "docker"', '"python", "llama-3", "whatsapp"'];
    const el = $('heroStackText');
    if (el) el.textContent = stacks[Math.floor(Math.random() * stacks.length)];
  }, 3000);
}
function showStack(k, btn) {
  if (btn) { document.querySelectorAll('.stk-tab').forEach(b => b.classList.remove('tab-active')); btn.classList.add('tab-active'); }
  const v = STACK[k];
  if (!$('stackDetail')) return;
  $('stackDetail').innerHTML = `<div class="flex items-center justify-between"><div class="font-mono font-bold text-cyan-300 text-lg">${k}</div><div class="text-[12px] text-white/50">${v.use}</div></div><div class="h-2.5 rounded-full bg-white/10 mt-3 overflow-hidden"><div class="h-full rounded-full bg-gradient-to-r from-cyan-400 via-blue-500 to-violet-500" style="width:${v.level}%"></div></div><p class="text-[13.5px] text-white/65 mt-3 leading-relaxed">${v.desc}</p>`;
}
function renderTesti() {
  if (!$('testiBox')) return;
  if (!TESTIS_DB.length) {
    $('testiBox').innerHTML = `<div class="text-white/50 text-[13.5px] leading-relaxed"><i class="fa-solid fa-quote-left text-cyan-300/60 mr-2"></i>${LANG === 'sw' ? 'Bado hakuna ushuhuda wa wateja — tutaonyesha maoni halisi mara tu mteja wa kwanza atakapokamilisha kazi nasi.' : 'No client testimonials yet — real reviews will appear here as soon as our first client completes a project with us.'}</div>`;
    return;
  }
  const idx = testiIdx % TESTIS_DB.length;
  const t = TESTIS_DB[idx] || TESTIS[0];
  const stars = Math.max(1, Math.min(5, Number(t.s || t.stars) || 5));
  const initials = (t.n || t.name || 'MTS').split(' ').map(w => w[0]).join('').slice(0, 2);
  $('testiBox').innerHTML = `<div class="flex gap-1 text-gold text-xs">${'<i class="fa-solid fa-star"></i>'.repeat(stars)}</div><p class="text-[16px] md:text-[18px] leading-relaxed mt-2 text-white/85">“${t.t || t.quote || '—'}”</p><div class="flex items-center gap-3 mt-4"><div class="w-11 h-11 rounded-full grad-btn flex items-center justify-center font-bold">${initials}</div><div><div class="font-bold text-[14px]">${t.n || t.name || ''}</div><div class="text-[12px] text-white/50">${t.r || t.role || ''}</div></div></div>`;
}
function moveTesti(d) {
  if (!TESTIS_DB.length) return;
  testiIdx = (testiIdx + d + TESTIS_DB.length) % TESTIS_DB.length;
  renderTesti();
}

/* AI DEMO */
let aiBrain = 'pharmacy';
document.addEventListener('click', e => { const b = e.target.closest('.brain-btn'); if (b) { document.querySelectorAll('.brain-btn').forEach(x => x.classList.remove('tab-active')); b.classList.add('tab-active'); aiBrain = b.dataset.brain; } });
function runAiDemo() {
  if (!$('aiDemoInput')) return;
  const q = $('aiDemoInput').value.trim() || 'What are your prices / Bei zenu ni zipi?';
  const ans = {
    pharmacy: `Karibu Amina Pharmacy! ✅ Dawa ya presha (Amlodipine)ipo — TZS 8,500/dose. Tunafungua 8:00–22:00, tunatuma Dar popote via bodaboda (TZS 3,000). Lipia M-Pesa 0796752645. Unahitaji prescription upload? Reply 1 kuagiza, 2 kuongea na pharmacist.`,
    school: `Hello! Moshi Academy Form One fees 2026: TZS 1,450,000/year (incl. meals). Pay via M-Pesa PayBill 555001, Ref: student name. Receipt + SMS in 2 mins. 2nd installment due 15 Aug. Need fee slip? Reply with admission no.`,
    hotel: `Karibu Seaview Hotel! 🌊 Deluxe sea-view tonight: TZS 180,000 (breakfast incl). 2 rooms left. Free airport pickup + WiFi 200Mbps. Lipa 30% deposit M-Pesa to confirm. Reply BOOK + name to reserve instantly.`
  };
  const badge = LANG === 'sw' ? 'IMEFUNZWA KWA DATA YAKO • SW/EN • 1.8s' : 'TRAINED ON YOUR DATA • SW/EN • 1.8s';
  $('aiDemoAnswer').innerHTML = `<span class="text-white/50 font-mono text-[12px]">Q: "${esc(q)}"</span><br><br>${ans[aiBrain]}<br><br><span class="badge bg-emerald-500/15 text-emerald-300 border border-emerald-400/30">${badge}</span>`;
  toast(LANG === 'sw' ? 'Jibu lililofunzwa limetengenezwa — fikiria hili likifunzwa kwa biashara YAKO!' : 'Fine-tuned answer generated — imagine this trained on YOUR business!', 'info');
  if (voiceOn) speak($('aiDemoAnswer').textContent);
}

/* The demo answer has no data-en pair (JS owns it), so read it directly. */
function speakAiDemo() {
  const txt = ($('aiDemoAnswer')?.textContent || '').trim();
  if (!txt) return;
  if (!('speechSynthesis' in window)) { toast('Voice is not supported in this browser.', 'err'); return; }
  const wasOff = !voiceOn;
  if (wasOff) voiceOn = true;
  speak(txt);
  if (wasOff) voiceOn = false;
}

/* ---------- TRACK ---------- */
function fillTrack(id) { $('trackInput').value = id; trackOrder(); }
function statusStep(s) { return { 'Pending': 1, 'Under Review': 2, 'Quoted': 3, 'Approved': 4, 'In Progress': 5, 'Completed': 6, 'Rejected': 0 }[s] ?? 1; }

async function trackOrder() {
  const id = ($('trackInput')?.value || '').trim().toUpperCase();
  if (!id) { toast('Enter a Tracking ID.', 'err'); return; }
  $('trackResult').innerHTML = `<div><i class="fa-solid fa-circle-notch fa-spin text-3xl text-white/30"></i><div class="text-[13px] mt-3">Looking up ${esc(id)}…</div></div>`;
  let d;
  try {
    d = await apiGet('api/track.php', { id });
  } catch (e) {
    $('trackResult').innerHTML = `<div><i class="fa-solid fa-magnifying-glass text-4xl text-white/15"></i><div class="font-bold mt-3 text-white">Not found</div><div class="text-[13px]">${esc(e.message)}</div></div>`;
    return;
  }
  const r = d.request, q = d.quote, c = d.contract;
  const steps = ['Submitted', 'Under Review', 'Quoted', 'Approved', 'In Progress', 'Completed'];
  const cur = statusStep(r.status);
  $('trackResult').innerHTML = `
    <div class="text-left w-full">
      <div class="flex flex-wrap items-center justify-between gap-2"><div><div class="font-mono font-bold text-cyan-300">${r.trackingId}</div><div class="font-bold text-[16px]">${esc(r.title)}</div><div class="text-[12px] text-white/50">${esc(r.service)} • ${fmtDate(r.createdAt)}</div></div><span class="badge status-${r.status.replace(' ', '')}">${r.status.toUpperCase()}</span></div>
      <div class="mt-5 relative pl-6"><div class="absolute left-[9px] top-2 bottom-2 w-[3px] rounded timeline-line opacity-60"></div>
        ${steps.map((s, i) => { const n = i + 1; const done = n <= cur; return `<div class="relative pb-4 last:pb-0"><span class="absolute -left-6 top-0 w-5 h-5 rounded-full ${done ? 'bg-cyan-400 step-dot' : 'bg-white/10 border border-white/15'} flex items-center justify-center">${done ? '<i class="fa-solid fa-check text-[9px] text-[#050914]"></i>' : ''}</span><div class="text-[13.5px] font-bold ${done ? 'text-white' : 'text-white/35'}">${s}</div><div class="text-[11.5px] text-white/40">${n === 3 && q ? 'Quotation ' + fmtTZS(q.total) + ' — valid till ' + fmtDate(q.validUntil) : n === 4 && c ? 'Contract ' + c.id + ' — ' + c.status : n === 1 ? 'Received by CEO office' : ''}</div></div>`; }).join('')}
      </div>
      ${r.adminNote ? `<div class="mt-4 rounded-xl bg-violet-500/[.08] border border-violet-400/25 p-3.5 text-[13px]"><span class="font-bold text-violet-300"><i class="fa-solid fa-user-tie mr-1"></i> ${T('Update from the CEO', 'Taarifa kutoka kwa CEO')}:</span> ${esc(r.adminNote)}</div>` : ''}
      <div class="grid grid-cols-3 gap-2 mt-4 text-center text-[12px]">
        <div class="rounded-xl bg-white/[.03] border border-white/10 p-3"><div class="font-bold ${q ? 'text-emerald-300' : 'text-white/30'}">${q ? '✓ Quoted' : '○ Quote'}</div><div class="text-white/45">${q ? fmtTZS(q.total) : 'Pending'}</div></div>
        <div class="rounded-xl bg-white/[.03] border border-white/10 p-3"><div class="font-bold ${c ? 'text-emerald-300' : 'text-white/30'}">${c ? '✓ Contract' : '○ Contract'}</div><div class="text-white/45">${c ? c.status : 'Pending'}</div></div>
        <div class="rounded-xl bg-white/[.03] border border-white/10 p-3"><div class="font-bold text-cyan-300">Support</div><a href="https://wa.me/255796752645?text=Tracking%20${r.trackingId}" target="_blank" class="text-white/60 underline">WhatsApp us</a></div>
      </div>
    </div>`;
}

/* ---------- CONTACT ---------- */
async function sendContactTicket() {
  const n = $('cName').value.trim(), p = $('cPhone').value.trim(), e = $('cEmail').value.trim(), s = $('cSubject').value.trim(), m = $('cMsg').value.trim();
  if (!n || !p || !e || !m) { toast(T('Please fill name, phone, email and message.', 'Jaza sehemu zote muhimu.'), 'err'); return; }
  if (!/^[^@\s]+@[^@\s]+\.[^@\s]+$/.test(e)) { toast('Invalid email address.', 'err'); return; }
  try {
    await apiPost('api/tickets.php', { action: 'create', name: n, email: e, subject: s || 'Website inquiry', message: `Phone: ${p}\n${m}` });
  } catch (err) { toast(esc(err.message), 'err'); return; }
  $('cName').value = $('cPhone').value = $('cEmail').value = $('cSubject').value = $('cMsg').value = '';
  toast('Message sent! Ticket opened — we reply within 4hrs. Asante! ✓', 'ok');
}

/* ---------- FEEDBACK ---------- */
let __FB_RATING = 0;
function initFeedbackStars() {
  const box = $('fbStars');
  if (!box || box.dataset.ready === '1') return;
  box.dataset.ready = '1';
  const paint = () => {
    box.innerHTML = [1, 2, 3, 4, 5].map(i =>
      `<button type="button" onclick="__setRating(${i})" class="w-7 h-7 rounded-lg border text-sm transition ${
        i <= __FB_RATING ? 'bg-amber-400/20 border-amber-400/60 text-amber-300' : 'bg-white/[.03] border-white/15 text-white/35 hover:border-white/40'
      }">${i <= __FB_RATING ? '★' : '☆'}</button>`).join('');
    const v = $('fbRatingVal'); if (v) v.textContent = __FB_RATING + '/5';
  };
  window.__setRating = (n) => { __FB_RATING = (n === __FB_RATING) ? 0 : n; paint(); };
  paint();
}
async function submitFeedback() {
  const n = ($('fbName')?.value || '').trim();
  const e = ($('fbEmail')?.value || '').trim();
  const s = ($('fbSubject')?.value || '').trim();
  const c = ($('fbCategory')?.value || 'general');
  const m = ($('fbMessage')?.value || '').trim();
  const out = $('fbResult');
  if (!n || !m) { toast(T('Please enter your name and message.', 'Jaza jina na ujumbe.'), 'err'); return; }
  if (e && !/^[^@\s]+@[^@\s]+\.[^@\s]+$/.test(e)) { toast(T('Invalid email address.', 'Barua pepe si sahihi.'), 'err'); return; }
  const btn = $('fbBtn');
  if (btn) btn.disabled = true;
  try {
    const r = await apiPost('api/settings.php', { action: 'feedback', name: n, email: e, subject: s, category: c, message: m, rating: __FB_RATING });
    if (out) {
      out.className = 'mt-4 rounded-xl p-4 text-[13.5px] bg-emerald-500/10 border border-emerald-400/35 text-emerald-100';
      out.innerHTML = `<i class="fa-solid fa-circle-check mr-2"></i>${esc(r.thanks || 'Asante!')} <span class="text-white/60">Ref: ${esc(r.id || '')}</span>`;
      out.classList.remove('hidden');
    }
    if ($('fbName')) $('fbName').value = '';
    if ($('fbEmail')) $('fbEmail').value = '';
    if ($('fbSubject')) $('fbSubject').value = '';
    if ($('fbMessage')) $('fbMessage').value = '';
    __FB_RATING = 0; const b = $('fbStars'); if (b) { b.dataset.ready = ''; initFeedbackStars(); }
    toast(T('Thanks for your feedback!', 'Asante kwa maoni yako!'), 'ok');
  } catch (err) {
    toast(esc(err.message), 'err');
    if (out) {
      out.className = 'mt-4 rounded-xl p-4 text-[13.5px] bg-rose-500/10 border border-rose-400/35 text-rose-100';
      out.innerHTML = `<i class="fa-solid fa-triangle-exclamation mr-2"></i>${esc(err.message)}`;
      out.classList.remove('hidden');
    }
  } finally { if (btn) btn.disabled = false; }
}

/* ---------- AUTH ---------- */
function switchAuthTab(t) {
  $('authLogin').classList.toggle('hidden', t !== 'login'); $('authRegister').classList.toggle('hidden', t !== 'register');
  $('tabLogin').className = t === 'login' ? 'py-3 rounded-xl font-bold text-sm grad-btn' : 'py-3 rounded-xl font-bold text-sm text-white/60';
  $('tabRegister').className = t === 'register' ? 'py-3 rounded-xl font-bold text-sm gold-btn text-[#2a1500]' : 'py-3 rounded-xl font-bold text-sm text-white/60';
}
function togglePw(id, btn) {
  const el = $(id); if (!el) return;
  const show = el.type === 'password';
  el.type = show ? 'text' : 'password';
  if (btn) btn.innerHTML = `<i class="fa-solid ${show ? 'fa-eye-slash' : 'fa-eye'}"></i>`;
}
/* Live password strength — purely advisory, the server still enforces 6+. */
function pwStrength(id, meterId) {
  const v = ($(id)?.value) || '';
  let score = 0;
  if (v.length >= 6) score++;
  if (v.length >= 10) score++;
  if (/[A-Z]/.test(v) && /[a-z]/.test(v)) score++;
  if (/\d/.test(v)) score++;
  if (/[^A-Za-z0-9]/.test(v)) score++;
  const bar = $(meterId)?.firstElementChild;
  if (!bar) return;
  const pct = [0, 25, 45, 65, 85, 100][Math.min(score, 5)];
  const tone = score <= 1 ? '#ef4444' : score === 2 ? '#f59e0b' : score === 3 ? '#eab308' : '#22c55e';
  bar.style.width = pct + '%';
  bar.style.background = tone;
}
/* Login ni moja — server inatambua role (admin/client) kiotomatiki baada ya kuingia. */

function returnTarget(role) {
  const rt = (new URLSearchParams(location.search).get('returnTo') || '').replace(/\.php$/, '');
  if (rt === 'client' || rt === 'admin') {
    if (rt === 'admin' && role !== 'admin') return 'client';
    return rt;
  }
  return role === 'admin' ? 'admin' : 'client';
}

let __OTP_EMAIL = '';
let __RESEND_COUNTDOWN = 0;
let __PIN_AVAILABLE = false;
let __PIN_KIND = '';
let __RESEND_TIMER = null;

function stopResendTimer() {
  if (__RESEND_TIMER) { clearInterval(__RESEND_TIMER); __RESEND_TIMER = null; }
  __RESEND_COUNTDOWN = 0;
}
function paintResendBtn(btnId = 'resendBtn') {
  const b = $(btnId); if (!b) return;
  if (__RESEND_COUNTDOWN > 0) {
    b.disabled = true;
    b.className = 'px-4 py-2.5 rounded-xl bg-white/[.04] border border-white/10 text-white/30 text-[12px] font-bold';
    b.innerHTML = `<span data-en="Resend code in" data-sw="Tuma tena baada ya">Resend code in</span> ${__RESEND_COUNTDOWN}s`;
  } else {
    b.disabled = false;
    b.className = 'px-4 py-2.5 rounded-xl border border-cyan-400/50 text-cyan-300 hover:text-white hover:bg-cyan-500/10 text-[12px] font-bold';
    b.innerHTML = `<i class="fa-solid fa-rotate-right mr-1.5"></i><span data-en="Resend 6-digit code" data-sw="Tuma tena code ya 6 tarakimu">Resend 6-digit code</span>`;
  }
}
function startResendTimer(seconds = 60) {
  stopResendTimer();
  __RESEND_COUNTDOWN = seconds;
  paintResendBtn();
  __RESEND_TIMER = setInterval(() => {
    __RESEND_COUNTDOWN = Math.max(0, __RESEND_COUNTDOWN - 1);
    paintResendBtn();
    if (__RESEND_COUNTDOWN === 0) stopResendTimer();
  }, 1000);
}

function openOtpModal(email, debugCode = null, pinAvailable = false, pinKind = '') {
  __OTP_EMAIL = email;
  __PIN_AVAILABLE = !!pinAvailable;
  __PIN_KIND = pinKind;
  const hint = debugCode
    ? `<div class="text-[11px] font-mono text-white/35 mt-2 rounded-lg bg-white/[.03] p-2 border border-white/10"><span class="text-white/50 font-bold">DEV debug</span> — your test OTP is: <span class="text-emerald-300 font-bold text-lg">${esc(debugCode)}</span></div>`
    : `<div class="text-[11px] text-white/35 mt-2"><i class="fa-solid fa-envelope-open-text mr-1 text-cyan-300"></i><span data-en="Check your inbox (and spam/junk folder) for a 6-digit code from mbilinyitech@gmail.com. Expires in 10 minutes." data-sw="Tafuta kwenye email yako (na spam/junk folder) code ya 6 tarakimu kutoka mbilinyitech@gmail.com. Itaisha baada ya dakika 10.">Check your inbox (and spam/junk folder) for a 6-digit code from mbilinyitech@gmail.com. Expires in 10 minutes.</span></div>`;
  const pinLabel = pinKind === 'phone'
    ? '<span data-en="ENTER THE LAST 4 DIGITS OF YOUR PHONE (account pattern)" data-sw="INGIZA TARAKIMU 4 ZA MWISHO ZA SIMU YAKO (pattern ya account)">ENTER THE LAST 4 DIGITS OF YOUR PHONE (account pattern)</span>'
    : '<span data-en="ENTER YOUR ACCOUNT PIN (4–6 digits)" data-sw="INGIZA PIN YA ACCOUNT YAKO (tarakimu 4–6)">ENTER YOUR ACCOUNT PIN (4–6 digits)</span>';
  openGen('<i class="fa-solid fa-shield-halved mr-1.5 text-cyan-300"></i>Verify Login', `
    <div class="text-[13px] text-white/65"><span data-en="Verifying that this account is really yours:" data-sw="Tunathibitisha kuwa account hii ni yako kweli:">Verifying that this account is really yours:</span> <span class="font-bold text-white">${esc(email)}</span></div>
    ${pinAvailable ? `<div class="grid grid-cols-2 gap-2 p-1.5 rounded-2xl bg-white/[.04] border border-white/10 mt-4">
      <button id="otpModeCode" onclick="switchVerifyMode('code')" class="py-2.5 rounded-xl font-bold text-[13px] grad-btn"><i class="fa-solid fa-envelope mr-1.5"></i><span data-en="Emailed Code" data-sw="Code ya Email">Emailed Code</span></button>
      <button id="otpModePin" onclick="switchVerifyMode('pin')" class="py-2.5 rounded-xl font-bold text-[13px] text-white/60"><i class="fa-solid fa-key mr-1.5"></i>PIN</button>
    </div>` : ''}
    <div id="verifyCodeWrap">
      ${hint}
      <label class="text-[12px] font-bold text-white/60 mt-4 block"><span data-en="ENTER 6-DIGIT CODE" data-sw="INGIZA CODE YA 6 TARAKIMU">ENTER 6-DIGIT CODE</span></label>
      <div class="grid grid-cols-6 gap-2 mt-2">
        ${[0,1,2,3,4,5].map(i => `<input id="otp${i}" inputmode="numeric" maxlength="1" class="input !text-center !text-xl !font-mono !tracking-[0.3em]" oninput="onOtpInput(${i})" onkeydown="onOtpKey(${i},event)">`).join('')}
      </div>
      <div class="flex flex-col sm:flex-row gap-2 mt-6">
        <button onclick="doVerifyOTP()" id="verifyBtn" class="grad-btn flex-1 py-3.5 rounded-xl font-bold text-sm"><i class="fa-solid fa-shield-check mr-2"></i><span data-en="Verify & Sign In" data-sw="Thibitisha na Uingie">Verify & Sign In</span></button>
        <button onclick="resendLoginCode()" id="resendBtn" class="px-4 py-3.5 rounded-xl border border-cyan-400/50 text-cyan-300 hover:text-white hover:bg-cyan-500/10 text-sm font-bold"><i class="fa-solid fa-rotate-right mr-1.5"></i><span data-en="Resend code" data-sw="Tuma tena">Resend code</span></button>
      </div>
    </div>
    <div id="verifyPinWrap" class="hidden">
      <div class="text-[11px] text-white/35 mt-3"><i class="fa-solid fa-lock mr-1 text-cyan-300"></i><span data-en="5 wrong attempts lock the account for 15 minutes." data-sw="Majaribio 5 mabaya yanafunga account kwa dakika 15.">5 wrong attempts lock the account for 15 minutes.</span></div>
      <label class="text-[12px] font-bold text-white/60 mt-4 block">${pinLabel}</label>
      <input id="loginPin" inputmode="numeric" maxlength="6" class="input mt-1.5 !text-center !text-2xl !font-mono !tracking-[0.4em]" onkeydown="if(event.key==='Enter')doVerifyPIN()">
      <button onclick="doVerifyPIN()" class="grad-btn w-full mt-5 py-3.5 rounded-xl font-bold text-sm"><i class="fa-solid fa-key mr-2"></i><span data-en="Verify PIN & Sign In" data-sw="Thibitisha PIN na Uingie">Verify PIN & Sign In</span></button>
    </div>
    <button onclick="closeGen(); stopResendTimer();" class="w-full mt-3 text-[12px] text-white/40 hover:text-white/80"><span data-en="← Cancel — go back to Sign In" data-sw="← Ghairi — rudi kwenye Kuingia">← Cancel — go back to Sign In</span></button>
  `);
  startResendTimer(60);
  setTimeout(() => { const el = (__PIN_AVAILABLE && __PIN_KIND === 'phone') ? null : $('otp0'); if (el) el.focus(); }, 50);
}
function switchVerifyMode(mode) {
  const codeOn = mode === 'code';
  $('verifyCodeWrap')?.classList.toggle('hidden', !codeOn);
  $('verifyPinWrap')?.classList.toggle('hidden', codeOn);
  const bc = $('otpModeCode'), bp = $('otpModePin');
  if (bc) bc.className = 'py-2.5 rounded-xl font-bold text-[13px] ' + (codeOn ? 'grad-btn' : 'text-white/60');
  if (bp) bp.className = 'py-2.5 rounded-xl font-bold text-[13px] ' + (!codeOn ? 'grad-btn' : 'text-white/60');
  if (codeOn) $('otp0')?.focus(); else $('loginPin')?.focus();
}
async function doVerifyPIN() {
  const pin = ($('loginPin')?.value || '').replace(/\D/g, '');
  if (pin.length < 4) { toast('Enter your 4–6 digit PIN.', 'err'); return; }
  try {
    const d = await apiPost('api/auth.php', { action: 'verify-login-pin', email: __OTP_EMAIL, pin });
    window.__ME__ = d.user;
    stopResendTimer(); closeGen();
    toast(`Welcome back, ${esc(d.user.name)}! ✓`, 'ok');
    location.href = returnTarget(d.user.role);
  } catch (e) { toast(esc(e.message), 'err'); if ($('loginPin')) $('loginPin').value = ''; }
}
function onOtpInput(i) {
  const v = $('otp'+i).value.replace(/\D/g, '');
  $('otp'+i).value = v.slice(0,1);
  if ($('otp'+i).value && i < 5) { const n = $('otp'+(i+1)); if (n) n.focus(); }
  if (i === 5 && $('otp5').value) setTimeout(doVerifyOTP, 120);
}
function onOtpKey(i, ev) {
  if (ev.key === 'Backspace' && !$('otp'+i).value && i > 0) { const p = $('otp'+(i-1)); if (p) { p.focus(); p.value=''; } }
  if (ev.key === 'Enter') doVerifyOTP();
}
async function doVerifyOTP() {
  const code = [0,1,2,3,4,5].map(i => ($('otp'+i)?.value || '')).join('');
  if (!/^\d{6}$/.test(code)) { toast('Enter all 6 digits of the code.', 'err'); return; }
  try {
    const d = await apiPost('api/auth.php', { action: 'verify-login-otp', email: __OTP_EMAIL, code });
    window.__ME__ = d.user;
    stopResendTimer(); closeGen();
    toast(`Welcome back, ${esc(d.user.name)}! ✓`, 'ok');
    location.href = returnTarget(d.user.role);
  } catch (e) { toast(esc(e.message), 'err'); $('otp0')?.focus(); [0,1,2,3,4,5].forEach(i => { if($('otp'+i)) $('otp'+i).value=''; }); }
}
async function resendLoginCode() {
  if (__RESEND_COUNTDOWN > 0) return;
  if (!__OTP_EMAIL) { toast(T('Enter your email first.', 'Andika barua pepe kwanza.'), 'err'); return; }
  try {
    const d = await apiPost('api/auth.php', { action: 'resend-login-otp', email: __OTP_EMAIL });
    toast(T('A new 6-digit code has been emailed ✓', 'Code mpya ya tarakimu 6 imetumwa kwa barua pepe ✓'), 'ok');
    startResendTimer(60);
    if (d.otp_debug_only && $('genBody')) {
      const hint = document.createElement('div');
      hint.className = 'text-[11px] font-mono mt-2 rounded-lg bg-white/[.03] p-2 border border-white/10';
      hint.innerHTML = `<span class="text-white/50 font-bold">DEV debug</span> — resent OTP: <span class="text-emerald-300 font-bold text-lg">${esc(d.otp_debug_only)}</span>`;
      $('genBody').appendChild(hint);
    }
  } catch (e) { toast(esc(e.message), 'err'); }
}
function openResendCodeFlow() {
  const e = prompt('Enter your account email / Ingiza email yako ya account:', $('loginEmail')?.value?.trim() || '');
  if (!e) return;
  __OTP_EMAIL = e.trim().toLowerCase();
  resendLoginCode().then(() => {
    if (!$('genRoot')?.classList.contains('hidden')) return; // already shown
    openOtpModal(__OTP_EMAIL);
  });
}

function openForgotPassword() {
  openGen('<i class="fa-solid fa-unlock-keyhole mr-1.5 text-violet-300"></i>Forgot Password', `
    <div class="text-[13px] text-white/65"><span data-en="Enter your account email — we'll send a secure link to reset your password. Link expires after 1 hour." data-sw="Ingiza email yako ya account — tutatuma link salama ya kubadili neno la Siri. Link itaisha baada ya saa 1.">Enter your account email — we'll send a secure link to reset your password. Link expires after 1 hour.</span></div>
    <label class="text-[12px] font-bold text-white/60 mt-4 block">EMAIL ADDRESS</label>
    <input id="fpEmail" class="input mt-1.5" value="${esc(($('loginEmail')?.value || '').trim())}" placeholder="you@company.com">
    <div class="flex gap-2 mt-6">
      <button onclick="submitForgotPassword()" class="grad-btn flex-1 py-3.5 rounded-xl font-bold text-sm"><i class="fa-solid fa-paper-plane mr-2"></i><span data-en="Send Reset Link" data-sw="Tuma Link ya Kubadili">Send Reset Link</span></button>
      <button onclick="closeGen()" class="px-5 py-3.5 rounded-xl border border-white/15 font-bold text-sm">Cancel</button>
    </div>
    <div class="text-[11px] text-white/35 mt-4"><i class="fa-solid fa-info-circle mr-1"></i><span data-en="For security, we show this exact confirmation whether or not the email is in the system. If it is registered, you will receive an email within 60 seconds." data-sw="Kwa usalama, tutatoa ushahidi huu kama vile email iko au sivyo. Kwa kweli ikiwa imesajiliwa, utapata email ndani ya sekunde 60.">For security, we show this exact confirmation whether or not the email is in the system. If it is registered, you will receive an email within 60 seconds.</span></div>
  `);
  setTimeout(() => $('fpEmail')?.focus(), 50);
}
async function submitForgotPassword() {
  const e = ($('fpEmail')?.value || '').trim().toLowerCase();
  if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(e)) { toast('Enter a valid email address.', 'err'); return; }
  try {
    await apiPost('api/auth.php', { action: 'forgot-password-init', email: e });
    closeGen();
    toast('If this email is registered, a password reset link has been sent. Check inbox & spam folder ✓', 'ok');
  } catch (err) { toast(esc(err.message), 'err'); }
}

function showLoginTabOnly() {
  if ($('authReset')) $('authReset').classList.add('hidden');
  if ($('authRegister')) $('authRegister').classList.add('hidden');
  if ($('authLogin')) $('authLogin').classList.remove('hidden');
  switchAuthTab('login');
}
function showResetTabOnly() {
  if ($('authLogin')) $('authLogin').classList.add('hidden');
  if ($('authRegister')) $('authRegister').classList.add('hidden');
  if ($('authReset')) $('authReset').classList.remove('hidden');
}
function getUrlParams() {
  try { return Object.fromEntries(new URLSearchParams(location.search)); } catch { return {}; }
}
async function doResetPasswordFromUrl() {
  const { reset_token, email } = getUrlParams();
  const np = $('resetNewPass')?.value || '', cp = $('resetConfirmPass')?.value || '';
  if (np.length < 6) { toast('Password must be 6+ characters.', 'err'); return; }
  if (np !== cp) { toast('Passwords do not match.', 'err'); return; }
  if (!reset_token || !email) { toast(T('Missing reset token or email.', 'Token ya kubadilisha neno la siri haipo.'), 'err'); return; }
  try {
    await apiPost('api/auth.php', { action: 'forgot-password-complete', token: reset_token, email: email, new_password: np });
    toast('Password updated! Sign in with your new password. ✓', 'ok');
    // Strip reset_token from URL without reload, then show login
    const u = new URL(location.href); u.searchParams.delete('reset_token'); u.searchParams.delete('email');
    history.replaceState({}, '', u.toString());
    showLoginTabOnly();
    $('loginEmail').value = email;
    $('loginPass').focus();
  } catch (err) { toast(esc(err.message), 'err'); }
}

async function doLogin() {
  const e = $('loginEmail').value.trim(), p = $('loginPass').value;
  if (!e || !p) { toast('Fill email and password.', 'err'); return; }
  try {
    const d = await apiPost('api/auth.php', { action: 'login', email: e, password: p });
    if (d && d.otp_required) {
      openOtpModal(d.email, d.otp_debug_only ?? null, !!d.pin_available, d.pin_kind || '');
      return;
    }
    if (d && d.user) {
      window.__ME__ = d.user;
      toast(`Welcome back, ${esc(d.user.name)}! ✓`, 'ok');
      location.href = returnTarget(d.user.role);
    }
  } catch (err) { toast(esc(err.message), 'err'); }
}

/* ---------- passkeys (WebAuthn) ----------
 * The device holds a private key that never leaves it. The browser asks for the
 * phone's or laptop's own unlock — fingerprint, face, PIN or pattern — and hands
 * back only a signature, which the server verifies. This site never sees that
 * unlock, so a passkey is both stronger and shorter than password + code. */

function waSupported() {
  return typeof window.PublicKeyCredential !== 'undefined' && !!(navigator.credentials && navigator.credentials.create);
}

/* The browser speaks ArrayBuffer; JSON speaks base64url. These two bridge them. */
function b64uToBuf(s) {
  s = String(s).replace(/-/g, '+').replace(/_/g, '/');
  while (s.length % 4) s += '=';
  const bin = atob(s), out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
  return out.buffer;
}

function bufToB64u(buf) {
  const b = new Uint8Array(buf);
  let s = '';
  for (let i = 0; i < b.length; i++) s += String.fromCharCode(b[i]);
  return btoa(s).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

/* Hide the passkey button on browsers without WebAuthn, or on a page served
 * over plain HTTP where the browser refuses to run the ceremony at all. */
function paintPasskeyLogin() {
  const wrap = $('passkeyLoginWrap');
  if (!wrap) return;
  wrap.classList.toggle('hidden', !waSupported());
}

function passkeyErr(e) {
  const name = e && e.name ? e.name : '';
  if (name === 'NotAllowedError') return LANG === 'sw' ? 'Umeghairi au muda uliisha.' : 'Cancelled or timed out.';
  if (name === 'InvalidStateError') return LANG === 'sw' ? 'Kifaa hiki kimesajiliwa tayari.' : 'This device is already registered.';
  if (name === 'NotSupportedError') return LANG === 'sw' ? 'Kivinjari hiki hakiwezi passkeys. Tumia HTTPS.' : 'This browser cannot use passkeys. Use HTTPS.';
  return esc(e && e.message ? e.message : 'Passkey failed.');
}

async function doPasskeyLogin() {
  if (!waSupported()) { toast(LANG === 'sw' ? 'Kivinjari hiki hakiwezi passkeys.' : 'This browser cannot use passkeys.', 'err'); return; }
  const btn = $('btnPasskeyLogin'), old = btn ? btn.innerHTML : '';
  if (btn) { btn.disabled = true; btn.innerHTML = '<i class="fa-solid fa-circle-notch fa-spin mr-2"></i>' + (LANG === 'sw' ? 'Inasubiri kifaa…' : 'Waiting for your device…'); }
  try {
    const email = ($('loginEmail')?.value || '').trim();
    const begin = await apiPost('api/auth.php', { action: 'passkey-login-begin', email });
    const pk = { challenge: b64uToBuf(begin.challenge), rpId: begin.rpId, timeout: begin.timeout, userVerification: begin.userVerification };
    if (begin.allowCredentials && begin.allowCredentials.length) {
      pk.allowCredentials = begin.allowCredentials.map(c => ({ type: 'public-key', id: b64uToBuf(c.id), transports: c.transports }));
    }
    const cred = await navigator.credentials.get({ publicKey: pk });
    const d = await apiPost('api/auth.php', {
      action: 'passkey-login-finish',
      id: bufToB64u(cred.rawId),
      response: {
        clientDataJSON: bufToB64u(cred.response.clientDataJSON),
        authenticatorData: bufToB64u(cred.response.authenticatorData),
        signature: bufToB64u(cred.response.signature),
      },
    });
    window.__ME__ = d.user;
    toast(LANG === 'sw' ? `Karibu, ${esc(d.user.name)}! ✓` : `Welcome back, ${esc(d.user.name)}! ✓`, 'ok');
    location.href = returnTarget(d.user.role);
  } catch (err) {
    toast(passkeyErr(err), 'err');
    if (btn) { btn.disabled = false; btn.innerHTML = old; }
  }
}

/** Register the current device as a passkey for the signed-in user. */
async function doPasskeyRegister() {
  if (!waSupported()) { toast(LANG === 'sw' ? 'Kivinjari hiki hakiwezi passkeys.' : 'This browser cannot use passkeys.', 'err'); return; }
  const btn = $('btnAddPasskey'), old = btn ? btn.innerHTML : '';
  if (btn) { btn.disabled = true; btn.innerHTML = '<i class="fa-solid fa-circle-notch fa-spin mr-2"></i>' + (LANG === 'sw' ? 'Inasajili…' : 'Registering…'); }
  try {
    const begin = await apiPost('api/auth.php', { action: 'passkey-register-begin' });
    const label = passkeyLabel();
    const cred = await navigator.credentials.create({ publicKey: {
      challenge: b64uToBuf(begin.challenge),
      rp: begin.rp,
      user: { id: b64uToBuf(begin.user.id), name: begin.user.name, displayName: begin.user.displayName },
      pubKeyCredParams: begin.pubKeyCredParams,
      timeout: begin.timeout,
      attestation: begin.attestation,
      authenticatorSelection: begin.authenticatorSelection,
      excludeCredentials: (begin.excludeCredentials || []).map(c => ({ type: 'public-key', id: b64uToBuf(c.id), transports: c.transports })),
    } });
    const transports = (cred.response.getTransports ? cred.response.getTransports() : []);
    await apiPost('api/auth.php', {
      action: 'passkey-register-finish',
      label,
      transports,
      response: {
        clientDataJSON: bufToB64u(cred.response.clientDataJSON),
        attestationObject: bufToB64u(cred.response.attestationObject),
      },
    });
    toast(LANG === 'sw' ? 'Passkey imesajiliwa ✓ Kifaa hiki sasa kinaweza kuingia bila neno la siri.' : 'Passkey added ✓ This device can now sign in without a password.', 'ok');
    loadPasskeys();
  } catch (err) {
    toast(passkeyErr(err), 'err');
  } finally {
    if (btn) { btn.disabled = false; btn.innerHTML = old; }
  }
}

/** A human name for the new passkey, guessed from the platform. */
function passkeyLabel() {
  const ua = navigator.userAgent;
  if (/iPhone/i.test(ua)) return 'iPhone';
  if (/iPad/i.test(ua)) return 'iPad';
  if (/Android/i.test(ua)) return 'Android phone';
  if (/Macintosh/i.test(ua)) return 'Mac';
  if (/Windows/i.test(ua)) return 'Windows PC';
  if (/Linux/i.test(ua)) return 'Linux device';
  return 'Passkey';
}

async function loadPasskeys() {
  const box = $('passkeyList');
  if (!box) return;
  try {
    const d = await apiGet('api/auth.php', { action: 'passkey-list' });
    const rows = d.passkeys || [];
    if (!rows.length) {
      box.innerHTML = `<div class="text-white/40 text-[12.5px]">${LANG === 'sw' ? 'Hakuna passkey bado. Ongeza kifaa chako ili kuingia bila neno la siri.' : 'No passkeys yet. Add your device to sign in without a password.'}</div>`;
      return;
    }
    box.innerHTML = rows.map(p => `
      <div class="flex items-center gap-3 py-2.5 border-b border-white/[.06] last:border-0">
        <i class="fa-solid fa-fingerprint text-cyan-300"></i>
        <div class="flex-1 min-w-0">
          <div class="font-bold text-[13px] truncate">${esc(p.label || 'Passkey')}</div>
          <div class="text-white/40 text-[11px]">${LANG === 'sw' ? 'Imeongezwa' : 'Added'} ${esc((p.created_at || '').slice(0, 10))}${p.last_used_at ? ' • ' + (LANG === 'sw' ? 'ilitumika' : 'last used') + ' ' + esc(p.last_used_at.slice(0, 10)) : ''}</div>
        </div>
        <button onclick="deletePasskey('${esc(p.id)}')" class="text-red-300/70 hover:text-red-300 text-[12px] font-bold px-2 py-1"><i class="fa-solid fa-trash"></i></button>
      </div>`).join('');
  } catch (e) {
    box.innerHTML = `<div class="text-red-300/80 text-[12.5px]">${esc(e.message)}</div>`;
  }
}

async function deletePasskey(id) {
  if (!confirm(LANG === 'sw' ? 'Ondoa passkey hii? Kifaa hicho hakitaweza kuingia kwa passkey tena.' : 'Remove this passkey? That device will no longer sign in with it.')) return;
  try {
    await apiPost('api/auth.php', { action: 'passkey-delete', id });
    toast(LANG === 'sw' ? 'Passkey imeondolewa.' : 'Passkey removed.', 'ok');
    loadPasskeys();
  } catch (e) { toast(esc(e.message), 'err'); }
}

/* Local phone patterns accepted at sign-up: +255XXXXXXXXX or 0XXXXXXXXX.
 * Mirrors mts_phone_valid() in api/auth.php so the two never disagree. */
function validTzPhone(raw) {
  let d = String(raw).replace(/\D/g, '');
  if (d.startsWith('255')) d = '0' + d.slice(3);
  return /^0[67]\d{8}$/.test(d);
}

async function doRegister() {
  const n = $('regName').value.trim(), ph = $('regPhone').value.trim(), e = $('regEmail').value.trim(), c = $('regCompany').value.trim(), p = $('regPass').value, p2 = $('regPass2').value;
  const pin = ($('regPin')?.value || '').replace(/\D/g, ''), pin2 = ($('regPin2')?.value || '').replace(/\D/g, '');
  const sw = LANG === 'sw';
  const say = (en, swTxt) => toast(sw ? swTxt : en, 'err');

  if (!n || !ph || !e) { say('Fill all required fields.', 'Jaza sehemu zote zinazohitajika.'); return; }
  if (!validTzPhone(ph)) { say('Enter a valid Tanzanian phone, e.g. 0712 345 678.', 'Ingiza namba sahihi ya simu ya Tanzania, mf. 0712 345 678.'); return; }
  if (!/^[^@\s]+@[^@\s]+\.[^@\s]+$/.test(e)) { say('Enter a valid email address.', 'Ingiza barua pepe sahihi.'); return; }
  if (p.length < 6) { say('Password must be 6+ characters.', 'Neno la siri liwe na herufi 6 au zaidi.'); return; }
  if (p !== p2) { say('Passwords do not match.', 'Maneno ya siri hayafanani.'); return; }
  if (pin !== '' && (pin.length < 4 || pin.length > 6)) { say('PIN must be 4–6 digits.', 'PIN iwe tarakimu 4–6.'); return; }
  if (pin !== '' && pin2 !== '' && pin !== pin2) { say('PINs do not match.', 'PIN hazifanani.'); return; }
  if ($('regAgree') && !$('regAgree').checked) { say('Please accept the Terms of Service and Privacy Policy.', 'Tafadhali kubali Masharti ya Huduma na Sera ya Faragha.'); return; }

  const btn = document.querySelector('#authRegister .gold-btn');
  const oldHtml = btn ? btn.innerHTML : '';
  if (btn) { btn.disabled = true; btn.innerHTML = '<i class="fa-solid fa-circle-notch fa-spin mr-2"></i>' + (sw ? 'Inafungua akaunti…' : 'Creating your account…'); }
  try {
    const d = await apiPost('api/auth.php', { action: 'register', name: n, phone: ph, email: e, company: c, password: p, password2: p2, pin, pin2 });
    window.__ME__ = d.user;
    toast(sw ? `Karibu, ${esc(n)}! Akaunti imefunguliwa ✓` : `Welcome, ${esc(n)}! Account created ✓`, 'ok');
    location.href = 'client';
  } catch (err) {
    toast(esc(err.message), 'err');
    if (btn) { btn.disabled = false; btn.innerHTML = oldHtml; }
  }
}

async function logout() {
  try { await apiPost('api/auth.php', { action: 'logout' }); } catch (e) { /* ignore */ }
  window.__ME__ = null;
  toast('Signed out securely.', 'info');
  location.href = '/';
}

/* ---------- QUOTE WIZARD ---------- */
let wizStep = 1, wizService = '';
let qTechSel = new Set();

function openQuoteWizard(preset = '') {
  wizStep = 1; qTechSel = new Set(); if (preset) wizService = preset;
  const u = currentUser();
  if (u) { $('qName').value = u.name; $('qEmail').value = u.email; $('qPhone').value = u.phone || ''; $('qCompany').value = u.company || ''; }
  $('qServiceGrid').innerHTML = VISIBLE_SERVICES.map(s => `<button onclick="pickService('${s.t}',this)" class="qsvc text-left px-3.5 py-3 rounded-xl border text-[12.5px] font-bold ${wizService === s.t ? 'tab-active border-transparent' : 'border-white/10 text-white/70 hover:border-cyan-400/40'}"><i class="fa-solid ${s.icon} mr-1.5"></i>${LANG === 'sw' ? (s.tSw || s.t) : s.t}</button>`).join('');
  renderStep3();
  renderServiceFields();
  updateWizardLabels();
  fillRegionChoices();
  paintWiz(); $('quoteModal').classList.remove('hidden'); $('quoteModal').classList.add('flex'); document.body.style.overflow = 'hidden';
}

/* The Tanzania regions, filled once from the backend. The list is not hard-coded
   in the page, so adding a region needs no frontend change. A client outside
   Tanzania simply leaves this blank. */
let REGION_CHOICES = null;
async function fillRegionChoices() {
  const sel = $('qRegion');
  if (!sel) return;
  if (!REGION_CHOICES) {
    try { REGION_CHOICES = (await apiGet('api/ofisi.php', { action: 'regions', country: 'Tanzania' })).regions || []; }
    catch (e) { REGION_CHOICES = []; }
  }
  const cur = sel.value;
  sel.innerHTML = '<option value="">—</option>' + REGION_CHOICES.map(r => `<option value="${esc(r.region)}">${esc(r.region)}</option>`).join('');
  sel.value = cur;
}
/* Each service carries its own request form — fields differ per service. */
const SERVICE_FORMS = {
  'Custom Software Development': [
    { id: 'sfSoftware', label: 'WHAT TYPE? *', type: 'select', options: ['Mobile App (Android/iOS)', 'Desktop App (Windows/macOS/Linux)', 'Web Application', 'Web + Mobile + Desktop', 'Full Custom System'] },
    { id: 'sfUsers', label: 'ESTIMATED USERS / SCALE', type: 'select', options: ['1–10 (small)', '11–100 (growing)', '101–1,000 (business)', '1,000+ (enterprise)'] },
    { id: 'sfIntegr', label: 'INTEGRATIONS NEEDED', type: 'text', ph: 'e.g. M-Pesa, SMS, Google Sheets, ERP…' },
  ],
  'System Analysis & Design': [
    { id: 'sfDoc', label: 'DOCUMENTS REQUIRED *', type: 'select', options: ['SRS Document', 'Database / ERD Design', 'UML Diagrams', 'Full Feasibility Study', 'All of the above'] },
    { id: 'sfScope', label: 'SYSTEM SCOPE', type: 'text', ph: 'What business process should the system cover?' },
  ],
  'IT Consultancy': [
    { id: 'sfArea', label: 'AREA OF CONSULTANCY *', type: 'select', options: ['Digital Strategy', 'System Audit', 'Tech Hiring / Team Advice', 'CTO-as-a-Service', 'Other'] },
    { id: 'sfWhen', label: 'WHEN DO YOU NEED THIS?', type: 'select', options: ['This week', 'This month', 'Next quarter', 'Flexible'] },
  ],
  'Software Maintenance': [
    { id: 'sfPlan', label: 'MAINTENANCE PLAN *', type: 'select', options: ['Starter Care (150K/mo)', 'Business SLA (450K/mo)', 'Enterprise 24/7 (1.2M/mo)', 'One-off fix (not a plan)'] },
    { id: 'sfSystem', label: 'SYSTEM TO MAINTAIN', type: 'text', ph: 'e.g. our pharmacy system built last year' },
    { id: 'sfAccess', label: 'DO YOU HAVE SOURCE CODE / ACCESS?', type: 'select', options: ['Yes, full access', 'Partial', 'No — need takeover'] },
  ],
  'Cybersecurity Services': [
    { id: 'sfSec', label: 'SECURITY SERVICE *', type: 'select', options: ['Security Audit (800K)', 'Penetration Test (2.5M)', 'Managed Security (900K/mo)', 'Malware Removal', 'SSL / Firewall Setup'] },
    { id: 'sfTarget', label: 'TARGET SYSTEM / WEBSITE', type: 'text', ph: 'URL or system name' },
    { id: 'sfUrgency', label: 'IS THIS AN ACTIVE INCIDENT?', type: 'select', options: ['No — preventive', 'Yes — suspected breach', 'Yes — active attack NOW'] },
  ],
  'Digital Forensics': [
    { id: 'sfFrKind', label: 'INVESTIGATION TYPE *', type: 'select', options: ['Cyber-crime / fraud investigation', 'Data breach / incident response', 'Deleted-file & data recovery', 'Employee misconduct (company device)', 'Mobile phone forensics', 'Court-ready expert report'] },
    { id: 'sfFrDevice', label: 'DEVICES / SOURCES *', type: 'text', ph: 'e.g. 1 laptop, 2 phones, server, USB drives' },
    { id: 'sfFrLegal', label: 'LEGAL / DEADLINE CONTEXT', type: 'text', ph: 'e.g. police case, court date, internal HR case (optional)' },
  ],
  'AI Chatbots': [
    { id: 'sfChannel', label: 'BOT CHANNEL *', type: 'select', options: ['Website Chatbot', 'WhatsApp Chatbot', 'Both Website + WhatsApp', 'Facebook / Instagram DM'] },
    { id: 'sfLang', label: 'LANGUAGES', type: 'select', options: ['English only', 'Kiswahili only', 'English + Kiswahili', 'Other'] },
    { id: 'sfBotGoal', label: 'WHAT SHOULD THE BOT DO?', type: 'text', ph: 'e.g. answer FAQs, sell, book appointments, track orders…' },
  ],
  'AI Fine-Tuning': [
    { id: 'sfData', label: 'TRAINING DATA YOU HAVE *', type: 'select', options: ['Price lists', 'Policies / manuals', 'Chats / FAQs', 'Mixed documents', 'Not sure yet'] },
    { id: 'sfModel', label: 'PREFERRED MODEL', type: 'select', options: ['Open-source (Llama/Mistral)', 'Commercial (GPT/Gemini)', 'Advise me'] },
    { id: 'sfDeploy', label: 'WHERE WILL IT RUN?', type: 'select', options: ['Website chatbot', 'Internal staff tool', 'App / API', 'On-premise'] },
  ],
  'Cloud & DevOps': [
    { id: 'sfCloud', label: 'PLATFORM *', type: 'select', options: ['AWS', 'VPS / DigitalOcean', 'Google Cloud', 'Azure', 'Advise me'] },
    { id: 'sfDevops', label: 'WHAT DO YOU NEED?', type: 'select', options: ['Deployment only', 'CI/CD pipeline', 'Docker / Kubernetes', 'Domains + business email', 'Full setup'] },
  ],
  'BRELA & Business Registration': [
    { id: 'sfBrela', label: 'REGISTRATION TYPE *', type: 'select', options: ['Company (Limited)', 'Business Name (Sole)', 'SME Registration', 'TIN / Tax registration', 'Certificate only'] },
    { id: 'sfBrelaName', label: 'PROPOSED BUSINESS NAME(S)', type: 'text', ph: '1–3 name options for search' },
  ],
  'TRA & Tax Services': [
    { id: 'sfTra', label: 'TRA SERVICE *', type: 'select', options: ['TIN Registration', 'File Tax Return', 'VAT Registration', 'PAYE Setup', 'TRA system support'] },
    { id: 'sfTraPeriod', label: 'PERIOD / YEAR', type: 'text', ph: 'e.g. 2025 annual return' },
  ],
  'Computer Troubleshooting': [
    { id: 'sfIssue', label: 'MAIN PROBLEM *', type: 'select', options: ['Slow PC', 'Crashes / errors', 'Malware / virus', 'Data recovery', 'Drivers / hardware', 'Other'] },
    { id: 'sfDevice', label: 'DEVICE & OS', type: 'text', ph: 'e.g. HP laptop, Windows 11' },
    { id: 'sfRemote', label: 'SUPPORT MODE', type: 'select', options: ['Remote (AnyDesk/TeamViewer)', 'Bring device to us', 'On-site visit'] },
  ],
  'Windows & OS Installation': [
    { id: 'sfOs', label: 'OPERATING SYSTEM *', type: 'select', options: ['Windows 10/11', 'Linux (Ubuntu/Fedora)', 'macOS', 'Dual boot', 'Server OS'] },
    { id: 'sfPc', label: 'DEVICE MODEL', type: 'text', ph: 'e.g. Dell Latitude 5520' },
    { id: 'sfKeep', label: 'KEEP EXISTING FILES?', type: 'select', options: ['Yes — backup first', 'No — clean install', 'Not sure'] },
  ],
  'Email & Communication Setup': [
    { id: 'sfEmail', label: 'EMAIL PLATFORM *', type: 'select', options: ['Google Workspace (G-Suite)', 'Zoho Mail', 'cPanel / Web host', 'Microsoft 365', 'Advise me'] },
    { id: 'sfDomain', label: 'DOMAIN NAME', type: 'text', ph: 'e.g. yourcompany.co.tz' },
    { id: 'sfMailJob', label: 'WHAT DO YOU NEED?', type: 'select', options: ['Create new mailboxes', 'Fix SMTP / delivery', 'SPF / DKIM / DMARC', 'Full setup'] },
  ],
  'Government e-Services': [
    { id: 'sfGov', label: 'GOVERNMENT SERVICE *', type: 'select', options: ['NIDA', 'NHIF', 'NSSF', 'Huduma Number', 'eCitizen', 'BRELA / TRA portal', 'Other'] },
    { id: 'sfGovId', label: 'YOUR ID / REFERENCE (optional)', type: 'text', ph: 'Avoid sharing passwords' },
  ],
  'Software Installation': [
    { id: 'sfSoft', label: 'SOFTWARE TO INSTALL *', type: 'select', options: ['MS Office', 'Adobe (Photoshop etc.)', 'IDEs / Dev tools', 'Antivirus', 'Design apps', 'Other / list'] },
    { id: 'sfCount', label: 'HOW MANY DEVICES?', type: 'select', options: ['1 device', '2–5 devices', '6–20 devices', '20+ devices'] },
  ],
};

function serviceFormFields(service) {
  return SERVICE_FORMS[service] || [];
}

/** The catalogue entry for a service, or a safe software default. */
function serviceMeta(service) {
  return SERVICES.find(s => s.t === service) || { kind: 'software' };
}
const isSoftwareService = (service) => serviceMeta(service).kind === 'software';

/**
 * Step 2 asks for a "project title + requirements" for builds, but a
 * done-for-you task (registration, repair, install) has no "project" — it has a
 * subject and a description. Relabel the fields so a service request never
 * borrows software-project wording.
 */
function updateWizardLabels() {
  const sw = LANG === 'sw';
  const soft = isSoftwareService(wizService);
  const set = (id, en, swText) => { const el = $(id); if (el) el.textContent = sw ? swText : en; };
  if (!wizService) return;
  if (soft) {
    set('qTitleLabel', 'PROJECT TITLE *', 'KICHWA CHA MRADI *');
    set('qDescLabel', 'DETAILED REQUIREMENTS *', 'MAHITAJI KWA UNDANI *');
    const t = $('qTitle'), d = $('qDesc');
    if (t) { t.dataset.phEn = 'e.g. Pharmacy stock + sales system with M-Pesa'; t.dataset.phSw = 'mf. Mfumo wa pharmacy wa stock na mauzo na M-Pesa'; t.placeholder = sw ? t.dataset.phSw : t.dataset.phEn; }
    if (d) { d.dataset.phEn = 'Describe features, users, integrations (M-Pesa, SMS), reports you need...'; d.dataset.phSw = 'Eleza vipengele, watumiaji, miunganisho (M-Pesa, SMS), ripoti unazohitaji...'; d.placeholder = sw ? d.dataset.phSw : d.dataset.phEn; }
  } else {
    set('qTitleLabel', 'WHAT DO YOU NEED? *', 'UNAHITAJI NINI? *');
    set('qDescLabel', 'DETAILS *', 'MAELEZO *');
    const t = $('qTitle'), d = $('qDesc');
    if (t) { t.dataset.phEn = 'e.g. Register a limited company named ...'; t.dataset.phSw = 'mf. Nisajiliwe kampuni ya Ltd inayoitwa ...'; t.placeholder = sw ? t.dataset.phSw : t.dataset.phEn; }
    if (d) { d.dataset.phEn = 'Tell us exactly what you need done, and anything we should know.'; d.dataset.phSw = 'Tuambie hasa unachotaka kifanyike, na lolote tunalopaswa kujua.'; d.placeholder = sw ? d.dataset.phSw : d.dataset.phEn; }
  }
}

/* Step 3 (technology/platform) is also per-service — no two services show the same set.
 * Services not about code (BRELA, TRA, Windows, Email, Government, Software Install)
 * show their own deliverable-focused options with NO programming-language picker. */
const SERVICE_STEP3 = {
  'Custom Software Development': { title: 'Technology & Platform', hint: 'Pick preferred languages (optional — our System Analyst will advise the best stack).', techs: ['Python', 'React', 'Rust', 'R', 'HTML/CSS/JS', 'Node.js', 'Flutter', 'PostgreSQL', 'Docker', 'AWS'], plabel: 'PLATFORM', platforms: ['Web Application', 'Mobile App (Android/iOS)', 'Desktop System', 'Web + Mobile', 'Web + Mobile + Desktop', 'Full System (All)'] },
  'System Analysis & Design': { title: 'Documents & Modelling Tools', hint: 'What modelling tools do you already use? (optional)', techs: ['Draw.io', 'Lucidchart', 'Visio', 'StarUML', 'MySQL Workbench', 'Power BI'], plabel: 'DOCUMENT FORMAT', platforms: ['Word / PDF', 'Editable diagrams', 'Both'] },
  'IT Consultancy': { title: 'Engagement Format', hint: 'How should the consultancy be delivered?', techs: [], plabel: 'ENGAGEMENT', platforms: ['One-off assessment', 'Monthly retainer', 'CTO-as-a-service', 'Workshop / training'] },
  'Software Maintenance': { title: 'Support Model', hint: 'What kind of support do you need?', techs: [], plabel: 'SUPPORT CHANNEL', platforms: ['Email only', 'Email + WhatsApp', '24/7 on-call', 'On-site visits'] },
  'Cybersecurity Services': { title: 'Audit Scope, Rules & Tools', hint: 'Any standards you must comply with? (optional)', techs: ['OWASP Top 10', 'ISO 27001', 'PCI-DSS', 'NIST', 'GDPR'], plabel: 'SCOPE', platforms: ['Web app', 'Network', 'Full infrastructure', 'Mail & endpoints'] },
  'Digital Forensics': { title: 'Forensic Scope & Tools', hint: 'Which tools and legal standards apply? (optional)', techs: ['Autopsy', 'FTK Imager', 'Volatility', 'Wireshark', 'Chain of custody', 'Court-ready report'], plabel: 'DELIVERABLE', platforms: ['Evidence acquisition only', 'Full investigation report', 'Expert witness support', 'Incident response'] },
  'AI Chatbots': { title: 'Platform & Model', hint: 'Where should the bot live and which AI model should power it?', techs: ['OpenAI GPT', 'Llama 3', 'Mistral', 'Gemini', 'Whisper STT', 'RAG + Vector DB'], plabel: 'DEPLOY TO', platforms: ['Website widget', 'WhatsApp Business', 'Facebook/Instagram', 'Both Website + WhatsApp'] },
  'AI Fine-Tuning': { title: 'Model & Training Tooling', hint: 'Fine-tuning approach and tools (optional).', techs: ['LoRA', 'QLoRA', 'Full fine-tune', 'RAG', 'HuggingFace', 'PyTorch'], plabel: 'SERVING', platforms: ['Cloud API', 'On-premise', 'Your app / API', 'Advise me'] },
  'Cloud & DevOps': { title: 'Cloud Stack & Tooling', hint: 'Which cloud and tooling do you use or prefer?', techs: ['AWS', 'Docker', 'Kubernetes', 'Terraform', 'GitHub Actions', 'Nginx', 'PostgreSQL'], plabel: 'PLATFORM', platforms: ['AWS', 'Google Cloud', 'Azure', 'VPS / DigitalOcean', 'Advise me'] },
  'BRELA & Business Registration': { title: 'Registration Deliverables', hint: 'Which documents should we obtain for you?', techs: [], plabel: 'SERVICE SPEED', platforms: ['Standard (5–10 working days)', 'Express (2–3 working days)', 'Advise me'] },
  'TRA & Tax Services': { title: 'TRA Filing Scope', hint: 'Which TRA obligations should we handle?', techs: [], plabel: 'FILING MODE', platforms: ['Online (TRA portal)', 'Physical documents', 'Both'] },
  'Computer Troubleshooting': { title: 'Service Channels & Tools', hint: 'How should our technicians reach your device?', techs: ['AnyDesk', 'TeamViewer', 'RustDesk', 'On-site', 'Drop-off'], plabel: 'SESSION TYPE', platforms: ['Remote session', 'On-site visit', 'Bring device to us'] },
  'Windows & OS Installation': { title: 'Licensing & Media', hint: 'How will the OS licence/installer be provided?', techs: [], plabel: 'LICENCE', platforms: ['Client provides licence', 'We provide licence', 'Free / open-source OS', 'Not sure'] },
  'Email & Communication Setup': { title: 'Mail Platform & Security', hint: 'Which mail stack and security standards?', techs: ['SPF', 'DKIM', 'DMARC', 'IMAP/SMTP', 'Microsoft 365', 'Google Workspace'], plabel: 'DELIVERY', platforms: ['Remote configuration', 'On-site', 'Both'] },
  'Government e-Services': { title: 'Government Portal & Documents', hint: 'Which portal(s) and what documents do you have ready?', techs: [], plabel: 'ASSISTANCE MODE', platforms: ['Guided (you apply, we guide)', 'We apply on your behalf', 'Advise me'] },
  'Software Installation': { title: 'Installation Logistics', hint: 'How should we install and license the software?', techs: ['Windows', 'macOS', 'Linux', 'Remote', 'On-site'], plabel: 'LICENSING', platforms: ['Client provides licence', 'We recommend', 'Free/open-source only'] },
};

function renderStep3() {
  const cfg = SERVICE_STEP3[wizService];
  const techWrap = $('qTechWrap'), techGrid = $('qTechGrid'), titleEl = $('wizStep3Title'), hintEl = $('wizStep3Hint');
  const platformSel = $('qPlatform'), plabelEl = $('qPlatformLabel');
  if (!platformSel) return;

  if (!cfg) {
    /* Fallback (should not happen — every service has a config). */
    if (titleEl) titleEl.innerHTML = '<i class="fa-solid fa-code text-emerald-300 mr-2"></i>' + (LANG === 'sw' ? 'Hatua 3: Maelezo' : 'Step 3: Details');
    if (hintEl) hintEl.textContent = LANG === 'sw' ? 'Tuambie jambo lolote lingine muhimu.' : 'Tell us anything else that matters.';
    if (techWrap) techWrap.classList.add('hidden');
    plabelEl.textContent = LANG === 'sw' ? 'JUKWAA' : 'PLATFORM';
    platformSel.innerHTML = (LANG === 'sw' ? ['Sijui'] : ['Not sure']).map(o => `<option>${o}</option>`).join('');
    return;
  }

  if (titleEl) titleEl.innerHTML = `<i class="fa-solid fa-code text-emerald-300 mr-2"></i>${LANG === 'sw' ? 'Hatua 3' : 'Step 3'}: ${esc(LANG === 'sw' && cfg.titleSw ? cfg.titleSw : cfg.title)}`;
  if (hintEl) hintEl.textContent = LANG === 'sw' && cfg.hintSw ? cfg.hintSw : cfg.hint;
  plabelEl.textContent = LANG === 'sw' && cfg.plabelSw ? cfg.plabelSw : cfg.plabel;
  platformSel.innerHTML = cfg.platforms.map(o => `<option>${esc(o)}</option>`).join('');

  if (cfg.techs.length) {
    techWrap.classList.remove('hidden');
    qTechSel = new Set();
    $('qTechLabel').textContent = (LANG === 'sw' && cfg.titleSw ? cfg.titleSw.toUpperCase() : cfg.title.toUpperCase()) + (LANG === 'sw' ? ' — CHAGUA YOYOTE (si lazima)' : ' — SELECT ANY (optional)');
    techGrid.innerHTML = cfg.techs.map(t => `<button onclick="toggleQTech('${t}',this)" class="px-3.5 py-2 rounded-full border border-white/10 text-[12.5px] font-mono hover:border-cyan-400/40 text-white/60">${t}</button>`).join('');
  } else {
    techWrap.classList.add('hidden');
    qTechSel = new Set();
    techGrid.innerHTML = '';
  }
}

function renderServiceFields() {
  const box = $('qServiceFields');
  if (!box) return;
  const fields = serviceFormFields(wizService);
  if (!fields.length) { box.innerHTML = ''; return; }
  box.innerHTML = `
    <div class="rounded-2xl border border-cyan-400/25 bg-cyan-500/[.05] p-4">
      <div class="text-[11px] font-bold tracking-[.16em] text-cyan-300 mb-2"><i class="fa-solid fa-list-check mr-1"></i>${esc(wizService).toUpperCase()} — SPECIFIC DETAILS</div>
      <div class="grid sm:grid-cols-2 gap-3">
        ${fields.map(f => `<div><label class="text-[12px] font-bold text-white/60">${f.label}</label>${
          f.type === 'select'
            ? `<select id="${f.id}" class="input mt-1">${f.options.map(o => `<option>${esc(o)}</option>`).join('')}</select>`
            : `<input id="${f.id}" class="input mt-1" placeholder="${esc(f.ph || '')}">`
        }</div>`).join('')}
      </div>
      <div class="text-[11px] text-white/40 mt-2">Only ask what matters for this service — no generic noise.</div>
    </div>`;
}

function collectServiceFields() {
  const out = {};
  serviceFormFields(wizService).forEach(f => { const el = $(f.id); if (el) out[f.label.replace(' *', '')] = el.value.trim(); });
  return out;
}

function pickService(t, btn) { wizService = t; document.querySelectorAll('.qsvc').forEach(b => { b.classList.remove('tab-active', 'border-transparent'); b.classList.add('border-white/10', 'text-white/70'); }); btn.classList.add('tab-active'); btn.classList.remove('border-white/10', 'text-white/70'); renderStep3(); renderServiceFields(); updateWizardLabels(); }
function toggleQTech(t, btn) { if (qTechSel.has(t)) { qTechSel.delete(t); btn.classList.remove('tab-active'); } else { qTechSel.add(t); btn.classList.add('tab-active'); } }
function closeQuoteWizard() { $('quoteModal').classList.add('hidden'); $('quoteModal').classList.remove('flex'); document.body.style.overflow = ''; }

function paintWiz() {
  document.querySelectorAll('.wizard-step').forEach(s => s.classList.toggle('active', +s.dataset.step === wizStep));
  $('wizProgress').innerHTML = [1, 2, 3, 4, 5].map(i => `<div class="flex-1 h-2 rounded-full ${i <= wizStep ? 'bg-gradient-to-r from-cyan-400 to-violet-500' : 'bg-white/10'}"></div>`).join('');
  $('wizLabel').textContent = LANG === 'sw' ? `Hatua ${wizStep} ya 5` : `Step ${wizStep} of 5`;
  $('wizBack').classList.toggle('invisible', wizStep === 1);
  $('wizNext').innerHTML = wizStep === 5
    ? (LANG === 'sw' ? '<i class="fa-solid fa-paper-plane mr-2"></i>Tuma Ombi' : '<i class="fa-solid fa-paper-plane mr-2"></i>Submit Request')
    : (LANG === 'sw' ? 'Endelea →' : 'Continue →');
  if (wizStep === 5) {
    const sf = collectServiceFields();
    const soft = isSoftwareService(wizService);
    const sfRows = Object.entries(sf).map(([k, v]) => `<div><span class="text-white/45">${esc(k)}:</span> <strong class="text-cyan-200">${esc(v || '—')}</strong></div>`).join('');
    /* A build has a tech stack & platform; a done-for-you task does not. */
    const techRow = soft
      ? `<div><span class="text-white/45">${T('Tech', 'Teknolojia')}:</span> <span class="font-mono text-cyan-200">${[...qTechSel].join(', ') || T('To be advised by analyst', 'Itashauriwa na mchambuzi')}</span> • ${esc($('qPlatform').value)} • ${esc($('qPriority').value)}</div>`
      : `<div><span class="text-white/45">${T('Priority', 'Kipaumbele')}:</span> <strong class="text-cyan-200">${esc($('qPriority').value)}</strong></div>`;
    $('qReview').innerHTML = `
    <div><span class="text-white/45">${T('Name', 'Jina')}:</span> <strong>${esc($('qName').value || '—')}</strong> • ${esc($('qPhone').value || '')} • ${esc($('qEmail').value || '')}</div>
    <div><span class="text-white/45">${T('Service', 'Huduma')}:</span> <strong class="text-cyan-300">${esc(wizService || '—')}</strong></div>
    <div><span class="text-white/45">${soft ? T('Project', 'Mradi') : T('Request', 'Ombi')}:</span> <strong>${esc($('qTitle').value || '—')}</strong></div>
    <div><span class="text-white/45">${T('Details', 'Maelezo')}:</span> ${esc(($('qDesc').value || '—').slice(0, 220))}</div>
    ${sfRows}
    ${techRow}
    <div><span class="text-white/45">${T('Budget', 'Bajeti')}:</span> ${esc($('qBudget').value)} • <span class="text-white/45">${T('Deadline', 'Muda')}:</span> ${esc($('qDeadline').value || T('Flexible', 'Bila sharti'))}</div>`;
  }
}

function wizMove(d) {
  if (d === 1) {
    if (wizStep === 1) {
      if (!$('qName').value.trim() || !$('qPhone').value.trim() || !$('qEmail').value.trim()) { toast('Fill name, phone & email.', 'err'); return; }
      if (!/^[^@\s]+@[^@\s]+\.[^@\s]+$/.test($('qEmail').value.trim())) { toast('Invalid email.', 'err'); return; }
    }
    if (wizStep === 2) {
      if (!wizService) { toast(T('Select a service.', 'Chagua huduma.'), 'err'); return; }
      if (!$('qTitle').value.trim() || $('qDesc').value.trim().length < 15) { toast('Add project title + detailed requirements (15+ chars).', 'err'); return; }
    }
    if (wizStep === 5) { submitRequest(); return; }
    wizStep = Math.min(5, wizStep + 1);
  } else wizStep = Math.max(1, wizStep - 1);
  paintWiz();
}

async function submitRequest() {
  if (!$('qAgree').checked) { toast('Please accept contact consent to continue.', 'err'); return; }
  const btn = $('wizNext');
  const label = btn.innerHTML;
  btn.disabled = true; btn.innerHTML = '<i class="fa-solid fa-circle-notch fa-spin mr-2"></i>Sending…';
  let d;
  try {
    d = await apiPost('api/requests.php', {
      action: 'create',
      name: $('qName').value.trim(), phone: $('qPhone').value.trim(), email: $('qEmail').value.trim(), company: $('qCompany').value.trim(),
      service: wizService, title: $('qTitle').value.trim(), description: $('qDesc').value.trim(),
      tech: [...qTechSel], serviceDetails: collectServiceFields(),
      platform: $('qPlatform').value, budget: $('qBudget').value,
      deadline: $('qDeadline').value, priority: $('qPriority').value,
      // Where the client is, so the growth view counts real work by place. The
      // browser resolves the country; the client names their region.
      country: (window.__PLATFORM__ && window.__PLATFORM__.country) || '',
      region: (($('qRegion') && $('qRegion').value) || '').trim(),
    });
  } catch (err) {
    btn.disabled = false; btn.innerHTML = label;
    toast(esc(err.message), 'err');
    return;
  }
  btn.disabled = false; btn.innerHTML = label;
  if (d.user) window.__ME__ = d.user;
  const tid = d.trackingId;
  closeQuoteWizard();
  openGen(`Request Submitted ✓ — ${tid}`, `
    <div class="text-center py-4">
      <div class="w-20 h-20 mx-auto rounded-full bg-emerald-500/15 border border-emerald-400/40 flex items-center justify-center text-3xl"><i class="fa-solid fa-check text-emerald-300"></i></div>
      <h3 class="font-display font-bold text-2xl mt-4">Asante! Request Received.</h3>
      <p class="text-white/55 text-[14px] mt-2 max-w-md mx-auto">Your Tracking ID is <span class="font-mono font-bold text-cyan-300 text-lg">${tid}</span>. CEO Jackson Mbilinyi will send your official quotation within 24hrs.</p>
      <div class="flex flex-wrap justify-center gap-3 mt-6">
        <button onclick="closeGen();location.href='client#requests'" class="grad-btn px-6 py-3 rounded-xl font-bold text-sm">Open My Portal →</button>
        <a href="https://wa.me/255796752645?text=Hello!%20My%20tracking%20ID%20is%20${tid}" target="_blank" class="px-6 py-3 rounded-xl font-bold text-sm bg-[#25D366]/15 border border-[#25D366]/40 text-[#4be584]"><i class="fa-brands fa-whatsapp mr-1"></i> Confirm on WhatsApp</a>
      </div>
    </div>`);
  toast('Request ' + tid + ' submitted successfully ✓', 'ok');
}

/* ---------- GENERIC MODAL ---------- */
function openGen(title, html) { $('genTitle').innerHTML = title; $('genBody').innerHTML = html; $('genModal').classList.remove('hidden'); $('genModal').classList.add('flex'); document.body.style.overflow = 'hidden'; }
function closeGen() { $('genModal').classList.add('hidden'); $('genModal').classList.remove('flex'); document.body.style.overflow = ''; }

/* ---------- CLIENT PORTAL ---------- */
let cTab = 'overview';

function paintClientHeader() {
  const u = currentUser(); if (!u) return;
  $('clientAvatar').textContent = u.name.split(' ').map(w => w[0]).join('').slice(0, 2).toUpperCase();
  $('clientName').textContent = u.name;
  $('clientEmail').textContent = u.email;
}

async function clientTab(t) {
  const valid = ['overview', 'requests', 'quotes', 'contracts', 'invoices', 'support', 'news'];
  if (!valid.includes(t)) t = 'overview';
  cTab = t;
  document.querySelectorAll('#clientNav .sidebar-link').forEach(b => b.classList.toggle('active', b.dataset.ctab === t));
  const u = currentUser(); if (!u) { showView('auth'); return; }
  const titles = {
    overview: ['Overview', 'Muhtasari'],
    requests: ['My Requests', 'Maombi Yangu'],
    quotes: ['Quotations', 'Nukuu za Bei'],
    contracts: ['Contracts', 'Mikataba'],
    invoices: ['Invoices & Payments', 'Ankara na Malipo'],
    support: ['Support Center', 'Kituo cha Msaada'],
    news: ['News & Updates', 'Habari na Taarifa'],
  };
  const [tEn, tSw] = titles[t] || titles.overview;
  $('clientTabTitle').textContent = LANG === 'sw' ? tSw : tEn;
  $('clientContent').innerHTML = `<div class="text-white/45 text-[13.5px] flex items-center gap-2"><i class="fa-solid fa-circle-notch fa-spin"></i> ${LANG === 'sw' ? 'Inapakia data yako…' : 'Loading your data…'}</div>`;

  let myReq, myQ, myC, myI, myT;
  try {
    [myReq, myQ, myC, myI, myT] = await Promise.all([
      apiGet('api/requests.php', { action: 'mine' }).then(d => d.requests),
      apiGet('api/quotes.php', { action: 'mine' }).then(d => d.quotes),
      apiGet('api/contracts.php', { action: 'mine' }).then(d => d.contracts),
      apiGet('api/invoices.php', { action: 'mine' }).then(d => { window.__MYI__ = d.invoices; return d.invoices; }),
      apiGet('api/tickets.php', { action: 'mine' }).then(d => d.tickets),
    ]);
  } catch (e) { toast(esc(e.message), 'err'); $('clientContent').innerHTML = ''; return; }

  let h = '';
  if (t === 'overview') {
    const pay = await apiGet('api/settings.php', { action: 'list' }).catch(() => null);
    const P = pay && pay.payment ? pay.payment : null;
    if (P) P.instructionsText = LANG === 'sw' ? (P.instructionsSw || P.instructions) : (P.instructionsEn || P.instructions);
    const payCard = P ? `
    <div class="mt-5 rounded-2xl border border-gold/30 overflow-hidden" style="background:linear-gradient(135deg,rgba(245,179,1,.07),rgba(34,211,238,.05))">
      <div class="px-5 py-3.5 border-b border-white/10 flex flex-wrap items-center gap-2 justify-between" style="border-color:rgba(245,179,1,.25)">
        <div class="font-bold text-[14px]"><i class="fa-solid fa-wallet text-gold mr-2"></i><span data-en="Payment Details" data-sw="Maelezo ya Malipo">Payment Details</span></div>
        <span class="badge" style="background:rgba(245,179,1,.15);border:1px solid rgba(245,179,1,.4);color:#F5B301">50% Deposit • 50% Mid-Project</span>
      </div>
      <div class="p-5 grid gap-3 sm:grid-cols-2">
        <div class="rounded-xl bg-white/[.03] border border-white/10 p-4">
          <div class="text-[10.5px] font-bold uppercase tracking-widest text-cyan-300 mb-2"><i class="fa-solid fa-building-columns mr-1.5"></i>${T('Bank Transfer', 'Uhamisho wa Benki')}</div>
          <div class="text-[13px] grid gap-1.5">
            <div class="flex justify-between gap-3"><span class="text-white/50">${T('Bank', 'Benki')}</span><strong>${esc(P.bank.name)}</strong></div>
            <div class="flex justify-between gap-3"><span class="text-white/50">${T('Account', 'Akaunti')}</span><strong class="font-mono text-gold">${esc(P.bank.account)}</strong></div>
            <div class="flex justify-between gap-3"><span class="text-white/50">${T('Name', 'Jina')}</span><strong>${esc(P.bank.holder)}</strong></div>
            ${P.bank.swift ? `<div class="flex justify-between gap-3"><span class="text-white/50">SWIFT</span><strong class="font-mono">${esc(P.bank.swift)}</strong></div>` : ''}
          </div>
        </div>
        <div class="rounded-xl bg-white/[.03] border border-white/10 p-4">
          <div class="text-[10.5px] font-bold uppercase tracking-widest text-emerald-300 mb-2"><i class="fa-solid fa-mobile-screen-button mr-1.5"></i>${T('Mobile Money', 'Pesa kwa Simu')}</div>
          <div class="text-[13px] grid gap-1.5">
            <div class="flex justify-between gap-3"><span class="text-white/50">${T('Network', 'Mtandao')}</span><strong>${esc(P.mobile.network)}</strong></div>
            <div class="flex justify-between gap-3"><span class="text-white/50">${T('Number', 'Namba')}</span><strong class="font-mono text-gold">${esc(P.mobile.number)}</strong></div>
            <div class="flex justify-between gap-3"><span class="text-white/50">${T('Name', 'Jina')}</span><strong>${esc(P.mobile.holder)}</strong></div>
          </div>
        </div>
        <div class="sm:col-span-2 text-[12.5px] text-white/65 rounded-xl bg-black/20 border border-white/10 p-3.5">
          <i class="fa-solid fa-circle-info text-cyan-300 mr-1.5"></i>${esc(P.instructionsText || P.instructions)}
        </div>
      </div>
    </div>` : '';
    h = `<div class="grid sm:grid-cols-4 gap-3">
      ${[['folder-open', T('Requests', 'Maombi'), myReq.length, 'violet'], ['file-invoice-dollar', T('Quotations', 'Nukuu'), myQ.length, 'gold'], ['file-contract', T('Contracts', 'Mikataba'), myC.length, 'emerald'], ['receipt', T('Invoices', 'Ankara'), myI.length, 'blue']].map(([i, l, n, c]) => `<div class="rounded-2xl bg-white/[.03] border border-white/10 p-4 text-center"><i class="fa-solid fa-${i} text-${c === 'gold' ? 'yellow-400' : c + '-300'} text-xl"></i><div class="font-display font-bold text-2xl mt-1">${n}</div><div class="text-[11px] text-white/50 font-bold uppercase tracking-wider">${l}</div></div>`).join('')}
    </div>
    <div class="mt-5 rounded-2xl border border-cyan-400/25 bg-cyan-500/[.05] p-5 flex flex-wrap items-center gap-4"><div class="flex-1 min-w-[220px]"><div class="font-bold">${T('Need a new system, maintenance or security audit?', 'Unahitaji mfumo mpya, matengenezo au ukaguzi wa usalama?')}</div><div class="text-[13px] text-white/55">${T('Submit requirements — get an official priced quotation within 24 hours.', 'Tuma mahitaji yako — upate nukuu rasmi yenye bei ndani ya saa 24.')}</div></div><button onclick="openQuoteWizard()" class="grad-btn px-6 py-3 rounded-xl font-bold text-sm">+ ${T('New Request', 'Ombi Jipya')}</button></div>
    <div class="font-bold mt-6 mb-3">${T('Recent Activity', 'Shughuli za Karibuni')}</div>
    <div class="grid gap-2">${myReq.slice(0, 4).map(r => `<div class="flex items-center gap-3 p-3.5 rounded-xl bg-white/[.02] border border-white/10 text-[13px]"><span class="font-mono text-cyan-300 text-[12px]">${r.trackingId}</span><span class="flex-1 truncate">${esc(r.title)}</span><span class="badge status-${r.status.replace(' ', '')}">${r.status}</span></div>`).join('') || `<div class="text-white/40 text-sm">${T('No requests yet — click New Request above.', 'Hakuna maombi bado — bofya Ombi Jipya juu.')}</div>`}</div>
    ${payCard}
    <div class="font-bold mt-6 mb-3">${T('Security', 'Usalama')}</div>
    <div class="rounded-2xl border border-white/10 bg-white/[.02] p-5">
      <div class="flex flex-wrap items-start gap-4">
        <div class="flex-1 min-w-[240px]">
          <div class="font-bold text-[14px]"><i class="fa-solid fa-fingerprint text-cyan-300 mr-2"></i>${T('Passkeys', 'Passkeys')}</div>
          <div class="text-[12.5px] text-white/55 mt-1">${T('Sign in with your fingerprint, face or screen lock instead of a password and a code. Nothing secret ever leaves your device.', 'Ingia kwa alama ya kidole, uso au lock ya simu badala ya neno la siri na code. Hakuna siri inayoondoka kwenye kifaa chako.')}</div>
        </div>
        <button id="btnAddPasskey" onclick="doPasskeyRegister()" class="grad-btn px-5 py-3 rounded-xl font-bold text-sm"><i class="fa-solid fa-plus mr-1.5"></i>${T('Add this device', 'Ongeza kifaa hiki')}</button>
      </div>
      <div id="passkeyList" class="mt-4"></div>
    </div>`;
  }
  if (t === 'requests') {
    // For each request, fetch the expert serving it and how far along it is —
    // the client asked to watch progress and be satisfied by what they see.
    const workers = {};
    await Promise.all(myReq.map(async r => {
      try { workers[r.id] = await apiGet('api/wafanyakazi.php', { action: 'my_worker', requestId: r.id }); }
      catch (e) { workers[r.id] = null; }
    }));
    h = `<div class="grid gap-3">${myReq.map(r => { const q = myQ.find(x => x.requestId === r.id); const w = workers[r.id] || {}; const prog = Number(w.progress) || 0; return `<div class="rounded-2xl border border-white/10 bg-white/[.02] p-5">
      <div class="flex flex-wrap items-center gap-2 justify-between"><div><span class="font-mono text-cyan-300 text-[12px] font-bold">${r.trackingId}</span><div class="font-bold text-[15px]">${esc(r.title)}</div><div class="text-[12px] text-white/45">${esc(r.service)} • ${fmtDate(r.createdAt)} • ${T('Priority', 'Kipaumbele')}: ${esc(r.priority)}</div></div><span class="badge status-${r.status.replace(' ', '')}">${r.status.toUpperCase()}</span></div>
      <div class="mt-3 rounded-xl border border-cyan-400/20 bg-cyan-500/[.04] p-3">
        <div class="flex items-center justify-between text-[11.5px] font-bold text-white/60 mb-1.5"><span>${T('Progress', 'Maendeleo')}</span><span class="text-cyan-300">${prog}%</span></div>
        <div class="progress-rail"><div class="progress-fill" style="width:${prog}%"></div></div>
        ${w.worker ? `<div class="flex items-center gap-2 mt-2.5">
          ${w.worker.photo ? `<img src="${esc(w.worker.photo)}" class="w-8 h-8 rounded-lg object-cover border border-white/15">` : `<div class="w-8 h-8 rounded-lg grad-btn flex items-center justify-center text-[11px] font-bold">${esc((w.worker.fullName || 'M').slice(0, 2).toUpperCase())}</div>`}
          <div class="text-[12px] text-white/70"><strong class="text-white">${esc(w.worker.fullName)}</strong> — ${esc(w.worker.profession)} <span class="chip ${'text-emerald-300'} ml-1">${esc(w.worker.level)}</span></div>
        </div>` : `<div class="text-[12px] text-white/45 mt-2">${T('An expert will be assigned to you shortly.', 'Mtaalamu atapewa ombi lako hivi karibuni.')}</div>`}
      </div>
      <p class="text-[13px] text-white/60 mt-2">${esc(r.description.slice(0, 180))}...</p>
      ${r.adminNote ? `<div class="mt-2 text-[13px] rounded-xl bg-violet-500/[.08] border border-violet-400/25 p-3"><strong class="text-violet-300">${T('Update from the CEO', 'Taarifa kutoka kwa CEO')}:</strong> ${esc(r.adminNote)}</div>` : ''}
      <div class="flex flex-wrap gap-2 mt-3 text-[12px] font-bold"><button onclick="viewRequestDetail('${r.id}')" class="px-4 py-2 rounded-full bg-white/5 border border-white/10 hover:border-cyan-400/40">${T('View Details', 'Ona Maelezo')}</button>${q ? `<button onclick="clientTab('quotes')" class="px-4 py-2 rounded-full grad-btn">${T('View Quotation', 'Ona Nukuu')} ${fmtTZS(q.total)}</button>` : ''}<a href="https://wa.me/255796752645?text=Hi!%20About%20${r.trackingId}" target="_blank" class="px-4 py-2 rounded-full bg-[#25D366]/10 border border-[#25D366]/30 text-[#4be584]">WhatsApp</a></div>
    </div>`; }).join('') || `<div class="text-white/40">${T('No requests yet.', 'Hakuna maombi bado.')} <button onclick="openQuoteWizard()" class="text-cyan-300 font-bold underline">${T('Submit your first request', 'Tuma ombi lako la kwanza')}</button></div>`}</div>`;
  }
  if (t === 'quotes') {
    h = `<div class="grid gap-3">${myQ.map(q => { const r = myReq.find(x => x.id === q.requestId); return `<div class="rounded-2xl border ${q.status === 'Sent' ? 'border-yellow-400/30' : 'border-white/10'} bg-white/[.02] p-5">
      <div class="flex flex-wrap justify-between gap-2 items-start"><div><div class="font-bold">${q.id} • ${esc(r ? r.title : '')}</div><div class="text-[12px] text-white/45 font-mono">${q.trackingId} • ${T('Valid till', 'Inafaa hadi')} ${fmtDate(q.validUntil)} • ${T('Status', 'Hali')}: <strong class="text-cyan-300">${q.status}</strong></div><span class="badge mt-1.5 inline-block" style="background:rgba(245,179,1,.12);border:1px solid rgba(245,179,1,.4);color:#F5B301"><i class="fa-solid fa-coins mr-1"></i>50/50 Terms</span></div><div class="text-right"><div class="font-display font-bold text-2xl grad-text">${fmtTZS(q.total)}</div><div class="text-[11px] text-white/40">${T('incl. 18% VAT', 'ikiwemo VAT 18%')}</div><div class="text-[11px] text-gold font-bold mt-0.5">Deposit ${fmtTZS(q.total / 2)} + Mid ${fmtTZS(q.total / 2)}</div></div></div>
      <div class="mt-3 rounded-xl border border-gold/30 p-3 text-[12.5px] text-white/70" style="background:rgba(245,179,1,.06)"><strong class="text-gold"><i class="fa-solid fa-circle-info mr-1"></i>${T('Payment Terms', 'Masharti ya Malipo')}:</strong> ${T('<strong class="text-gold">50% deposit is paid before the project starts</strong>, and <strong class="text-gold">50% mid-project when the timeline reaches half</strong> (before delivery). It is not deposit + delivery.', '<strong class="text-gold">50% ya amana hulipwa kabla mradi kuanza</strong>, na <strong class="text-gold">50% katikati ya mradi mradi ukifika nusu</strong> ya ratiba (kabla ya kukabidhi). Si amana + kukabidhi.')}</div>
      ${q.adminMessage ? `<div class="text-[13px] mt-2 p-3 rounded-xl bg-white/[.03] border border-white/10 whitespace-pre-line"><strong>${T('From the CEO', 'Kutoka kwa CEO')}:</strong> ${esc(q.adminMessage)}</div>` : ''}
      <div class="flex flex-wrap gap-2 mt-3"><button onclick="viewQuote('${q.id}')" class="px-4 py-2 rounded-full bg-white/5 border border-white/10 text-[12px] font-bold hover:border-cyan-400/40">${T('View Breakdown', 'Ona Mchanganuo')}</button>
      ${q.status === 'Sent' ? `<button onclick="respondQuote('${q.id}','Approved')" class="px-4 py-2 rounded-full bg-emerald-500/15 border border-emerald-400/40 text-emerald-200 text-[12px] font-bold">✓ ${T('Approve & Generate Contract', 'Kubali na Tengeneza Mkataba')}</button><button onclick="respondQuote('${q.id}','Rejected')" class="px-4 py-2 rounded-full bg-red-500/10 border border-red-400/30 text-red-200 text-[12px] font-bold">${T('Decline', 'Kataa')}</button>` : q.status === 'Approved' ? `<span class="badge status-Approved">${T('APPROVED ✓ — contract generated', 'IMEKUBALIWA ✓ — mkataba umetengenezwa')}</span>` : ''}</div>
    </div>`; }).join('') || `<div class="text-white/40 text-sm">${T('No quotations yet. Your official quotation appears here within 24 hours of your request.', 'Hakuna nukuu bado. Nukuu yako rasmi itaonekana hapa ndani ya saa 24 baada ya ombi lako.')}</div>`}</div>`;
  }
  if (t === 'contracts') {
    h = `<div class="grid gap-3">${myC.map(c => `<div class="rounded-2xl border border-white/10 bg-white/[.02] p-5 flex flex-wrap items-center gap-3 justify-between"><div><div class="font-bold">${c.id} — ${esc(c.projectTitle)}</div><div class="text-[12px] text-white/45 font-mono">${c.trackingId} • ${fmtTZS(c.total)} • ${c.duration}</div><span class="badge ${c.status === 'Signed' ? 'status-Completed' : c.status === 'Client Signed' ? 'status-Approved' : 'status-Quoted'} mt-1 inline-block">${c.status === 'Client Signed' ? T('SIGNED BY YOU — AWAITING CEO', 'UMESAINI — UNASUBIRI CEO') : c.status.toUpperCase()}</span></div><div class="flex gap-2"><button onclick="viewContract('${c.id}')" class="px-4 py-2 rounded-full grad-btn text-[12px] font-bold"><i class="fa-solid fa-file-contract mr-1"></i> ${c.status === 'Signed' ? T('View Signed', 'Ona Iliyosainiwa') : c.status === 'Client Signed' ? T('Track CEO Signature', 'Fuatilia Saini ya CEO') : T('Review & E-Sign', 'Hakiki na Saini')}</button></div></div>`).join('') || `<div class="text-white/40 text-sm">${T('Contracts appear here after you approve a quotation.', 'Mikataba itaonekana hapa baada ya kukubali nukuu.')}</div>`}</div>
    <div class="mt-3 rounded-2xl border border-cyan-400/25 bg-cyan-500/[.05] p-4 text-[12.5px] text-white/70"><strong class="text-white"><i class="fa-solid fa-coins text-gold mr-1"></i>${T('Payment rule (50/50)', 'Kanuni ya malipo (50/50)')}:</strong> ${T('<strong class="text-gold">50% is paid before the project starts</strong>; the remaining half is paid at mid-project, before handover. Your deposit invoice is under', '<strong class="text-gold">50% hulipwa kabla mradi kuanza</strong>; nusu iliyobaki hulipwa katikati ya mradi, kabla ya kukabidhi. Ankara yako ya amana ipo kwenye')} <button onclick="clientTab('invoices')" class="text-cyan-300 underline font-bold">${T('Invoices & Pay', 'Ankara na Malipo')}</button>.</div>`;
  }
  if (t === 'invoices') {
    h = `<div class="grid gap-3">${myI.map(v => `<div class="rounded-2xl border border-white/10 bg-white/[.02] p-5 flex flex-wrap items-center justify-between gap-3"><div><div class="font-bold">${v.id} — ${esc(v.title)}</div><div class="text-[12px] text-white/45">${T('Due', 'Inalipwa')} ${fmtDate(v.dueDate)} • ${v.method}</div><span class="badge ${v.status === 'Paid' ? 'status-Completed' : 'status-Pending'} mt-1 inline-block">${v.status.toUpperCase()}</span></div><div class="text-right"><div class="font-display font-bold text-xl">${fmtMoney(v.amount, v.currency)}</div>${v.status !== 'Paid' ? `<button onclick="payInvoice('${v.id}')" class="mt-2 px-5 py-2.5 rounded-xl gold-btn text-[13px] font-bold text-[#2a1500]"><i class="fa-solid fa-credit-card mr-1"></i> ${T('Pay now (card / mobile money)', 'Lipa sasa (kadi / simu)')}</button>` : `<div class="text-emerald-300 text-[13px] font-bold">✓ ${T('Paid — thank you!', 'Imelipwa — asante!')}</div>`}</div></div>`).join('') || `<div class="text-white/40 text-sm">${T('No invoices yet.', 'Hakuna ankara bado.')}</div>`}</div>
    <div class="mt-4 rounded-2xl border border-cyan-400/25 bg-cyan-500/[.05] p-4 text-[12.5px] text-white/70"><strong class="text-white"><i class="fa-solid fa-coins text-gold mr-1"></i>${T('Payment rule — 50/50', 'Kanuni ya malipo — 50/50')}:</strong> ${T('<strong class="text-gold">Pay 50% (deposit) before the project starts</strong> — work begins officially after the deposit and the CEO signature; the remaining half is paid at mid-project. <strong class="text-white">Payment:</strong> M-Pesa 0796 752 645 (Jackson Mbilinyi) • bank details on the invoice. Send the receipt on WhatsApp after paying.', '<strong class="text-gold">Lipa 50% (amana) kabla mradi kuanza</strong> — kazi huanza rasmi baada ya amana na saini ya CEO; nusu iliyobaki hulipwa katikati ya mradi. <strong class="text-white">Malipo:</strong> M-Pesa 0796 752 645 (Jackson Mbilinyi) • taarifa za benki zipo kwenye ankara. Tuma risiti kwa WhatsApp baada ya kulipa.')}</div>`;
  }
  if (t === 'news') {
    let news = [];
    try { news = (await apiGet('api/news.php', { action: 'list' })).news || []; } catch (e) { news = []; }
    h = `<div class="rounded-2xl border border-cyan-400/25 bg-cyan-500/[.05] p-4 text-[13px] text-white/70 mb-4">
      <i class="fa-solid fa-newspaper text-cyan-300 mr-2"></i><span data-en="Announcements and updates from the Mbilinyi Tech team." data-sw="Matangazo na taarifa kutoka timu ya Mbilinyi Tech.">Announcements and updates from the Mbilinyi Tech team.</span>
    </div>
    <div class="grid md:grid-cols-2 gap-4">
    ${news.map(n => `
      <article class="rounded-2xl border border-white/10 bg-white/[.02] overflow-hidden flex flex-col">
        ${n.imageUrl ? `<div class="h-40 overflow-hidden"><img src="${esc(n.imageUrl)}" class="w-full h-full object-cover" loading="lazy" alt="${esc(n.title)}"></div>` : ''}
        <div class="p-5 flex-1 flex flex-col">
          <div class="flex flex-wrap items-center gap-2"><span class="badge bg-cyan-500/10 text-cyan-300 border border-cyan-400/30">${esc(n.category)}</span><span class="text-[11.5px] text-white/40">${fmtDate(n.createdAt)}</span></div>
          <div class="font-bold text-[15px] mt-2">${esc(n.title)}</div>
          <p class="text-[13px] text-white/55 mt-1.5 leading-relaxed flex-1">${esc(n.excerpt)}</p>
          <button onclick="viewNews('${n.id}')" class="mt-3 text-[13px] font-bold text-cyan-300 hover:text-white text-left">${T('Read more →', 'Soma zaidi →')}</button>
        </div>
      </article>`).join('') || `<div class="md:col-span-2 text-white/40 glass rounded-3xl p-8 text-center">${T('No news yet — check back soon.', 'Hakuna habari bado — angalia tena hivi karibuni.')}</div>`}</div>`;
  }
  if (t === 'support') {
    h = `<div class="flex flex-wrap gap-2"><input id="tkSub" class="input !w-auto flex-1 min-w-[200px]" placeholder="${T('Subject', 'Somo')}"><button onclick="clientNewTicket()" class="grad-btn px-5 py-3 rounded-xl font-bold text-sm">+ ${T('New Ticket', 'Tiketi Mpya')}</button></div>
    <textarea id="tkMsg" rows="2" class="input mt-2" placeholder="${T('Describe your issue...', 'Eleza tatizo lako...')}"></textarea>
    <div class="grid gap-3 mt-4">${myT.map(x => `<div class="rounded-2xl border border-white/10 bg-white/[.02] p-4"><div class="flex justify-between gap-2 flex-wrap"><div class="font-bold text-[14px]">${esc(x.subject)}</div><span class="badge ${x.status === 'Open' ? 'status-Pending' : 'status-Completed'}">${x.status.toUpperCase()}</span></div><div class="text-[13px] text-white/60 mt-1 whitespace-pre-line">${esc(x.message)}</div>${(x.replies || []).map(r => `<div class="mt-2 ml-4 p-3 rounded-xl bg-cyan-500/[.06] border border-cyan-400/20 text-[13px]"><strong class="text-cyan-300">${esc(r.by)}:</strong> ${esc(r.text)}</div>`).join('')}<div class="flex gap-2 mt-2"><input id="rep-${x.id}" class="input !py-2 text-[13px]" placeholder="${T('Reply...', 'Jibu...')}"><button onclick="clientReply('${x.id}')" class="px-4 rounded-xl bg-white/5 border border-white/10 text-[12px] font-bold">${T('Send', 'Tuma')}</button></div></div>`).join('') || `<div class="text-white/40 text-sm">${T('No tickets yet.', 'Hakuna tiketi bado.')}</div>`}</div>`;
  }
  $('clientContent').innerHTML = h;
  if (t === 'overview') loadPasskeys();
}

async function viewRequestDetail(id) {
  let r;
  try { r = (await apiGet('api/requests.php', { action: 'get', id })).request; } catch (e) { toast(esc(e.message), 'err'); return; }
  const sd = r.serviceDetails || {};
  const sdRows = Object.entries(sd).map(([k, v]) => `<div class="flex gap-2"><span class="text-white/40 text-[11px] font-bold uppercase mt-0.5">${esc(k)}</span><span class="text-white/85">${esc(v)}</span></div>`).join('');
  openGen(`Request ${r.trackingId}`, `<div class="grid gap-3 text-[14px]">
    <div class="flex justify-between flex-wrap gap-2"><span class="badge status-${r.status.replace(' ', '')}">${r.status}</span><span class="text-white/40 text-[12px]">${fmtDate(r.createdAt)} • ${esc(r.priority)}</span></div>
    <div><div class="text-[11px] font-bold text-white/40">SERVICE</div><div class="font-bold text-cyan-300">${esc(r.service)}</div></div>
    <div><div class="text-[11px] font-bold text-white/40">PROJECT TITLE</div><div class="font-bold">${esc(r.title)}</div></div>
    <div><div class="text-[11px] font-bold text-white/40">FULL REQUIREMENTS</div><div class="p-4 rounded-xl bg-white/[.03] border border-white/10 whitespace-pre-line text-white/75">${esc(r.description)}</div></div>
    ${sdRows ? `<div><div class="text-[11px] font-bold text-white/40">${esc(r.service).toUpperCase()} — SPECIFIC DETAILS</div><div class="p-4 rounded-xl bg-cyan-500/[.05] border border-cyan-400/25 grid gap-1.5 text-[13px]">${sdRows}</div></div>` : ''}
    <div class="grid sm:grid-cols-3 gap-2 text-[13px]"><div class="p-3 rounded-xl bg-white/[.03] border border-white/10"><div class="text-white/40 text-[11px] font-bold">TECH</div><span class="font-mono">${(r.tech || []).join(', ') || '—'}</span></div><div class="p-3 rounded-xl bg-white/[.03] border border-white/10"><div class="text-white/40 text-[11px] font-bold">BUDGET</div>${esc(r.budget)}</div><div class="p-3 rounded-xl bg-white/[.03] border border-white/10"><div class="text-white/40 text-[11px] font-bold">DEADLINE</div>${esc(r.deadline || 'Flexible')}</div></div>
    ${r.adminNote ? `<div class="p-3 rounded-xl bg-violet-500/[.08] border border-violet-400/25 text-[13px]"><strong class="text-violet-300">${T('Note from the CEO', 'Taarifa kutoka kwa CEO')}:</strong> ${esc(r.adminNote)}</div>` : ''}
  </div>`);
}

async function viewQuote(id) {
  let d;
  try { d = await apiGet('api/quotes.php', { action: 'get', id }); } catch (e) { toast(esc(e.message), 'err'); return; }
  const q = d.quote;
  openGen(`Official Quotation ${q.id}`, `
    <div class="rounded-2xl overflow-hidden border border-white/10">
      <div class="p-5 bg-gradient-to-r from-cyan-600/20 to-violet-600/20 flex justify-between flex-wrap gap-3"><div><div class="font-display font-bold">MBILINYI TECH SOLUTIONS</div><div class="text-[11px] text-white/50">BRELA Reg • TIN 192-147-522 • 0796752645</div></div><div class="text-right text-[12px] text-white/60">Quotation <strong class="text-white font-mono">${q.id}</strong><br>Valid till ${fmtDate(q.validUntil)}</div></div>
      <table class="w-full data text-[13px]"><tr><th>#</th><th>Item</th><th>Qty</th><th class="!text-right">Amount</th></tr>
      ${q.items.map((it, i) => `<tr><td>${i + 1}</td><td>${esc(it.desc)}</td><td>${it.qty}</td><td class="!text-right font-mono">${fmtTZS(it.qty * it.price)}</td></tr>`).join('')}
      </table>
      <div class="p-5 text-right text-[14px] grid gap-1"><div class="text-white/55">Subtotal: <span class="font-mono">${fmtTZS(q.subtotal)}</span></div><div class="text-white/55">VAT 18%: <span class="font-mono">${fmtTZS(q.vat)}</span></div><div class="font-display font-bold text-2xl grad-text">Total: ${fmtTZS(q.total)}</div></div>
      <div class="mx-5 mb-5 rounded-xl border p-3.5 text-[12.5px] leading-relaxed" style="background:rgba(245,179,1,.06);border-color:rgba(245,179,1,.35);color:#c9d4e8"><strong style="color:#F5B301"><i class="fa-solid fa-coins mr-1"></i>PAYMENT TERMS — MASHARTI YA MALIPO (50/50):</strong><br>
      <strong style="color:#F5B301">1. Deposit 50% (${fmtTZS(q.total / 2)}):</strong> inalipwa <strong>kabla mradi kuanza</strong> — mradi hauanzi hadi deposit ilipwe na CEO aweke saini.<br>
      <strong style="color:#F5B301">2. Mid-Project 50% (${fmtTZS(q.total / 2)}):</strong> inalipwa mradi ukifika <strong>nusu ya timeline</strong>, kabla ya delivery ya mwisho.<br>
      <span style="opacity:.8">Si "Deposit + Delivery". Source code na handover hazinapelekwi kabla installment ya 2 haijawekwa. Late payment: 2% kwa wiki.</span></div>
    </div>
    ${q.status === 'Sent' ? `<div class="flex gap-2 mt-4"><button onclick="closeGen();respondQuote('${q.id}','Approved')" class="grad-btn flex-1 py-3 rounded-xl font-bold text-sm">✓ Approve</button><button onclick="closeGen();respondQuote('${q.id}','Rejected')" class="flex-1 py-3 rounded-xl font-bold text-sm border border-red-400/30 text-red-300">Decline</button></div>` : ''}
    <button onclick="printQuote('${q.id}')" class="w-full mt-2 py-3 rounded-xl border border-white/15 font-bold text-sm no-print"><i class="fa-solid fa-print mr-2"></i>Print / Save PDF</button>`);
}

async function respondQuote(id, dec) {
  try {
    await apiPost('api/quotes.php', { action: 'respond', id, decision: dec });
  } catch (e) { toast(esc(e.message), 'err'); return; }
  if (dec === 'Approved') toast('Quotation approved! Contract & 50% deposit invoice generated ✓', 'ok');
  else toast('Quotation declined. The CEO has been notified.', 'info');
  clientTab('quotes');
}

/* ---------- PAYMENTS (gateways + USD/TZS) ----------
   The payer picks a method and the currency they will send. A Tanzanian
   mobile-money gateway takes TZS; Stripe/PayPal take USD. When the two
   currencies differ the amount is converted at the CEO's rate, and the
   gateway's own currency list decides what may be offered. */
let PAY_GW = null, PAY_PICK = null, PAY_CUR = 'TZS', PAY_INVOICE = '', PAY_BASE = 'TZS';

async function loadGateways() {
  if (PAY_GW) return PAY_GW;
  try { PAY_GW = await apiGet('api/payments.php', { action: 'gateways' }); }
  catch (e) { PAY_GW = { gateways: [], currency: 'TZS', fx: { usd_tzs: 0 } }; }
  return PAY_GW;
}

function money(v, cur) {
  const n = Number(v) || 0;
  if (cur === 'USD') return '$' + n.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
  if (cur === 'EUR') return '€' + n.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
  if (cur === 'GBP') return '£' + n.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
  return 'TZS ' + n.toLocaleString('en-US', { maximumFractionDigits: 0 });
}

async function payInvoice(id) {
  let inv = (window.__MYI__ || []).find(x => x.id === id);
  if (!inv) {
    // Reached from a link rather than the invoice list — fetch it.
    try { inv = (await apiGet('api/invoices.php', { action: 'mine' })).invoices.find(x => x.id === id); }
    catch (e) { inv = null; }
  }
  const gw = await loadGateways();
  const base = (inv && inv.currency) || gw.currency || 'TZS';
  const amount = inv ? Number(inv.amount) || 0 : 0;
  const list = (gw.gateways || []).filter(g => g.enabled);
  PAY_PICK = null; PAY_CUR = base; PAY_INVOICE = id; PAY_BASE = base;
  const rate = Number(gw.fx && gw.fx.usd_tzs) || 0;

  if (!list.length) {
    openGen(`Pay ${id}`, `<div class="text-center py-6"><div class="w-16 h-16 mx-auto rounded-2xl bg-amber-500/15 border border-amber-400/30 flex items-center justify-center text-3xl"><i class="fa-solid fa-hourglass-half text-amber-300"></i></div>
      <div class="font-bold text-lg mt-3">No online method is switched on yet</div>
      <p class="text-white/55 text-[13px] mt-2">The CEO has not yet enabled a payment gateway. Pay by M-Pesa <strong class="font-mono text-white">0796 752 645</strong> (Jackson Mbilinyi) and confirm below, or contact us.</p>
      <input id="payRef" class="input mt-4 font-mono" placeholder="Transaction code (e.g. TX98412...)">
      <button onclick="confirmPay('${id}')" class="gold-btn w-full mt-3 py-3.5 rounded-xl font-bold text-[#2a1500]">I Have Paid — Confirm</button></div>`);
    return;
  }

  openGen(`Pay ${id}`, `
    <div class="rounded-2xl border border-cyan-400/25 bg-cyan-500/[.05] p-4 mb-4">
      <div class="flex flex-wrap items-center justify-between gap-2">
        <div><div class="text-[11px] font-bold tracking-wider text-white/50 uppercase">Amount due</div>
        <div id="payAmount" class="font-display font-bold text-2xl grad-text">${money(amount, base)}</div></div>
        <div class="text-right"><div class="text-[11px] font-bold tracking-wider text-white/50 uppercase">Invoice</div>
        <div class="font-mono text-[13px] text-cyan-300">${esc(id)}</div></div>
      </div>
      <div id="payConvert" class="text-[12px] text-white/60 mt-2"></div>
    </div>
    <label class="text-[12px] font-bold text-white/60">1. CHOOSE THE CURRENCY YOU WILL PAY IN</label>
    <div id="payCurRow" class="flex flex-wrap gap-2 mt-1.5">${payCurrencyChoices(base, list).map(c => `<button onclick="pickPayCurrency('${c}')" data-cur="${c}" class="paycur px-5 py-2.5 rounded-xl border font-bold text-[13px] ${c === base ? 'tab-active border-transparent' : 'border-white/10 text-white/70 hover:border-cyan-400/40'}">${c === 'USD' ? '<i class="fa-solid fa-dollar-sign mr-1"></i>' : ''}${c}</button>`).join('')}</div>
    ${rate > 0 ? '' : `<div class="text-[12px] text-amber-300 mt-2"><i class="fa-solid fa-triangle-exclamation mr-1"></i>No exchange rate is set — paying in a currency other than ${base} is blocked until the CEO sets it.</div>`}
    <label class="text-[12px] font-bold text-white/60 mt-4 block">2. CHOOSE HOW YOU WILL PAY</label>
    <div id="payGwRow" class="grid gap-2 mt-1.5"></div>
    <div id="payStage" class="mt-4"></div>
    <div id="payManual" class="mt-4 hidden"><input id="payRef" class="input font-mono" placeholder="Transaction code (e.g. TX98412...)"><button onclick="confirmPay('${id}')" class="gold-btn w-full mt-3 py-3.5 rounded-xl font-bold text-[#2a1500]">I Have Paid — Confirm</button></div>
  `);
  renderPayGateways(amount, base);
}

/* Only offer currencies that at least one switched-on gateway accepts. The
   invoice's own currency always stays, so the client can never reach a dead end
   where every choice is refused. */
function payCurrencyChoices(base, enabledGateways) {
  const accepted = new Set();
  (enabledGateways || []).forEach(g => (g.currencies || []).forEach(c => accepted.add(c)));
  const out = [base];
  ['USD', 'TZS', 'EUR', 'GBP'].forEach(c => { if (c !== base && accepted.has(c)) out.push(c); });
  return out;
}

function pickPayCurrency(cur) {
  PAY_CUR = cur;
  document.querySelectorAll('.paycur').forEach(b => {
    const on = b.dataset.cur === cur;
    b.classList.toggle('tab-active', on); b.classList.toggle('border-transparent', on);
    b.classList.toggle('border-white/10', !on); b.classList.toggle('text-white/70', !on);
  });
  renderPayGateways(Number(PAY_GW && PAY_GW.lastAmount) || 0, PAY_BASE);
}

function renderPayGateways(amount, base) {
  PAY_GW.lastAmount = amount;
  const rate = Number(PAY_GW.fx && PAY_GW.fx.usd_tzs) || 0;
  const show = c => c === base ? amount : (rate > 0 ? Math.round((c === 'USD' ? amount / rate : amount * rate) * 100) / 100 : null);
  const conv = show(PAY_CUR);
  const convEl = $('payConvert');
  if (convEl) {
    convEl.innerHTML = PAY_CUR === base
      ? `Charged in ${base}.`
      : (conv == null ? `<span class="text-amber-300">Cannot convert ${base} → ${PAY_CUR} without a rate.</span>`
        : `You will be charged <strong class="text-white">${money(conv, PAY_CUR)}</strong> at 1 USD = ${rate.toLocaleString()} TZS.`);
  }
  const amountEl = $('payAmount');
  if (amountEl) amountEl.textContent = conv == null ? money(amount, base) : money(conv, PAY_CUR);

  const row = $('payGwRow');
  if (!row) return;
  const allowed = (PAY_GW.gateways || []).filter(g => g.enabled && (g.currencies || []).includes(PAY_CUR));
  if (!allowed.length) {
    row.innerHTML = `<div class="text-[12.5px] text-amber-300 rounded-xl border border-amber-400/25 bg-amber-500/[.05] p-3"><i class="fa-solid fa-circle-info mr-1"></i>No enabled method accepts ${PAY_CUR}. ${PAY_CUR === 'USD' ? 'Turn on Stripe or PayPal in Admin → Settings, or pay in TZS.' : 'Choose USD, or pay by bank transfer.'}</div>`;
    return;
  }
  row.innerHTML = allowed.map(g => `<button onclick="pickGateway('${g.key}','${esc(g.name)}')" class="paygw flex items-center gap-3 p-3.5 rounded-xl border border-white/10 hover:border-cyan-400/50 hover:bg-white/5 text-left" data-key="${g.key}">
    <span class="w-10 h-10 rounded-xl flex items-center justify-center text-lg" style="background:${g.brand}22;color:${g.brand}"><i class="${g.icon}"></i></span>
    <span class="flex-1"><span class="font-bold text-[13.5px] block">${esc(g.name)}</span>
    <span class="text-[11.5px] text-white/50">${esc(g.kind)} • ${(g.currencies || []).join(', ')}</span></span>
    ${g.configured ? '<i class="fa-solid fa-circle-check text-emerald-300"></i>' : '<span class="chip text-amber-300">setup needed</span>'}
  </button>`).join('');
}

async function pickGateway(key, name) {
  document.querySelectorAll('.paygw').forEach(b => b.classList.toggle('tab-active', b.dataset.key === key));
  PAY_PICK = key;
  const stage = $('payStage');
  const g = (PAY_GW.gateways || []).find(x => x.key === key) || {};
  if (!g.configured) {
    stage.innerHTML = `<div class="rounded-xl border border-amber-400/25 bg-amber-500/[.05] p-3 text-[12.5px] text-white/70"><i class="fa-solid fa-key text-amber-300 mr-1"></i><strong>${esc(name)}</strong> is enabled but its credentials are not filled in. The CEO must add them in Admin → Settings before money can move.</div>`;
    return;
  }
  stage.innerHTML = `<div class="text-[12.5px] text-white/60"><i class="fa-solid fa-circle-notch fa-spin text-cyan-300 mr-1.5"></i>Starting ${esc(name)}…</div>`;
  try {
    const r = await apiPost('api/payments.php', {
      action: 'pay', invoiceId: PAY_INVOICE, gateway: key, currency: PAY_CUR,
    });
    const code = r.verifyCode || '';
    if (r.redirect) {
      stage.innerHTML = `<div class="rounded-xl border border-cyan-400/30 bg-cyan-500/[.06] p-4">
        <div class="font-bold text-[13.5px]">Complete the payment</div>
        <p class="text-[12.5px] text-white/65 mt-1.5">${esc(r.instructions || 'You will be taken to the secure payment page.')}</p>
        <a href="${esc(r.redirect)}" target="_blank" rel="noopener" class="grad-btn inline-block mt-3 px-5 py-3 rounded-xl font-bold text-sm"><i class="fa-solid fa-arrow-up-right-from-square mr-1"></i>Open Secure Payment Page</a>
        <button onclick="verifyPayment('${r.paymentId}')" class="ml-2 px-5 py-3 rounded-xl border border-white/15 font-bold text-sm">I Have Paid — Check</button>
        ${code ? `<div class="text-[11.5px] text-white/45 mt-3">Reference <span class="font-mono text-cyan-300">${esc(r.reference || '')}</span> • Verification code <span class="font-mono text-white">${esc(code)}</span></div>` : ''}
      </div>`;
    } else {
      stage.innerHTML = `<div class="rounded-xl border border-emerald-400/30 bg-emerald-500/[.06] p-4">
        <div class="font-bold text-[13.5px]"><i class="fa-solid fa-circle-info text-emerald-300 mr-1"></i>Payment instructions</div>
        <div class="text-[12.5px] text-white/75 mt-2 whitespace-pre-line">${esc(r.instructions || '')}</div>
        ${code ? `<div class="text-[11.5px] text-white/45 mt-2">Verification code <span class="font-mono text-white">${esc(code)}</span></div>` : ''}
        <button onclick="verifyPayment('${r.paymentId}')" class="grad-btn mt-3 px-5 py-3 rounded-xl font-bold text-sm">I Have Paid — Check</button>
      </div>`;
    }
  } catch (e) {
    stage.innerHTML = `<div class="rounded-xl border border-red-400/30 bg-red-500/[.06] p-3 text-[12.5px] text-red-200"><i class="fa-solid fa-circle-xmark mr-1"></i>${esc(e.message)}</div>`;
  }
}

async function verifyPayment(paymentId) {
  try {
    const r = await apiPost('api/payments.php', { action: 'verify', paymentId });
    if (r.settled) { closeGen(); clientTab('invoices'); toast('Payment confirmed — asante sana ✓', 'ok'); return; }
    toast(r.message || 'Not settled yet. Try again in a moment.', 'info');
  } catch (e) { toast(esc(e.message), 'err'); }
}

async function confirmPay(id) {
  const ref = ($('payRef')?.value || '').trim();
  if (ref.length < 4) { toast('Enter transaction reference code.', 'err'); return; }
  try {
    await apiPost('api/invoices.php', { action: 'pay', id, ref });
  } catch (e) { toast(esc(e.message), 'err'); return; }
  closeGen(); clientTab('invoices');
  toast('Payment recorded! Receipt sent. Asante sana ✓', 'ok');
}

async function clientNewTicket() {
  const u = currentUser(); const s = ($('tkSub')?.value || '').trim(), m = ($('tkMsg')?.value || '').trim();
  if (!s || !m) { toast('Add subject + message.', 'err'); return; }
  try {
    await apiPost('api/tickets.php', { action: 'create', name: u.name, email: u.email, subject: s, message: m });
  } catch (e) { toast(esc(e.message), 'err'); return; }
  clientTab('support');
  toast('Ticket opened — we respond within hours ✓', 'ok');
}

async function clientReply(id) {
  const v = ($('rep-' + id)?.value || '').trim();
  if (!v) return;
  try { await apiPost('api/tickets.php', { action: 'reply', id, text: v }); } catch (e) { toast(esc(e.message), 'err'); return; }
  clientTab('support');
}

/* ---------- CONTRACTS (shared) ----------
   Flow: client e-signs FIRST → CEO/admin countersigns SECOND → both signed = Signed. */
function contractStatusChip(c) {
  if (c.status === 'Signed') return '<span class="inline-block mt-1 px-2 py-0.5 rounded text-[11px] font-bold bg-emerald-100 text-emerald-700">FULLY SIGNED ✓</span>';
  if (c.status === 'Client Signed') return '<span class="inline-block mt-1 px-2 py-0.5 rounded text-[11px] font-bold bg-blue-100 text-blue-700">CLIENT SIGNED — CEO COUNTERSIGNATURE PENDING</span>';
  return '<span class="inline-block mt-1 px-2 py-0.5 rounded text-[11px] font-bold bg-amber-100 text-amber-700">AWAITING CLIENT SIGNATURE</span>';
}
function adminSigBlock(c) {
  if (c.adminSig && c.adminSig.indexOf('data:image') === 0) {
    return `<div class="mt-1"><img src="${c.adminSig}" class="h-14 border-b"><div class="text-[11px] text-emerald-600 font-bold">Countersigned ✓ ${fmtDate(c.adminSignedAt || c.createdAt)}</div></div>`;
  }
  if (c.status === 'Signed') {
    return '<div class="font-bold italic text-lg mt-1" style="font-family:cursive">Jackson Mbilinyi</div><div class="text-[11px] text-slate-500">Signed & stamped ✓</div>';
  }
  if (c.status === 'Client Signed') {
    return '<div class="text-[12px] text-blue-600 font-bold mt-2">Client signed — CEO saini inasubiri hapa</div>';
  }
  return '<div class="text-[12px] text-slate-400 mt-2">Inasubiri saini ya mteja kwanza</div>';
}
function clientSigBlock(c) {
  if (c.clientSig && c.clientSig.indexOf('data:image') === 0) {
    return `<div class="mt-1"><img src="${c.clientSig}" class="h-14 border-b"><div class="text-[11px] text-emerald-600 font-bold">E-signed ✓ ${fmtDate(c.clientSignedAt || c.createdAt)}</div></div>`;
  }
  return '<div class="text-[12px] text-amber-600 font-bold mt-2">Awaiting client e-signature below</div>';
}
function contractHTML(c) {
  const total = Number(c.total) || 0;
  const deposit = Math.round(total * 0.5);
  const installment = Math.round(total * 0.5);
  return `<div class="rounded-2xl overflow-hidden border border-white/10 bg-white text-slate-800" id="contractDoc">
    <div class="p-6 border-b-4 border-cyan-500 flex justify-between flex-wrap gap-3">
      <div><div class="font-bold text-lg tracking-tight">MBILINYI TECH SOLUTIONS</div><div class="text-[11px] text-slate-500">BRELA Registered (706668) • TIN 192-147-522 • Dodoma, Tanzania<br>0796752645 • mbilinyitech@gmail.com • @mbilinyitech</div></div>
      <div class="text-right text-[12px]"><div class="font-bold">SERVICE CONTRACT ${c.id}</div><div>Tracking: ${c.trackingId}</div><div>Date: ${fmtDate(c.createdAt)}</div>${contractStatusChip(c)}</div>
    </div>
    <div class="p-6 text-[13px] leading-relaxed grid gap-3">
      <p><strong>PARTIES:</strong> (1) <strong>Mbilinyi Tech Solutions</strong>, represented by CEO <strong>Jackson Mbilinyi</strong> (Service Provider) and (2) <strong>${esc(c.clientName)}</strong> (Client).</p>
      <p><strong>1. PROJECT:</strong> ${esc(c.projectTitle)}. Scope: ${esc(c.scope)}</p>
      <p><strong>2. CONTRACT VALUE:</strong> <strong>${fmtTZS(total)}</strong> (inclusive of 18% VAT where applicable). Payable via M-Pesa 0796752645 or bank transfer per the strict 50/50 schedule below.</p>
      <div class="rounded-2xl border-2 border-cyan-400/40 bg-cyan-50 p-5 grid gap-3">
        <div class="text-center"><strong class="text-cyan-700 uppercase tracking-widest text-[11px]">MASHARTI YA MALIPO — 50 / 50 PAYMENT TERMS</strong></div>
        <div class="grid sm:grid-cols-2 gap-3 text-[13px]">
          <div class="rounded-xl border border-cyan-200 bg-white p-4">
            <div class="text-[10px] font-bold text-cyan-600 uppercase tracking-wider">① INSTALLMENT 1 — BEFORE START (KABLA YA KUANZA)</div>
            <div class="font-display font-bold text-2xl grad-text mt-1">${fmtTZS(deposit)}</div>
            <div class="text-[11px] text-slate-500 mt-1">50% of total contract value</div>
            <div class="text-[11.5px] text-slate-600 mt-2 leading-relaxed"><strong>Deposit Rules:</strong> Mradi <u>HAUTAANZISHI</u> hadi hela hii iwe lipewe na <u>ku-thibitishwa na CEO Jackson Mbilinyi</u>. Invoice hutoa kiotomatiki mteja akikubali Quote. Hii ndio saini ya ku-enable kusainiwa kwa CEO kwenye mkataba.</div>
          </div>
          <div class="rounded-xl border border-violet-200 bg-white p-4">
            <div class="text-[10px] font-bold text-violet-600 uppercase tracking-wider">② INSTALLMENT 2 — MID-PROJECT (KATI YA MRADI)</div>
            <div class="font-display font-bold text-2xl grad-text mt-1">${fmtTZS(installment)}</div>
            <div class="text-[11px] text-slate-500 mt-1">50% of total contract value</div>
            <div class="text-[11.5px] text-slate-600 mt-2 leading-relaxed"><strong>Mid-Project Rules:</strong> Inalipwa <u>wakati timeline imekamilika 50%</u> — kabla ya final handover, test na training. Final acceptance, source code delivery na handover zinafanyika baada ya installment hii kukamilika.</div>
          </div>
        </div>
        <div class="text-[11.5px] text-slate-600 leading-relaxed bg-white/60 rounded-lg p-3 border border-slate-200"><strong>Si "50/50 — Deposit + Delivery."</strong> Hii ni 50% kabla ya kuanza + 50% kati ya mradi (mid-project). <u>Final delivery na source code hazinapelekwi kabla installment ya 2 haijawekwa.</u> Timelines zinaongezwa kwa muda wa malipo. Late payments inavuta penalty ya 2% kwa wiki.</div>
      </div>
      <p><strong>3. DURATION:</strong> ${esc(c.duration)} from signing & deposit confirmation. Delays caused by client (content, feedback, approvals, assets) extend timelines fairly on written notice.</p>
      <p><strong>4. PROVIDER OBLIGATIONS:</strong> professional development, testing, deployment, training, documentation, 6-month defect warranty, data confidentiality & security best practice (OWASP).</p>
      <p><strong>5. CLIENT OBLIGATIONS:</strong> timely requirements, content, feedback (within 5 working days per milestone) and strict adherence to the 50/50 payment schedule above.</p>
      <p><strong>6. OWNERSHIP:</strong> full source code & rights transfer to client upon final payment of Installment 2. Provider may showcase non-confidential work in portfolio.</p>
      <p><strong>7. CONFIDENTIALITY & DATA:</strong> both parties protect shared data; provider complies with Tanzania data protection laws.</p>
      <p><strong>8. TERMINATION:</strong> either party may terminate with 14-day written notice; completed non-refundable milestones remain payable. Deposit is non-refundable after work starts.</p>
      <p><strong>9. GOVERNING LAW:</strong> Laws of the United Republic of Tanzania. Disputes first via mediation in Dodoma.</p>
      <div class="grid sm:grid-cols-2 gap-4 mt-2">
        <div class="border rounded-xl p-4"><div class="text-[11px] font-bold text-slate-500">FOR CLIENT — ${esc(c.clientName)} (saini kwanza)</div>${clientSigBlock(c)}</div>
        <div class="border rounded-xl p-4"><div class="text-[11px] font-bold text-slate-500">FOR PROVIDER — Jackson Mbilinyi, CEO (baada ya mteja)</div>${adminSigBlock(c)}</div>
      </div>
    </div></div>`;
}

async function viewContract(id) {
  let c;
  try { c = (await apiGet('api/contracts.php', { action: 'get', id })).contract; } catch (e) { toast(esc(e.message), 'err'); return; }
  const me = currentUser(); const isAdmin = me?.role === 'admin';

  let signPanel = '';
  if (c.status === 'Sent' && !isAdmin) {
    signPanel = `<div class="rounded-2xl border border-cyan-400/30 bg-cyan-500/[.05] p-4"><div class="font-bold text-[14px]"><i class="fa-solid fa-pen-nib text-cyan-300 mr-2"></i>Sign with your finger / mouse (wewe ndio unasaini kwanza):</div><canvas id="sigPad" width="600" height="160" class="sig-pad w-full h-36 rounded-xl bg-white mt-2"></canvas><div class="flex gap-2 mt-2"><button onclick="clearSig()" class="px-4 py-2 rounded-xl border border-white/15 text-[12px] font-bold">Clear</button><button onclick="saveSig('${c.id}','client')" class="grad-btn flex-1 py-2.5 rounded-xl text-[13px] font-bold">✓ E-Sign Contract</button></div></div>`;
  } else if (c.status === 'Sent' && isAdmin) {
    signPanel = `<div class="rounded-2xl border border-amber-400/30 bg-amber-500/[.07] p-4 text-[13px] text-amber-100"><i class="fa-solid fa-hourglass-half text-amber-300 mr-2"></i><strong>Inasubiri saini ya mteja (${esc(c.clientName)}).</strong> Mteja anasaini kwanza, kisha ndipo wewe (CEO) utasaini. Bei/scope bado unaweza kubadilisha kabla ya mteja kusaini.<div class="mt-3"><button onclick="openContractEditor('${c.id}')" class="px-4 py-2 rounded-xl grad-btn text-[12px] font-bold"><i class="fa-solid fa-pen mr-1"></i>Edit Bei / Scope / Muda</button></div></div>`;
  } else if (c.status === 'Client Signed' && isAdmin) {
    let dep = { paid: true, invoiceId: null, message: '' };
    try { dep = (await apiGet('api/contracts.php', { action: 'deposit_info', id: c.id })).deposit; } catch (e) { /* server nayo inazuia */ }
    if (dep.paid) {
      signPanel = `<div class="rounded-2xl border border-blue-400/30 bg-blue-500/[.07] p-4"><div class="font-bold text-[14px] text-blue-200"><i class="fa-solid fa-badge-check mr-2"></i>Mteja amesaini na deposit ya 50% imelipwa ✓. Sasa ni zamu yako (CEO) kusaini — project inaanza rasmi:</div><canvas id="sigPad" width="600" height="160" class="sig-pad w-full h-36 rounded-xl bg-white mt-2"></canvas><div class="flex gap-2 mt-2"><button onclick="clearSig()" class="px-4 py-2 rounded-xl border border-white/15 text-[12px] font-bold">Clear</button><button onclick="saveSig('${c.id}','admin')" class="grad-btn flex-1 py-2.5 rounded-xl text-[13px] font-bold">✓ CEO Countersign</button></div></div>`;
    } else {
      signPanel = `<div class="rounded-2xl border border-amber-400/30 bg-amber-500/[.07] p-4 text-[13px] text-amber-100"><i class="fa-solid fa-hourglass-half text-amber-300 mr-2"></i><strong>Mteja amesaini, lakini project HAWEZI kuanza bila 50% deposit.</strong><div class="mt-1">${esc(dep.message || 'Lipa 50% kwanza kabla project kuanza.')}</div><div class="mt-3 flex flex-wrap gap-2">${dep.invoiceId ? `<button onclick="markPaid('${dep.invoiceId}')" class="px-4 py-2 rounded-xl gold-btn text-[12px] font-bold text-[#2a1500]"><i class="fa-solid fa-check mr-1"></i>Mark Deposit Paid</button>` : `<button onclick="createDepositInvoice('${c.id}')" class="px-4 py-2 rounded-xl grad-btn text-[12px] font-bold"><i class="fa-solid fa-file-invoice-dollar mr-1"></i>Create 50% Deposit Invoice</button>`}</div><div class="text-[11.5px] text-amber-200/80 mt-2">Kanuni ya malipo: 50% kabla project kuanza, nusu iliyobaki baada ya kukabidhi.</div></div>`;
    }
  } else if (c.status === 'Client Signed' && !isAdmin) {
    signPanel = `<div class="rounded-2xl border border-blue-400/30 bg-blue-500/[.07] p-4 text-[13px] text-blue-100"><i class="fa-solid fa-circle-check text-blue-300 mr-2"></i><strong>Umesomeza saini.</strong> CEO Jackson Mbilinyi sasa anasaini, kisha mradi utaanza rasmi. <span class="text-amber-200 font-bold">Kumbuka: lipa 50% deposit (Invoices & Pay) — mradi unanza baada ya deposit.</span></div>`;
  } else if (c.status === 'Signed') {
    signPanel = `<div class="rounded-2xl border border-emerald-400/30 bg-emerald-500/[.07] p-4 text-[13px] text-emerald-100"><i class="fa-solid fa-shield-halved text-emerald-300 mr-2"></i><strong>Mkataba umesainiwa na pande zote mbili ✓</strong> — umetunzwa kwenye mfumo (Client: ${fmtDate(c.clientSignedAt)} • CEO: ${fmtDate(c.adminSignedAt)}).</div>`;
  }

  openGen(`Service Contract ${c.id}`, contractHTML(c) + `
    <div class="no-print mt-4 grid gap-3">
      ${signPanel}
      <div class="flex gap-2"><button onclick="printContract('${c.id}')" class="flex-1 py-3 rounded-xl border border-white/15 font-bold text-sm"><i class="fa-solid fa-print mr-2"></i>Print / PDF</button><a href="https://wa.me/255796752645?text=Contract%20${c.id}%20reviewed" target="_blank" class="flex-1 text-center py-3 rounded-xl bg-[#25D366]/10 border border-[#25D366]/30 text-[#4be584] font-bold text-sm">Discuss on WhatsApp</a></div>
    </div>`);
  initSigPad();
}

/* admin edits price / scope / duration while the client has not signed yet */
async function openContractEditor(id) {
  let c;
  try { c = (await apiGet('api/contracts.php', { action: 'get', id })).contract; } catch (e) { toast(esc(e.message), 'err'); return; }
  if (c.status !== 'Sent') { toast('Contract is locked — the client already signed.', 'err'); return; }
  openGen(`Edit Contract ${c.id} — Bei / Scope`, `
    <label class="text-[12px] font-bold text-white/60">PROJECT TITLE</label><input id="ceTitle" class="input mt-1" value="${esc(c.projectTitle)}">
    <label class="text-[12px] font-bold text-white/60 mt-3 block">SCOPE</label><textarea id="ceScope" rows="3" class="input mt-1">${esc(c.scope)}</textarea>
    <div class="grid sm:grid-cols-2 gap-3 mt-3"><div><label class="text-[12px] font-bold text-white/60">DURATION</label><input id="ceDuration" class="input mt-1" value="${esc(c.duration)}"></div><div><label class="text-[12px] font-bold text-white/60">CONTRACT VALUE (TZS)</label><input id="ceTotal" type="number" min="0" step="1000" class="input mt-1 font-mono" value="${c.total}"></div></div>
    <div class="text-[12px] text-white/45 mt-3"><i class="fa-solid fa-circle-info mr-1"></i> Baada ya mteja kusaini, bei hii haitoweza kubadilishwa.</div>
    <div class="flex gap-2 mt-3"><button onclick="saveContractEdit('${c.id}')" class="grad-btn flex-1 py-3 rounded-xl font-bold text-sm">Save Changes</button><button onclick="closeGen()" class="px-5 py-3 rounded-xl border border-white/15 font-bold text-sm">Cancel</button></div>`);
}
async function saveContractEdit(id) {
  const total = parseFloat($('ceTotal').value);
  if (!isFinite(total) || total < 0) { toast('Enter a valid contract value.', 'err'); return; }
  try {
    await apiPost('api/contracts.php', { action: 'update', id, projectTitle: $('ceTitle').value.trim(), scope: $('ceScope').value.trim(), duration: $('ceDuration').value.trim(), total });
  } catch (e) { toast(esc(e.message), 'err'); return; }
  closeGen(); toast('Contract updated ✓', 'ok');
  if (currentUser()?.role === 'admin') adminTab('contracts'); else clientTab('contracts');
}

let sigDraw = false;
function initSigPad() {
  const cv = $('sigPad'); if (!cv) return;
  const ctx = cv.getContext('2d'); ctx.lineWidth = 2.5; ctx.lineCap = 'round'; ctx.strokeStyle = '#0A1226';
  const pos = e => { const r = cv.getBoundingClientRect(); const t = e.touches ? e.touches[0] : e; return [(t.clientX - r.left) * (cv.width / r.width), (t.clientY - r.top) * (cv.height / r.height)]; };
  cv.onmousedown = e => { sigDraw = true; const [x, y] = pos(e); ctx.beginPath(); ctx.moveTo(x, y); };
  cv.onmousemove = e => { if (!sigDraw) return; const [x, y] = pos(e); ctx.lineTo(x, y); ctx.stroke(); };
  window.onmouseup = () => sigDraw = false;
  cv.ontouchstart = e => { e.preventDefault(); sigDraw = true; const [x, y] = pos(e); ctx.beginPath(); ctx.moveTo(x, y); };
  cv.ontouchmove = e => { e.preventDefault(); if (!sigDraw) return; const [x, y] = pos(e); ctx.lineTo(x, y); ctx.stroke(); };
  cv.ontouchend = () => sigDraw = false;
}
function clearSig() { const cv = $('sigPad'); cv.getContext('2d').clearRect(0, 0, cv.width, cv.height); }

/* ---------- KANUNI YA 50% DEPOSIT ---------- */
async function createDepositInvoice(cid) {
  try {
    const d = await apiPost('api/contracts.php', { action: 'create_deposit_invoice', id: cid });
    closeGen();
    toast('Deposit invoice ' + d.invoice.id + ' (' + fmtTZS(d.invoice.amount) + ') imeundwa ✓', 'ok');
    if (currentUser()?.role === 'admin') adminTab('invoices');
  } catch (e) { toast(esc(e.message), 'err'); }
}

async function saveSig(id, who) {
  const cv = $('sigPad');
  if (!cv) return;
  const blank = document.createElement('canvas'); blank.width = cv.width; blank.height = cv.height;
  if (cv.toDataURL() === blank.toDataURL()) { toast('Please draw your signature first.', 'err'); return; }
  try {
    await apiPost('api/contracts.php', { action: who === 'admin' ? 'sign_admin' : 'sign_client', id, sig: cv.toDataURL() });
  } catch (e) { toast(esc(e.message), 'err'); return; }
  closeGen();
  if (who === 'admin') {
    toast('Contract countersigned by CEO! Both parties signed ✓', 'ok');
    adminTab('contracts');
  } else {
    toast('Contract signed by you! CEO countersignature pending ✓', 'ok');
    clientTab('contracts');
  }
}

async function printContract(id) {
  let c;
  try { c = (await apiGet('api/contracts.php', { action: 'get', id })).contract; } catch (e) { toast(esc(e.message), 'err'); return; }
  $('printArea').innerHTML = contractHTML(c);
  window.print();
}

async function printQuote(id) {
  let d;
  try { d = await apiGet('api/quotes.php', { action: 'get', id }); } catch (e) { toast(esc(e.message), 'err'); return; }
  const q = d.quote;
  $('printArea').innerHTML = `<div style="font-family:Arial;color:#111"><h2>MBILINYI TECH SOLUTIONS — QUOTATION ${q.id}</h2><p>BRELA Reg • TIN 192-147-522 • 0796752645 • mbilinyitech@gmail.com</p><p>Client: ${esc(d.clientName)} (${esc(d.clientEmail)})<br>Project: ${esc(d.requestTitle)}<br>Date: ${fmtDate(q.createdAt)} • Valid till ${fmtDate(q.validUntil)}</p><table border="1" cellpadding="8" cellspacing="0" width="100%">${q.items.map((it, i) => `<tr><td>${i + 1}</td><td>${esc(it.desc)}</td><td>${it.qty}</td><td>${fmtTZS(it.qty * it.price)}</td></tr>`).join('')}</table><h3>Total: ${fmtTZS(q.total)} (incl. VAT)</h3><div style="border:1px solid #333;padding:12px;background:#f7f7f7"><strong>PAYMENT TERMS — MASHARTI YA MALIPO (50/50)</strong><br><strong>1. Deposit 50% (${fmtTZS(q.total / 2)}):</strong> inalipwa KABLA mradi kuanza — mradi hauanzi hadi deposit ilipwe na CEO aweke saini.<br><strong>2. Mid-Project 50% (${fmtTZS(q.total / 2)}):</strong> inalipwa mradi ukifika nusu ya timeline, kabla ya delivery ya mwisho.<br>Si "Deposit + Delivery". Source code na handover hazinapelekwi kabla installment ya 2 haijawekwa. Late payment: penalty 2% kwa wiki.</div><p>CEO: Jackson Mbilinyi</p></div>`;
  window.print();
}

/* ---------- ADMIN PORTAL ---------- */
let aTab = 'overview', adminSearch = '', adminStatusFilter = 'All';
// State for the staff, hiring, agent and tool screens added with the MTECHOS merge.
let DEPARTMENTS = [], EMPLOYEES = [], AGENTS = [], TOOLS = [], JOBS = [];
let adminEmpDept = '', adminJobFilter = '', adminAgentDept = '', adminToolHolder = null;

/* Growth bars: each place's real request count against the busiest place, so a
   single request still shows a visible bar rather than a zero-width line. */
function growthBars(rows, labelKey, colour) {
  const list = rows || [];
  if (!list.length) return '<div class="text-white/40 text-[12.5px] py-3">No requests from any ' + (labelKey === 'country' ? 'country' : 'region') + ' yet. The first one will appear here.</div>';
  const max = Math.max(...list.map(r => Number(r.count) || 0), 1);
  return '<div class="space-y-2">' + list.map(r => {
    const n = Number(r.count) || 0;
    const pct = Math.max(4, Math.round((n / max) * 100));
    return `<div>
      <div class="flex items-center justify-between text-[12.5px] mb-1"><span class="text-white/80">${esc(r[labelKey] || '—')}</span><span class="chip text-${colour}-300">${n}</span></div>
      <div class="h-2 rounded-full bg-white/5 overflow-hidden"><div class="h-full rounded-full bg-${colour}-400/70" style="width:${pct}%"></div></div>
    </div>`;
  }).join('') + '</div>';
}

async function adminTab(t, quiet) {
  if (!quiet) stopQueueWatch();
  aTab = t;
  document.querySelectorAll('#adminNav .sidebar-link').forEach(b => b.classList.toggle('active', b.dataset.atab === t));
  const titles = { overview: 'Business Dashboard', requests: 'Client Requests Inbox', quotes: 'Quotations & Pricing (Admin Sets Prices)', contracts: 'Contracts Manager', portfolio: 'Portfolio Projects (Admin CRUD)', testimonials: 'Client Testimonials (Admin CRUD)', news: 'News & Updates (Admin CRUD)', team: 'Our Team (Admin CRUD)', users: 'Manage All Accounts', invoices: 'Invoices & Revenue', tickets: 'Support Tickets', settings: 'Settings — Payments, Prices & Company Info', feedback: 'Client Feedback & Reviews', email: 'Email / SMTP Configuration',
    wafanyakazi: 'Wafanyakazi — Hire Staff per Service (CEO Only)', ajira: 'Ajira — Vacancies, Applications & Assessment', agents: 'Agent Company — 100 Agents & Their Manager', tools: 'Tools & Assignments — Every Tool Lives Here', ofisi: 'Online Office — Meetings, Calls & SMS', ukuaji: 'Growth — Countries & Tanzania Regions', usalama: 'Security, ECDSA & Company Account',
    queue: 'Work Queue — What the Company Is Doing and What Waits for Approval',
    runs: 'Agent Runs — Every Job the Company Carries Out', runtimes: 'Runtimes — Languages the Platform Really Runs', ai: 'AI Engine — The Provider Every Agent Thinks With', sms: 'SMS Gateway — Real Message Delivery' };
  $('adminTabTitle').textContent = titles[t];
  if (!quiet) $('adminContent').innerHTML = '<div class="text-white/45 text-[13.5px] flex items-center gap-2"><i class="fa-solid fa-circle-notch fa-spin"></i> Loading…</div>';

  try {
    const stats = await apiGet('api/dashboard.php', { action: 'stats' });
    $('adminReqCount').textContent = stats.pending;
    let h = '';

    if (t === 'overview') {
      const vs = await apiGet('api/dashboard.php', { action: 'visitor_stats' }).catch(() => ({ today: 0, yesterday: 0, this_week: 0, this_month: 0, last_7_days: [], top_pages: [] }));
      const cards = [
        [stats.revenue, 'Revenue Collected', 'emerald', 'fa-sack-dollar'],
        [stats.pipeline, 'Quoted Pipeline', 'cyan', 'fa-file-invoice-dollar'],
        [stats.requestCount, 'Total Requests', 'violet', 'fa-inbox'],
        [stats.userCount, 'Total Accounts', 'blue', 'fa-users'],
        [vs.today, 'Visitors Today', 'fuchsia', 'fa-eye'],
      ];
      h = `<div class="grid sm:grid-cols-2 md:grid-cols-3 xl:grid-cols-5 gap-3">
        ${cards.map(([v, l, c, i]) => `<div class="rounded-2xl bg-white/[.03] border border-white/10 p-5"><div class="flex justify-between items-start"><div><div class="font-display font-bold text-xl">${typeof v === 'number' && v > 999 && l !== 'Visitors Today' ? fmtTZS(v) : v}</div><div class="text-[11px] text-white/50 font-bold uppercase tracking-wider mt-1">${l}</div></div><div class="w-10 h-10 rounded-xl bg-${c}-500/15 flex items-center justify-center"><i class="fa-solid ${i} text-${c}-300"></i></div></div>${l === 'Visitors Today' ? `<div class="text-[11px] text-white/40 mt-2">Yesterday: <span class="text-white/70 font-bold">${vs.yesterday}</span> • Week: <span class="text-white/70 font-bold">${vs.this_week}</span> • Month: <span class="text-white/70 font-bold">${vs.this_month}</span></div>` : ''}</div>`).join('')}
      </div>
      <div class="grid lg:grid-cols-2 gap-4 mt-4">
        <div class="rounded-2xl border border-white/10 bg-white/[.02] p-5"><div class="font-bold text-[14px] mb-3">Revenue vs Pipeline (TZS M)</div><canvas id="chRev" height="180"></canvas></div>
        <div class="rounded-2xl border border-white/10 bg-white/[.02] p-5"><div class="font-bold text-[14px] mb-3">Requests by Service</div><canvas id="chSvc" height="180"></canvas></div>
        <div class="rounded-2xl border border-white/10 bg-white/[.02] p-5"><div class="font-bold text-[14px] mb-3">Visitors — Last 7 Days</div><canvas id="chVis" height="180"></canvas></div>
        <div class="rounded-2xl border border-white/10 bg-white/[.02] p-5">
          <div class="font-bold text-[14px] mb-3">Top 5 Most-Visited Pages (30 days)</div>
          <div class="grid gap-2">
            ${(vs.top_pages || []).map((p, idx) => `<div class="flex items-center gap-3 p-2.5 rounded-xl bg-white/[.02] border border-white/5 text-[13px]"><div class="w-7 h-7 rounded-lg bg-fuchsia-500/15 border border-fuchsia-400/20 flex items-center justify-center font-bold text-[11px] text-fuchsia-300">${idx + 1}</div><div class="flex-1 min-w-0 font-mono text-[12px] text-white/80 truncate">${esc(p.page)}</div><div class="font-bold text-white/70">${p.views} views</div></div>`).join('') || '<div class="text-white/40 text-[13px]">No tracked pages yet — visit the homepage to begin logging.</div>'}
          </div>
        </div>
      </div>
      <div class="font-bold mt-5 mb-2">Latest Requests Needing Action</div>
      <div class="grid gap-2">${stats.recent.map(r => `<div class="flex flex-wrap items-center gap-3 p-3.5 rounded-xl bg-white/[.02] border border-white/10 text-[13px]"><span class="font-mono text-cyan-300">${r.trackingId}</span><span class="flex-1 min-w-[160px]">${esc(r.title)} — <span class="text-white/45">${esc(r.name)}</span></span><span class="badge status-${r.status.replace(' ', '')}">${r.status}</span><button onclick="adminTab('requests')" class="text-cyan-300 font-bold">Review →</button></div>`).join('')}</div>`;
      $('adminContent').innerHTML = h;
      setTimeout(() => {
        if (typeof Chart !== 'undefined') {
          if ($('chRev')) {
            new Chart($('chRev'), { type: 'bar', data: { labels: ['Collected', 'Unpaid', 'Pipeline'], datasets: [{ data: [(stats.paid || 0) / 1e6, (stats.unpaid || 0) / 1e6, (stats.pipeline || 0) / 1e6], backgroundColor: ['#10b981', '#F5B301', '#22d3ee'], borderRadius: 10 }] }, options: { plugins: { legend: { display: false } }, scales: { x: { ticks: { color: '#8EA0BF' } }, y: { ticks: { color: '#8EA0BF' } } } } });
          }
          const by = stats.byService || {};
          if ($('chSvc') && Object.keys(by).length) {
            new Chart($('chSvc'), { type: 'doughnut', data: { labels: Object.keys(by), datasets: [{ data: Object.values(by), backgroundColor: ['#22d3ee', '#8b5cf6', '#F5B301', '#10b981', '#f472b6', '#60a5fa'] }] }, options: { plugins: { legend: { labels: { color: '#C7D2E8', boxWidth: 12 } } } } });
          }
          if ($('chVis')) {
            const days = (vs.last_7_days || []);
            new Chart($('chVis'), { type: 'line', data: { labels: days.map(d => d.label), datasets: [{ label: 'Page Views', data: days.map(d => d.count), borderColor: '#e879f9', backgroundColor: 'rgba(232,121,249,.15)', tension: .35, fill: true, pointRadius: 4, pointBackgroundColor: '#d946ef', borderWidth: 2.5 }] }, options: { plugins: { legend: { display: false } }, scales: { x: { ticks: { color: '#8EA0BF' } }, y: { beginAtZero: true, ticks: { color: '#8EA0BF', precision: 0 } } } } });
          }
        }
      }, 80);
      return;
    }

    if (t === 'requests') {
      const [reqData, userData, quoteData] = await Promise.all([
        apiGet('api/requests.php', { action: 'admin_list', q: adminSearch, status: adminStatusFilter }),
        apiGet('api/users.php', { action: 'list', q: '' }),
        apiGet('api/quotes.php', { action: 'admin_list' }),
      ]);
      const list = reqData.requests;
      const allQuotes = quoteData.quotes;
      const usersById = {};
      userData.users.forEach(u => { usersById[u.id] = u; });
      h = `<div class="flex flex-wrap gap-2 mb-4"><input oninput="adminSearch=this.value;adminTab('requests')" value="${esc(adminSearch)}" class="input !w-auto flex-1 min-w-[200px]" placeholder="Search title, client, tracking ID...">
      <select onchange="adminStatusFilter=this.value;adminTab('requests')" class="input !w-auto"><option ${adminStatusFilter === 'All' ? 'selected' : ''}>All</option>${['Pending', 'Under Review', 'Quoted', 'Approved', 'In Progress', 'Completed', 'Rejected'].map(s => `<option ${adminStatusFilter === s ? 'selected' : ''}>${s}</option>`).join('')}</select></div>
      <div class="grid gap-3">${list.map(r => { const u = usersById[r.userId]; const q = allQuotes.find(x => x.requestId === r.id); return `<div class="rounded-2xl border border-white/10 bg-white/[.02] p-5">
        <div class="flex flex-wrap justify-between gap-2"><div><span class="font-mono text-cyan-300 text-[12px] font-bold">${r.trackingId}</span> <span class="text-[11px] text-white/40">• ${fmtDate(r.createdAt)} • ${esc(r.priority)}</span><div class="font-bold text-[15px] mt-0.5">${esc(r.title)}</div><div class="text-[12px] text-white/50">${esc(r.name)} • ${esc(r.phone)} • ${esc(r.email)} ${u ? `• <span class="${u.status === 'Active' ? 'text-emerald-300' : 'text-red-300'}">[${u.status}]</span>` : ''}</div></div><span class="badge status-${r.status.replace(' ', '')} h-fit">${r.status.toUpperCase()}</span></div>
        <div class="text-[12.5px] text-white/60 mt-2"><strong class="text-white/80">${esc(r.service)}</strong> • Budget: ${esc(r.budget)} • Deadline: ${esc(r.deadline || '—')} • Tech: <span class="font-mono">${(r.tech || []).join(', ') || '—'}</span></div>
        ${Object.keys(r.serviceDetails || {}).length ? `<div class="mt-2 flex flex-wrap gap-1.5">${Object.entries(r.serviceDetails).map(([k, v]) => `<span class="text-[11px] px-2 py-0.5 rounded-full bg-cyan-500/10 border border-cyan-400/25 text-cyan-200"><b>${esc(k)}:</b> ${esc(v)}</span>`).join('')}</div>` : ''}
        <p class="text-[13px] text-white/65 mt-1.5 whitespace-pre-line">${esc(r.description)}</p>
        <div class="flex flex-wrap gap-2 mt-3">
          <select onchange="setReqStatus('${r.id}',this.value)" class="input !w-auto !py-2 text-[12px] font-bold">${['Pending', 'Under Review', 'Quoted', 'Approved', 'In Progress', 'Completed', 'Rejected'].map(s => `<option ${r.status === s ? 'selected' : ''}>${s}</option>`).join('')}</select>
          <button onclick="openRequestEditor('${r.id}')" class="px-4 py-2 rounded-xl bg-white/5 border border-white/10 text-[12px] font-bold hover:border-cyan-400/40"><i class="fa-solid fa-pen mr-1"></i>Edit</button>
          <button onclick="openAssignWorker('${r.id}','${esc(r.service)}')" class="px-4 py-2 rounded-xl bg-emerald-500/10 border border-emerald-400/30 text-emerald-200 text-[12px] font-bold"><i class="fa-solid fa-user-check mr-1"></i>Assign Worker</button>
          <button onclick="openQuoteBuilder('${r.id}')" class="px-4 py-2 rounded-xl grad-btn text-[12px] font-bold"><i class="fa-solid fa-tag mr-1"></i>Set Price & Quote</button>
          <button onclick="adminNote('${r.id}')" class="px-4 py-2 rounded-xl bg-white/5 border border-white/10 text-[12px] font-bold">CEO Note</button>
          <button onclick="quoteToContract('${r.id}')" class="px-4 py-2 rounded-xl bg-emerald-500/10 border border-emerald-400/30 text-emerald-200 text-[12px] font-bold">Generate Contract</button>
          <button onclick="delRequest('${r.id}')" class="px-4 py-2 rounded-xl bg-red-500/10 border border-red-400/30 text-red-200 text-[12px] font-bold"><i class="fa-solid fa-trash"></i></button>
        </div>
        ${q ? `<div class="mt-2 text-[12px] text-emerald-300 font-bold">Quoted: ${q.id} — ${fmtTZS(q.total)} (${q.status})</div>` : ''}
      </div>`; }).join('') || '<div class="text-white/40">No requests match.</div>'}</div>`;
    }

    if (t === 'quotes') {
      const data = await apiGet('api/quotes.php', { action: 'admin_list' });
      h = `<div class="rounded-2xl border border-cyan-400/25 bg-cyan-500/[.05] p-4 text-[13px] mb-4"><i class="fa-solid fa-circle-info text-cyan-300 mr-1"></i> <strong>Admin Pricing Desk:</strong> select any request → add line items → VAT 18% auto → send. Client gets instant portal + WhatsApp-ready quotation.</div>
      <div class="grid gap-3">${data.quotes.map(q => `<div class="rounded-2xl border border-white/10 bg-white/[.02] p-5"><div class="flex flex-wrap justify-between gap-2"><div><div class="font-bold">${q.id} • ${esc(q.requestTitle)}</div><div class="text-[12px] text-white/45 font-mono">${q.trackingId} • ${fmtDate(q.createdAt)} • Valid: ${fmtDate(q.validUntil)}</div></div><div class="text-right"><div class="font-display font-bold text-xl grad-text">${fmtTZS(q.total)}</div><span class="badge ${q.status === 'Sent' ? 'status-Quoted' : q.status === 'Approved' ? 'status-Completed' : 'status-Rejected'}">${q.status.toUpperCase()}</span></div></div>
      <div class="flex flex-wrap gap-2 mt-3"><button onclick="viewQuote('${q.id}')" class="px-4 py-2 rounded-full bg-white/5 border border-white/10 text-[12px] font-bold">Preview</button><button onclick="openQuoteBuilder('${q.requestId}')" class="px-4 py-2 rounded-full grad-btn text-[12px] font-bold">Edit Prices</button><button onclick="delQuote('${q.id}')" class="px-4 py-2 rounded-full bg-red-500/10 border border-red-400/30 text-red-200 text-[12px] font-bold">Delete</button></div></div>`).join('') || '<div class="text-white/40">No quotations yet — open Requests and click "Set Price & Quote".</div>'}</div>`;
    }

    if (t === 'contracts') {
      const data = await apiGet('api/contracts.php', { action: 'admin_list' });
      h = `<div class="rounded-2xl border border-cyan-400/25 bg-cyan-500/[.05] p-4 text-[13px] mb-4"><i class="fa-solid fa-circle-info text-cyan-300 mr-1"></i> <strong>Mtiririko wa saini:</strong> mteja anasaini <strong>kwanza</strong> → kisha wewe (CEO) unapigia saini → mkataba umekamilika na kutunzwa. Bei hubadilishwa kabla tu mteja hajasaini. <strong>Kanuni ya 50/50:</strong> huwezi kusaini mpaka deposit ya 50% ilipwe — deposit invoice inaundwa kiotomatiki wakati mteja anakubali nukuu.</div>
      <div class="grid gap-3">${data.contracts.map(c => `<div class="rounded-2xl border border-white/10 bg-white/[.02] p-5 flex flex-wrap justify-between gap-3 items-center"><div><div class="font-bold">${c.id} — ${esc(c.projectTitle)}</div><div class="text-[12px] text-white/45">${esc(c.clientName)} • ${c.trackingId} • ${fmtTZS(c.total)} • Client: ${c.clientSig ? '✓ ' + fmtDate(c.clientSignedAt) : '—'} • CEO: ${c.adminSig ? '✓ ' + fmtDate(c.adminSignedAt) : '—'}</div><span class="badge ${c.status === 'Signed' ? 'status-Completed' : c.status === 'Client Signed' ? 'status-Approved' : 'status-Quoted'} mt-1 inline-block">${c.status === 'Client Signed' ? 'CLIENT SIGNED — SIGN NOW' : c.status.toUpperCase()}</span></div><div class="flex gap-2">${c.status === 'Sent' ? `<button onclick="openContractEditor('${c.id}')" class="px-4 py-2 rounded-xl bg-white/5 border border-white/10 text-[12px] font-bold hover:border-cyan-400/40"><i class="fa-solid fa-pen mr-1"></i>Edit</button>` : ''}<button onclick="viewContract('${c.id}')" class="px-4 py-2 rounded-xl grad-btn text-[12px] font-bold">${c.status === 'Client Signed' ? 'Saini (CEO)' : c.status === 'Signed' ? 'Open' : 'Open / Preview'}</button><button onclick="delContract('${c.id}')" title="Contracts are permanent — cannot be deleted" class="px-4 py-2 rounded-xl bg-white/5 border border-white/10 text-white/35 text-[12px] font-bold cursor-not-allowed"><i class="fa-solid fa-lock"></i></button></div></div>`).join('') || '<div class="text-white/40">No contracts yet.</div>'}</div>`;
    }

    if (t === 'users') {
      const data = await apiGet('api/users.php', { action: 'list', q: adminSearch });
      h = `<div class="flex flex-wrap gap-2 mb-4"><input oninput="adminSearch=this.value;adminTab('users')" value="${esc(adminSearch)}" class="input !w-auto flex-1 min-w-[220px]" placeholder="Search name, email, phone, company..."><button onclick="openUserEditor('new')" class="grad-btn px-5 py-3 rounded-xl font-bold text-sm"><i class="fa-solid fa-user-plus mr-1"></i> Add Account</button></div>
      <div class="overflow-x-auto rounded-2xl border border-white/10"><table class="w-full data min-w-[760px]"><tr><th>Account</th><th>Contact</th><th>Role</th><th>Status</th><th>Joined</th><th class="!text-right">Actions</th></tr>
      ${data.users.map(u => `<tr><td><div class="flex items-center gap-2.5"><div class="w-9 h-9 rounded-xl ${u.role === 'admin' ? 'bg-gradient-to-br from-red-500 to-violet-600' : 'grad-btn'} flex items-center justify-center font-bold text-xs">${esc(u.name.split(' ').map(w => w[0]).join('').slice(0, 2))}</div><div><div class="font-bold">${esc(u.name)}</div><div class="text-[11px] text-white/40">${esc(u.company || '—')} • ${u.requestCount || 0} requests</div></div></div></td>
      <td><div class="text-[12.5px]">${esc(u.email)}</div><div class="text-[12px] text-white/45 font-mono">${esc(u.phone)}</div></td>
      <td><span class="badge ${u.role === 'admin' ? 'bg-red-500/15 text-red-300 border border-red-400/30' : 'bg-cyan-500/10 text-cyan-300 border border-cyan-400/30'}">${u.role.toUpperCase()}</span></td>
      <td><button onclick="toggleUserStatus('${u.id}')" class="badge ${u.status === 'Active' ? 'status-Completed' : 'status-Rejected'} cursor-pointer">${u.status.toUpperCase()}</button></td>
      <td class="text-white/50 text-[12px]">${fmtDate(u.createdAt)}</td>
      <td><div class="flex justify-end gap-1.5"><button onclick="openUserEditor('${u.id}')" title="Edit" class="w-8 h-8 rounded-lg bg-white/5 border border-white/10 hover:border-cyan-400/50"><i class="fa-solid fa-pen text-[11px] text-cyan-300"></i></button><button onclick="toggleUserRole('${u.id}')" title="Switch role" class="w-8 h-8 rounded-lg bg-white/5 border border-white/10 hover:border-violet-400/50"><i class="fa-solid fa-user-gear text-[11px] text-violet-300"></i></button><button onclick="impersonate('${u.id}')" title="View as client" class="w-8 h-8 rounded-lg bg-white/5 border border-white/10 hover:border-emerald-400/50"><i class="fa-solid fa-eye text-[11px] text-emerald-300"></i></button><button onclick="delUser('${u.id}')" title="Delete" class="w-8 h-8 rounded-lg bg-red-500/10 border border-red-400/20 hover:border-red-400/60"><i class="fa-solid fa-trash text-[11px] text-red-300"></i></button></div></td></tr>`).join('')}</table></div>
      <div class="text-[12px] text-white/40 mt-2">Click status badge to suspend/activate • eye icon to preview client portal • role icon toggles admin/client.</div>`;
    }

    if (t === 'invoices') {
      const data = await apiGet('api/invoices.php', { action: 'admin_list' });
      const tot = data.invoices.reduce((s, v) => s + v.amount, 0);
      const paid = data.invoices.filter(v => v.status === 'Paid').reduce((s, v) => s + v.amount, 0);
      h = `<div class="grid sm:grid-cols-3 gap-3 mb-4"><div class="rounded-2xl bg-white/[.03] border border-white/10 p-4"><div class="font-display font-bold text-xl">${fmtTZS(tot)}</div><div class="text-[11px] text-white/50 font-bold">TOTAL BILLED</div></div><div class="rounded-2xl bg-emerald-500/[.07] border border-emerald-400/25 p-4"><div class="font-display font-bold text-xl text-emerald-300">${fmtTZS(paid)}</div><div class="text-[11px] text-white/50 font-bold">COLLECTED</div></div><div class="rounded-2xl bg-yellow-500/[.07] border border-yellow-400/25 p-4"><div class="font-display font-bold text-xl text-yellow-300">${fmtTZS(tot - paid)}</div><div class="text-[11px] text-white/50 font-bold">OUTSTANDING</div></div></div>
      <div class="rounded-2xl border border-cyan-400/25 bg-cyan-500/[.05] p-4 text-[12.5px] text-white/70 mb-4"><strong class="text-white"><i class="fa-solid fa-coins text-gold mr-1"></i>Kanuni ya 50/50:</strong> deposit invoice ya 50% inaundwa kiotomatiki mteja akikubali nukuu. Mradi HAUWEZI kuanza (CEO hawezi kusaini, status haiwezi In Progress) mpaka deposit ilipwe — tumia "Mark Paid" baada ya kuthibitisha malipo.</div>
      <div class="flex justify-end mb-3"><button onclick="openInvoiceEditor('new')" class="grad-btn px-5 py-3 rounded-xl font-bold text-sm"><i class="fa-solid fa-plus mr-1"></i> New Invoice</button></div>
      <div class="grid gap-2">${data.invoices.map(v => `<div class="flex flex-wrap items-center justify-between gap-3 p-4 rounded-2xl bg-white/[.02] border border-white/10"><div><div class="font-bold text-[14px]">${v.id} — ${esc(v.title)}</div><div class="text-[12px] text-white/45 font-mono">${v.trackingId} • Due ${fmtDate(v.dueDate)}</div></div><div class="flex items-center gap-3"><div class="font-bold font-mono">${fmtTZS(v.amount)}</div><span class="badge ${v.status === 'Paid' ? 'status-Completed' : 'status-Pending'}">${v.status.toUpperCase()}</span><button onclick="openInvoiceEditor('${v.id}')" class="px-3 py-2 rounded-xl bg-white/5 border border-white/10 text-[12px] font-bold hover:border-cyan-400/40"><i class="fa-solid fa-pen"></i></button>${v.status !== 'Paid' ? `<button onclick="markPaid('${v.id}')" class="px-4 py-2 rounded-xl gold-btn text-[12px] font-bold text-[#2a1500]">Mark Paid</button>` : ''}<button onclick="delInvoice('${v.id}')" class="px-3 py-2 rounded-xl bg-red-500/10 border border-red-400/30 text-red-200 text-[12px] font-bold"><i class="fa-solid fa-trash"></i></button></div></div>`).join('') || '<div class="text-white/40">No invoices yet — click "New Invoice".</div>'}</div>`;
    }

    if (t === 'tickets') {
      const data = await apiGet('api/tickets.php', { action: 'all' });
      h = `<div class="grid gap-3">${data.tickets.map(x => `<div class="rounded-2xl border border-white/10 bg-white/[.02] p-4"><div class="flex justify-between flex-wrap gap-2"><div class="font-bold">${esc(x.subject)} <span class="text-[11px] text-white/40 font-normal">— ${esc(x.name)} (${esc(x.email)}) • ${fmtDate(x.createdAt)}</span></div><div class="flex gap-2 items-center"><span class="badge ${x.status === 'Open' ? 'status-Pending' : 'status-Completed'}">${x.status.toUpperCase()}</span><button onclick="toggleTicket('${x.id}')" class="text-[11px] font-bold text-cyan-300 underline">${x.status === 'Open' ? 'Close' : 'Reopen'}</button></div></div>
      <div class="text-[13px] text-white/60 mt-1 whitespace-pre-line">${esc(x.message)}</div>
      ${(x.replies || []).map(r => `<div class="mt-2 ml-3 p-2.5 rounded-xl bg-white/[.03] border border-white/10 text-[13px]"><strong>${esc(r.by)}:</strong> ${esc(r.text)}</div>`).join('')}
      <div class="flex gap-2 mt-2"><input id="arep-${x.id}" class="input !py-2 text-[13px]" placeholder="Reply as CEO..."><button onclick="adminReply('${x.id}')" class="grad-btn px-4 rounded-xl text-[12px] font-bold">Reply</button><a href="https://wa.me/255796752645" target="_blank" class="px-4 py-2 rounded-xl bg-[#25D366]/10 border border-[#25D366]/30 text-[#4be584] text-[12px] font-bold">WhatsApp</a><button onclick="adminDeleteTicket('${x.id}')" class="px-4 py-2 rounded-xl bg-red-500/10 border border-red-400/30 text-red-200 text-[12px] font-bold"><i class="fa-solid fa-trash"></i></button></div></div>`).join('') || '<div class="text-white/40">No tickets.</div>'}</div>`;
    }

    if (t === 'portfolio') {
      await loadPortfolioAndTestimonials(true);
      h = `<div class="flex flex-wrap items-center justify-between gap-2 mb-4">
        <div class="text-[13px] text-white/55"><i class="fa-solid fa-briefcase text-fuchsia-300 mr-1.5"></i>Projects currently visible on <strong>work</strong>. Demo data will be replaced once you delete the seeded ones & add your own. Image URL is required.</div>
        <button onclick="openPortfolioEditor('new')" class="grad-btn px-5 py-3 rounded-xl font-bold text-sm"><i class="fa-solid fa-plus mr-1"></i> Add Project</button>
      </div>
      <div class="overflow-x-auto rounded-2xl border border-white/10"><table class="w-full data min-w-[880px]"><tr><th>Preview</th><th>Title / Category</th><th>Stats / Link</th><th>Description</th><th class="!text-right">Actions</th></tr>
      ${PORTFOLIO_DB.map(p => {
        const id = p.id ?? '0';
        const img = p.img || p.imageUrl || p.image_url || '';
        const title = p.t || p.title || '';
        const cat = p.c || p.category || '';
        const stat = p.s || p.stat || '';
        const link = p.link || p.url || '';
        const desc = p.d || p.description || '';
        return `<tr>
          <td style="width:120px"><div class="w-[100px] h-[70px] rounded-xl overflow-hidden border border-white/10">${img ? `<img src="${img}" class="w-full h-full object-cover">` : `<div class="w-full h-full bg-gradient-to-br from-cyan-500/20 to-violet-500/20 flex items-center justify-center text-[10px] text-white/60">No Image</div>`}</div></td>
          <td><div class="font-bold">${esc(title)}</div><div class="text-[11px] text-white/50 mt-0.5">${esc(cat)}</div></td>
          <td>${stat ? `<span class="badge status-Completed text-[11px]">${esc(stat)}</span>` : '<span class="text-white/35 text-[11px]">—</span>'}<br>${link ? `<a href="${link}" target="_blank" class="text-[11px] text-cyan-300 underline font-bold mt-1 inline-block">Open ↗</a>` : ''}</td>
          <td class="text-[12.5px] text-white/60 max-w-[320px] line-clamp-2">${esc(desc)}</td>
          <td class="!text-right"><div class="flex justify-end gap-1.5"><button onclick="openPortfolioEditor('${id}')" title="Edit" class="w-9 h-9 rounded-lg bg-white/5 border border-white/10 hover:border-fuchsia-400/50"><i class="fa-solid fa-pen text-[11px] text-fuchsia-300"></i></button><button onclick="delPortfolio('${id}')" title="Delete" class="w-9 h-9 rounded-lg bg-red-500/10 border border-red-400/20 hover:border-red-400/60"><i class="fa-solid fa-trash text-[11px] text-red-300"></i></button></div></td>
        </tr>`;
      }).join('') || '<tr><td colspan="5" class="!text-center text-white/40 py-8">No portfolio projects yet. Click "Add Project" to add your first one — it will appear on work immediately.</td></tr>'}</table></div>`;
    }

    if (t === 'testimonials') {
      await loadPortfolioAndTestimonials(true);
      h = `<div class="flex flex-wrap items-center justify-between gap-2 mb-4">
        <div class="text-[13px] text-white/55"><i class="fa-solid fa-comments text-emerald-300 mr-1.5"></i>Testimonials shown on the landing page hero testimonial section. Demo clients can be deleted and replaced with your real customers.</div>
        <button onclick="openTestimonialEditor('new')" class="grad-btn px-5 py-3 rounded-xl font-bold text-sm"><i class="fa-solid fa-plus mr-1"></i> Add Testimonial</button>
      </div>
      <div class="grid md:grid-cols-2 gap-4">
      ${TESTIS_DB.map(t2 => {
        const id = t2.id ?? '0';
        const stars = Math.max(1, Math.min(5, Number(t2.s || t2.stars) || 5));
        const name = t2.n || t2.name || '';
        const role = t2.r || t2.role || '';
        const quote = t2.t || t2.quote || '';
        return `<div class="rounded-2xl border border-white/10 bg-white/[.02] p-5">
          <div class="flex items-start justify-between gap-3">
            <div class="flex-1 min-w-0">
              <div class="flex gap-1 text-gold text-xs">${'<i class="fa-solid fa-star"></i>'.repeat(stars)}</div>
              <p class="text-[14px] text-white/75 mt-2 leading-relaxed">“${esc(quote)}”</p>
              <div class="flex items-center gap-3 mt-4"><div class="w-11 h-11 rounded-full grad-btn flex items-center justify-center font-bold">${(name || 'MTS').split(' ').map(w => w[0]).join('').slice(0, 2)}</div><div><div class="font-bold text-[14px]">${esc(name)}</div><div class="text-[11.5px] text-white/45">${esc(role)}</div></div></div>
            </div>
            <div class="flex flex-col gap-1.5">
              <button onclick="openTestimonialEditor('${id}')" title="Edit" class="w-9 h-9 rounded-lg bg-white/5 border border-white/10 hover:border-emerald-400/50"><i class="fa-solid fa-pen text-[11px] text-emerald-300"></i></button>
              <button onclick="delTestimonial('${id}')" title="Delete" class="w-9 h-9 rounded-lg bg-red-500/10 border border-red-400/20 hover:border-red-400/60"><i class="fa-solid fa-trash text-[11px] text-red-300"></i></button>
            </div>
          </div>
        </div>`;
      }).join('') || '<div class="md:col-span-2 text-white/40 glass rounded-3xl p-8 text-center">No testimonials yet. Click "Add Testimonial" — they will appear on the homepage hero immediately.</div>'}</div>`;
    }

    if (t === 'news') {
      await loadNewsAndTeam(true);
      let all;
      try { all = await apiGet('api/news.php', { action: 'list', all: '1' }); } catch (e) { all = { news: NEWS_DB }; }
      NEWS_DB = all.news || [];
      h = `<div class="flex flex-wrap items-center justify-between gap-2 mb-4">
        <div class="text-[13px] text-white/55"><i class="fa-solid fa-newspaper text-cyan-300 mr-1.5"></i>Published posts appear immediately on the <strong>news</strong> page and in the client portal. Unpublished posts stay hidden from users.</div>
        <button onclick="openNewsEditor('new')" class="grad-btn px-5 py-3 rounded-xl font-bold text-sm"><i class="fa-solid fa-plus mr-1"></i> Add News Post</button>
      </div>
      <div class="grid md:grid-cols-2 gap-4">
      ${NEWS_DB.map(n => `
        <div class="rounded-2xl border ${n.isActive ? 'border-cyan-400/25' : 'border-white/10'} bg-white/[.02] p-5">
          <div class="flex items-start justify-between gap-3">
            <div class="flex-1 min-w-0">
              <div class="flex flex-wrap items-center gap-2"><span class="badge bg-cyan-500/10 text-cyan-300 border border-cyan-400/30">${esc(n.category)}</span><span class="badge ${n.isActive ? 'status-Completed' : 'status-Rejected'}">${n.isActive ? 'PUBLISHED' : 'HIDDEN'}</span><span class="text-[11px] text-white/40">${fmtDate(n.createdAt)}</span></div>
              <div class="font-bold text-[15px] mt-2">${esc(n.title)}</div>
              <p class="text-[12.5px] text-white/55 mt-1 line-clamp-3">${esc(n.excerpt)}</p>
            </div>
            <div class="flex flex-col gap-1.5">
              <button onclick="openNewsEditor('${n.id}')" title="Edit" class="w-9 h-9 rounded-lg bg-white/5 border border-white/10 hover:border-cyan-400/50"><i class="fa-solid fa-pen text-[11px] text-cyan-300"></i></button>
              <button onclick="toggleNews('${n.id}')" title="${n.isActive ? 'Unpublish' : 'Publish'}" class="w-9 h-9 rounded-lg bg-white/5 border border-white/10 hover:border-yellow-400/50"><i class="fa-solid fa-eye${n.isActive ? '-slash' : ''} text-[11px] text-yellow-300"></i></button>
              <button onclick="delNews('${n.id}')" title="Delete" class="w-9 h-9 rounded-lg bg-red-500/10 border border-red-400/20 hover:border-red-400/60"><i class="fa-solid fa-trash text-[11px] text-red-300"></i></button>
            </div>
          </div>
        </div>`).join('') || '<div class="md:col-span-2 text-white/40 glass rounded-3xl p-8 text-center">No news posts yet. Click "Add News Post" — published posts appear on the news page immediately.</div>'}</div>`;
    }

    if (t === 'team') {
      await loadNewsAndTeam(true);
      let all;
      try { all = await apiGet('api/team.php', { action: 'list', all: '1' }); } catch (e) { all = { team: TEAM_DB }; }
      TEAM_DB = all.team || [];
      h = `<div class="flex flex-wrap items-center justify-between gap-2 mb-4">
        <div class="text-[13px] text-white/55"><i class="fa-solid fa-people-group text-violet-300 mr-1.5"></i>Team members appear on the homepage <strong>Our Team</strong> section and the team page. Hidden members are not shown to visitors.</div>
        <button onclick="openTeamEditor('new')" class="grad-btn px-5 py-3 rounded-xl font-bold text-sm"><i class="fa-solid fa-plus mr-1"></i> Add Team Member</button>
      </div>
      <div class="grid md:grid-cols-2 lg:grid-cols-3 gap-4">
      ${TEAM_DB.map(m => {
        const initials = (m.name || 'MTS').split(' ').map(w => w[0]).join('').slice(0, 2).toUpperCase();
        return `<div class="rounded-2xl border ${m.isActive ? 'border-violet-400/25' : 'border-white/10'} bg-white/[.02] p-5">
          <div class="flex items-start justify-between gap-3">
            <div class="flex-1 min-w-0 flex items-center gap-3">
              ${m.imageUrl ? `<img src="${esc(m.imageUrl)}" class="w-12 h-12 rounded-2xl object-cover border border-white/10">` : `<div class="w-12 h-12 rounded-2xl grad-btn flex items-center justify-center font-bold">${esc(initials)}</div>`}
              <div class="min-w-0"><div class="font-bold text-[14px] truncate">${esc(m.name)}</div><div class="text-[11.5px] text-violet-300 truncate">${esc(m.role)}</div><span class="badge ${m.isActive ? 'status-Completed' : 'status-Rejected'} mt-1 inline-block">${m.isActive ? 'VISIBLE' : 'HIDDEN'}</span></div>
            </div>
            <div class="flex flex-col gap-1.5">
              <button onclick="openTeamEditor('${m.id}')" title="Edit" class="w-9 h-9 rounded-lg bg-white/5 border border-white/10 hover:border-violet-400/50"><i class="fa-solid fa-pen text-[11px] text-violet-300"></i></button>
              <button onclick="toggleTeam('${m.id}')" title="${m.isActive ? 'Hide' : 'Show'}" class="w-9 h-9 rounded-lg bg-white/5 border border-white/10 hover:border-yellow-400/50"><i class="fa-solid fa-eye${m.isActive ? '-slash' : ''} text-[11px] text-yellow-300"></i></button>
              <button onclick="delTeam('${m.id}')" title="Delete" class="w-9 h-9 rounded-lg bg-red-500/10 border border-red-400/20 hover:border-red-400/60"><i class="fa-solid fa-trash text-[11px] text-red-300"></i></button>
            </div>
          </div>
          <p class="text-[12.5px] text-white/55 mt-3 line-clamp-3">${esc(m.bio)}</p>
        </div>`;
      }).join('') || '<div class="md:col-span-2 lg:col-span-3 text-white/40 glass rounded-3xl p-8 text-center">No team members yet. Click "Add Team Member" — they appear on the homepage immediately.</div>'}</div>`;
    }

    if (t === 'wafanyakazi') {
      const [deps, emp] = await Promise.all([
        apiGet('api/wafanyakazi.php', { action: 'departments' }),
        apiGet('api/wafanyakazi.php', { action: 'list', department: adminEmpDept || '' }),
      ]);
      DEPARTMENTS = deps.departments || [];
      const list = emp.employees || [];
      const lvlColor = { junior: 'text-sky-300', mid: 'text-cyan-300', senior: 'text-emerald-300', lead: 'text-amber-300', principal: 'text-fuchsia-300' };
      h = `<div class="rounded-2xl border border-emerald-400/25 bg-emerald-500/[.05] p-4 text-[13px] text-white/70 mb-4">
        <i class="fa-solid fa-user-tie text-emerald-300 mr-2"></i>
        Hire a worker for a named service. The client's request then goes to whoever serves that service — the client is served by a named person, not a queue. <strong>Only you, the CEO, hire and assign tools.</strong>
      </div>
      <div class="flex flex-wrap items-center gap-2 mb-4">
        <select onchange="adminEmpDept=this.value;adminTab('wafanyakazi')" class="input !w-auto text-[13px]">
          <option value="">All departments (${list.length})</option>
          ${DEPARTMENTS.map(d => `<option value="${d.key}" ${adminEmpDept === d.key ? 'selected' : ''}>${d.name}</option>`).join('')}
        </select>
        <button onclick="openEmployeeEditor('new')" class="grad-btn px-5 py-3 rounded-xl font-bold text-sm"><i class="fa-solid fa-user-plus mr-1"></i> Hire Employee</button>
      </div>
      <div class="grid sm:grid-cols-2 xl:grid-cols-3 gap-4">
      ${list.map(m => {
        const initials = (m.fullName || 'MTS').split(' ').map(w => w[0]).join('').slice(0, 2).toUpperCase();
        const pop = ['pop-cyan', 'pop-violet', 'pop-emerald', 'pop-amber', 'pop-rose'][(m.fullName || '').length % 5];
        return `<div class="card3d ${pop} p-5">
          <div class="flex items-start gap-3">
            ${m.photo ? `<img src="${esc(m.photo)}" class="w-14 h-14 rounded-2xl object-cover border border-white/15">` : `<div class="w-14 h-14 rounded-2xl grad-btn flex items-center justify-center font-bold">${esc(initials)}</div>`}
            <div class="flex-1 min-w-0">
              <div class="font-bold text-[14.5px] truncate">${esc(m.fullName)}</div>
              <div class="text-[12px] text-emerald-300 truncate">${esc(m.profession || '—')}</div>
              <div class="flex flex-wrap gap-1.5 mt-1.5">
                <span class="chip ${lvlColor[m.level] || 'text-white/60'}"><i class="fa-solid fa-layer-group"></i>${esc(m.level)}</span>
                <span class="chip text-white/60">${esc(m.department)}</span>
                <span class="chip ${m.isActive ? 'text-emerald-300' : 'text-red-300'}">${m.isActive ? 'ACTIVE' : 'PAUSED'}</span>
              </div>
            </div>
          </div>
          <div class="text-[12px] text-white/50 mt-3 space-y-1">
            <div><i class="fa-solid fa-location-dot mr-1.5 text-white/35"></i>${esc(m.region || '—')} • ${esc(m.country || 'Tanzania')}</div>
            <div><i class="fa-solid fa-briefcase mr-1.5 text-white/35"></i>${Number(m.experienceYears) || 0} yrs • ${esc(m.languages || '—')}</div>
            <div><i class="fa-solid fa-toolbox mr-1.5 text-white/35"></i>${(m.tools || []).length} tools granted</div>
            <div class="text-white/70"><i class="fa-solid fa-diagram-project mr-1.5 text-white/35"></i>${(m.services || []).map(s => esc(s)).join(', ') || 'No service assigned'}</div>
          </div>
          <div class="flex gap-1.5 mt-4">
            <button onclick="openEmployeeEditor('${m.id}')" title="Edit" class="flex-1 py-2 rounded-lg bg-white/5 border border-white/10 hover:border-emerald-400/50 text-[12px] font-bold"><i class="fa-solid fa-pen mr-1"></i>Edit</button>
            <button onclick="openToolGranter('${m.id}','employee')" title="Tools" class="flex-1 py-2 rounded-lg bg-white/5 border border-white/10 hover:border-sky-400/50 text-[12px] font-bold"><i class="fa-solid fa-toolbox mr-1"></i>Tools</button>
            <button onclick="toggleEmployee('${m.id}')" title="Pause/Resume" class="px-3 py-2 rounded-lg bg-white/5 border border-white/10 hover:border-amber-400/50"><i class="fa-solid fa-power-off text-[11px] text-amber-300"></i></button>
            <button onclick="delEmployee('${m.id}')" title="Remove" class="px-3 py-2 rounded-lg bg-red-500/10 border border-red-400/20 hover:border-red-400/60"><i class="fa-solid fa-trash text-[11px] text-red-300"></i></button>
          </div>
        </div>`;
      }).join('') || '<div class="sm:col-span-2 xl:col-span-3 glass rounded-3xl p-8 text-center text-white/40">No employees yet. Click <strong>Hire Employee</strong> — they start serving clients immediately.</div>'}</div>`;
    }

    if (t === 'ajira') {
      const [jobs, apps, st] = await Promise.all([
        apiGet('api/ajira.php', { action: 'admin_openings' }),
        apiGet('api/ajira.php', { action: 'admin_applications', jobId: adminJobFilter || '' }),
        apiGet('api/ajira.php', { action: 'status' }),
      ]);
      const openings = jobs.openings || [];
      const applications = apps.applications || [];
      h = `<div class="rounded-2xl border border-amber-400/25 bg-amber-500/[.05] p-4 text-[13px] text-white/70 mb-4">
        <i class="fa-solid fa-briefcase text-amber-300 mr-2"></i>
        Post a vacancy; applicants fill in their details (photo, profession, level) and are assessed <strong>theoretically and practically</strong>. The two scores are weighted into one total. Hiring creates the employee record in one step.
      </div>
      <div class="flex flex-wrap items-center justify-between gap-2 mb-3">
        <div class="font-bold text-[14px]"><i class="fa-solid fa-bullhorn text-amber-300 mr-1.5"></i>Open Vacancies (${openings.length})</div>
        <button onclick="openJobEditor('new')" class="grad-btn px-5 py-2.5 rounded-xl font-bold text-[13px]"><i class="fa-solid fa-plus mr-1"></i> Post Vacancy</button>
      </div>
      <div class="grid sm:grid-cols-2 gap-3 mb-6">
      ${openings.map(j => `<div class="card3d pop-amber p-4">
        <div class="flex justify-between items-start gap-2">
          <div><div class="font-bold text-[14px]">${esc(j.title)}</div>
          <div class="text-[11.5px] text-amber-300">${esc(j.department)} • ${esc(j.service || '—')} • ${esc(j.level)}</div></div>
          <span class="chip ${j.isOpen ? 'text-emerald-300' : 'text-red-300'}">${j.isOpen ? 'OPEN' : 'CLOSED'}</span>
        </div>
        <div class="text-[12px] text-white/50 mt-2">${esc(j.location)} • ${esc(j.kind)} • ${esc(j.salary || 'Salary on request')}</div>
        <div class="text-[12px] text-white/45 mt-1.5">Assessment: theory ${j.theoreticalWeight}% / practical ${j.practicalWeight}% • ${j.positions} position(s)</div>
        <div class="flex gap-1.5 mt-3">
          <button onclick="adminJobFilter='${j.id}';adminTab('ajira')" class="flex-1 py-2 rounded-lg bg-white/5 border border-white/10 hover:border-amber-400/50 text-[12px] font-bold">View Applicants</button>
          <button onclick="openJobEditor('${j.id}')" class="px-3 py-2 rounded-lg bg-white/5 border border-white/10 hover:border-emerald-400/50"><i class="fa-solid fa-pen text-[11px] text-emerald-300"></i></button>
          <button onclick="delJob('${j.id}')" class="px-3 py-2 rounded-lg bg-red-500/10 border border-red-400/20"><i class="fa-solid fa-trash text-[11px] text-red-300"></i></button>
        </div>
      </div>`).join('') || '<div class="sm:col-span-2 glass rounded-3xl p-6 text-center text-white/40">No vacancies posted.</div>'}</div>
      <div class="flex flex-wrap items-center gap-2 mb-3">
        <div class="font-bold text-[14px]"><i class="fa-solid fa-users-viewfinder text-cyan-300 mr-1.5"></i>Applications (${applications.length})</div>
        <select onchange="adminJobFilter=this.value;adminTab('ajira')" class="input !w-auto text-[12.5px]"><option value="">All vacancies</option>${openings.map(j => `<option value="${j.id}" ${adminJobFilter === j.id ? 'selected' : ''}>${esc(j.title)}</option>`).join('')}</select>
      </div>
      <div class="grid gap-2.5">
      ${applications.map(a => `<div class="rounded-2xl border border-white/10 bg-white/[.02] p-4">
        <div class="flex flex-wrap items-center gap-3">
          ${a.photo ? `<img src="${esc(a.photo)}" class="w-11 h-11 rounded-xl object-cover border border-white/15">` : `<div class="w-11 h-11 rounded-xl bg-white/5 border border-white/10 flex items-center justify-center font-bold text-[13px]">${esc((a.fullName || 'A').slice(0, 2).toUpperCase())}</div>`}
          <div class="flex-1 min-w-[180px]"><div class="font-bold text-[13.5px]">${esc(a.fullName)} <span class="text-white/40 font-normal">— ${esc(a.profession || '')}</span></div>
          <div class="text-[11.5px] text-white/45">${esc(a.jobTitle || '—')} • ${esc(a.region || '—')} • ${Number(a.experienceYears) || 0} yrs</div></div>
          <div class="text-center px-3"><div class="font-display font-bold text-lg ${a.totalScore >= 70 ? 'text-emerald-300' : a.totalScore >= 50 ? 'text-amber-300' : 'text-white/50'}">${a.totalScore}%</div><div class="text-[10px] text-white/40 font-bold uppercase">Total</div></div>
          <span class="badge status-${(a.status || '').replace(' ', '')}">${esc(a.status)}</span>
          <div class="flex gap-1.5">
            <button onclick="openAssess('${a.id}','${esc(a.fullName)}',${a.theoreticalScore},${a.practicalScore})" class="px-3 py-2 rounded-lg bg-white/5 border border-white/10 hover:border-cyan-400/50 text-[12px] font-bold"><i class="fa-solid fa-clipboard-check mr-1 text-cyan-300"></i>Assess</button>
            <button onclick="hireApplicant('${a.id}')" class="px-3 py-2 rounded-lg bg-emerald-500/10 border border-emerald-400/30 hover:border-emerald-400/70 text-[12px] font-bold text-emerald-300"><i class="fa-solid fa-user-plus mr-1"></i>Hire</button>
            <button onclick="delApplication('${a.id}')" class="px-3 py-2 rounded-lg bg-red-500/10 border border-red-400/20"><i class="fa-solid fa-trash text-[11px] text-red-300"></i></button>
          </div>
        </div>
        <div class="flex flex-wrap gap-3 mt-2.5 text-[11.5px] text-white/50">
          <span>Theory: <strong class="text-white/80">${a.theoreticalScore}%</strong></span>
          <span>Practical: <strong class="text-white/80">${a.practicalScore}%</strong></span>
          <span>Email: <strong class="text-white/80">${esc(a.email || '—')}</strong></span>
          <span>Phone: <strong class="text-white/80">${esc(a.phone || '—')}</strong></span>
        </div>
        ${a.assessmentNotes ? `<div class="text-[12px] text-white/55 mt-2 italic">“${esc(a.assessmentNotes)}”</div>` : ''}
      </div>`).join('') || '<div class="glass rounded-3xl p-6 text-center text-white/40">No applications yet. The public Ajira page is open to everyone.</div>'}</div>`;
    }

    if (t === 'queue') {
      const q = await apiGet('api/engine.php', { action: 'queue', limit: 200 });
      const st = q.stats || {};
      const jobs = q.jobs || [];
      const badge = (s) => ({ held: 'bg-amber-500/20 text-amber-200 border-amber-400/40', ready: 'bg-cyan-500/20 text-cyan-200 border-cyan-400/40', running: 'bg-violet-500/20 text-violet-200 border-violet-400/40', done: 'bg-emerald-500/20 text-emerald-200 border-emerald-400/40', failed: 'bg-red-500/20 text-red-200 border-red-400/40' }[s] || 'bg-white/10 text-white/60 border-white/20');
      if ($('adminQueueCount')) $('adminQueueCount').textContent = (st.held || 0) + (st.ready || 0);
      const card = (n, l, c) => `<div class="glass rounded-xl p-3.5 border border-white/10"><div class="text-[22px] font-extrabold text-${c}">${n}</div><div class="text-[11.5px] text-white/50">${l}</div></div>`;
      h = `
        <div class="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-2.5 mb-4">
          ${card(st.held || 0, 'Waiting for your approval', 'amber-300')}
          ${card(st.ready || 0, 'Ready to start', 'cyan-300')}
          ${card(st.running || 0, 'Running now', 'violet-300')}
          ${card(st.done || 0, 'Done', 'emerald-300')}
          ${card(st.failed || 0, 'Failed', 'red-300')}
          ${card((st.busyAgents || 0) + '/' + (st.activeAgents || 0), 'Agents busy', 'fuchsia-300')}
        </div>
        <div class="glass rounded-2xl border border-white/10 p-4 mb-4">
          <div class="flex flex-wrap items-center gap-2">
            <span class="text-[13px] text-white/60"><i class="fa-solid fa-heart-pulse mr-1 ${st.runningLoop ? 'text-emerald-300' : 'text-red-300'}"></i>Autonomous loop: <b class="${st.runningLoop ? 'text-emerald-300' : 'text-red-300'}">${st.runningLoop ? 'running' : 'stopped'}</b> · ticks: <span class="font-mono">${st.ticks || 0}</span></span>
            <span class="flex-1"></span>
            <button onclick="queueSweep()" class="grad-btn px-4 py-2 rounded-xl text-[12.5px] font-bold"><i class="fa-solid fa-bolt mr-1"></i>Run queue now</button>
            <button onclick="adminTab('queue')" class="px-4 py-2 rounded-xl bg-white/5 border border-white/10 text-[12.5px] font-bold hover:border-cyan-400/50"><i class="fa-solid fa-rotate mr-1"></i>Refresh</button>
          </div>
          <div class="text-[11.5px] text-white/45 mt-2">A job marked <b class="text-amber-300">held</b> is a client request waiting for you. Approve the request and the loop starts it by itself.</div>
        </div>
        <div class="flex flex-wrap gap-2 mb-3">
          ${['', 'held', 'ready', 'running', 'done', 'failed'].map(f => `<button onclick="adminQueueFilter='${f}';adminTab('queue')" class="px-3.5 py-1.5 rounded-lg text-[12px] font-bold border ${adminQueueFilter === f ? 'bg-cyan-500/20 border-cyan-400/50 text-cyan-200' : 'bg-white/5 border-white/10 text-white/60'}">${f || 'All'}</button>`).join('')}
        </div>
        <div class="space-y-2">${jobs.filter(j => !adminQueueFilter || j.status === adminQueueFilter).map(j => `
          <div class="glass rounded-xl border border-white/10 p-3.5">
            <div class="flex flex-wrap items-center gap-2">
              <span class="text-[10.5px] px-2 py-0.5 rounded-md border ${badge(j.status)} font-bold uppercase">${esc(j.status)}</span>
              <span class="font-mono text-[11.5px] text-white/50">${esc(j.id)}</span>
              <span class="text-[12.5px] text-white/80"><i class="fa-solid fa-robot mr-1 text-fuchsia-300"></i>${esc(j.agentName || j.agentId || '—')}</span>
              ${j.service ? `<span class="text-[11.5px] text-cyan-300"><i class="fa-solid fa-tag mr-1"></i>${esc(j.service)}</span>` : ''}
              ${j.attempts > 0 ? `<span class="text-[11px] text-white/40">attempts ${j.attempts}</span>` : ''}
              <span class="flex-1"></span>
              ${j.status === 'held' ? `<button onclick="queueRelease('${esc(j.id)}')" class="px-3 py-1.5 rounded-lg bg-emerald-500/20 border border-emerald-400/40 text-emerald-200 text-[11.5px] font-bold hover:bg-emerald-500/30"><i class="fa-solid fa-check mr-1"></i>Approve &amp; start</button>` : ''}
              ${j.status === 'failed' ? `<button onclick="queueRetry('${esc(j.id)}')" class="px-3 py-1.5 rounded-lg bg-white/5 border border-white/15 text-[11.5px] font-bold hover:border-cyan-400/50"><i class="fa-solid fa-rotate-right mr-1"></i>Retry</button>` : ''}
              ${(j.status === 'held' || j.status === 'ready' || j.status === 'running') ? `<button onclick="queueCancel('${esc(j.id)}')" class="px-3 py-1.5 rounded-lg bg-red-500/15 border border-red-400/30 text-red-200 text-[11.5px] font-bold"><i class="fa-solid fa-ban mr-1"></i>Cancel</button>` : ''}
              ${j.runId ? `<button onclick="viewRun('${esc(j.runId)}')" class="px-3 py-1.5 rounded-lg bg-white/5 border border-white/15 text-[11.5px] font-bold"><i class="fa-solid fa-eye mr-1"></i>Steps</button>` : ''}
            </div>
            <div class="text-[12.5px] text-white/75 mt-2">${esc(j.task || '')}</div>
            ${j.error ? `<div class="text-[11.5px] text-red-300 mt-1.5"><i class="fa-solid fa-triangle-exclamation mr-1"></i>${esc(j.error)}</div>` : ''}
            ${j.result ? `<details class="mt-1.5"><summary class="text-[11.5px] text-emerald-300 cursor-pointer">Result</summary><div class="text-[12px] text-white/70 mt-1 whitespace-pre-wrap">${esc(j.result).slice(0, 1500)}</div></details>` : ''}
          </div>`).join('') || '<div class="text-white/40 text-[13px] p-4">The queue is empty.</div>'}</div>`;
      if (!quiet) startQueueWatch();
    }

    if (t === 'agents') {
      const [st, list, mgr] = await Promise.all([
        apiGet('api/agents.php', { action: 'stats' }),
        apiGet('api/agents.php', { action: 'list', department: adminAgentDept || '' }),
        apiGet('api/agents.php', { action: 'manager' }),
      ]);
      AGENTS = list.agents || [];
      const manager = mgr.manager;
      h = `<div class="rounded-2xl border border-fuchsia-400/25 bg-fuchsia-500/[.05] p-4 text-[13px] text-white/70 mb-4">
        <i class="fa-solid fa-robot text-fuchsia-300 mr-2"></i>
        A self-running company of <strong>${st.total}</strong> agents. The manager leads every agent; each agent has one job and only the tools you grant it. You give work and inspect it — the agents carry it out.
      </div>
      ${manager ? `<div class="card3d pop-violet p-5 mb-5">
        <div class="flex flex-wrap items-center gap-4">
          <div class="w-14 h-14 rounded-2xl bg-gradient-to-br from-fuchsia-500 to-violet-600 flex items-center justify-center text-2xl"><i class="fa-solid fa-diagram-project"></i></div>
          <div class="flex-1 min-w-[200px]"><div class="font-display font-bold text-lg">${esc(manager.name)}</div>
          <div class="text-[12.5px] text-fuchsia-300">${esc(manager.role)}</div>
          <div class="text-[12px] text-white/50 mt-1">${esc(manager.speciality)}</div></div>
          <div class="flex flex-wrap gap-1.5">${(manager.tools || []).map(x => `<span class="chip text-fuchsia-300">${esc(x)}</span>`).join('')}</div>
          <span class="chip text-emerald-300"><i class="fa-solid fa-circle-check"></i>${esc(manager.status)}</span>
        </div>
      </div>` : ''}
      <div class="grid grid-cols-2 md:grid-cols-4 gap-3 mb-5">
        ${[['Total agents', st.total, 'fa-users', 'fuchsia'], ['With tools', st.withTools, 'fa-toolbox', 'sky'], ['Departments', (st.byDepartment || []).length, 'fa-sitemap', 'violet'], ['Autonomous', 'YES', 'fa-bolt', 'emerald']].map(([l, v, i, c]) => `<div class="card3d pop-${c} p-4"><div class="flex justify-between items-start"><div><div class="font-display font-bold text-xl">${v}</div><div class="text-[10.5px] text-white/50 font-bold uppercase tracking-wider mt-0.5">${l}</div></div><i class="fa-solid ${i} text-${c}-300"></i></div></div>`).join('')}
      </div>
      <div class="flex flex-wrap items-center gap-2 mb-3">
        <div class="font-bold text-[14px]"><i class="fa-solid fa-hexagon-nodes text-fuchsia-300 mr-1.5"></i>Every Seat (${AGENTS.length})</div>
        <select onchange="adminAgentDept=this.value;adminTab('agents')" class="input !w-auto text-[12.5px]"><option value="">All departments</option>${(st.byDepartment || []).map(d => `<option value="${d.department}" ${adminAgentDept === d.department ? 'selected' : ''}>${d.department} (${d.count})</option>`).join('')}</select>
      </div>
      <div class="grid sm:grid-cols-2 xl:grid-cols-3 gap-2.5">
      ${AGENTS.map(a => `<div class="agent-seat">
        <div class="w-9 h-9 rounded-xl bg-gradient-to-br from-fuchsia-500/70 to-violet-600/70 flex items-center justify-center text-[11px] font-bold shrink-0"><i class="fa-solid fa-robot"></i></div>
        <div class="flex-1 min-w-0">
          <div class="font-bold text-[12.5px] truncate">${esc(a.name)}</div>
          <div class="text-[11px] text-white/45 truncate">${esc(a.speciality || a.role)}</div>
        </div>
        <div class="text-right shrink-0">
          <div class="text-[10.5px] text-sky-300 font-bold">${(a.tools || []).length} tools</div>
          <div class="text-[10px] text-white/40">${a.tasksDone} done</div>
        </div>
        <div class="flex flex-col gap-1 shrink-0">
          <button onclick="openToolGranter('${a.id}','agent')" title="Assign tools" class="w-7 h-7 rounded-lg bg-white/5 border border-white/10 hover:border-sky-400/60"><i class="fa-solid fa-toolbox text-[10px] text-sky-300"></i></button>
          <button onclick="toggleAgent('${a.id}')" title="Pause/Resume" class="w-7 h-7 rounded-lg bg-white/5 border border-white/10 hover:border-amber-400/60"><i class="fa-solid fa-power-off text-[10px] text-amber-300"></i></button>
        </div>
      </div>`).join('') || '<div class="sm:col-span-2 xl:col-span-3 glass rounded-3xl p-6 text-center text-white/40">No agents in this department.</div>'}</div>`;
    }

    if (t === 'tools') {
      const cat = await apiGet('api/tools.php', { action: 'catalogue' });
      TOOLS = cat.tools || [];
      const famColor = { software: 'cyan', security: 'rose', ai: 'fuchsia', design: 'violet', business: 'amber', comms: 'sky', platform: 'emerald', agent: 'teal', media: 'orange' };
      const famLabel = { software: 'Software Development', security: 'Cyber Security', ai: 'AI — Fine-Tuning, Chatbots & Custom AI', design: 'Design & Documents', business: 'Business & Tanzania Services', comms: 'Communication', platform: 'Platform', agent: 'Agent Runtime (OpenHands-class)', media: 'Media & Vision' };
      h = `<div class="rounded-2xl border border-sky-400/25 bg-sky-500/[.05] p-4 text-[13px] text-white/70 mb-4">
        <i class="fa-solid fa-toolbox text-sky-300 mr-2"></i>
        Every tool lives inside the platform — <strong>nothing is called from outside</strong>. You grant each tool to a named employee or agent. A worker who was not granted a tool is refused at the endpoint, so no-one works outside their own job.
      </div>
      <div class="grid grid-cols-2 md:grid-cols-4 gap-3 mb-5">
        ${[['Total tools', cat.count], ['Families', (cat.families || []).length], ['Employees', (EMPLOYEES || []).length || '—'], ['Agents', (AGENTS || []).length || 101]].map(([l, v]) => `<div class="card3d pop-sky p-4"><div class="font-display font-bold text-xl">${v}</div><div class="text-[10.5px] text-white/50 font-bold uppercase tracking-wider mt-0.5">${l}</div></div>`).join('')}
      </div>
      <div class="grid gap-4">
      ${(cat.families || []).map(f => `<div class="rounded-2xl border border-white/10 bg-white/[.02] p-4">
        <div class="font-bold text-[13.5px] mb-3"><i class="fa-solid fa-layer-group text-${famColor[f.family] || 'cyan'}-300 mr-1.5"></i>${esc(famLabel[f.family] || f.family)} <span class="text-white/40 font-normal">(${f.tools.length})</span></div>
        <div class="flex flex-wrap gap-2">
        ${f.tools.map(x => `<span class="chip text-${famColor[f.family] || 'cyan'}-300 !text-[11px] !px-3 !py-1.5"><i class="${TOOLS.find(y => y.code === x.code) ? (TOOLS.find(y => y.code === x.code).icon || 'fa-solid fa-gear') : 'fa-solid fa-gear'}"></i>${esc(x.name)}${x.agentic ? ' <i class="fa-solid fa-bolt text-amber-300 ml-1" title="Agentic — runs itself"></i>' : ''}${x.tanzaniaOnly ? ' <span class="text-[9px] text-amber-300 font-bold">TZ</span>' : ''}</span>`).join('')}
        </div>
      </div>`).join('')}
      </div>`;
    }

    if (t === 'ofisi') {
      const [rooms, sms] = await Promise.all([
        apiGet('api/ofisi.php', { action: 'rooms' }),
        apiGet('api/ofisi.php', { action: 'sms_log' }),
      ]);
      const kinds = rooms.kinds || [];
      h = `<div class="rounded-2xl border border-rose-400/25 bg-rose-500/[.05] p-4 text-[13px] text-white/70 mb-4">
        <i class="fa-solid fa-video text-rose-300 mr-2"></i>
        The online office: live video calls, voice calls, screen sharing, online meetings with an AI scribe — and every SMS the platform sent. A request that arrives by SMS is logged here, ready for you to approve and sign.
      </div>
      <div class="flex flex-wrap items-center justify-between gap-2 mb-3">
        <div class="font-bold text-[14px]"><i class="fa-solid fa-users-rectangle text-rose-300 mr-1.5"></i>Rooms (${(rooms.rooms || []).length})</div>
        <button onclick="openRoomEditor()" class="grad-btn px-5 py-2.5 rounded-xl font-bold text-[13px]"><i class="fa-solid fa-plus mr-1"></i> New Room</button>
      </div>
      <div class="grid sm:grid-cols-2 xl:grid-cols-3 gap-3 mb-6">
      ${(rooms.rooms || []).map(r => `<div class="card3d pop-rose p-4">
        <div class="flex justify-between items-start gap-2"><div class="font-bold text-[13.5px]">${esc(r.name)}</div><span class="chip ${r.status === 'live' ? 'text-emerald-300' : 'text-white/60'}">${esc(r.status)}</span></div>
        <div class="text-[11.5px] text-rose-300 mt-1"><i class="fa-solid fa-video mr-1"></i>${esc(r.kind)} • ${r.durationMin} min</div>
        <div class="text-[11.5px] text-white/45 mt-1">Host: ${esc(r.host || '—')} • ${esc(r.startsAt || 'now')}</div>
        <div class="flex flex-wrap gap-1.5 mt-2">${r.scribe ? '<span class="chip text-cyan-300"><i class="fa-solid fa-pen-nib"></i>AI Scribe</span>' : ''}${r.recording ? '<span class="chip text-amber-300"><i class="fa-solid fa-circle-dot"></i>Recording</span>' : ''}<span class="chip text-white/50">${(r.invitees || []).length} invitees</span></div>
        <div class="flex gap-1.5 mt-3">
          <button onclick="openCall('${r.id}')" class="flex-1 py-2 rounded-lg grad-btn text-[12px] font-bold"><i class="fa-solid fa-video mr-1"></i>Join Call</button>
          <button onclick="setRoomStatus('${r.id}','live')" class="flex-1 py-2 rounded-lg bg-emerald-500/10 border border-emerald-400/30 text-[12px] font-bold text-emerald-300"><i class="fa-solid fa-play mr-1"></i>Go Live</button>
          <button onclick="setRoomStatus('${r.id}','ended')" class="px-3 py-2 rounded-lg bg-white/5 border border-white/10 text-[12px] font-bold">End</button>
        </div>
      </div>`).join('') || '<div class="sm:col-span-2 xl:col-span-3 glass rounded-3xl p-6 text-center text-white/40">No rooms yet. Create one to meet a client online.</div>'}</div>
      <div class="font-bold text-[14px] mb-2"><i class="fa-solid fa-comment-sms text-sky-300 mr-1.5"></i>SMS Log (${sms.count})</div>
      <div class="grid gap-2">
      ${(sms.messages || []).map(m => `<div class="flex flex-wrap items-center gap-3 p-3 rounded-xl bg-white/[.02] border border-white/10 text-[12.5px]">
        <span class="chip text-sky-300"><i class="fa-solid fa-phone"></i>${esc(m.phone)}</span>
        <span class="flex-1 min-w-[200px] text-white/75">${esc(m.message)}</span>
        <span class="chip text-white/50">${esc(m.kind)}</span>
        <span class="chip ${m.status === 'queued' ? 'text-amber-300' : 'text-emerald-300'}">${esc(m.status)}</span>
        <span class="text-[11px] text-white/35">${esc(m.createdAt)}</span>
      </div>`).join('') || '<div class="glass rounded-3xl p-6 text-center text-white/40">No SMS yet. They are logged automatically when a request is assigned.</div>'}</div>`;
    }

    if (t === 'ukuaji') {
      const g = await apiGet('api/ofisi.php', { action: 'growth' });
      const regions = await apiGet('api/ofisi.php', { action: 'regions', country: 'Tanzania' });
      h = `<div class="rounded-2xl border border-teal-400/25 bg-teal-500/[.05] p-4 text-[13px] text-white/70 mb-4">
        <i class="fa-solid fa-earth-africa text-teal-300 mr-2"></i>
        Ready to serve <strong>${g.countries} countries</strong> and all <strong>${g.regions} regions</strong> of Tanzania. The bars below count <strong>real requests</strong>, so you see where the work actually comes from — not just what the platform can reach. The Tanzania-only services (BRELA, TRA, e-services) are never shown abroad.
      </div>
      <div class="grid grid-cols-2 md:grid-cols-4 gap-3 mb-5">
        ${[['Countries used', g.activeCountries || 0, 'fa-globe', 'teal'], ['TZ regions used', g.activeRegions || 0, 'fa-map-location-dot', 'cyan'], ['Requests', g.requests, 'fa-inbox', 'violet'], ['Reach', g.countries + ' / ' + g.regions, 'fa-check-double', 'emerald']].map(([l, v, i, c]) => `<div class="card3d pop-${c} p-4"><div class="flex justify-between items-start"><div><div class="font-display font-bold text-xl">${v}</div><div class="text-[10.5px] text-white/50 font-bold uppercase tracking-wider mt-0.5">${l}</div></div><i class="fa-solid ${i} text-${c}-300"></i></div></div>`).join('')}
      </div>
      <div class="grid lg:grid-cols-2 gap-4">
        <div class="rounded-2xl border border-white/10 bg-white/[.02] p-4">
          <div class="font-bold text-[13.5px] mb-3"><i class="fa-solid fa-globe text-teal-300 mr-1.5"></i>Countries the Work Came From</div>
          ${growthBars(g.byCountry, 'country', 'teal')}
        </div>
        <div class="rounded-2xl border border-white/10 bg-white/[.02] p-4">
          <div class="font-bold text-[13.5px] mb-3"><i class="fa-solid fa-map-location-dot text-cyan-300 mr-1.5"></i>Tanzania Regions the Work Came From</div>
          ${growthBars(g.byRegion, 'region', 'cyan')}
          <div class="text-[11.5px] text-white/40 mt-3 pt-3 border-t border-white/10">${(regions.regions || []).length} regions are enabled and ready to serve.</div>
        </div>
      </div>`;
    }

    if (t === 'usalama') {
      const [sec, acct, log] = await Promise.all([
        apiGet('api/security.php', { action: 'status' }),
        apiGet('api/security.php', { action: 'company_account' }),
        apiGet('api/security.php', { action: 'log' }),
      ]);
      const a = acct.account || {};
      h = `<div class="rounded-2xl border border-cyan-400/25 bg-cyan-500/[.05] p-4 text-[13px] text-white/70 mb-4">
        <i class="fa-solid fa-shield-halved text-cyan-300 mr-2"></i>
        Contracts are signed with <strong>ECDSA (secp256k1)</strong>. Change any term after signing and verification fails — that is what makes the signature worth having. Money leaves escrow into the company account below.
      </div>
      <div class="grid md:grid-cols-3 gap-3 mb-5">
        <div class="card3d pop-cyan p-4"><div class="text-[10.5px] text-white/50 font-bold uppercase tracking-wider">Signature</div><div class="font-display font-bold text-lg mt-1">${esc(sec.ecdsa.algorithm)}</div><div class="text-[11.5px] text-emerald-300 mt-1"><i class="fa-solid fa-circle-check mr-1"></i>${sec.ecdsa.enabled ? 'Active' : 'Off'}</div><div class="font-mono text-[10px] text-white/35 mt-2 break-all">${esc((sec.ecdsa.publicKey || '').slice(0, 42))}…</div></div>
        <div class="card3d pop-violet p-4"><div class="text-[10.5px] text-white/50 font-bold uppercase tracking-wider">Quantum readiness</div><div class="font-display font-bold text-lg mt-1">${esc(sec.quantum.signatureScheme)}</div><div class="text-[11.5px] text-white/50 mt-1">${esc(sec.quantum.plan)}</div></div>
        <div class="card3d pop-emerald p-4"><div class="text-[10.5px] text-white/50 font-bold uppercase tracking-wider">Escrow held</div><div class="font-display font-bold text-lg mt-1">${fmtTZS(sec.escrow.held || 0)}</div><div class="text-[11.5px] text-white/50 mt-1">Gateways: ${sec.gateways.enabled}/${sec.gateways.catalogue} on • Stripe: ${sec.gateways.stripeReady ? 'ready' : 'needs key'}</div></div>
      </div>
      <div class="rounded-2xl border border-fuchsia-400/25 bg-fuchsia-500/[.04] p-4 mb-4">
        <div class="flex flex-wrap items-center justify-between gap-3">
          <div>
            <div class="font-bold text-[13.5px]"><i class="fa-solid fa-image text-fuchsia-300 mr-1.5"></i>Company Logo &amp; Brand Name</div>
            <div class="text-[12.5px] text-white/55 mt-1">Upload the logo shown across the whole platform. The current default is used until you replace it.</div>
          </div>
          <button onclick="openLogoEditor()" class="grad-btn px-5 py-3 rounded-xl font-bold text-sm"><i class="fa-solid fa-upload mr-1"></i>Change Logo</button>
        </div>
      </div>
      <div class="rounded-2xl border border-emerald-400/25 bg-emerald-500/[.04] p-4 mb-4">
        <div class="font-bold text-[13.5px] mb-3"><i class="fa-solid fa-building-columns text-emerald-300 mr-1.5"></i>Company Account — where the money settles after escrow</div>
        <div class="grid sm:grid-cols-2 gap-3">
          <div><label class="text-[11px] font-bold text-white/55 uppercase tracking-wider">Account name</label><input id="caName" class="input mt-1" value="${esc(a.name || '')}"></div>
          <div><label class="text-[11px] font-bold text-white/55 uppercase tracking-wider">Bank</label><input id="caBank" class="input mt-1" value="${esc(a.bank || '')}"></div>
          <div><label class="text-[11px] font-bold text-white/55 uppercase tracking-wider">Account number</label><input id="caNo" class="input mt-1 font-mono" value="${esc(a.accountNo || '')}"></div>
          <div><label class="text-[11px] font-bold text-white/55 uppercase tracking-wider">SWIFT</label><input id="caSwift" class="input mt-1 font-mono" value="${esc(a.swift || '')}"></div>
          <div><label class="text-[11px] font-bold text-white/55 uppercase tracking-wider">Mobile money</label><input id="caMobile" class="input mt-1 font-mono" value="${esc(a.mobile || '')}"></div>
          <div class="flex items-end"><button onclick="saveCompanyAccount()" class="grad-btn w-full py-3 rounded-xl font-bold text-sm">Save Company Account</button></div>
        </div>
      </div>
      <div class="rounded-2xl border border-white/10 bg-white/[.02] p-4">
        <div class="font-bold text-[13.5px] mb-3"><i class="fa-solid fa-file-signature text-cyan-300 mr-1.5"></i>Signature Log (${(log.entries || []).length})</div>
        <div class="grid gap-1.5">
        ${(log.entries || []).map(e => `<div class="flex flex-wrap items-center gap-3 p-2.5 rounded-xl bg-white/[.02] border border-white/5 text-[12.5px]">
          <span class="chip ${e.action === 'sign' ? 'text-cyan-300' : 'text-violet-300'}">${esc(e.action)}</span>
          <span class="font-mono text-[11.5px] text-white/60">${esc(e.contractId)}</span>
          <span class="flex-1 text-[11.5px] text-white/40">${esc(e.algorithm)}</span>
          <span class="chip ${e.ok ? 'text-emerald-300' : 'text-red-300'}">${e.ok ? 'valid' : 'FAILED'}</span>
          <span class="text-[11px] text-white/35">${esc(e.createdAt)}</span>
        </div>`).join('') || '<div class="text-white/40 text-[12.5px]">No signatures yet. Sign a contract from the Contracts tab.</div>'}
        </div>
      </div>
      <div class="rounded-2xl border border-cyan-400/25 bg-cyan-500/[.04] p-4 mt-4">
        <div class="flex flex-wrap items-center justify-between gap-3">
          <div>
            <div class="font-bold text-[13.5px]"><i class="fa-solid fa-radar text-cyan-300 mr-1.5"></i>Security Sandbox — the real scanner</div>
            <div class="text-[12.5px] text-white/55 mt-1">The security tools open real connections and read real responses. Nothing is scanned until you authorise the client's domain here, after their written permission.</div>
          </div>
          <button onclick="loadSandbox()" class="px-4 py-2.5 rounded-xl text-[13px] font-bold border border-white/15 hover:bg-white/5"><i class="fa-solid fa-rotate mr-1"></i>Refresh</button>
        </div>
        <div id="sandboxBox" class="mt-4"></div>
      </div>`;
      setTimeout(() => { loadSandbox(); }, 60);
    }

    /* ---- Runtimes: what the platform can actually compile and run ---- */
    if (t === 'runtimes') {
      const [rt, sb] = await Promise.all([
        apiGet('api/runtime.php', { action: 'status' }),
        apiGet('api/runtime.php', { action: 'sandbox' }).catch(() => null),
      ]);
      h = `<div class="rounded-2xl border border-emerald-400/25 bg-emerald-500/[.05] p-4 text-[13px] text-white/75 mb-4">
        <i class="fa-solid fa-microchip text-emerald-300 mr-2"></i>
        The platform compiles and runs code for real. <strong>${(rt.ready || []).length}</strong> runtime(s) are installed and ready.
        Code runs in a scratch directory with no inherited environment and a hard timeout, so a snippet cannot read a key or run forever.
      </div>
      <div class="grid sm:grid-cols-2 lg:grid-cols-3 gap-3">
        ${(rt.languages || []).map(l => `<div class="card3d ${l.available ? 'pop-emerald' : 'pop-amber'} p-4">
          <div class="flex justify-between items-start">
            <div><div class="font-bold text-[14px]">${esc(l.name)}</div><div class="font-mono text-[11px] text-white/45 mt-0.5">${esc(l.binary)}</div></div>
            <span class="chip ${l.available ? 'text-emerald-300' : 'text-amber-300'}"><i class="fa-solid ${l.available ? 'fa-circle-check' : 'fa-triangle-exclamation'}"></i>${l.available ? 'ready' : 'missing'}</span>
          </div>
          ${l.canTest ? `<div class="text-[11.5px] text-cyan-300 mt-2"><i class="fa-solid fa-vial mr-1"></i>Test runner available</div>` : ''}
        </div>`).join('')}
      </div>
      <div class="rounded-2xl border border-white/10 bg-white/[.02] p-4 mt-4">
        <div class="font-bold text-[13.5px] mb-3"><i class="fa-solid fa-play text-cyan-300 mr-1.5"></i>Run a snippet (admin only)</div>
        <div class="grid sm:grid-cols-[160px_1fr] gap-3">
          <select id="rtLang" class="input">${(rt.languages || []).map(l => `<option value="${esc(l.key)}">${esc(l.name)}</option>`).join('')}</select>
          <input id="rtCode" class="input font-mono text-[12.5px]" placeholder="print(6 * 7)">
        </div>
        <button onclick="runSnippet()" class="grad-btn mt-3 px-5 py-2.5 rounded-xl font-bold text-[13px]"><i class="fa-solid fa-bolt mr-1"></i>Run</button>
        <pre id="rtOut" class="mt-3 p-3 rounded-xl bg-black/40 border border-white/10 text-[12px] text-white/80 max-h-72 overflow-auto hidden"></pre>
      </div>`;
    }

    /* ---- Agent Runs: watch every job the company carries out ---- */
    if (t === 'runs') {
      const [runs, st, cap] = await Promise.all([
        apiGet('api/engine.php', { action: 'runs' }),
        apiGet('api/engine.php', { action: 'stats' }),
        apiGet('api/engine.php', { action: 'capabilities' }),
      ]);
      const R = runs.runs || [];
      const statusColor = s => s === 'done' ? 'emerald' : s === 'running' ? 'sky' : s === 'error' ? 'red' : s === 'blocked' ? 'amber' : 'white';
      h = `<div class="rounded-2xl border ${cap.ready ? 'border-emerald-400/25 bg-emerald-500/[.05]' : 'border-amber-400/30 bg-amber-500/[.06]'} p-4 text-[13px] text-white/75 mb-4">
        <i class="fa-solid ${cap.ready ? 'fa-circle-check text-emerald-300' : 'fa-triangle-exclamation text-amber-300'} mr-2"></i>
        ${cap.ready
          ? `The engine is live on <strong>${esc(cap.provider)} / ${esc(cap.model)}</strong>. Hand an agent a job below and watch every step it takes — which tool it used and what came back.`
          : `No AI provider is configured yet, so jobs cannot start. Add a provider key on the <strong>AI Engine</strong> tab and the whole company starts working.`}
      </div>
      <div class="grid grid-cols-2 md:grid-cols-4 gap-3 mb-4">
        ${[['Total runs', st.total, 'fa-list', 'violet'], ['Running', st.running, 'fa-spinner', 'sky'], ['Finished', st.done, 'fa-circle-check', 'emerald'], ['Tools', cap.toolCount, 'fa-toolbox', 'cyan']].map(([l, v, i, c]) => `<div class="card3d pop-${c} p-4"><div class="flex justify-between items-start"><div><div class="font-display font-bold text-xl">${v}</div><div class="text-[10.5px] text-white/50 font-bold uppercase tracking-wider mt-0.5">${l}</div></div><i class="fa-solid ${i} text-${c}-300"></i></div></div>`).join('')}
      </div>
      <div class="rounded-2xl border border-fuchsia-400/25 bg-fuchsia-500/[.04] p-4 mb-4">
        <div class="font-bold text-[13.5px] mb-3"><i class="fa-solid fa-play text-fuchsia-300 mr-1.5"></i>Give the company a job</div>
        <div class="grid sm:grid-cols-3 gap-2">
          <input id="runTask" class="input sm:col-span-3" placeholder="Describe the job — e.g. 'Audit example.com for OWASP Top-10 issues and write the report'">
          <input id="runService" class="input" placeholder="Service (optional) — e.g. Cybersecurity">
          <input id="runTool" class="input" placeholder="Tool (optional) — e.g. owasp_audit">
          <button onclick="runAgentJob()" class="grad-btn py-3 rounded-xl font-bold text-sm"><i class="fa-solid fa-bolt mr-1"></i>Start job</button>
        </div>
        <div class="text-[11.5px] text-white/45 mt-2">Leave the agent blank and the manager (Magma Commander) picks the right agent for the job.</div>
      </div>
      <div class="font-bold text-[14px] mb-2"><i class="fa-solid fa-clock-rotate-left text-emerald-300 mr-1.5"></i>Run history (${R.length})</div>
      <div class="grid gap-2">
      ${R.map(r => `<div class="rounded-xl bg-white/[.02] border border-white/10 p-3.5">
        <div class="flex flex-wrap items-center gap-3 text-[12.5px]">
          <span class="chip text-${statusColor(r.status)}-300">${esc(r.status)}</span>
          <span class="font-mono text-[11px] text-white/50">${esc(r.id)}</span>
          <span class="flex-1 min-w-[180px] font-bold">${esc(r.agentName)}</span>
          <span class="text-[11px] text-white/40">${r.steps} steps • ${esc(r.provider)}/${esc(r.model)}</span>
          <button onclick="viewRun('${r.id}')" class="text-cyan-300 font-bold text-[12px]">Inspect →</button>
        </div>
        <div class="text-[12px] text-white/55 mt-1.5 truncate">${esc((r.task || '').slice(0, 140))}</div>
      </div>`).join('') || '<div class="glass rounded-3xl p-6 text-center text-white/40">No runs yet. Start one above and it appears here immediately.</div>'}
      </div>`;
    }

    /* ---- AI Engine: the provider the whole platform thinks with ---- */
    if (t === 'ai') {
      const s = await apiGet('api/llm.php', { action: 'status' });
      h = `<div class="rounded-2xl border ${s.ready ? 'border-emerald-400/25 bg-emerald-500/[.05]' : 'border-amber-400/30 bg-amber-500/[.06]'} p-4 text-[13px] text-white/75 mb-4">
        <i class="fa-solid ${s.ready ? 'fa-circle-check text-emerald-300' : 'fa-triangle-exclamation text-amber-300'} mr-2"></i>
        ${s.ready ? 'The AI engine is connected. Every agent and every AI tool works through it.' : 'No key yet. Paste a provider key below and the whole company switches on — agents, tools, reports, chatbots and fine-tuning.'}
        ${s.keySet ? ` <span class="chip text-emerald-300 ml-2">key ${esc(s.keyHint)}</span>` : ''}
      </div>
      <div class="rounded-2xl border border-fuchsia-400/25 bg-fuchsia-500/[.04] p-5">
        <div class="font-bold text-[14px] mb-3"><i class="fa-solid fa-microchip text-fuchsia-300 mr-1.5"></i>Provider</div>
        <div class="grid sm:grid-cols-2 gap-3">
          <div><label class="text-[11px] font-bold text-white/55 uppercase tracking-wider">Provider</label>
            <select id="llmProvider" class="input mt-1">
              ${(s.providers || []).map(p => `<option value="${p.key}" ${s.provider === p.key ? 'selected' : ''}>${esc(p.name)}</option>`).join('')}
            </select></div>
          <div><label class="text-[11px] font-bold text-white/55 uppercase tracking-wider">Model</label>
            <input id="llmModel" class="input mt-1 font-mono" value="${esc(s.model || '')}" placeholder="leave blank for the provider default"></div>
          <div><label class="text-[11px] font-bold text-white/55 uppercase tracking-wider">API key ${s.keySet ? '(leave blank to keep the current one)' : ''}</label>
            <input id="llmKey" type="password" class="input mt-1 font-mono" placeholder="${s.keySet ? '•••••••• stored' : 'paste the key'}"></div>
          <div><label class="text-[11px] font-bold text-white/55 uppercase tracking-wider">Base URL (advanced)</label>
            <input id="llmBase" class="input mt-1 font-mono" value="${esc(s.baseUrl || '')}"></div>
          <div><label class="text-[11px] font-bold text-white/55 uppercase tracking-wider">Max tokens</label>
            <input id="llmMax" class="input mt-1" value="${s.maxTokens || 4096}"></div>
          <div><label class="text-[11px] font-bold text-white/55 uppercase tracking-wider">Temperature</label>
            <input id="llmTemp" class="input mt-1" value="${s.temperature}"></div>
        </div>
        <div class="flex flex-wrap gap-2 mt-4">
          <button onclick="saveLlm()" class="grad-btn px-5 py-3 rounded-xl font-bold text-sm"><i class="fa-solid fa-floppy-disk mr-1"></i>Save</button>
          <button onclick="testLlm()" class="px-5 py-3 rounded-xl font-bold text-sm border border-cyan-400/40 text-cyan-200 hover:bg-cyan-500/10"><i class="fa-solid fa-plug-circle-check mr-1"></i>Test connection</button>
          ${s.keySet ? `<button onclick="clearLlmKey()" class="px-5 py-3 rounded-xl font-bold text-sm border border-red-400/30 text-red-300 hover:bg-red-500/10">Remove key</button>` : ''}
        </div>
        <div id="llmResult" class="mt-3"></div>
      </div>
      <div class="rounded-2xl border border-white/10 bg-white/[.02] p-4 mt-4 text-[12.5px] text-white/60">
        <div class="font-bold text-white/80 mb-1.5"><i class="fa-solid fa-lock text-emerald-300 mr-1.5"></i>How the key is kept</div>
        The key is stored in the settings table and is <strong>never</strong> sent back to a page — the form shows only the last four characters. Every provider error is scrubbed of anything key-shaped before it is logged or shown.
      </div>`;
    }

    /* ---- SMS Gateway: make messages actually leave the platform ---- */
    if (t === 'sms') {
      const s = await apiGet('api/ofisi.php', { action: 'sms_config' });
      const log = await apiGet('api/ofisi.php', { action: 'sms_log' });
      h = `<div class="rounded-2xl border ${s.configured ? 'border-emerald-400/25 bg-emerald-500/[.05]' : 'border-amber-400/30 bg-amber-500/[.06]'} p-4 text-[13px] text-white/75 mb-4">
        <i class="fa-solid ${s.configured ? 'fa-circle-check text-emerald-300' : 'fa-triangle-exclamation text-amber-300'} mr-2"></i>
        ${s.configured ? `Messages are sent through <strong>${esc(s.gateway)}</strong>. Every send records what the gateway actually replied.` : 'No gateway configured, so messages are logged but never sent. Fill this in and every SMS the platform produces goes out for real.'}
      </div>
      <div class="rounded-2xl border border-sky-400/25 bg-sky-500/[.04] p-5">
        <div class="font-bold text-[14px] mb-3"><i class="fa-solid fa-comment-sms text-sky-300 mr-1.5"></i>Gateway</div>
        <div class="grid sm:grid-cols-2 gap-3">
          <div><label class="text-[11px] font-bold text-white/55 uppercase tracking-wider">Gateway</label>
            <select id="smsGw" class="input mt-1">
              <option value="">— none —</option>
              ${(s.gateways || []).map(g => `<option value="${g.key}" ${s.gateway === g.key ? 'selected' : ''}>${esc(g.name)}</option>`).join('')}
            </select></div>
          <div><label class="text-[11px] font-bold text-white/55 uppercase tracking-wider">Sender ID / username</label>
            <input id="smsSender" class="input mt-1" value="${esc(s.senderId || '')}" placeholder="MBILINYI"></div>
          <div><label class="text-[11px] font-bold text-white/55 uppercase tracking-wider">API key ${s.keySet ? '(blank keeps the current one)' : ''}</label>
            <input id="smsKey" type="password" class="input mt-1 font-mono" placeholder="${s.keySet ? '•••••••• stored' : 'paste the key'}"></div>
          <div><label class="text-[11px] font-bold text-white/55 uppercase tracking-wider">API secret</label>
            <input id="smsSecret" type="password" class="input mt-1 font-mono" placeholder="Beem and Twilio need this"></div>
          <div class="sm:col-span-2"><label class="text-[11px] font-bold text-white/55 uppercase tracking-wider">Endpoint URL</label>
            <input id="smsBase" class="input mt-1 font-mono" value="${esc(s.baseUrl || '')}" placeholder="https://apisms.beem.africa"></div>
        </div>
        <div class="flex flex-wrap gap-2 mt-4">
          <button onclick="saveSms()" class="grad-btn px-5 py-3 rounded-xl font-bold text-sm"><i class="fa-solid fa-floppy-disk mr-1"></i>Save</button>
          <input id="smsTestPhone" class="input !w-auto flex-1 min-w-[180px]" placeholder="number for a test message">
          <button onclick="testSms()" class="px-5 py-3 rounded-xl font-bold text-sm border border-cyan-400/40 text-cyan-200 hover:bg-cyan-500/10"><i class="fa-solid fa-paper-plane mr-1"></i>Send test</button>
        </div>
        <div id="smsResult" class="mt-3"></div>
      </div>
      <div class="font-bold text-[14px] mb-2 mt-5"><i class="fa-solid fa-list text-sky-300 mr-1.5"></i>Message log (${log.count})</div>
      <div class="grid gap-2">
      ${(log.messages || []).map(m => `<div class="flex flex-wrap items-center gap-3 p-3 rounded-xl bg-white/[.02] border border-white/10 text-[12.5px]">
        <span class="chip text-sky-300"><i class="fa-solid fa-phone"></i>${esc(m.phone)}</span>
        <span class="flex-1 min-w-[200px] text-white/75">${esc(m.message)}</span>
        <span class="chip ${m.status.startsWith('sent') ? 'text-emerald-300' : m.status.startsWith('failed') ? 'text-red-300' : 'text-amber-300'}">${esc(m.status.slice(0, 60))}</span>
        <span class="text-[11px] text-white/35">${esc(m.createdAt)}</span>
      </div>`).join('') || '<div class="glass rounded-3xl p-6 text-center text-white/40">No messages yet.</div>'}
      </div>`;
    }

    if (t === 'settings') {
      const [sd, pl, gwc] = await Promise.all([
        apiGet('api/settings.php', { action: 'list' }),
        apiGet('api/settings.php', { action: 'plans' }),
        apiGet('api/settings.php', { action: 'gateways' }),
      ]);
      const S = sd.settings || {};
      const g = (k, d = '') => (S[k] !== undefined && S[k] !== null ? S[k] : d);
      const money = v => Number(v || 0).toLocaleString('en-US');
      const fld = (k, label, hint) => `<div>
        <label class="text-[11px] font-bold text-white/55 uppercase tracking-wider">${label}</label>
        <input class="input mt-1" data-setkey="${k}" value="${esc(g(k))}" placeholder="${esc(hint || '')}"></div>`;

      h = `<div class="rounded-2xl border border-cyan-400/25 bg-cyan-500/[.05] p-4 text-[13px] text-white/70 mb-4">
        <i class="fa-solid fa-circle-info text-cyan-300 mr-2"></i>
        Everything below is stored in the <strong>settings</strong> table and shown to clients after they sign in. Edit and press Save — changes apply immediately, no code changes needed.
      </div>

      <!-- PAYMENT DETAILS -->
      <div class="rounded-2xl border border-gold/30 overflow-hidden" style="background:linear-gradient(135deg,rgba(245,179,1,.06),transparent)">
        <div class="px-5 py-3.5 border-b border-white/10 font-bold text-[14px]" style="border-color:rgba(245,179,1,.25)">
          <i class="fa-solid fa-wallet text-gold mr-2"></i>Payment Details — shown to every logged-in client
        </div>
        <div class="p-5 grid sm:grid-cols-2 lg:grid-cols-3 gap-3">
          ${fld('pay_bank_name', 'Bank name')}
          ${fld('pay_bank_account', 'Bank account number')}
          ${fld('pay_bank_holder', 'Account holder name')}
          ${fld('pay_bank_branch', 'Branch')}
          ${fld('pay_bank_swift', 'SWIFT code')}
          ${fld('pay_contact_person', 'Payment contact person')}
          ${fld('pay_mobile_network', 'Mobile money network')}
          ${fld('pay_mobile_number', 'Mobile money number')}
          ${fld('pay_mobile_holder', 'Mobile money holder')}
          ${fld('pay_currency', 'Currency')}
          ${fld('fx_usd_tzs', 'Exchange rate — TZS per 1 USD (used when a client pays in the other currency)')}
        </div>
        <div class="px-5 pb-5 grid gap-3">
          <div>
            <label class="text-[11px] font-bold text-white/55 uppercase tracking-wider">Payment instructions (shown under the numbers)</label>
            <textarea class="input mt-1" rows="2" data-setkey="pay_instructions">${esc(g('pay_instructions'))}</textarea>
          </div>
          <div class="rounded-xl bg-black/30 border border-white/10 p-3.5 text-[12.5px] text-white/60">
            <i class="fa-solid fa-hand-holding-dollar text-gold mr-2"></i>
            <strong>Software projects:</strong> 50% deposit before start + 50% mid-project.
            <strong>Other services:</strong> terms agreed in writing first.
          </div>
        </div>
      </div>

      <!-- ONLINE PAYMENT GATEWAYS -->
      <div class="rounded-2xl border border-emerald-400/25 overflow-hidden mt-4" style="background:linear-gradient(135deg,rgba(16,185,129,.06),transparent)">
        <div class="px-5 py-3.5 border-b border-white/10 font-bold text-[14px]" style="border-color:rgba(16,185,129,.25)">
          <i class="fa-solid fa-plug-circle-bolt text-emerald-300 mr-2"></i>Online Payment Gateways — money lands in the company account
        </div>
        <div class="px-5 py-3 text-[12.5px] text-white/60 border-b border-white/5">
          Switch a gateway on and money settles straight into the company account. A gateway cannot be switched on until every credential below it is filled in, so nothing is left half-configured. Clients only see what is switched on.
        </div>
        <div class="p-5 grid gap-3">
          ${(gwc.gateways || []).map(gw => {
            const fields = (gwc.fields || []).filter(f => f.gateway === gw.key);
            return `<div class="rounded-xl border ${gw.enabled ? 'border-emerald-400/40 bg-emerald-500/[.04]' : 'border-white/10 bg-white/[.02]'} p-4">
              <div class="flex flex-wrap items-center justify-between gap-3">
                <div class="flex items-center gap-3">
                  <span class="w-10 h-10 rounded-xl flex items-center justify-center text-lg" style="background:${gw.brand}22;color:${gw.brand}"><i class="${gw.icon}"></i></span>
                  <div>
                    <div class="font-bold text-[13.5px]">${esc(gw.name)}</div>
                    <div class="text-[11.5px] text-white/45">${esc(gw.kind)} • ${(gw.currencies || []).join(', ')}</div>
                  </div>
                </div>
                <div class="flex items-center gap-2">
                  <span class="chip ${gw.configured ? 'text-emerald-300' : 'text-amber-300'}">${gw.configured ? 'credentials set' : 'setup needed'}</span>
                  <button onclick="toggleGateway('${gw.key}', ${gw.enabled ? 0 : 1})" class="${gw.enabled ? 'px-4 py-2 rounded-xl border border-red-400/40 text-red-200 text-[12.5px] font-bold' : 'grad-btn px-4 py-2 rounded-xl text-[12.5px] font-bold'}">${gw.enabled ? 'Switch off' : 'Switch on'}</button>
                </div>
              </div>
              <div class="grid sm:grid-cols-2 lg:grid-cols-3 gap-3 mt-3">
                ${fields.map(f => `<div>
                  <label class="text-[10.5px] font-bold text-white/50 uppercase tracking-wider">${esc(f.key.replace('gw_' + gw.key + '_', '').replace(/_/g, ' '))}${f.secret ? ' (secret)' : ''}</label>
                  <input class="input mt-1 ${f.secret ? 'font-mono text-[12px]' : ''}" data-setkey="${f.key}" ${f.secret ? 'data-secret="1"' : ''} type="${f.secret ? 'password' : 'text'}" value="${esc(f.value || '')}" placeholder="${f.isSet ? '•••• saved — type to replace' : 'not set'}">
                </div>`).join('')}
              </div>
            </div>`;
          }).join('')}
        </div>
      </div>

      <!-- COMPANY -->
      <div class="rounded-2xl border border-white/10 bg-white/[.02] p-5 mt-4">
        <div class="font-bold text-[14px] mb-4"><i class="fa-solid fa-building text-cyan-300 mr-2"></i>Company Information</div>
        <div class="grid sm:grid-cols-2 lg:grid-cols-3 gap-3">
          ${fld('company_location', 'Location')}
          ${fld('company_work_mode', 'How you work')}
          ${fld('company_domain', 'Domain')}
          ${fld('company_email', 'Public email')}
          ${fld('company_phone', 'Public phone')}
        </div>
      </div>

      <!-- MBILINYITECH PLATFORM -->
      <div class="rounded-2xl border border-fuchsia-400/30 bg-fuchsia-500/[.05] p-5 mt-4">
        <div class="flex flex-wrap items-center justify-between gap-2 mb-1">
          <div class="font-bold text-[14px]"><i class="fa-solid fa-diagram-project text-fuchsia-300 mr-2"></i>MbilinyiTech Platform — the "OUR PROJECTS" button</div>
          <span class="badge bg-fuchsia-500/15 text-fuchsia-200">Your control</span>
        </div>
        <div class="text-[12.5px] text-white/55 mb-4">
          Paste the link to your platform here. The <strong>OUR PROJECTS</strong> section on the homepage then sends visitors straight there, where they can see every project.
          Leave it blank and the button is hidden automatically — no dead links.
        </div>
        <div class="grid sm:grid-cols-2 gap-3">
          ${fld('platform_url', 'Platform link (https://…)', 'https://…')}
          ${fld('platform_name', 'Platform name shown on the button', 'MbilinyiTech Platform')}
        </div>
        <div class="mt-3 rounded-xl bg-black/25 border border-white/10 p-3.5 text-[12px] text-white/60">
          <i class="fa-solid fa-circle-info text-fuchsia-300 mr-2"></i>
          Type the full address including <strong>https://</strong>. Press <strong>Save Settings</strong> below, then reload the homepage.
        </div>
      </div>

      <!-- PASSKEYS -->
      <div class="rounded-2xl border border-cyan-400/30 bg-cyan-500/[.05] p-5 mt-4">
        <div class="flex flex-wrap items-center justify-between gap-2 mb-1">
          <div class="font-bold text-[14px]"><i class="fa-solid fa-fingerprint text-cyan-300 mr-2"></i>Passkeys — sign in with a fingerprint</div>
          <span class="badge bg-cyan-500/15 text-cyan-200">Advanced</span>
        </div>
        <div class="text-[12.5px] text-white/55 mb-4">
          Leave both fields blank and the site figures them out from the address visitors use — correct for the live domain. Only fill them in if the site sits behind a proxy that changes the host or scheme, otherwise the browser will refuse every passkey.
        </div>
        <div class="grid sm:grid-cols-3 gap-3">
          ${fld('webauthn_rp_name', 'Name shown in the passkey prompt')}
          ${fld('webauthn_rp_id', 'Relying party ID (bare domain)', 'mbilinyitech.co.tz')}
          ${fld('webauthn_origin', 'Origin (scheme + host)', 'https://mbilinyitech.co.tz')}
        </div>
      </div>

      <div class="mt-4"><button onclick="saveAllSettings()" class="grad-btn px-7 py-3.5 rounded-xl font-bold text-sm"><i class="fa-solid fa-floppy-disk mr-2"></i>Save Settings</button></div>

      <!-- SERVICE VISIBILITY -->
      <div class="rounded-2xl border border-amber-400/30 bg-amber-500/[.05] p-5 mt-5">
        <div class="flex flex-wrap items-center justify-between gap-2 mb-1">
          <div class="font-bold text-[14px]"><i class="fa-solid fa-toggle-on text-amber-300 mr-2"></i>Service Visibility — switch services on or off</div>
          <span class="badge bg-amber-500/15 text-amber-200">Your control</span>
        </div>
        <div class="text-[12.5px] text-white/55 mb-4">
          Switch any service <strong>ON</strong> to show it on the website, or <strong>OFF</strong> to hide it everywhere (services grid, quote wizard &amp; dropdowns).
          <strong class="text-amber-200">Cyber Security and Digital Forensics start switched OFF</strong> — switch them on when you are ready to offer them.
        </div>
        <div class="grid sm:grid-cols-2 lg:grid-cols-3 gap-2.5" id="svcVisGrid">
          ${SERVICES.map(s => {
            const on = (window.__SERVICES_ON__ || {})[s.k] !== false;
            const tz = TZ_ONLY.includes(s.k);
            return `<label class="flex items-center gap-3 rounded-xl bg-black/20 border border-white/10 p-3 cursor-pointer hover:border-amber-400/40">
              <input type="checkbox" class="w-4 h-4 accent-amber-400" data-svckey="${esc(s.k)}" ${on ? 'checked' : ''}>
              <span class="w-8 h-8 rounded-lg bg-white/5 border border-white/10 flex items-center justify-center shrink-0"><i class="fa-solid ${s.icon} text-[13px] text-white/70"></i></span>
              <span class="flex-1"><span class="text-[12.5px] font-bold leading-snug block">${esc(s.t)}</span>
              ${tz ? '<span class="chip text-cyan-300 mt-1">Tanzania only</span>' : ''}</span>
            </label>`;
          }).join('')}
        </div>
        <div class="mt-4 flex flex-wrap items-center gap-3">
          <button onclick="saveServiceVisibility()" class="grad-btn px-6 py-3 rounded-xl font-bold text-sm"><i class="fa-solid fa-floppy-disk mr-2"></i>Save Service Visibility</button>
          <span class="text-[12px] text-white/45">Takes effect on the next page load.</span>
        </div>
        <div class="mt-5 pt-5 border-t border-white/10">
          <div class="font-bold text-[13.5px] mb-1"><i class="fa-solid fa-earth-africa text-cyan-300 mr-2"></i>Tanzania-only services</div>
          <div class="text-[12.5px] text-white/55 mb-3">
            Tick a service to keep it for Tanzania only — visitors from other countries will not see it or be able to order it, but everyone can still buy your international services.
          </div>
          <div class="grid sm:grid-cols-2 lg:grid-cols-3 gap-2.5">
            ${SERVICES.map(s => `<label class="flex items-center gap-3 rounded-xl bg-black/20 border border-white/10 p-3 cursor-pointer hover:border-cyan-400/40">
              <input type="checkbox" class="w-4 h-4 accent-cyan-400" data-tzkey="${esc(s.k)}" ${TZ_ONLY.includes(s.k) ? 'checked' : ''}>
              <span class="text-[12.5px] font-bold leading-snug">${esc(s.t)}</span>
            </label>`).join('')}
          </div>
          <div class="mt-4"><button onclick="saveTzOnly()" class="px-6 py-3 rounded-xl font-bold text-sm border border-cyan-400/40 text-cyan-200 hover:bg-cyan-500/10"><i class="fa-solid fa-floppy-disk mr-2"></i>Save Tanzania-only list</button></div>
        </div>
      </div>

      <!-- SERVICE PRICES -->
      <div class="rounded-2xl border border-violet-400/25 bg-violet-500/[.05] p-5 mt-5">
        <div class="flex flex-wrap items-center justify-between gap-2 mb-1">
          <div class="font-bold text-[14px]"><i class="fa-solid fa-coins text-violet-300 mr-2"></i>Service Prices — pricing &amp; AI chatbot</div>
          <span class="badge bg-violet-500/15 text-violet-200">Editable by you</span>
        </div>
        <div class="text-[12.5px] text-white/55 mb-4">Type a new amount and press <strong>Save Settings</strong>. The pricing page and the chatbot quote the new figure immediately.</div>
        <div class="grid sm:grid-cols-2 lg:grid-cols-3 gap-3">
          ${(sd.servicePrices || []).map(p => `<div class="rounded-xl bg-black/20 border border-white/10 p-3.5">
            <div class="text-[12.5px] text-white/75 min-h-[34px] leading-snug">${esc(p.en)}</div>
            <div class="text-[11px] text-white/40 mb-1.5">${esc(p.sw)}</div>
            <div class="flex items-center gap-2">
              <span class="text-[12px] font-bold text-violet-300 shrink-0">TZS</span>
              <input class="input !py-1.5 !text-[13px] font-mono" type="number" min="0" step="10000"
                     data-setkey="price_${esc(p.code)}" value="${Math.round(Number(p.price) || 0)}">
            </div>
            <div class="text-[10.5px] text-white/35 mt-1">per ${esc(String(p.period).replace('/', ''))}</div>
          </div>`).join('')}
        </div>
        <div class="mt-4"><button onclick="saveAllSettings()" class="px-6 py-3 rounded-xl font-bold text-sm border border-violet-400/40 text-violet-200 hover:bg-violet-500/10"><i class="fa-solid fa-floppy-disk mr-2"></i>Save Service Prices</button></div>
      </div>

      <!-- PRICES -->
      <div class="rounded-2xl border border-emerald-400/25 bg-emerald-500/[.04] p-5 mt-5">
        <div class="flex flex-wrap items-center justify-between gap-2 mb-4">
          <div>
            <div class="font-bold text-[14px]"><i class="fa-solid fa-tags text-emerald-300 mr-2"></i>Public Price List — pricing</div>
            <div class="text-[12.5px] text-white/55">Change any price below and the public pricing page updates instantly.</div>
          </div>
          <a href="pricing" target="_blank" class="text-[12px] font-bold px-4 py-2 rounded-full border border-white/15 hover:bg-white/5">Open pricing page ↗</a>
        </div>
        <div class="overflow-x-auto rounded-xl border border-white/10">
          <table class="w-full data min-w-[720px]">
            <tr><th>Plan</th><th>Group</th><th>Price (TZS)</th><th>Period</th><th class="!text-right">Actions</th></tr>
            ${(pl.plans || []).map(p => `<tr data-plangroup="${esc(p.group)}">
              <td><input class="input !py-1.5 !text-[13px]" data-planname="${esc(p.id)}" value="${esc(p.name)}"></td>
              <td><span class="badge ${p.group === 'sec' ? 'bg-red-500/15 text-red-300' : 'bg-emerald-500/15 text-emerald-300'}">${p.group === 'sec' ? 'Security' : 'Maintenance'}</span></td>
              <td><input class="input !py-1.5 !text-[13px] font-mono" type="number" min="0" step="1000" data-planprice="${esc(p.id)}" value="${Number(p.price || 0)}"><div class="text-[10.5px] text-white/40 mt-0.5">TZS ${money(p.price)}</div></td>
              <td><input class="input !py-1.5 !text-[13px]" data-planperiod="${esc(p.id)}" value="${esc(p.period || '')}"></td>
              <td class="!text-right"><button onclick="savePlanPrice('${esc(p.id)}')" class="grad-btn px-4 py-2 rounded-lg text-[12px] font-bold">Save</button></td>
            </tr>`).join('')}
          </table>
        </div>
        <div class="mt-4"><button onclick="saveAllPlans()" class="px-6 py-3 rounded-xl font-bold text-sm border border-emerald-400/40 text-emerald-200 hover:bg-emerald-500/10"><i class="fa-solid fa-check-double mr-2"></i>Save All Prices</button></div>
      </div>`;
    }

    if (t === 'feedback') {
      const fb = await apiGet('api/settings.php', { action: 'feedback_list' }).catch(() => ({ feedback: [] }));
      const list = fb.feedback || [];
      const badge = s => ({ New: 'status-Pending', Read: 'status-Quoted', Replied: 'status-Completed', Archived: 'text-white/50' }[s] || 'status-Pending');
      h = `<div class="flex flex-wrap items-center justify-between gap-2 mb-4">
        <div class="text-[13px] text-white/55"><i class="fa-solid fa-comment-dots text-violet-300 mr-1.5"></i>Messages sent from the public <strong>feedback</strong> page. Mark them as you handle them.</div>
        <a href="feedback" target="_blank" class="text-[12px] font-bold px-4 py-2 rounded-full border border-white/15 hover:bg-white/5">Open feedback page ↗</a>
      </div>
      <div class="grid gap-3">${list.map(x => `<div class="rounded-2xl border border-white/10 bg-white/[.02] p-5">
        <div class="flex flex-wrap items-start justify-between gap-3">
          <div class="min-w-0">
            <div class="flex flex-wrap items-center gap-2">
              <span class="font-bold text-[15px]">${esc(x.name || 'Anonymous')}</span>
              ${x.email ? `<a href="mailto:${esc(x.email)}" class="text-[12px] text-cyan-300 underline">${esc(x.email)}</a>` : ''}
              <span class="badge bg-violet-500/15 text-violet-200">${esc(x.category)}</span>
              <span class="badge ${badge(x.status)}">${esc(x.status)}</span>
            </div>
            ${x.subject ? `<div class="text-[13.5px] font-bold mt-1.5">${esc(x.subject)}</div>` : ''}
            <div class="flex gap-1 text-gold text-[11px] mt-1.5">${'<i class="fa-solid fa-star"></i>'.repeat(Math.max(0, Math.min(5, Number(x.rating) || 0)))}${Number(x.rating) ? '' : '<span class="text-white/35">No rating</span>'}</div>
            <p class="text-[13.5px] text-white/70 mt-2 leading-relaxed whitespace-pre-line">${esc(x.message)}</p>
            <div class="text-[11px] text-white/40 mt-2">${fmtDate(x.createdAt)}</div>
          </div>
          <div class="flex flex-wrap gap-1.5 shrink-0">
            ${['New', 'Read', 'Replied', 'Archived'].map(s => `<button onclick="setFeedbackStatus('${esc(x.id)}','${s}')" class="px-3 py-1.5 rounded-lg text-[11px] font-bold ${x.status === s ? 'grad-btn' : 'bg-white/5 border border-white/10 hover:border-cyan-400/40'}">${s}</button>`).join('')}
            <a href="${x.email ? `mailto:${esc(x.email)}` : 'https://wa.me/255796752645'}" target="_blank" class="px-3 py-1.5 rounded-lg text-[11px] font-bold bg-[#25D366]/10 border border-[#25D366]/30 text-[#4be584]">Reply</a>
          </div>
        </div>
      </div>`).join('') || '<div class="text-white/40 glass rounded-3xl p-8 text-center">No feedback yet. Share <strong>feedback</strong> with clients to start collecting reviews.</div>'}</div>`;
    }

    if (t === 'email') {
      /* Email / SMTP Configuration — iframe the setup page so admin never leaves the panel */
      h = `<div class="rounded-2xl border border-teal-400/25 bg-teal-500/[.05] p-5">
        <div class="flex items-center gap-3 mb-4">
          <div class="w-12 h-12 rounded-xl bg-teal-500/20 flex items-center justify-center"><i class="fa-solid fa-envelope-gear text-teal-300 text-xl"></i></div>
          <div><div class="font-bold text-lg">Email / SMTP Configuration</div>
            <div class="text-[13px] text-white/55">Enter your Gmail App Password once — this page updates <code>.env</code>, pushes to Supabase, and sends a test email. No manual file editing.</div></div>
        </div>
        <iframe src="/setup" style="width:100%;height:700px;border:none;border-radius:12px;background:#050914" title="Email Setup"></iframe>
      </div>`;
    }

    $('adminContent').innerHTML = h;
  } catch (e) {
    $('adminContent').innerHTML = '';
    toast(esc(e.message), 'err');
  }
}

/* admin actions */
async function saveAllSettings() {
  const settings = {};
  document.querySelectorAll('[data-setkey]').forEach(i => {
    // A blank secret box means "leave the stored key alone", never "erase it".
    if (i.dataset.secret === '1' && !i.value.trim()) return;
    settings[i.dataset.setkey] = i.value;
  });
  try {
    await apiPost('api/settings.php', { action: 'save_all', settings });
    toast(`Saved ${Object.keys(settings).length} settings ✓ — clients see them immediately.`, 'ok');
    // Re-render so "credentials set / setup needed" reflects what was just saved.
    if (aTab === 'settings') adminTab('settings');
  } catch (e) { toast(esc(e.message), 'err'); }
}

/* Switching a gateway on runs the server's own completeness check, so the
   reason a gateway refuses to start is the server's, not a guess made here. */
async function toggleGateway(key, on) {
  try {
    await apiPost('api/settings.php', { action: 'gateway_toggle', gateway: key, enabled: on ? '1' : '0' });
    toast(on ? 'Gateway switched on ✓ — clients can now use it.' : 'Gateway switched off.', 'ok');
    adminTab('settings');
  } catch (e) { toast(esc(e.message), 'err'); }
}
/* Service visibility is stored as one comma-separated `services_hidden` key. */
async function saveServiceVisibility() {
  const hidden = [...document.querySelectorAll('[data-svckey]')]
    .filter(c => !c.checked)
    .map(c => c.dataset.svckey);
  try {
    await apiPost('api/settings.php', { action: 'save', key: 'services_hidden', value: hidden.join(',') });
    toast('Service visibility saved ✓ — reload the page to see it applied.', 'ok');
  } catch (e) { toast(esc(e.message), 'err'); }
}
async function saveTzOnly() {
  const tz = [...document.querySelectorAll('[data-tzkey]')]
    .filter(c => c.checked)
    .map(c => c.dataset.tzkey);
  try {
    await apiPost('api/settings.php', { action: 'save', key: 'services_tz_only', value: tz.join(',') });
    toast('Tanzania-only services saved ✓ — visitors abroad will not see them.', 'ok');
    TZ_ONLY.length = 0; tz.forEach(k => TZ_ONLY.push(k));
  } catch (e) { toast(esc(e.message), 'err'); }
}
async function savePlanPrice(id) {
  const n = document.querySelector(`[data-planname="${CSS.escape(id)}"]`);
  const p = document.querySelector(`[data-planprice="${CSS.escape(id)}"]`);
  const d = document.querySelector(`[data-planperiod="${CSS.escape(id)}"]`);
  const row = n ? n.closest('[data-plangroup]') : null;
  const grp = row ? row.dataset.plangroup : '';
  if (!n || !p) return;
  try {
    await apiPost('api/settings.php', { action: 'plan_save', id, name: n.value.trim(), price: Number(p.value || 0), period: (d ? d.value : '').trim(), group: grp });
    toast(`Price updated ✓ — "${n.value.trim()}" now TZS ${Number(p.value || 0).toLocaleString('en-US')}`, 'ok');
    await loadPublicPrices();
  } catch (e) { toast(esc(e.message), 'err'); }
}
async function saveAllPlans() {
  const ids = [...document.querySelectorAll('[data-planprice]')].map(i => i.dataset.planprice);
  if (!ids.length) return;
  try {
    for (const id of ids) {
      const n = document.querySelector(`[data-planname="${CSS.escape(id)}"]`);
      const p = document.querySelector(`[data-planprice="${CSS.escape(id)}"]`);
      const d = document.querySelector(`[data-planperiod="${CSS.escape(id)}"]`);
      const row = n ? n.closest('[data-plangroup]') : null;
      const grp = row ? row.dataset.plangroup : '';
      if (!n || !p) continue;
      await apiPost('api/settings.php', { action: 'plan_save', id, name: n.value.trim(), price: Number(p.value || 0), period: (d ? d.value : '').trim(), group: grp });
    }
    toast('All prices saved ✓ — the pricing page now shows them.', 'ok');
    await loadPublicPrices();
  } catch (e) { toast(esc(e.message), 'err'); }
}
async function setFeedbackStatus(id, status) {
  try {
    await apiPost('api/settings.php', { action: 'feedback_status', id, status });
    adminTab('feedback');
  } catch (e) { toast(esc(e.message), 'err'); }
}

/**
 * Overlay the admin-maintained price list onto the built-in MAINT / SEC arrays
 * and re-render. Fails soft: if the API is unreachable the page keeps showing
 * the built-in prices, so the pricing page never renders empty.
 */
async function loadPublicPrices() {
  try {
    const r = await apiGet('api/settings.php', { action: 'plans' });
    const plans = r.plans || [];
    if (!plans.length) return;
    const byName = {};
    plans.forEach(p => { byName[String(p.name).trim().toLowerCase()] = p; });
    const fmt = v => 'TZS ' + Number(v || 0).toLocaleString('en-US');
    let changed = false;
    [...MAINT, ...SEC].forEach(m => {
      const p = byName[String(m.n).trim().toLowerCase()];
      if (!p) return;
      m.p = fmt(p.price);
      if (p.period) m.per = p.period;
      m.__price = Number(p.price || 0);
      changed = true;
    });
    if (changed) renderPricing();
  } catch (e) { /* keep the built-in list */ }
}

async function setReqStatus(id, s) {
  try { await apiPost('api/requests.php', { action: 'set_status', id, status: s }); } catch (e) { toast(esc(e.message), 'err'); return; }
  toast('Status updated → ' + s, 'info');
  adminTab('requests');
}
async function delRequest(id) {
  if (!confirm('Delete this request + linked docs?')) return;
  try { await apiPost('api/requests.php', { action: 'delete', id }); } catch (e) { toast(esc(e.message), 'err'); return; }
  toast('Request deleted.', 'info');
  adminTab('requests');
}

/* ---------- Portfolio admin CRUD ---------- */
async function openPortfolioEditor(id) {
  let p = { id: 'new', title: '', category: '', imageUrl: '', description: '', stat: '', link: '', sortOrder: 0 };
  if (id !== 'new') {
    const found = PORTFOLIO_DB.find(x => String(x.id ?? x.ID ?? '') === String(id));
    if (!found) { toast('Project not found.', 'err'); return; }
    p = {
      id: found.id ?? id,
      title: found.t || found.title || '',
      category: found.c || found.category || '',
      imageUrl: found.img || found.imageUrl || found.image_url || '',
      description: found.d || found.description || '',
      stat: found.s || found.stat || '',
      link: found.link || found.linkUrl || found.url || '',
      sortOrder: found.sortOrder ?? found.sort_order ?? 0,
    };
  }
  openGen(id === 'new' ? 'Add Portfolio Project' : `Edit Project — ${p.title || id}`, `
    <div class="grid sm:grid-cols-2 gap-3">
      <div class="sm:col-span-2"><label class="text-[12px] font-bold text-white/60">TITLE *</label><input id="pfTitle" class="input mt-1" value="${esc(p.title)}" placeholder="e.g. ShulePay Fees Portal"></div>
      <div><label class="text-[12px] font-bold text-white/60">CATEGORY</label><input id="pfCat" class="input mt-1" value="${esc(p.category)}" placeholder="e.g. Education • React + Node + SMS"></div>
      <div><label class="text-[12px] font-bold text-white/60">STAT / RESULT TAG</label><input id="pfStat" class="input mt-1" value="${esc(p.stat)}" placeholder="e.g. Multi-branch stock control, OWASP hardening"></div>
      <div class="sm:col-span-2"><label class="text-[12px] font-bold text-white/60">IMAGE URL * (recommended 800x600)</label><input id="pfImg" class="input mt-1 font-mono text-[12px]" value="${esc(p.imageUrl)}" placeholder="https://.../photo.jpg or a data URI"></div>
      <div class="sm:col-span-2"><label class="text-[12px] font-bold text-white/60">OPTIONAL EXTERNAL LINK</label><input id="pfLink" class="input mt-1 font-mono text-[12px]" value="${esc(p.link)}" placeholder="https:// (optional — live project URL)"></div>
      <div class="sm:col-span-2"><label class="text-[12px] font-bold text-white/60">DESCRIPTION *</label><textarea id="pfDesc" rows="3" class="input mt-1">${esc(p.description)}</textarea></div>
      <div><label class="text-[12px] font-bold text-white/60">SORT ORDER</label><input id="pfSort" type="number" class="input mt-1" value="${Number(p.sortOrder) || 0}"></div>
    </div>
    <div class="flex gap-2 mt-5"><button onclick="savePortfolioEdit('${id}')" class="grad-btn flex-1 py-3 rounded-xl font-bold text-sm">${id === 'new' ? 'Create Project' : 'Save Changes'}</button><button onclick="closeGen()" class="px-5 py-3 rounded-xl border border-white/15 font-bold text-sm">Cancel</button></div>
  `);
}
async function savePortfolioEdit(id) {
  const payload = {
    action: id === 'new' ? 'add' : 'update',
    title: $('pfTitle').value.trim(),
    category: $('pfCat').value.trim(),
    image_url: $('pfImg').value.trim(),
    description: $('pfDesc').value.trim(),
    stat: $('pfStat').value.trim(),
    link: $('pfLink').value.trim(),
    sort_order: Number($('pfSort').value) || 0,
  };
  if (!payload.title) { toast('Title is required.', 'err'); return; }
  if (id !== 'new') payload.id = id;
  try { await apiPost('api/portfolio.php', payload); } catch (e) { toast(esc(e.message), 'err'); return; }
  closeGen(); adminTab('portfolio'); toast(id === 'new' ? 'Project added ✓' : 'Project updated ✓', 'ok');
  loadPortfolioAndTestimonials(true);
}
async function delPortfolio(id) {
  if (!confirm('Delete this portfolio project permanently?')) return;
  try { await apiPost('api/portfolio.php', { action: 'delete', id }); } catch (e) { toast(esc(e.message), 'err'); return; }
  adminTab('portfolio'); toast('Project deleted.', 'info');
  loadPortfolioAndTestimonials(true);
}

/* ---------- Testimonials admin CRUD ---------- */
async function openTestimonialEditor(id) {
  let t3 = { id: 'new', name: '', role: '', quote: '', stars: 5, sortOrder: 0 };
  if (id !== 'new') {
    const found = TESTIS_DB.find(x => String(x.id ?? '') === String(id));
    if (!found) { toast('Testimonial not found.', 'err'); return; }
    t3 = {
      id: found.id ?? id,
      name: found.n || found.name || '',
      role: found.r || found.role || '',
      quote: found.t || found.quote || '',
      stars: Math.max(1, Math.min(5, Number(found.s || found.stars) || 5)),
      sortOrder: found.sort ?? found.sortOrder ?? 0,
    };
  }
  openGen(id === 'new' ? 'Add Testimonial' : `Edit Testimonial — ${t3.name || id}`, `
    <div class="grid sm:grid-cols-2 gap-3">
      <div><label class="text-[12px] font-bold text-white/60">NAME *</label><input id="tmName" class="input mt-1" value="${esc(t3.name)}" placeholder="e.g. Amina Juma"></div>
      <div><label class="text-[12px] font-bold text-white/60">ROLE / COMPANY</label><input id="tmRole" class="input mt-1" value="${esc(t3.role)}" placeholder="e.g. Director, Amina Pharmacy — Dodoma"></div>
      <div class="sm:col-span-2"><label class="text-[12px] font-bold text-white/60">STARS</label><select id="tmStars" class="input mt-1">${[5,4,3,2,1].map(n => `<option ${t3.stars===n?'selected':''}>${n}</option>`).join('')}</select></div>
      <div class="sm:col-span-2"><label class="text-[12px] font-bold text-white/60">QUOTE *</label><textarea id="tmQuote" rows="4" class="input mt-1">${esc(t3.quote)}</textarea></div>
      <div><label class="text-[12px] font-bold text-white/60">SORT ORDER</label><input id="tmSort" type="number" class="input mt-1" value="${Number(t3.sortOrder) || 0}"></div>
    </div>
    <div class="flex gap-2 mt-5"><button onclick="saveTestimonialEdit('${id}')" class="grad-btn flex-1 py-3 rounded-xl font-bold text-sm">${id === 'new' ? 'Publish Testimonial' : 'Save Changes'}</button><button onclick="closeGen()" class="px-5 py-3 rounded-xl border border-white/15 font-bold text-sm">Cancel</button></div>
  `);
}
async function saveTestimonialEdit(id) {
  const payload = {
    action: id === 'new' ? 'add' : 'update',
    name: $('tmName').value.trim(),
    role: $('tmRole').value.trim(),
    quote: $('tmQuote').value.trim(),
    stars: Math.max(1, Math.min(5, Number($('tmStars').value) || 5)),
    sort_order: Number($('tmSort').value) || 0,
  };
  if (!payload.name || !payload.quote) { toast('Name and quote are required.', 'err'); return; }
  if (id !== 'new') payload.id = id;
  try { await apiPost('api/testimonials.php', payload); } catch (e) { toast(esc(e.message), 'err'); return; }
  closeGen(); adminTab('testimonials'); toast(id === 'new' ? 'Testimonial published ✓' : 'Testimonial updated ✓', 'ok');
  loadPortfolioAndTestimonials(true);
}
async function delTestimonial(id) {
  if (!confirm('Delete this testimonial permanently?')) return;
  try { await apiPost('api/testimonials.php', { action: 'delete', id }); } catch (e) { toast(esc(e.message), 'err'); return; }
  adminTab('testimonials'); toast('Testimonial deleted.', 'info');
  loadPortfolioAndTestimonials(true);
}

/* ---------- News admin CRUD ---------- */
const NEWS_CATEGORIES = ['Announcement', 'New Service', 'Product Update', 'Event', 'Case Study', 'Tender', 'Other'];

async function openNewsEditor(id) {
  let n = { id: 'new', title: '', category: 'Announcement', excerpt: '', body: '', imageUrl: '' };
  if (id !== 'new') {
    try { n = (await apiGet('api/news.php', { action: 'get', id })).post; } catch (e) { toast(esc(e.message), 'err'); return; }
  }
  const cats = NEWS_CATEGORIES.includes(n.category) ? NEWS_CATEGORIES : [n.category, ...NEWS_CATEGORIES];
  openGen(id === 'new' ? 'Add News Post' : `Edit News — ${n.title || id}`, `
    <div class="grid sm:grid-cols-2 gap-3">
      <div class="sm:col-span-2"><label class="text-[12px] font-bold text-white/60">TITLE *</label><input id="nwTitle" class="input mt-1" value="${esc(n.title)}" placeholder="e.g. New AI fine-tuning service now available"></div>
      <div><label class="text-[12px] font-bold text-white/60">CATEGORY</label><select id="nwCat" class="input mt-1">${cats.map(c => `<option ${n.category === c ? 'selected' : ''}>${esc(c)}</option>`).join('')}</select></div>
      <div><label class="text-[12px] font-bold text-white/60">IMAGE URL (optional)</label><input id="nwImg" class="input mt-1 font-mono text-[12px]" value="${esc(n.imageUrl)}" placeholder="https://.../photo.jpg"></div>
      <div class="sm:col-span-2"><label class="text-[12px] font-bold text-white/60">SHORT SUMMARY (leave blank to auto-generate)</label><input id="nwExcerpt" class="input mt-1" value="${esc(n.excerpt)}" placeholder="One-line summary shown on the card"></div>
      <div class="sm:col-span-2"><label class="text-[12px] font-bold text-white/60">FULL ARTICLE *</label><textarea id="nwBody" rows="7" class="input mt-1">${esc(n.body)}</textarea></div>
    </div>
    <div class="flex gap-2 mt-5"><button onclick="saveNewsEdit('${id}')" class="grad-btn flex-1 py-3 rounded-xl font-bold text-sm">${id === 'new' ? 'Publish Post' : 'Save Changes'}</button><button onclick="closeGen()" class="px-5 py-3 rounded-xl border border-white/15 font-bold text-sm">Cancel</button></div>
  `);
}
async function saveNewsEdit(id) {
  const payload = {
    action: id === 'new' ? 'add' : 'update',
    title: $('nwTitle').value.trim(),
    category: $('nwCat').value,
    excerpt: $('nwExcerpt').value.trim(),
    body: $('nwBody').value.trim(),
    image_url: $('nwImg').value.trim(),
  };
  if (!payload.title || !payload.body) { toast('Title and article body are required.', 'err'); return; }
  if (id !== 'new') payload.id = id;
  try { await apiPost('api/news.php', payload); } catch (e) { toast(esc(e.message), 'err'); return; }
  closeGen(); adminTab('news'); toast(id === 'new' ? 'News post published ✓' : 'News post updated ✓', 'ok');
}
async function toggleNews(id) {
  try { await apiPost('api/news.php', { action: 'toggle', id }); } catch (e) { toast(esc(e.message), 'err'); return; }
  adminTab('news'); toast('Publish status changed.', 'info');
}
async function delNews(id) {
  if (!confirm('Delete this news post permanently?')) return;
  try { await apiPost('api/news.php', { action: 'delete', id }); } catch (e) { toast(esc(e.message), 'err'); return; }
  adminTab('news'); toast('News post deleted.', 'info');
}

/* ---------- Team admin CRUD ---------- */
async function openTeamEditor(id) {
  let m = { id: 'new', name: '', role: '', roleSw: '', bio: '', bioSw: '', imageUrl: '', email: '', phone: '', linkedin: '', sortOrder: 0 };
  if (id !== 'new') {
    await loadNewsAndTeam(true);
    const found = TEAM_DB.find(x => x.id === id);
    if (found) m = found;
  }
  openGen(id === 'new' ? 'Add Team Member' : `Edit Member — ${m.name || id}`, `
    <div class="grid sm:grid-cols-2 gap-3">
      <div><label class="text-[12px] font-bold text-white/60">FULL NAME *</label><input id="tmName" class="input mt-1" value="${esc(m.name)}" placeholder="e.g. Jackson Mbilinyi"></div>
      <div><label class="text-[12px] font-bold text-white/60">ROLE / TITLE</label><input id="tmRole" class="input mt-1" value="${esc(m.role)}" placeholder="e.g. CEO &amp; Founder"></div>
      <div class="sm:col-span-2"><label class="text-[12px] font-bold text-white/60">PHOTO URL (optional)</label><input id="tmImg" class="input mt-1 font-mono text-[12px]" value="${esc(m.imageUrl)}" placeholder="https://.../photo.jpg"></div>
      <div><label class="text-[12px] font-bold text-white/60">ROLE / TITLE (KISWAHILI)</label><input id="tmRoleSw" class="input mt-1" value="${esc(m.roleSw)}" placeholder="mf. Mkurugenzi Mtendaji"></div>
      <div class="sm:col-span-2"><label class="text-[12px] font-bold text-white/60">SHORT BIO (ENGLISH)</label><textarea id="tmBio" rows="3" class="input mt-1" placeholder="One or two sentences about this person">${esc(m.bio)}</textarea></div>
      <div class="sm:col-span-2"><label class="text-[12px] font-bold text-white/60">SHORT BIO (KISWAHILI)</label><textarea id="tmBioSw" rows="3" class="input mt-1" placeholder="Sentensi moja au mbili kuhusu mtu huyu">${esc(m.bioSw)}</textarea></div>
      <div><label class="text-[12px] font-bold text-white/60">EMAIL</label><input id="tmEmail" class="input mt-1" value="${esc(m.email)}" placeholder="mbilinyitech@gmail.com"></div>
      <div><label class="text-[12px] font-bold text-white/60">PHONE</label><input id="tmPhone" class="input mt-1" value="${esc(m.phone)}" placeholder="07XX XXX XXX"></div>
      <div><label class="text-[12px] font-bold text-white/60">LINKEDIN URL</label><input id="tmLinkedin" class="input mt-1 font-mono text-[12px]" value="${esc(m.linkedin)}" placeholder="https://linkedin.com/in/..."></div>
      <div><label class="text-[12px] font-bold text-white/60">SORT ORDER</label><input id="tmSort" type="number" class="input mt-1" value="${Number(m.sortOrder) || 0}"></div>
    </div>
    <div class="flex gap-2 mt-5"><button onclick="saveTeamEdit('${id}')" class="grad-btn flex-1 py-3 rounded-xl font-bold text-sm">${id === 'new' ? 'Add Member' : 'Save Changes'}</button><button onclick="closeGen()" class="px-5 py-3 rounded-xl border border-white/15 font-bold text-sm">Cancel</button></div>
  `);
}
async function saveTeamEdit(id) {
  const payload = {
    action: id === 'new' ? 'add' : 'update',
    name: $('tmName').value.trim(),
    role: $('tmRole').value.trim(),
    role_sw: $('tmRoleSw').value.trim(),
    bio: $('tmBio').value.trim(),
    bio_sw: $('tmBioSw').value.trim(),
    image_url: $('tmImg').value.trim(),
    email: $('tmEmail').value.trim(),
    phone: $('tmPhone').value.trim(),
    linkedin: $('tmLinkedin').value.trim(),
    sort_order: Number($('tmSort').value) || 0,
  };
  if (!payload.name) { toast('Name is required.', 'err'); return; }
  if (id !== 'new') payload.id = id;
  try { await apiPost('api/team.php', payload); } catch (e) { toast(esc(e.message), 'err'); return; }
  closeGen(); adminTab('team'); toast(id === 'new' ? 'Team member added ✓' : 'Team member updated ✓', 'ok');
}
async function toggleTeam(id) {
  try { await apiPost('api/team.php', { action: 'toggle', id }); } catch (e) { toast(esc(e.message), 'err'); return; }
  adminTab('team'); toast('Visibility changed.', 'info');
}
async function delTeam(id) {
  if (!confirm('Remove this team member permanently?')) return;
  try { await apiPost('api/team.php', { action: 'delete', id }); } catch (e) { toast(esc(e.message), 'err'); return; }
  adminTab('team'); toast('Team member removed.', 'info');
}

/* full edit of a request (admin) */
async function openRequestEditor(id) {
  let r;
  try { r = (await apiGet('api/requests.php', { action: 'get', id })).request; } catch (e) { toast(esc(e.message), 'err'); return; }
  const services = VISIBLE_SERVICES.map(s => `<option ${r.service === s.t ? 'selected' : ''}>${s.t}</option>`).join('');
  const extra = SERVICES.some(s => s.t === r.service) ? '' : `<option selected>${esc(r.service)}</option>`;
  openGen(`Edit Request — ${r.trackingId}`, `
    <label class="text-[12px] font-bold text-white/60">SERVICE</label><select id="reService" class="input mt-1">${extra}${services}</select>
    <label class="text-[12px] font-bold text-white/60 mt-3 block">PROJECT TITLE</label><input id="reTitle" class="input mt-1" value="${esc(r.title)}">
    <label class="text-[12px] font-bold text-white/60 mt-3 block">REQUIREMENTS</label><textarea id="reDesc" rows="4" class="input mt-1">${esc(r.description)}</textarea>
    ${Object.keys(r.serviceDetails || {}).length ? `<div class="mt-3 rounded-xl border border-cyan-400/25 bg-cyan-500/[.05] p-3"><div class="text-[11px] font-bold text-cyan-300 mb-1.5">CLIENT'S SERVICE-SPECIFIC ANSWERS</div><div class="grid gap-1 text-[12.5px]">${Object.entries(r.serviceDetails).map(([k, v]) => `<div class="flex gap-2"><span class="text-white/40 font-bold uppercase text-[10.5px] mt-0.5">${esc(k)}</span><span class="text-white/85">${esc(v)}</span></div>`).join('')}</div></div>` : ''}
    <div class="grid sm:grid-cols-3 gap-3 mt-3"><div><label class="text-[12px] font-bold text-white/60">PRIORITY</label><select id="rePriority" class="input mt-1"><option ${r.priority === 'Normal' ? 'selected' : ''}>Normal</option><option ${r.priority === 'Urgent' ? 'selected' : ''}>Urgent</option><option ${r.priority === 'Critical / ASAP' ? 'selected' : ''}>Critical / ASAP</option></select></div><div><label class="text-[12px] font-bold text-white/60">BUDGET</label><input id="reBudget" class="input mt-1" value="${esc(r.budget)}"></div><div><label class="text-[12px] font-bold text-white/60">DEADLINE</label><input id="reDeadline" class="input mt-1" value="${esc(r.deadline || '')}" placeholder="YYYY-MM-DD"></div></div>
    <label class="text-[12px] font-bold text-white/60 mt-3 block">STATUS</label><select id="reStatus" class="input mt-1">${['Pending', 'Under Review', 'Quoted', 'Approved', 'In Progress', 'Completed', 'Rejected'].map(s => `<option ${r.status === s ? 'selected' : ''}>${s}</option>`).join('')}</select>
    <label class="text-[12px] font-bold text-white/60 mt-3 block">CEO NOTE (client will see it)</label><textarea id="reNote" rows="2" class="input mt-1">${esc(r.adminNote || '')}</textarea>
    <div class="flex gap-2 mt-4"><button onclick="saveRequestEdit('${id}')" class="grad-btn flex-1 py-3 rounded-xl font-bold text-sm">Save Changes</button><button onclick="closeGen()" class="px-5 py-3 rounded-xl border border-white/15 font-bold text-sm">Cancel</button></div>`);
}
async function saveRequestEdit(id) {
  try {
    await apiPost('api/requests.php', {
      action: 'admin_update', id,
      service: $('reService').value, title: $('reTitle').value.trim(), description: $('reDesc').value.trim(),
      priority: $('rePriority').value, budget: $('reBudget').value.trim(), deadline: $('reDeadline').value.trim(),
      status: $('reStatus').value, adminNote: $('reNote').value.trim(),
    });
  } catch (e) { toast(esc(e.message), 'err'); return; }
  closeGen(); adminTab('requests'); toast('Request updated ✓', 'ok');
}

/* invoice create / edit / delete (admin) */
async function openInvoiceEditor(id) {
  let v = { id: 'new', title: '', amount: 0, trackingId: '', method: 'M-Pesa / Bank', status: 'Unpaid', dueDate: new Date(Date.now() + 14 * 864e5).toISOString() };
  let trackingIds = [];
  try {
    const [invData, reqData] = await Promise.all([
      apiGet('api/invoices.php', { action: 'admin_list' }),
      apiGet('api/requests.php', { action: 'admin_list', q: '', status: 'All' }),
    ]);
    trackingIds = reqData.requests.map(r => r.trackingId);
    if (id !== 'new') v = invData.invoices.find(x => x.id === id) || v;
  } catch (e) { toast(esc(e.message), 'err'); return; }
  const options = trackingIds.map(t => `<option ${t === v.trackingId ? 'selected' : ''}>${t}</option>`).join('');
  openGen(id === 'new' ? 'New Invoice' : `Edit Invoice ${v.id}`, `
    <label class="text-[12px] font-bold text-white/60">TITLE *</label><input id="ivTitle" class="input mt-1" value="${esc(v.title)}" placeholder="e.g. Phase 1 — 50% Deposit">
    <div class="grid sm:grid-cols-2 gap-3 mt-3"><div><label class="text-[12px] font-bold text-white/60">AMOUNT (TZS) *</label><input id="ivAmount" type="number" min="0" step="1000" class="input mt-1 font-mono" value="${v.amount}"></div><div><label class="text-[12px] font-bold text-white/60">TRACKING ID</label><input id="ivTrack" class="input mt-1 font-mono" list="trackList" value="${esc(v.trackingId)}" placeholder="MTS-2026-XXXX"><datalist id="trackList">${options}</datalist></div></div>
    <div class="grid sm:grid-cols-3 gap-3 mt-3"><div><label class="text-[12px] font-bold text-white/60">DUE DATE</label><input id="ivDue" type="date" class="input mt-1" value="${(v.dueDate || '').substring(0, 10)}"></div><div><label class="text-[12px] font-bold text-white/60">STATUS</label><select id="ivStatus" class="input mt-1"><option ${v.status === 'Unpaid' ? 'selected' : ''}>Unpaid</option><option ${v.status === 'Paid' ? 'selected' : ''}>Paid</option></select></div><div><label class="text-[12px] font-bold text-white/60">METHOD</label><input id="ivMethod" class="input mt-1" value="${esc(v.method || 'M-Pesa / Bank')}"></div></div>
    <div class="flex gap-2 mt-4"><button onclick="saveInvoice('${id}')" class="grad-btn flex-1 py-3 rounded-xl font-bold text-sm">${id === 'new' ? 'Create Invoice' : 'Save Invoice'}</button><button onclick="closeGen()" class="px-5 py-3 rounded-xl border border-white/15 font-bold text-sm">Cancel</button></div>`);
}
async function saveInvoice(id) {
  const title = $('ivTitle').value.trim();
  const amount = parseFloat($('ivAmount').value);
  if (!title) { toast('Invoice title is required.', 'err'); return; }
  if (!isFinite(amount) || amount <= 0) { toast('Amount must be greater than zero.', 'err'); return; }
  try {
    await apiPost('api/invoices.php', {
      action: 'admin_save', id, title, amount,
      trackingId: $('ivTrack').value.trim().toUpperCase(),
      dueDate: $('ivDue').value, status: $('ivStatus').value, method: $('ivMethod').value.trim(),
    });
  } catch (e) { toast(esc(e.message), 'err'); return; }
  closeGen(); adminTab('invoices'); toast(id === 'new' ? 'Invoice created ✓' : 'Invoice updated ✓', 'ok');
}
async function delInvoice(id) {
  if (!confirm('Delete this invoice?')) return;
  try { await apiPost('api/invoices.php', { action: 'admin_delete', id }); } catch (e) { toast(esc(e.message), 'err'); return; }
  adminTab('invoices'); toast('Invoice deleted.', 'info');
}
async function adminDeleteTicket(id) {
  if (!confirm('Delete this ticket and all replies?')) return;
  try { await apiPost('api/tickets.php', { action: 'admin_delete', id }); } catch (e) { toast(esc(e.message), 'err'); return; }
  adminTab('tickets'); toast('Ticket deleted.', 'info');
}
async function adminNote(id) {
  let r;
  try { r = (await apiGet('api/requests.php', { action: 'get', id })).request; } catch (e) { toast(esc(e.message), 'err'); return; }
  openGen('CEO Note — ' + r.trackingId, `<textarea id="noteTxt" rows="3" class="input">${esc(r.adminNote || '')}</textarea><button onclick="saveNote('${id}')" class="grad-btn w-full mt-3 py-3 rounded-xl font-bold text-sm">Save Note</button>`);
}
async function saveNote(id) {
  try { await apiPost('api/requests.php', { action: 'note', id, note: $('noteTxt').value }); } catch (e) { toast(esc(e.message), 'err'); return; }
  closeGen(); adminTab('requests'); toast('Note saved ✓', 'ok');
}
async function delQuote(id) {
  if (!confirm('Delete quotation?')) return;
  try { await apiPost('api/quotes.php', { action: 'delete', id }); } catch (e) { toast(esc(e.message), 'err'); return; }
  adminTab('quotes');
}
async function delContract(id) {
  /* Contracts are permanent legal records — deletion is intentionally blocked. */
  toast(T('Contracts are permanent and cannot be deleted.', 'Mkataba hauwezi kufutwa — ni rekodi ya kisheria inayohifadhiwa milele kwa mteja na kampuni.'), 'info');
}
async function markPaid(id) {
  try { await apiPost('api/invoices.php', { action: 'mark_paid', id }); } catch (e) { toast(esc(e.message), 'err'); return; }
  adminTab('invoices'); toast('Invoice marked PAID ✓', 'ok');
}
async function toggleUserStatus(id) {
  try { const d = await apiPost('api/users.php', { action: 'toggle_status', id }); toast('Status → ' + d.status, 'info'); } catch (e) { toast(esc(e.message), 'err'); return; }
  adminTab('users');
}
async function toggleUserRole(id) {
  try { const d = await apiPost('api/users.php', { action: 'toggle_role', id }); toast('Role → ' + d.role, 'info'); } catch (e) { toast(esc(e.message), 'err'); return; }
  adminTab('users');
}
async function delUser(id) {
  if (!confirm('Delete this account permanently?')) return;
  try { await apiPost('api/users.php', { action: 'delete', id }); } catch (e) { toast(esc(e.message), 'err'); return; }
  adminTab('users'); toast('Account deleted.', 'info');
}
async function impersonate(id) {
  try {
    const d = await apiPost('api/users.php', { action: 'impersonate', id });
    window.__ME__ = d.user;
  } catch (e) { toast(esc(e.message), 'err'); return; }
  location.href = 'client';
}
async function toggleTicket(id) {
  try { await apiPost('api/tickets.php', { action: 'toggle', id }); } catch (e) { toast(esc(e.message), 'err'); return; }
  adminTab('tickets');
}
async function adminReply(id) {
  const v = ($('arep-' + id)?.value || '').trim();
  if (!v) return;
  try { await apiPost('api/tickets.php', { action: 'admin_reply', id, text: v }); } catch (e) { toast(esc(e.message), 'err'); return; }
  adminTab('tickets'); toast('Reply sent ✓', 'ok');
}

async function openUserEditor(id) {
  let u = { name: '', email: '', phone: '', company: '', role: 'client', status: 'Active', id: 'new' };
  if (id !== 'new') {
    try {
      const d = await apiGet('api/users.php', { action: 'list', q: '' });
      u = d.users.find(x => x.id === id) || u;
    } catch (e) { toast(esc(e.message), 'err'); return; }
  }
  openGen(id === 'new' ? 'Add New Account' : 'Edit Account — ' + esc(u.name), `
    <div class="grid sm:grid-cols-2 gap-3"><div><label class="text-[12px] font-bold text-white/60">FULL NAME</label><input id="ueName" class="input mt-1" value="${esc(u.name)}"></div><div><label class="text-[12px] font-bold text-white/60">PHONE</label><input id="uePhone" class="input mt-1" value="${esc(u.phone)}"></div></div>
    <div class="grid sm:grid-cols-2 gap-3 mt-3"><div><label class="text-[12px] font-bold text-white/60">EMAIL</label><input id="ueEmail" class="input mt-1" value="${esc(u.email)}"></div><div><label class="text-[12px] font-bold text-white/60">COMPANY</label><input id="ueCompany" class="input mt-1" value="${esc(u.company || '')}"></div></div>
    <div class="grid sm:grid-cols-3 gap-3 mt-3"><div><label class="text-[12px] font-bold text-white/60">ROLE</label><select id="ueRole" class="input mt-1"><option ${u.role === 'client' ? 'selected' : ''}>client</option><option ${u.role === 'admin' ? 'selected' : ''}>admin</option></select></div><div><label class="text-[12px] font-bold text-white/60">STATUS</label><select id="ueStatus" class="input mt-1"><option ${u.status === 'Active' ? 'selected' : ''}>Active</option><option ${u.status === 'Suspended' ? 'selected' : ''}>Suspended</option></select></div><div><label class="text-[12px] font-bold text-white/60">NEW PASSWORD</label><input id="uePass" class="input mt-1" placeholder="(leave blank)"></div></div>
    <button onclick="saveUserEditor('${id}')" class="grad-btn w-full mt-4 py-3.5 rounded-xl font-bold">Save Account</button>`);
}

async function saveUserEditor(id) {
  const n = $('ueName').value.trim(), e = $('ueEmail').value.trim(), p = $('uePhone').value.trim();
  if (!n || !e) { toast('Name & email required.', 'err'); return; }
  try {
    await apiPost('api/users.php', {
      action: 'save', id, name: n, email: e, phone: p,
      company: $('ueCompany').value.trim(), role: $('ueRole').value, status: $('ueStatus').value,
      password: $('uePass').value,
    });
  } catch (err) { toast(esc(err.message), 'err'); return; }
  closeGen(); adminTab('users'); toast('Account saved ✓', 'ok');
}

async function quoteToContract(rid) {
  try {
    const d = await apiPost('api/contracts.php', { action: 'create', requestId: rid });
    toast(d.created ? 'Contract + 50% deposit invoice generated ✓' : 'Contract already exists ✓', 'ok');
  } catch (e) { toast(esc(e.message), 'err'); return; }
  adminTab('contracts');
}

/* QUOTE BUILDER */
let qbItems = [];
async function openQuoteBuilder(rid) {
  let r, ex = null;
  try {
    r = (await apiGet('api/requests.php', { action: 'get', id: rid })).request;
    const quotes = (await apiGet('api/quotes.php', { action: 'admin_list' })).quotes;
    ex = quotes.find(q => q.requestId === rid) || null;
  } catch (e) { toast(esc(e.message), 'err'); return; }

  qbItems = ex ? JSON.parse(JSON.stringify(ex.items)) : [{ desc: 'System Analysis & Design', qty: 1, price: 300000 }, { desc: 'Development & Implementation', qty: 1, price: 1500000 }];
  openGen(`Set Price — ${r.trackingId}`, `
    <div class="text-[13px] text-white/55">Client: <strong class="text-white">${esc(r.name)}</strong> • ${esc(r.title)} • Budget hint: ${esc(r.budget)}</div>
    <div id="qbRows" class="grid gap-2 mt-3"></div>
    <button onclick="qbAdd()" class="mt-2 text-[13px] font-bold text-cyan-300">+ Add line item</button>
    <div class="grid sm:grid-cols-2 gap-3 mt-3"><div><label class="text-[12px] font-bold text-white/60">VALID UNTIL</label><input id="qbValid" type="date" class="input mt-1" value="${ex && ex.validUntil ? ex.validUntil.substring(0, 10) : new Date(Date.now() + 14 * 864e5).toISOString().substring(0, 10)}"></div><div><label class="text-[12px] font-bold text-white/60">STATUS</label><select id="qbStatus" class="input mt-1"><option>Sent</option><option>Draft</option></select></div></div>
    <label class="text-[12px] font-bold text-white/60 mt-3 block">MESSAGE TO CLIENT</label><textarea id="qbMsg" rows="2" class="input mt-1" placeholder="Explain pricing, timeline, what's included...">${ex ? esc(ex.adminMessage || '') : ''}</textarea>
    <label class="mt-3 flex items-start gap-2.5 rounded-xl border border-cyan-400/25 bg-cyan-500/[.05] p-3 cursor-pointer"><input type="checkbox" id="qbTerms" checked class="mt-0.5 w-4 h-4 accent-cyan-400"><span class="text-[12.5px] text-white/70"><strong class="text-white">Apply standard 50/50 payment terms</strong><br><span class="text-white/50">Deposit 50% kabla mradi kuanza + 50% mid-project (kati ya mradi). Itaongezwa kwenye ujumbe kwa mteja.</span></span></label>
    <div id="qbTotal" class="mt-3 p-4 rounded-2xl bg-white/[.03] border border-white/10 text-right"></div>
    <div class="flex gap-2 mt-3"><button onclick="qbSave('${rid}')" class="grad-btn flex-1 py-3.5 rounded-xl font-bold text-sm"><i class="fa-solid fa-paper-plane mr-2"></i>Send Official Quotation</button><button onclick="closeGen()" class="px-5 py-3.5 rounded-xl border border-white/15 font-bold text-sm">Cancel</button></div>`);
  paintQb();
}
function paintQb() {
  $('qbRows').innerHTML = qbItems.map((it, i) => `<div class="grid grid-cols-[1fr_70px_130px_36px] gap-2"><input oninput="qbItems[${i}].desc=this.value" class="input !py-2.5 text-[13px]" value="${esc(it.desc)}" placeholder="Item description"><input type="number" min="1" oninput="qbItems[${i}].qty=+this.value||1;paintQbTotal()" class="input !py-2.5 text-[13px]" value="${it.qty}"><input type="number" min="0" step="1000" oninput="qbItems[${i}].price=+this.value||0;paintQbTotal()" class="input !py-2.5 text-[13px] font-mono" value="${it.price}"><button onclick="qbItems.splice(${i},1);paintQb()" class="rounded-xl bg-red-500/10 border border-red-400/25 text-red-300"><i class="fa-solid fa-trash text-xs"></i></button></div>`).join('');
  paintQbTotal();
}
function paintQbTotal() {
  const sub = qbItems.reduce((s, i) => s + i.qty * i.price, 0), vat = Math.round(sub * .18);
  if ($('qbTotal')) $('qbTotal').innerHTML = `<div class="text-white/55 text-[13px]">Subtotal ${fmtTZS(sub)} • VAT 18% ${fmtTZS(vat)}</div><div class="font-display font-bold text-2xl grad-text">Total ${fmtTZS(sub + vat)}</div>`;
}
function qbAdd() { qbItems.push({ desc: '', qty: 1, price: 0 }); paintQb(); }

const QB_TERMS_NOTICE = '\n\n— PAYMENT TERMS (50/50): 50% Deposit inalipwa KABLA mradi kuanza; 50% Mid-Project inalipwa mradi ukifika nusu ya timeline (kabla ya delivery). Si Deposit + Delivery. Late payment: 2% kwa wiki.';

async function qbSave(rid) {
  if (!qbItems.length || qbItems.some(i => !i.desc.trim())) { toast('Describe every line item.', 'err'); return; }
  let msg = $('qbMsg').value;
  const applyTerms = $('qbTerms') && $('qbTerms').checked;
  if (applyTerms && msg.indexOf('PAYMENT TERMS (50/50)') === -1) msg += QB_TERMS_NOTICE;
  if (!applyTerms) msg = msg.split(QB_TERMS_NOTICE).join('');
  try {
    await apiPost('api/quotes.php', {
      action: 'admin_save', requestId: rid, items: qbItems,
      validUntil: $('qbValid').value, status: $('qbStatus').value, adminMessage: msg,
    });
  } catch (e) { toast(esc(e.message), 'err'); return; }
  closeGen(); adminTab('quotes'); toast('Quotation sent to client ✓', 'ok');
}

/* ---------- VOICE (speech in + speech out) ---------- */
/* The assistant can be heard and can be talked to. Both directions use the
 * browser's own Web Speech API, so nothing extra is downloaded and no audio
 * ever leaves the device. */
let voiceOn = false;
try { voiceOn = localStorage.getItem('mts_voice') === 'on'; } catch (e) {}

function stripForSpeech(html) {
  const tmp = document.createElement('div');
  tmp.innerHTML = String(html)
    .replace(/<br\s*\/?>/gi, '. ')
    .replace(/<\/(p|div|li|h[1-6])>/gi, '. ');
  tmp.querySelectorAll('a').forEach(a => { a.textContent = a.textContent.replace(/[→➜]/g, '') + '. '; });
  return (tmp.textContent || '')
    .replace(/[\u{1F300}-\u{1FAFF}\u{2600}-\u{27BF}\u{FE0F}\u{2190}-\u{21FF}]/gu, '')
    .replace(/https?:\/\/\S+/g, '')
    .replace(/\s*\.\s*\./g, '.')
    .replace(/\s+/g, ' ')
    .trim();
}

function speechLang() { return LANG === 'sw' ? 'sw-TZ' : 'en-GB'; }

function pickVoice() {
  if (!('speechSynthesis' in window)) return null;
  const voices = window.speechSynthesis.getVoices() || [];
  const want = LANG === 'sw' ? ['sw'] : ['en-GB', 'en-US', 'en'];
  for (const tag of want) {
    const v = voices.find(x => (x.lang || '').toLowerCase().startsWith(tag.toLowerCase()));
    if (v) return v;
  }
  return voices[0] || null;
}

function speak(html) {
  if (!voiceOn || !('speechSynthesis' in window)) return;
  const text = stripForSpeech(html);
  if (!text) return;
  window.speechSynthesis.cancel();
  const u = new SpeechSynthesisUtterance(text);
  const v = pickVoice();
  if (v) u.voice = v;
  u.lang = v?.lang || speechLang();
  u.rate = 0.98;
  u.pitch = 1;
  window.speechSynthesis.speak(u);
}

function stopSpeaking() { if ('speechSynthesis' in window) window.speechSynthesis.cancel(); }

function paintVoiceBtn() {
  const b = $('voiceBtn');
  if (!b) return;
  b.className = voiceOn
    ? 'w-8 h-8 rounded-full bg-white/40 ring-2 ring-white/60'
    : 'w-8 h-8 rounded-full bg-white/15';
  b.innerHTML = `<i class="fa-solid ${voiceOn ? 'fa-volume-high' : 'fa-volume-xmark'} text-sm"></i>`;
  b.title = voiceOn ? 'Voice ON — tap to mute / Sauti IMEWASHWA' : 'Voice OFF — tap to enable / Sauti IMEZIMWA';
}

function toggleVoice() {
  if (!('speechSynthesis' in window)) { toast(T('Your browser does not support voice.', 'Kivinjari chako hakitumii sauti.'), 'err'); return; }
  voiceOn = !voiceOn;
  try { localStorage.setItem('mts_voice', voiceOn ? 'on' : 'off'); } catch (e) {}
  paintVoiceBtn();
  if (voiceOn) {
    toast(LANG === 'sw' ? 'Sauti imewashwa ✓' : 'Voice enabled ✓', 'ok');
    speak(LANG === 'sw' ? 'Sauti imewashwa. Naweza kukujibu kwa sauti sasa.' : 'Voice is on. I will read my answers aloud now.');
  } else {
    stopSpeaking();
    toast(LANG === 'sw' ? 'Sauti imezimwa' : 'Voice muted', 'info');
  }
}

/* Speech-to-text: hold the mic, speak, and the transcript lands in the input. */
let recognizer = null, listening = false;

function paintMicBtn() {
  const b = $('micBtn');
  if (!b) return;
  b.className = listening
    ? 'w-11 h-11 rounded-xl shrink-0 bg-red-500/30 border border-red-400/70 animate-pulse'
    : 'w-11 h-11 rounded-xl shrink-0 bg-white/5 border border-white/15';
  b.innerHTML = `<i class="fa-solid ${listening ? 'fa-microphone-lines' : 'fa-microphone'} text-sm"></i>`;
}

function toggleMic() {
  const SR = window.SpeechRecognition || window.webkitSpeechRecognition;
  if (!SR) { toast(LANG === 'sw' ? 'Kivinjari chako hakitumii kusikiliza kwa sauti.' : 'Speech input is not supported in this browser.', 'err'); return; }
  if (listening) { try { recognizer.stop(); } catch (e) {} return; }

  recognizer = new SR();
  recognizer.lang = speechLang();
  recognizer.interimResults = false;
  recognizer.maxAlternatives = 1;

  recognizer.onstart = () => {
    listening = true;
    paintMicBtn();
    stopSpeaking();
    toast(LANG === 'sw' ? 'Ongea sasa… 🎙️' : 'Listening… speak now 🎙️', 'info');
  };
  recognizer.onresult = (ev) => {
    const said = ev.results?.[0]?.[0]?.transcript || '';
    if (said.trim()) { $('chatInput').value = said.trim(); sendChat(); }
  };
  recognizer.onerror = (ev) => {
    listening = false; paintMicBtn();
    if (ev.error === 'not-allowed') toast(LANG === 'sw' ? 'Ruhusa ya maikrofoni imekataliwa.' : 'Microphone permission was denied.', 'err');
    else if (ev.error !== 'aborted') toast(LANG === 'sw' ? 'Sikukusikia. Jaribu tena.' : 'I did not catch that. Try again.', 'err');
  };
  recognizer.onend = () => { listening = false; paintMicBtn(); };
  try { recognizer.start(); } catch (e) { toast('Microphone busy. Try again.', 'err'); }
}

/* Voices load asynchronously in some browsers. */
if ('speechSynthesis' in window) {
  window.speechSynthesis.onvoiceschanged = () => { pickVoice(); };
}

/* ---------- CHATBOT ---------- */
let chatOpened = false;
function toggleChat() {
  const w = $('chatWindow'); w.classList.toggle('hidden-chat');
  if (!w.classList.contains('hidden-chat')) {
    if (!chatOpened) {
      chatOpened = true;
      const sw = LANG === 'sw';
      const intro = sw
        ? `Habari! 👋 Mimi ni <strong>Mbilinyi AI</strong> — msaidizi wa <strong>Mbilinyi Tech Solutions</strong> (CEO Jackson Mbilinyi).<br><br>Naweza kukusaidia kuhusu <strong>huduma, bei, ufuatiliaji, matengenezo, usalama na AI</strong> — kwa Kiswahili. Naweza pia <strong>kuongea kwa sauti</strong> 🎙️ na <strong>kusikiliza</strong> swali lako. Unahitaji nini leo?`
        : `Hello! 👋 I am <strong>Mbilinyi AI</strong> — the assistant of <strong>Mbilinyi Tech Solutions</strong> (CEO Jackson Mbilinyi).<br><br>I can help with <strong>services, prices, tracking, maintenance, security and AI</strong> — in English. I can also <strong>speak my answers aloud</strong> 🎙️ and <strong>listen</strong> to your question. What do you need today?`;
      botSay(intro);
      renderQuick(sw
        ? ['💰 Bei', '🛠 Omba Huduma', '📦 Fuatilia Oda', '🔒 Usalama', '🤖 AI & Chatbots', '📞 Mawasiliano']
        : ['💰 Prices', '🛠 Request Service', '📦 Track Order', '🔒 Security', '🤖 AI & Chatbots', '📞 Contact']);
      paintVoiceBtn();
      paintMicBtn();
    }
  } else {
    stopSpeaking();
  }
  setTimeout(() => $('chatInput').focus(), 300);
}
/* ---------- CHATBOT (Ollama Cloud AI + legacy fallback) ---------- */
let chatHistory = [];
let chatOllamaState = 'unknown'; // 'enabled' | 'disabled' | 'unknown'

function renderQuick(arr) { $('chatQuick').innerHTML = arr.map(q => `<button onclick="quickChat('${q}')" class="text-[11.5px] font-bold px-3 py-1.5 rounded-full bg-white/5 border border-white/15 hover:border-cyan-400/50">${q}</button>`).join(''); }
function quickChat(q) { $('chatInput').value = q.replace(/^[^\s]+\s/, ''); sendChat(); }
function userSay(t) { $('chatBody').insertAdjacentHTML('beforeend', `<div class="flex justify-end"><div class="chat-bubble-me px-4 py-2.5 max-w-[85%]">${t}</div></div>`); $('chatBody').scrollTop = 99999; }
function botSay(html) {
  $('chatBody').insertAdjacentHTML('beforeend', `<div class="flex gap-2"><div class="w-8 h-8 rounded-full grad-btn flex items-center justify-center shrink-0 text-sm"><i class="fa-solid fa-robot"></i></div><div class="chat-bubble-ai px-4 py-2.5 max-w-[85%]">${html}</div></div>`);
  $('chatBody').scrollTop = 99999;
  speak(html);
}
function botTyping() {
  return new Promise(resolve => {
    const id = 'tp' + Date.now();
    $('chatBody').insertAdjacentHTML('beforeend', `<div id="${id}" class="flex gap-2"><div class="w-8 h-8 rounded-full grad-btn flex items-center justify-center shrink-0 text-sm"><i class="fa-solid fa-robot"></i></div><div class="chat-bubble-ai px-4 py-3 typing flex gap-1"><span></span><span></span><span></span></div></div>`);
    $('chatBody').scrollTop = 99999;
    setTimeout(() => { document.getElementById(id)?.remove(); resolve(); }, 700 + Math.random() * 600);
  });
}
async function sendChat() {
  const inp = $('chatInput'); const raw = inp.value.trim(); if (!raw) return;
  userSay(esc(raw)); inp.value = '';

  chatHistory.push({ role: 'user', content: raw });
  if (chatHistory.length > 30) chatHistory = chatHistory.slice(-30);

  // Always run track ID lookup first (guaranteed accuracy via API on server)
  const trackMatch = raw.toUpperCase().match(/MTS-\d{4}-\d{3,4}/);
  if (trackMatch) {
    await botTyping();
    try {
      const d = await apiGet('api/track.php', { id: trackMatch[0] });
      const r = d.request, q = d.quote;
      const reply = `📦 <strong>${r.trackingId}</strong> — <strong>${esc(r.title)}</strong><br>Status: <strong>${r.status}</strong>${q ? `<br>Quotation: <strong>${fmtTZS(q.total)}</strong> (${q.status})` : ''}<br>${r.adminNote ? `${T('CEO update', 'Taarifa kutoka kwa CEO')}: ${esc(r.adminNote)}<br>` : ''}<a href="track" class="text-cyan-300 underline font-bold">${T('Open full timeline →', 'Fungua ratiba kamili →')}</a>`;
      chatHistory.push({ role: 'assistant', content: reply.replace(/<[^>]+>/g, ' ') });
      botSay(reply);
      return;
    } catch (e) {
      const reply = `Sikupata <strong>${trackMatch[0]}</strong>. Hakikisha ID ni sahihi au WhatsApp 0796752645. Track IDs look like <span class="font-mono">MTS-2026-XXXX</span>.`;
      chatHistory.push({ role: 'assistant', content: reply.replace(/<[^>]+>/g, ' ') });
      botSay(reply);
      return;
    }
  }

  // Check Ollama state once
  if (chatOllamaState === 'unknown') {
    try {
      const cfg = await apiGet('api/ollama.php', { action: 'config' });
      chatOllamaState = cfg.enabled ? 'enabled' : 'disabled';
    } catch { chatOllamaState = 'disabled'; }
  }

  await botTyping();

  if (chatOllamaState === 'enabled') {
    try {
      const res = await apiPost('api/ollama.php', { action: 'chat', messages: chatHistory });
      const text = String(res.reply || '');
      const html = esc(text).replace(/\n\n/g, '<br><br>').replace(/\n/g, '<br>')
        .replace(/(https?:\/\/[^\s]+)/g, '<a href="$1" target="_blank" class="text-cyan-300 underline">$1</a>')
        .replace(/(WhatsApp|0796\s?\d{3}\s?\d{3}|\+255\s?\d{3}\s?\d{3}\s?\d{3})/g, '<a href="https://wa.me/255796752645" target="_blank" class="text-emerald-300 font-bold underline">$1</a>');
      chatHistory.push({ role: 'assistant', content: text });
      botSay(html);
      return;
    } catch (e) {
      chatOllamaState = 'disabled'; // one-time fallback, do not retry this session
    }
  }

  // Fallback: legacy brain() (hardcoded rules — always works)
  const reply = await brain(raw);
  chatHistory.push({ role: 'assistant', content: String(reply).replace(/<[^>]+>/g, ' ') });
  botSay(reply);
}

async function brain(raw) {
  const t = raw.toLowerCase();
  const trackMatch = raw.toUpperCase().match(/MTS-\d{4}-\d{3,4}/);
  if (trackMatch) {
    try {
      const d = await apiGet('api/track.php', { id: trackMatch[0] });
      const r = d.request, q = d.quote;
      return `📦 <strong>${r.trackingId}</strong> — <strong>${esc(r.title)}</strong><br>Status: <strong>${r.status}</strong>${q ? `<br>Quotation: <strong>${fmtTZS(q.total)}</strong> (${q.status})` : ''}<br>${r.adminNote ? `${T('CEO update', 'Taarifa kutoka kwa CEO')}: ${esc(r.adminNote)}<br>` : ''}<a href="track" class="text-cyan-300 underline font-bold">${T('Open full timeline →', 'Fungua ratiba kamili →')}</a>`;
    } catch (e) {
      return `Sikupata <strong>${trackMatch[0]}</strong>. Hakikisha ID ni sahihi au WhatsApp 0796752645. Track IDs look like <span class="font-mono">MTS-2026-XXXX</span>.`;
    }
  }
  /* Identity questions are answered as themselves — never with a price list. */
  if (/(wewe ni nani|wewe ni nini|nani wewe|unaitwa nani|jina lako|kuhusu wewe|wewe ni|who are you|who r u|what are you|whats your name|what'?s your name|your name|about you|introduce yourself|tell me about yourself)/.test(t)) {
    const sw = /\b(wewe|nani|jina|kuhusu|unaitwa|nini|habari|naomba)\b/.test(t);
    return sw
      ? `Mimi ni <strong>Mbilinyi AI</strong> 🤖 — msaidizi rasmi wa <strong>Mbilinyi Tech Solutions</strong> (BRELA Reg. 706668), kampuni ya Jackson Mbilinyi iliyoko Dodoma, Tanzania.<br><br>Ninawasaidia wateja kuelewa <strong>huduma zetu</strong> (software maalum, uchambuzi wa mifumo, ushauri wa IT, matengenezo, usalama, AI na chatbots), kufuatilia maombi, na kuwaelekeza kwenye nukuu rasmi.<br><br>Uliza kuhusu huduma, au andika <strong>"bei"</strong> kwa mwongozo wa gharama. Naweza pia kuzungumza kwa sauti 🎙️.`
      : `I am <strong>Mbilinyi AI</strong> 🤖 — the official assistant of <strong>Mbilinyi Tech Solutions</strong> (BRELA Reg. 706668), the company of CEO Jackson Mbilinyi based in Dodoma, Tanzania.<br><br>I help visitors understand <strong>our services</strong> (custom software, system analysis, IT consultancy, maintenance, cybersecurity, AI & chatbots), track requests, and reach an official quotation.<br><br>Ask about a service, or type <strong>"prices"</strong> for the cost guide. I can also speak aloud 🎙️.`;
  }
  if (/track|fuatilia|status|oda|order|wapi/.test(t)) return `Kufuatilia oda yako, niandikie <strong>Tracking ID</strong> yako (mfano <span class="font-mono">MTS-2026-4810</span>) — au <a href="track" class="text-cyan-300 underline font-bold">fungua ukurasa wa Tracking →</a>`;
  if (/bei|price|cost|gharama|pesa|how much|quotation|nukuu/.test(t)) return `💰 <strong>Pricing guide (TZS):</strong><br>• Websites: 500K+ • Business systems: 1.5M–5M • Mobile apps: 2M+ • Chatbots: 600K+ • Maintenance: 150K/mo+ • Security audit: 800K+<br><br>Exact price = official quotation from CEO within 24hrs after you submit the <a href="#" onclick="openQuoteWizard();return false;" class="text-cyan-300 underline font-bold">Requirements Form →</a>`;
  if (/maint|matengenezo|support|update|bug|warranty/.test(t)) return `🛠 <strong>Software Maintenance:</strong> Starter 150K/mo • Business SLA 450K/mo (popular) • Enterprise 24/7 1.2M/mo.<br>Includes updates, backups, bug fixes & monitoring. <a href="#" onclick="openQuoteWizard('Software Maintenance');return false;" class="text-cyan-300 underline font-bold">Subscribe →</a>`;
  if (/secur|hack|virus|pentest|audit|usalama/.test(t)) return `🔒 <strong>Security Services:</strong> Audit 800K • Pentest 2.5M • Managed 900K/mo.<br>OWASP Top-10 checks, malware removal, WAF, SSL & staff training. <a href="#" onclick="openQuoteWizard('Cybersecurity Services');return false;" class="text-cyan-300 underline font-bold">Secure my system →</a>`;
  if (/fine|tuning|ai|chatbot|llm|robot|bot/.test(t)) return `🤖 <strong>AI & Chatbot:</strong> Build your own AI & Chatbot with MBILINYITECH — WhatsApp & website bots (Swahili + English) + LLM fine-tuning on YOUR data (price lists, FAQs, policies). Try the <a href="ai" class="text-cyan-300 underline font-bold">live AI demo →</a> or <a href="#" onclick="openQuoteWizard('AI Solutions & Fine-Tuning');return false;" class="text-cyan-300 underline font-bold">build your own AI →</a>`;
  if (/service|huduma|offer|fanya|mnatoa/.test(t)) return `We offer <strong>15</strong> service lines:<br><strong>1.</strong> Custom Software (Mobile + Desktop + Web)<br><strong>2.</strong> System Analysis<br><strong>3.</strong> IT Consultancy<br><strong>4.</strong> Maintenance<br><strong>5.</strong> Cybersecurity<br><strong>6.</strong> AI Chatbots<br><strong>7.</strong> AI Fine-Tuning (Build your own AI & Chatbot with MBILINYITECH)<br><strong>8.</strong> Cloud & DevOps<br><strong>9.</strong> BRELA & Business Registration<br><strong>10.</strong> TRA & Tax Services<br><strong>11.</strong> Computer Troubleshooting<br><strong>12.</strong> Windows & OS Installation<br><strong>13.</strong> Email Setup & Repair<br><strong>14.</strong> Government e-Services<br><strong>15.</strong> Software Installation<br><a href="services" class="text-cyan-300 underline font-bold">View details →</a> or <a href="#" onclick="openQuoteWizard();return false;" class="text-cyan-300 underline font-bold">Start a request →</a>`;
  if (/brela|tin|register|sajili|license|certificate/.test(t)) return `✅ <strong>100% Legitimate:</strong> BRELA Registered (706668) • TRA TIN <span class="font-mono">192-147-522</span>. We also DO BRELA + TRA registrations for YOUR business. Every project includes a stamped service contract, e-signature & official receipts.`;
  if (/contact|mawasiliano|number|namba|email|instagram|whatsapp|call|ceo|jackson|mbilinyi/.test(t)) return `📞 <strong>CEO Jackson Mbilinyi</strong><br>• Phone/WhatsApp/SMS: <a href="tel:+255796752645" class="text-cyan-300 font-bold">0796 752 645</a><br>• Email: <a href="mailto:mbilinyitech@gmail.com" class="text-cyan-300 font-bold">mbilinyitech@gmail.com</a><br>• Instagram: <strong>@mbilinyitech</strong><br>• Hours: Mon–Sat 8:00–20:00 EAT`;
  if (/\br\b|rust|react|python|html|php|laravel|language|stack|teknolojia/.test(t)) return `💻 <strong>Our stack:</strong> PHP • Laravel • Python • React • Rust • R • HTML/CSS/JS • Node.js • Flutter • SQL & Cloud. We pick the right tool per project, with security and scaling planned from day one.`;
  if (/contract|mkataba|sign|saini|agreement/.test(t)) return `📄 <strong>Contracts:</strong> after you approve a quotation, a BRELA-compliant contract with strict 50/50 payment terms is generated. You e-sign with finger/mouse in your portal FIRST, then CEO signs. Legally binding under Tanzanian law — archived on both portals & emailed on completion.`;
  if (/pay|lipia|mpesa|invoice|bill|deposit|malipo/.test(t)) return `💳 <strong>Payments — 50/50 STRICT (NOT deposit + delivery):</strong><br>① <strong>50% Deposit</strong> → payable BEFORE project start (verified by CEO; project does NOT start until paid).<br>② <strong>50% Mid-Project</strong> → at 50% of timeline (NOT at final delivery). Final source code, training & handover only AFTER second installment paid.<br><br>M-Pesa: <strong>0796 752 645</strong> (Jackson Mbilinyi). Bank on request. Late payments: 2% per week penalty.`;
  if (/hi|hello|hey|habari|mambo|niaje|salut|jambo/.test(t)) return `Habari! 👋 Karibu <strong>Mbilinyi Tech Solutions</strong>. <span class="grad-text font-bold">"Your Problem, Our Solution"</span> — <em>"Shida Yako, Tatizo Letu."</em> Mimi ni AI assistant. Uliza kuhusu bei, huduma, tracking, contracts — au andika <strong>"request"</strong> kuanza ombi. How can I help?`;
  if (/request|omba|form|quote|start|anza|need|nataka|nahitaji/.test(t)) return `Great! 🚀 Click <a href="#" onclick="openQuoteWizard();return false;" class="text-cyan-300 underline font-bold">HERE to open the Requirements Form</a> — it takes 2 minutes, and CEO Jackson responds with official prices within 24hrs.`;
  if (/thank|asante|shukran/.test(t)) return `Karibu sana! 🙏 Tunathamini. Anything else — tracking, bei, au msaada? Tuko hapa kukusaidia.`;
  if (/human|agent|mtu|ceo|call me|nipigie/.test(t)) return `Nitakupigia! 📲 CEO Jackson is available on <a href="https://wa.me/255796752645" target="_blank" class="text-emerald-300 font-bold underline">WhatsApp 0796752645</a> — usually replies within minutes (8:00–20:00). Meanwhile, unaweza kuacha ticket hapa: eleza tatizo lako.`;
  return `Nimekuelewa — "<em>${esc(raw).slice(0, 80)}</em>". 🤔<br>Let me connect the dots: for <strong>prices</strong> type <em>"bei"</em>, for <strong>tracking</strong> paste your <em>MTS ID</em>, for a <strong>new project</strong> type <em>"request"</em>, or chat with a human on <a href="https://wa.me/255796752645" target="_blank" class="text-emerald-300 font-bold underline">WhatsApp →</a>`;
}

/* ---------- INIT ---------- */
document.addEventListener('DOMContentLoaded', () => {
  const u = currentUser();
  const btn = $('navAuthBtn');
  if (btn && u) {
    btn.innerHTML = `<i class="fa-solid fa-gauge text-cyan-300"></i> ${u.role === 'admin' ? T('Panel', 'Panel') : T('My Portal', 'Portal Yangu')}`;
  }

  if (typeof paintVoiceBtn === 'function') paintVoiceBtn();
  if (typeof paintMicBtn === 'function') paintMicBtn();
  renderHero();
  renderServices();
  renderStack();
  renderPricing();
  loadPublicPrices();
  renderWork();
  renderNews();
  renderTeam();
  paintPlatformCta();
  paintBrand();
  initFeedbackStars();

  if ($('view-client')) {
    const hash = (location.hash || '').replace('#', '');
    paintClientHeader();
    clientTab(hash || 'overview');
  }

  if ($('view-admin')) {
    if (!u || u.role !== 'admin') {
      location.href = 'auth?returnTo=admin';
    } else {
      adminTab('overview');
    }
  }

  /* The staff portal. A signed-out visitor is sent to sign in; a signed-in
   * one sees only their own desk, which the API decides. */
  if ($('view-staff')) {
    if (!u) {
      location.href = 'auth?returnTo=mtaalamu';
    } else {
      staffTab((location.hash || '').replace('#', '') || 'home');
    }
  }

  if (typeof gsap !== 'undefined' && typeof ScrollTrigger !== 'undefined') {
    gsap.registerPlugin(ScrollTrigger);
  }

  if ($('view-landing')) {
    setTimeout(() => { toast(T('👋 Welcome to Mbilinyi Tech Solutions! Click the robot for AI help, or Request Service to start.', '👋 Karibu Mbilinyi Tech Solutions! Gusa roboti kwa msaada wa AI, au Omba Huduma kuanza.'), 'info'); }, 1200);
  }

  // The passkey button only appears where WebAuthn can actually run.
  paintPasskeyLogin();

  // Auto-open password reset panel if URL has ?reset_token=...&email=...
  if ($('authLogin') || $('authReset')) {
    const q = getUrlParams();
    if (q.tab === 'register' && $('authRegister')) switchAuthTab('register');
    if (q.reset_token && q.email) {
      showResetTabOnly();
      setTimeout(() => {
        toast('🔐 Password reset link detected — enter your new password below to continue.', 'info');
        $('resetNewPass')?.focus();
      }, 200);
    }
  }

  $('quoteModal')?.addEventListener('click', e => { if (e.target === $('quoteModal')) closeQuoteWizard(); });
  $('genModal')?.addEventListener('click', e => { if (e.target === $('genModal')) closeGen(); });

  applyLang(LANG); // re-apply after every first-paint renderer so the chosen language is never mixed
});

document.addEventListener('keydown', e => { if (e.key === 'Escape') { closeQuoteWizard(); closeGen(); } });

/* ---- assign a client's request to the worker who serves it ---- */
async function openAssignWorker(requestId, service) {
  let emps = [];
  try { emps = (await apiGet('api/wafanyakazi.php', { action: 'list' })).employees || []; } catch (e) { toast(esc(e.message), 'err'); return; }
  const active = emps.filter(m => m.isActive);
  if (!active.length) { toast('Hire an employee first — open the Wafanyakazi tab.', 'err'); return; }
  // The workers who serve this service come first, since they are the right pick.
  const serves = m => (m.services || []).some(s => String(s).toLowerCase() === String(service).toLowerCase());
  const best = active.filter(serves);
  const rest = active.filter(m => !serves(m));
  const opt = m => `<option value="${m.id}">${esc(m.fullName)} — ${esc(m.profession)} (${esc(m.level)})${serves(m) ? ' ★ serves this' : ''}</option>`;
  openGen(`Assign Worker — ${service || 'request'}`, `
    <div class="text-[12.5px] text-white/60 mb-3"><i class="fa-solid fa-user-check text-emerald-300 mr-1.5"></i>The client will see who is serving them and can watch the progress. The worker may use only the tools you granted them.</div>
    <label class="text-[12px] font-bold text-white/60">CHOOSE THE WORKER</label>
    <select id="awWorker" class="input mt-1">
      ${best.length ? `<optgroup label="Serves ${esc(service)}">${best.map(opt).join('')}</optgroup>` : ''}
      ${rest.length ? `<optgroup label="Other workers">${rest.map(opt).join('')}</optgroup>` : ''}
    </select>
    <label class="text-[12px] font-bold text-white/60 mt-3 block">NOTE (optional)</label>
    <input id="awNote" class="input mt-1" placeholder="e.g. Priority client — start today">
    <div class="flex gap-2 mt-5">
      <button onclick="saveAssignWorker('${requestId}')" class="grad-btn flex-1 py-3 rounded-xl font-bold text-sm">Assign Worker</button>
      <button onclick="autoAssignWorker('${requestId}')" class="px-4 py-3 rounded-xl border border-cyan-400/40 text-cyan-200 font-bold text-sm"><i class="fa-solid fa-wand-magic-sparkles mr-1"></i>Auto-pick</button>
      <button onclick="closeGen()" class="px-4 py-3 rounded-xl border border-white/15 font-bold text-sm">Cancel</button>
    </div>
  `);
}

async function saveAssignWorker(requestId) {
  const employeeId = $('awWorker').value;
  try {
    const r = await apiPost('api/wafanyakazi.php', { action: 'assign', requestId, employeeId, note: $('awNote').value.trim() });
    closeGen(); adminTab('requests');
    toast(`${r.employee} will serve this client ✓`, 'ok');
  } catch (e) { toast(esc(e.message), 'err'); }
}

async function autoAssignWorker(requestId) {
  try {
    const r = await apiPost('api/wafanyakazi.php', { action: 'auto_assign', requestId });
    closeGen(); adminTab('requests');
    if (r.assigned) toast(`${r.employee} auto-assigned ✓`, 'ok');
    else toast(r.reason || 'No worker serves this service yet.', 'info');
  } catch (e) { toast(esc(e.message), 'err'); }
}

/* ============================================================
   WAFANYAKAZI — hiring staff, assigning tools, routing clients
   ============================================================ */
const LEVELS = ['junior', 'mid', 'senior', 'lead', 'principal'];

async function openEmployeeEditor(id) {
  let m = { id: 'new', fullName: '', photo: '', profession: '', level: 'mid', department: 'employee', email: '', phone: '', region: '', country: 'Tanzania', languages: 'Kiswahili, English', experienceYears: 0, skills: [], services: [], tools: [], salary: 0, currency: 'TZS', bio: '', status: 'active' };
  if (id !== 'new') {
    try { m = (await apiGet('api/wafanyakazi.php', { action: 'get', id })).employee; } catch (e) { toast(esc(e.message), 'err'); return; }
  }
  if (!DEPARTMENTS.length) { try { DEPARTMENTS = (await apiGet('api/wafanyakazi.php', { action: 'departments' })).departments || []; } catch (e) { /* defaults below */ } }
  if (!TOOLS.length) { try { TOOLS = (await apiGet('api/tools.php', { action: 'catalogue' })).tools || []; } catch (e) { /* tools optional */ } }
  const services = VISIBLE_SERVICES.map(s => s.t);
  const toolByFam = {};
  TOOLS.forEach(t => { (toolByFam[t.family] = toolByFam[t.family] || []).push(t); });
  openGen(id === 'new' ? 'Hire Employee' : `Edit Employee — ${m.fullName || id}`, `
    <div class="grid sm:grid-cols-2 gap-3">
      <div class="sm:col-span-2"><label class="text-[12px] font-bold text-white/60">FULL NAME *</label><input id="emName" class="input mt-1" value="${esc(m.fullName)}" placeholder="e.g. Neema Mwakyusa"></div>
      <div class="sm:col-span-2"><label class="text-[12px] font-bold text-white/60">PHOTO (upload an image)</label>
        <div class="flex items-center gap-3 mt-1">
          <img id="emPhotoPrev" src="${esc(m.photo)}" class="w-14 h-14 rounded-xl object-cover border border-white/15 ${m.photo ? '' : 'hidden'}">
          <input id="emPhotoFile" type="file" accept="image/*" onchange="readEmployeePhoto(this)" class="input flex-1 text-[12px]">
          <input type="hidden" id="emPhoto" value="${esc(m.photo)}">
        </div>
        <div class="text-[11px] text-white/40 mt-1">Upload a photo under 512 KB — it appears on the client's progress page and the public staff page.</div>
      </div>
      <div><label class="text-[12px] font-bold text-white/60">PROFESSION / TAALUMA *</label><input id="emProf" class="input mt-1" value="${esc(m.profession)}" placeholder="e.g. Cyber Security Analyst"></div>
      <div><label class="text-[12px] font-bold text-white/60">LEVEL</label><select id="emLevel" class="input mt-1">${LEVELS.map(l => `<option ${m.level === l ? 'selected' : ''}>${l}</option>`).join('')}</select></div>
      <div><label class="text-[12px] font-bold text-white/60">DEPARTMENT</label><select id="emDept" class="input mt-1">${DEPARTMENTS.map(d => `<option value="${d.key}" ${m.department === d.key ? 'selected' : ''}>${d.name}</option>`).join('')}</select></div>
      <div><label class="text-[12px] font-bold text-white/60">REGION</label><input id="emRegion" class="input mt-1" value="${esc(m.region)}" placeholder="e.g. Dar es Salaam"></div>
      <div><label class="text-[12px] font-bold text-white/60">EMAIL</label><input id="emEmail" class="input mt-1" value="${esc(m.email)}"></div>
      <div><label class="text-[12px] font-bold text-white/60">PHONE</label><input id="emPhone" class="input mt-1" value="${esc(m.phone)}"></div>
      <div><label class="text-[12px] font-bold text-white/60">LANGUAGES</label><input id="emLangs" class="input mt-1" value="${esc(m.languages)}"></div>
      <div><label class="text-[12px] font-bold text-white/60">YEARS OF EXPERIENCE</label><input id="emExp" type="number" class="input mt-1" value="${Number(m.experienceYears) || 0}"></div>
      <div><label class="text-[12px] font-bold text-white/60">SKILLS (comma separated)</label><input id="emSkills" class="input mt-1" value="${esc((m.skills || []).join(', '))}"></div>
      <div><label class="text-[12px] font-bold text-white/60">SALARY</label><input id="emSalary" type="number" class="input mt-1" value="${Number(m.salary) || 0}"></div>
      <div><label class="text-[12px] font-bold text-white/60">CURRENCY</label><select id="emCurrency" class="input mt-1"><option ${m.currency === 'TZS' ? 'selected' : ''}>TZS</option><option ${m.currency === 'USD' ? 'selected' : ''}>USD</option></select></div>
      <div><label class="text-[12px] font-bold text-white/60">STATUS</label><select id="emStatus" class="input mt-1"><option value="active" ${m.status === 'active' ? 'selected' : ''}>Active (serving clients)</option><option value="inactive" ${m.status !== 'active' ? 'selected' : ''}>Paused</option></select></div>
    </div>
    <label class="text-[12px] font-bold text-white/60 mt-3 block">SERVICES THIS WORKER HANDLES * <span class="text-white/40 font-normal">(client requests for these go to them)</span></label>
    <div class="grid sm:grid-cols-2 gap-1.5 mt-1 max-h-52 overflow-y-auto p-2 rounded-xl border border-white/10 bg-white/[.02]">
      ${services.map(s => `<label class="flex items-center gap-2 text-[12.5px] p-1.5 rounded-lg hover:bg-white/5"><input type="checkbox" class="emSvc" value="${esc(s)}" ${(m.services || []).includes(s) ? 'checked' : ''}> ${esc(s)}</label>`).join('')}
    </div>
    <label class="text-[12px] font-bold text-white/60 mt-3 block">TOOLS GRANTED <span class="text-white/40 font-normal">(they may use only these)</span></label>
    <div class="grid sm:grid-cols-2 gap-3 mt-1 max-h-64 overflow-y-auto p-2 rounded-xl border border-white/10 bg-white/[.02]">
      ${Object.keys(toolByFam).map(f => `<div><div class="text-[10.5px] font-bold text-cyan-300 uppercase tracking-wider mb-1">${esc(f)}</div>
        ${toolByFam[f].map(t => `<label class="flex items-center gap-2 text-[12px] p-1 rounded-lg hover:bg-white/5"><input type="checkbox" class="emTool" value="${t.code}" ${(m.tools || []).includes(t.code) ? 'checked' : ''}> ${esc(t.name)}</label>`).join('')}</div>`).join('')}
    </div>
    <label class="text-[12px] font-bold text-white/60 mt-3 block">BIO</label>
    <textarea id="emBio" rows="2" class="input mt-1">${esc(m.bio)}</textarea>
    <div class="flex gap-2 mt-5"><button onclick="saveEmployee('${id}')" class="grad-btn flex-1 py-3 rounded-xl font-bold text-sm">${id === 'new' ? 'Hire Employee' : 'Save Changes'}</button><button onclick="closeGen()" class="px-5 py-3 rounded-xl border border-white/15 font-bold text-sm">Cancel</button></div>
  `);
}

function readEmployeePhoto(input) {
  const f = input.files && input.files[0];
  if (!f) return;
  if (f.size > 512 * 1024) { toast('Photo must be under 512 KB.', 'err'); input.value = ''; return; }
  const r = new FileReader();
  r.onload = () => { $('emPhoto').value = r.result; const p = $('emPhotoPrev'); p.src = r.result; p.classList.remove('hidden'); };
  r.readAsDataURL(f);
}

async function saveEmployee(id) {
  const payload = {
    action: id === 'new' ? 'add' : 'update',
    fullName: $('emName').value.trim(),
    photo: $('emPhoto').value,
    profession: $('emProf').value.trim(),
    level: $('emLevel').value,
    department: $('emDept').value,
    region: $('emRegion').value.trim(),
    email: $('emEmail').value.trim(),
    phone: $('emPhone').value.trim(),
    languages: $('emLangs').value.trim(),
    experienceYears: Number($('emExp').value) || 0,
    skills: $('emSkills').value.split(',').map(s => s.trim()).filter(Boolean),
    services: [...document.querySelectorAll('.emSvc:checked')].map(x => x.value),
    tools: [...document.querySelectorAll('.emTool:checked')].map(x => x.value),
    salary: Number($('emSalary').value) || 0,
    currency: $('emCurrency').value,
    status: $('emStatus').value,
    bio: $('emBio').value.trim(),
  };
  if (!payload.fullName) { toast('The name is required.', 'err'); return; }
  if (!payload.profession) { toast('The profession (taaluma) is required.', 'err'); return; }
  if (!payload.services.length) { toast('Choose at least one service this worker handles.', 'err'); return; }
  if (id !== 'new') payload.id = id;
  try { await apiPost('api/wafanyakazi.php', payload); } catch (e) { toast(esc(e.message), 'err'); return; }
  closeGen(); adminTab('wafanyakazi'); toast(id === 'new' ? 'Employee hired ✓' : 'Employee updated ✓', 'ok');
}

async function toggleEmployee(id) {
  try { await apiPost('api/wafanyakazi.php', { action: 'toggle', id }); } catch (e) { toast(esc(e.message), 'err'); return; }
  adminTab('wafanyakazi'); toast('Status changed.', 'info');
}
async function delEmployee(id) {
  if (!confirm('Remove this employee permanently?')) return;
  try { await apiPost('api/wafanyakazi.php', { action: 'delete', id }); } catch (e) { toast(esc(e.message), 'err'); return; }
  adminTab('wafanyakazi'); toast('Employee removed.', 'info');
}

/* ---- tool granter (shared by employees and agents) ---- */
async function openToolGranter(id, kind) {
  adminToolHolder = { id, kind };
  let held = [];
  try {
    const r = await apiGet('api/tools.php', { action: kind === 'agent' ? 'for_agent' : 'for_employee', id });
    held = r.codes || [];
  } catch (e) { toast(esc(e.message), 'err'); return; }
  if (!TOOLS.length) { try { TOOLS = (await apiGet('api/tools.php', { action: 'catalogue' })).tools || []; } catch (e) { /* ignore */ } }
  const famColor = { software: 'cyan', security: 'rose', ai: 'fuchsia', design: 'violet', business: 'amber', comms: 'sky', platform: 'emerald', agent: 'teal', media: 'orange' };
  const byFam = {};
  TOOLS.forEach(t => { (byFam[t.family] = byFam[t.family] || []).push(t); });
  openGen(`Assign Tools — ${kind === 'agent' ? 'Agent' : 'Employee'}`, `
    <div class="text-[12.5px] text-white/60 mb-3"><i class="fa-solid fa-circle-info text-cyan-300 mr-1.5"></i>Every tool lives in the platform. A holder may use only what is ticked here; anything else is refused at the endpoint.</div>
    <div class="grid sm:grid-cols-2 gap-3 max-h-[60vh] overflow-y-auto pr-1">
    ${Object.keys(byFam).map(f => `<div class="rounded-xl border border-white/10 bg-white/[.02] p-2.5">
      <div class="text-[10.5px] font-bold text-${famColor[f] || 'cyan'}-300 uppercase tracking-wider mb-1.5">${esc(f)} (${byFam[f].length})</div>
      ${byFam[f].map(t => `<label class="flex items-start gap-2 text-[12px] p-1 rounded-lg hover:bg-white/5"><input type="checkbox" class="tgTool" value="${t.code}" ${held.includes(t.code) ? 'checked' : ''}><span>${esc(t.name)}${t.agentic ? ' <i class="fa-solid fa-bolt text-amber-300 text-[9px]" title="Agentic"></i>' : ''}${t.tanzaniaOnly ? ' <span class="text-[9px] text-amber-300 font-bold">TZ</span>' : ''}<br><span class="text-white/40 text-[11px]">${esc(t.desc)}</span></span></label>`).join('')}
    </div>`).join('')}
    </div>
    <div class="flex gap-2 mt-4"><button onclick="saveToolGrant()" class="grad-btn flex-1 py-3 rounded-xl font-bold text-sm">Save Tool Assignment</button><button onclick="closeGen()" class="px-5 py-3 rounded-xl border border-white/15 font-bold text-sm">Cancel</button></div>
  `);
}

async function saveToolGrant() {
  if (!adminToolHolder) return;
  const codes = [...document.querySelectorAll('.tgTool:checked')].map(x => x.value);
  const { id, kind } = adminToolHolder;
  try {
    if (kind === 'agent') {
      await apiPost('api/agents.php', { action: 'set_tools', id, tools: codes });
    } else {
      const r = await apiGet('api/wafanyakazi.php', { action: 'get', id });
      const m = r.employee;
      await apiPost('api/wafanyakazi.php', {
        action: 'update', id,
        fullName: m.fullName, photo: m.photo, profession: m.profession, level: m.level,
        department: m.department, email: m.email, phone: m.phone, region: m.region,
        languages: m.languages, experienceYears: m.experienceYears, skills: m.skills,
        services: m.services, salary: m.salary, currency: m.currency, bio: m.bio, status: m.status,
        tools: codes,
      });
    }
  } catch (e) { toast(esc(e.message), 'err'); return; }
  closeGen();
  adminTab(kind === 'agent' ? 'agents' : 'wafanyakazi');
  toast(`${codes.length} tool(s) assigned ✓`, 'ok');
}

/* ============================================================
   AJIRA — vacancies, applications, assessment
   ============================================================ */
async function openJobEditor(id) {
  let j = { id: 'new', title: '', department: 'employee', service: '', level: 'mid', location: 'Tanzania', kind: 'Full-time', salary: '', description: '', requirements: [], theoreticalWeight: 50, practicalWeight: 50, positions: 1, isOpen: true };
  if (id !== 'new') {
    try { j = ((await apiGet('api/ajira.php', { action: 'admin_openings' })).openings || []).find(x => x.id === id) || j; } catch (e) { toast(esc(e.message), 'err'); return; }
  }
  if (!DEPARTMENTS.length) { try { DEPARTMENTS = (await apiGet('api/wafanyakazi.php', { action: 'departments' })).departments || []; } catch (e) { /* ignore */ } }
  const services = VISIBLE_SERVICES.map(s => s.t);
  openGen(id === 'new' ? 'Post a Vacancy' : `Edit Vacancy — ${j.title}`, `
    <div class="grid sm:grid-cols-2 gap-3">
      <div class="sm:col-span-2"><label class="text-[12px] font-bold text-white/60">JOB TITLE *</label><input id="jbTitle" class="input mt-1" value="${esc(j.title)}" placeholder="e.g. AI Engineer"></div>
      <div><label class="text-[12px] font-bold text-white/60">DEPARTMENT</label><select id="jbDept" class="input mt-1">${DEPARTMENTS.map(d => `<option value="${d.key}" ${j.department === d.key ? 'selected' : ''}>${d.name}</option>`).join('')}</select></div>
      <div><label class="text-[12px] font-bold text-white/60">SERVICE THE HIRE WILL HANDLE</label><select id="jbService" class="input mt-1"><option value="">— none —</option>${services.map(s => `<option ${j.service === s ? 'selected' : ''}>${esc(s)}</option>`).join('')}</select></div>
      <div><label class="text-[12px] font-bold text-white/60">LEVEL</label><select id="jbLevel" class="input mt-1">${LEVELS.map(l => `<option ${j.level === l ? 'selected' : ''}>${l}</option>`).join('')}</select></div>
      <div><label class="text-[12px] font-bold text-white/60">LOCATION</label><input id="jbLocation" class="input mt-1" value="${esc(j.location)}"></div>
      <div><label class="text-[12px] font-bold text-white/60">TYPE</label><input id="jbKind" class="input mt-1" value="${esc(j.kind)}" placeholder="Full-time / Contract / Internship"></div>
      <div><label class="text-[12px] font-bold text-white/60">SALARY</label><input id="jbSalary" class="input mt-1" value="${esc(j.salary)}" placeholder="e.g. TZS 2,000,000 / month"></div>
      <div><label class="text-[12px] font-bold text-white/60">THEORY WEIGHT %</label><input id="jbTW" type="number" class="input mt-1" value="${j.theoreticalWeight}"></div>
      <div><label class="text-[12px] font-bold text-white/60">PRACTICAL WEIGHT %</label><input id="jbPW" type="number" class="input mt-1" value="${j.practicalWeight}"></div>
      <div><label class="text-[12px] font-bold text-white/60">POSITIONS</label><input id="jbPos" type="number" class="input mt-1" value="${j.positions}"></div>
      <div><label class="text-[12px] font-bold text-white/60">VISIBLE TO APPLICANTS</label><select id="jbOpen" class="input mt-1"><option value="1" ${j.isOpen ? 'selected' : ''}>Open</option><option value="0" ${!j.isOpen ? 'selected' : ''}>Closed</option></select></div>
    </div>
    <label class="text-[12px] font-bold text-white/60 mt-3 block">DESCRIPTION</label>
    <textarea id="jbDesc" rows="3" class="input mt-1">${esc(j.description)}</textarea>
    <label class="text-[12px] font-bold text-white/60 mt-3 block">REQUIREMENTS (one per line)</label>
    <textarea id="jbReqs" rows="3" class="input mt-1">${esc((j.requirements || []).join('\n'))}</textarea>
    <div class="flex gap-2 mt-5"><button onclick="saveJob('${id}')" class="grad-btn flex-1 py-3 rounded-xl font-bold text-sm">${id === 'new' ? 'Post Vacancy' : 'Save Vacancy'}</button><button onclick="closeGen()" class="px-5 py-3 rounded-xl border border-white/15 font-bold text-sm">Cancel</button></div>
  `);
}

async function saveJob(id) {
  const payload = {
    action: 'opening_save', title: $('jbTitle').value.trim(), department: $('jbDept').value,
    service: $('jbService').value, level: $('jbLevel').value, location: $('jbLocation').value.trim(),
    kind: $('jbKind').value.trim(), salary: $('jbSalary').value.trim(), description: $('jbDesc').value.trim(),
    requirements: $('jbReqs').value.split('\n').map(s => s.trim()).filter(Boolean),
    theoreticalWeight: Number($('jbTW').value) || 50, practicalWeight: Number($('jbPW').value) || 50,
    positions: Number($('jbPos').value) || 1, isOpen: Number($('jbOpen').value),
  };
  if (!payload.title) { toast('The vacancy needs a title.', 'err'); return; }
  if (id !== 'new') payload.id = id;
  try { await apiPost('api/ajira.php', payload); } catch (e) { toast(esc(e.message), 'err'); return; }
  closeGen(); adminTab('ajira'); toast(id === 'new' ? 'Vacancy posted ✓' : 'Vacancy updated ✓', 'ok');
}

async function delJob(id) {
  if (!confirm('Delete this vacancy? Applications already received are kept.')) return;
  try { await apiPost('api/ajira.php', { action: 'opening_delete', id }); } catch (e) { toast(esc(e.message), 'err'); return; }
  adminTab('ajira'); toast('Vacancy deleted.', 'info');
}

function openAssess(id, name, theory, practical) {
  openGen(`Assessment — ${name}`, `
    <div class="text-[12.5px] text-white/60 mb-3"><i class="fa-solid fa-clipboard-check text-cyan-300 mr-1.5"></i>Record both scores out of 100. The total is computed from the vacancy's weights, so it can never disagree with its parts.</div>
    <label class="text-[12px] font-bold text-white/60">THEORETICAL SCORE (0–100)</label><input id="asTheory" type="number" min="0" max="100" class="input mt-1" value="${Number(theory) || 0}">
    <label class="text-[12px] font-bold text-white/60 mt-3 block">PRACTICAL SCORE (0–100)</label><input id="asPractical" type="number" min="0" max="100" class="input mt-1" value="${Number(practical) || 0}">
    <label class="text-[12px] font-bold text-white/60 mt-3 block">ASSESSOR NOTES</label><textarea id="asNotes" rows="3" class="input mt-1" placeholder="What did the applicant do well or badly?"></textarea>
    <div class="flex gap-2 mt-5"><button onclick="saveAssess('${id}')" class="grad-btn flex-1 py-3 rounded-xl font-bold text-sm">Save Assessment</button><button onclick="closeGen()" class="px-5 py-3 rounded-xl border border-white/15 font-bold text-sm">Cancel</button></div>
  `);
}

async function saveAssess(id) {
  const t = Number($('asTheory').value), p = Number($('asPractical').value);
  if (t < 0 || t > 100 || p < 0 || p > 100) { toast('Both scores must be between 0 and 100.', 'err'); return; }
  try {
    await apiPost('api/ajira.php', { action: 'assess', id, theoreticalScore: t, practicalScore: p, assessmentNotes: $('asNotes').value.trim() });
  } catch (e) { toast(esc(e.message), 'err'); return; }
  closeGen(); adminTab('ajira'); toast('Assessment saved ✓', 'ok');
}

async function hireApplicant(id) {
  if (!confirm('Hire this applicant? An employee record is created for them.')) return;
  try {
    const r = await apiPost('api/ajira.php', { action: 'hire', id });
    toast(`${r.employee} hired into ${r.department} ✓`, 'ok');
  } catch (e) { toast(esc(e.message), 'err'); return; }
  adminTab('ajira');
}

async function delApplication(id) {
  if (!confirm('Delete this application?')) return;
  try { await apiPost('api/ajira.php', { action: 'delete_application', id }); } catch (e) { toast(esc(e.message), 'err'); return; }
  adminTab('ajira'); toast('Application deleted.', 'info');
}

/* ============================================================
   AGENTS + ONLINE OFFICE + SECURITY
   ============================================================ */
async function toggleAgent(id) {
  try { await apiPost('api/agents.php', { action: 'toggle', id }); } catch (e) { toast(esc(e.message), 'err'); return; }
  adminTab('agents'); toast('Agent status changed.', 'info');
}

function openRoomEditor() {
  const kinds = ['meeting', 'video_call', 'voice_call', 'screen_share', 'webinar'];
  openGen('New Online Room', `
    <label class="text-[12px] font-bold text-white/60">ROOM NAME *</label><input id="rmName" class="input mt-1" placeholder="e.g. Client kick-off — Pharmacy System">
    <div class="grid sm:grid-cols-2 gap-3 mt-3">
      <div><label class="text-[12px] font-bold text-white/60">KIND</label><select id="rmKind" class="input mt-1">${kinds.map(k => `<option value="${k}">${k.replace('_', ' ')}</option>`).join('')}</select></div>
      <div><label class="text-[12px] font-bold text-white/60">DURATION (MIN)</label><input id="rmDur" type="number" class="input mt-1" value="30"></div>
      <div><label class="text-[12px] font-bold text-white/60">HOST</label><input id="rmHost" class="input mt-1" placeholder="Who runs the meeting?"></div>
      <div><label class="text-[12px] font-bold text-white/60">AI SCRIBE</label><select id="rmScribe" class="input mt-1"><option value="1">On — write the minutes</option><option value="0">Off</option></select></div>
    </div>
    <label class="text-[12px] font-bold text-white/60 mt-3 block">INVITEES (comma separated emails or names)</label>
    <input id="rmInvitees" class="input mt-1" placeholder="client@example.com, neema@mbilinyitech.co.tz">
    <div class="flex gap-2 mt-5"><button onclick="createRoom()" class="grad-btn flex-1 py-3 rounded-xl font-bold text-sm">Create Room</button><button onclick="closeGen()" class="px-5 py-3 rounded-xl border border-white/15 font-bold text-sm">Cancel</button></div>
  `);
}

async function createRoom() {
  const payload = {
    action: 'room_create', name: $('rmName').value.trim(), kind: $('rmKind').value,
    durationMin: Number($('rmDur').value) || 30, host: $('rmHost').value.trim(),
    scribe: Number($('rmScribe').value),
    invitees: $('rmInvitees').value.split(',').map(s => s.trim()).filter(Boolean),
  };
  if (!payload.name) { toast('The room needs a name.', 'err'); return; }
  try { await apiPost('api/ofisi.php', payload); } catch (e) { toast(esc(e.message), 'err'); return; }
  closeGen(); adminTab('ofisi'); toast('Room created ✓', 'ok');
}

async function setRoomStatus(id, status) {
  try { await apiPost('api/ofisi.php', { action: 'room_status', id, status }); } catch (e) { toast(esc(e.message), 'err'); return; }
  adminTab('ofisi'); toast(`Room ${status}.`, 'info');
}

/* ============================================================
   LIVE CALLS — the WebRTC room inside the platform
   ============================================================ */
/* Work Queue actions — the CEO's hands on the scheduler. */
let adminQueueFilter = '';
// The queue is a live thing: agents start and finish while the CEO watches, so
// the panel refreshes itself. Cleared whenever another tab is opened.
let queueTimer = null;
function startQueueWatch() {
  stopQueueWatch();
  queueTimer = setInterval(() => {
    if (aTab !== 'queue' || document.hidden) return;
    adminTab('queue', true);
  }, 5000);
}
function stopQueueWatch() {
  if (queueTimer) { clearInterval(queueTimer); queueTimer = null; }
}
async function queueRelease(id) {
  if (!confirm('Approve this job and let the agent start now?')) return;
  try { const r = await apiPost('api/engine.php', { action: 'queue_release', jobId: id }); toast(r.released ? 'Job released — the agent is starting.' : 'That job is no longer waiting.', r.released ? 'ok' : 'info'); } catch (e) { toast(esc(e.message), 'err'); return; }
  adminTab('queue');
}
async function queueCancel(id) {
  if (!confirm('Cancel this job?')) return;
  try { await apiPost('api/engine.php', { action: 'queue_cancel', jobId: id }); toast('Job cancelled.', 'info'); } catch (e) { toast(esc(e.message), 'err'); return; }
  adminTab('queue');
}
async function queueRetry(id) {
  try { const r = await apiPost('api/engine.php', { action: 'queue_retry', jobId: id }); toast(r.retried ? 'Job returned to the queue.' : 'That job cannot be retried.', r.retried ? 'ok' : 'info'); } catch (e) { toast(esc(e.message), 'err'); return; }
  adminTab('queue');
}
async function queueSweep() {
  try { const r = await apiPost('api/engine.php', { action: 'queue_sweep' }); toast(`Started ${r.started} job(s).`, 'ok'); } catch (e) { toast(esc(e.message), 'err'); return; }
  adminTab('queue');
}

function openCall(roomId) {
  const who = (currentUser()?.name || 'guest');
  openGen('Live Call', `
    <div data-call-room="${esc(roomId)}" data-call-name="${esc(who)}" class="call-root">
      <div class="flex flex-wrap items-center justify-between gap-2 mb-2">
        <div class="font-bold text-[13.5px]"><i class="fa-solid fa-video text-rose-300 mr-1.5"></i>Chumba: <span class="font-mono text-[12px]">${esc(roomId)}</span></div>
        <div class="flex flex-wrap gap-1.5">
          <button data-call-action="mute" class="call-btn" title="Zima/Kuwasha sauti"><i class="fa-solid fa-microphone"></i></button>
          <button data-call-action="camera" class="call-btn" title="Zima/Kuwasha kamera"><i class="fa-solid fa-video"></i></button>
          <button data-call-action="share" class="call-btn" title="Shiriki skrini"><i class="fa-solid fa-display"></i></button>
          <button data-call-action="hangup" class="call-btn call-btn-end" title="Ondoka"><i class="fa-solid fa-phone-slash"></i></button>
        </div>
      </div>
      <div data-call-videos class="call-grid"></div>
      <div data-call-status class="text-[12px] text-white/55 mt-2">Inaandaa kamera na maikrofoni…</div>
      <div class="text-[11.5px] text-white/40 mt-1">Sauti na picha zinasafiri moja kwa moja kati ya vivinjari (WebRTC, imesimbwa kwa DTLS-SRTP). Server inapitisha tu ishara ya kuunganisha.</div>
    </div>`);
  loadScript('/assets/call.js');
}

/* Load a script once, so opening the call twice does not run it twice. */
function loadScript(src) {
  if (document.querySelector(`script[data-src="${src}"]`)) return;
  const s = document.createElement('script');
  s.src = src;
  s.setAttribute('data-src', src);
  document.body.appendChild(s);
}

/* ============================================================
   AGENT ENGINE — run jobs, watch the steps, inspect the result
   ============================================================ */
async function runAgentJob() {
  const task = ($('runTask').value || '').trim();
  if (!task) { toast('Describe the job first.', 'err'); return; }
  try {
    const r = await apiPost('api/engine.php', {
      action: 'run_job', task,
      service: ($('runService').value || '').trim(),
      tool: ($('runTool').value || '').trim(),
    });
    toast(`${r.agent} ameanza kazi. Fungua Runs kuona hatua.`, 'ok');
    setTimeout(() => adminTab('runs'), 1200);
  } catch (e) { toast(esc(e.message), 'err'); }
}

async function viewRun(id) {
  let d;
  try { d = await apiGet('api/engine.php', { action: 'run', id }); } catch (e) { toast(esc(e.message), 'err'); return; }
  const r = d.run, steps = d.steps || [];
  const kindColor = k => k === 'tool' ? 'cyan' : k === 'refuse' ? 'red' : k === 'say' ? 'emerald' : 'white';
  openGen(`Run ${r.id}`, `
    <div class="flex flex-wrap items-center gap-2 mb-3 text-[12.5px]">
      <span class="chip text-${r.status === 'done' ? 'emerald' : r.status === 'error' ? 'red' : 'sky'}-300">${esc(r.status)}</span>
      <span class="font-bold">${esc(r.agentName)}</span>
      <span class="chip text-white/50">${esc(r.provider)}/${esc(r.model)}</span>
      <span class="text-white/40">${r.steps} steps</span>
    </div>
    <div class="rounded-xl bg-white/[.03] border border-white/10 p-3 text-[12.5px] mb-3"><strong>Job:</strong> ${esc(r.task)}</div>
    ${r.error ? `<div class="rounded-xl bg-red-500/10 border border-red-400/30 p-3 text-[12.5px] text-red-200 mb-3"><i class="fa-solid fa-triangle-exclamation mr-1"></i>${esc(r.error)}</div>` : ''}
    <div class="font-bold text-[13px] mb-2">Steps</div>
    <div class="grid gap-1.5 max-h-[40vh] overflow-auto">
    ${steps.map(s => `<div class="rounded-lg bg-white/[.02] border border-white/10 p-2.5 text-[12px]">
      <div class="flex items-center gap-2 mb-1">
        <span class="chip text-${kindColor(s.kind)}-300">${esc(s.kind)}</span>
        <span class="text-white/50">#${s.step}</span>
        ${s.tool ? `<span class="font-mono text-[11px] text-cyan-300">${esc(s.tool)}</span>` : ''}
        <span class="chip ${s.ok ? 'text-emerald-300' : 'text-red-300'}">${s.ok ? 'ok' : 'refused'}</span>
      </div>
      <div class="text-white/65 whitespace-pre-wrap break-words">${esc((s.output || '').slice(0, 900))}</div>
    </div>`).join('') || '<div class="text-white/40 text-[12.5px]">No steps recorded.</div>'}
    </div>
    <div class="mt-3 rounded-xl bg-emerald-500/[.07] border border-emerald-400/25 p-3">
      <div class="font-bold text-[12.5px] text-emerald-200 mb-1">Finished deliverable</div>
      <div class="text-[12.5px] text-white/75 whitespace-pre-wrap">${esc(r.result || '—')}</div>
    </div>
    <div class="flex gap-2 mt-4"><button onclick="closeGen()" class="grad-btn flex-1 py-3 rounded-xl font-bold text-sm">Close</button></div>`);
}

/* ============================================================
   AI ENGINE — provider configuration
   ============================================================ */
async function saveLlm() {
  try {
    await apiPost('api/llm.php', {
      action: 'save',
      provider: $('llmProvider').value,
      model: $('llmModel').value.trim(),
      apiKey: $('llmKey').value,
      baseUrl: $('llmBase').value.trim(),
      maxTokens: $('llmMax').value.trim(),
      temperature: $('llmTemp').value.trim(),
    });
  } catch (e) { toast(esc(e.message), 'err'); return; }
  adminTab('ai'); toast('AI engine settings saved ✓', 'ok');
}

async function testLlm() {
  const box = $('llmResult');
  box.innerHTML = '<div class="text-[12.5px] text-white/50"><i class="fa-solid fa-circle-notch fa-spin mr-1"></i>Calling the provider…</div>';
  let r;
  try { r = await apiPost('api/llm.php', { action: 'test' }); } catch (e) {
    box.innerHTML = `<div class="rounded-xl bg-red-500/10 border border-red-400/30 p-3 text-[12.5px] text-red-200">${esc(e.message)}</div>`; return;
  }
  box.innerHTML = r.connected
    ? `<div class="rounded-xl bg-emerald-500/10 border border-emerald-400/30 p-3 text-[12.5px] text-emerald-100"><i class="fa-solid fa-circle-check mr-1"></i>Connected to <strong>${esc(r.provider)}/${esc(r.model)}</strong> in ${r.ms} ms.<br>“${esc(r.reply)}”</div>`
    : `<div class="rounded-xl bg-red-500/10 border border-red-400/30 p-3 text-[12.5px] text-red-200"><i class="fa-solid fa-triangle-exclamation mr-1"></i>${esc(r.error)}</div>`;
}

async function clearLlmKey() {
  try { await apiPost('api/llm.php', { action: 'clear_key' }); } catch (e) { toast(esc(e.message), 'err'); return; }
  adminTab('ai'); toast('Key removed. The engine is off until a new key is added.', 'info');
}

/* ============================================================
   SMS GATEWAY — real delivery
   ============================================================ */
async function saveSms() {
  try {
    await apiPost('api/ofisi.php', {
      action: 'sms_config_save',
      gateway: $('smsGw').value,
      senderId: $('smsSender').value.trim(),
      apiKey: $('smsKey').value,
      secret: $('smsSecret').value,
      baseUrl: $('smsBase').value.trim(),
    });
  } catch (e) { toast(esc(e.message), 'err'); return; }
  adminTab('sms'); toast('SMS gateway saved ✓', 'ok');
}

async function testSms() {
  const box = $('smsResult');
  const phone = ($('smsTestPhone').value || '').trim();
  if (!phone) { toast('Give a number to send the test to.', 'err'); return; }
  box.innerHTML = '<div class="text-[12.5px] text-white/50"><i class="fa-solid fa-circle-notch fa-spin mr-1"></i>Sending…</div>';
  let r;
  try { r = await apiPost('api/ofisi.php', { action: 'sms_test', phone }); } catch (e) {
    box.innerHTML = `<div class="rounded-xl bg-red-500/10 border border-red-400/30 p-3 text-[12.5px] text-red-200">${esc(e.message)}</div>`; return;
  }
  box.innerHTML = r.sent
    ? `<div class="rounded-xl bg-emerald-500/10 border border-emerald-400/30 p-3 text-[12.5px] text-emerald-100"><i class="fa-solid fa-circle-check mr-1"></i>${esc(r.detail)}</div>`
    : `<div class="rounded-xl bg-red-500/10 border border-red-400/30 p-3 text-[12.5px] text-red-200"><i class="fa-solid fa-triangle-exclamation mr-1"></i>${esc(r.detail)}</div>`;
}

async function saveCompanyAccount() {
  try {
    await apiPost('api/security.php', {
      action: 'company_account_set', name: $('caName').value.trim(), bank: $('caBank').value.trim(),
      accountNo: $('caNo').value.trim(), swift: $('caSwift').value.trim(), mobile: $('caMobile').value.trim(),
    });
  } catch (e) { toast(esc(e.message), 'err'); return; }
  toast('Company account saved ✓', 'ok');
}

/* ---- logo upload (CEO) ---- */
async function openLogoEditor() {
  let cur = { logo: '', name: '', tagline: '' };
  try { cur = await apiGet('api/security.php', { action: 'logo_get' }); } catch (e) { /* defaults */ }
  openGen('Company Logo & Name', `
    <div class="text-[12.5px] text-white/60 mb-3"><i class="fa-solid fa-image text-cyan-300 mr-1.5"></i>Upload the company logo. It replaces the default everywhere in the platform.</div>
    <div class="flex items-center gap-3">
      <img id="lgPrev" src="${esc(cur.logo)}" class="w-16 h-16 rounded-xl object-contain bg-white/5 border border-white/15 p-1">
      <input id="lgFile" type="file" accept="image/*" onchange="readLogo(this)" class="input flex-1 text-[12px]">
    </div>
    <input type="hidden" id="lgLogo" value="${esc(cur.logo)}">
    <label class="text-[12px] font-bold text-white/60 mt-3 block">COMPANY NAME</label>
    <input id="lgName" class="input mt-1" value="${esc(cur.name)}">
    <label class="text-[12px] font-bold text-white/60 mt-3 block">TAGLINE (kauli mbiu)</label>
    <input id="lgTagline" class="input mt-1" value="${esc(cur.tagline)}" placeholder="No-one enters a section that is not theirs.">
    <div class="flex gap-2 mt-5"><button onclick="saveLogo()" class="grad-btn flex-1 py-3 rounded-xl font-bold text-sm">Save Logo</button><button onclick="resetLogo()" class="px-4 py-3 rounded-xl border border-white/15 font-bold text-sm">Reset to Default</button><button onclick="closeGen()" class="px-4 py-3 rounded-xl border border-white/15 font-bold text-sm">Cancel</button></div>
  `);
}

function readLogo(input) {
  const f = input.files && input.files[0];
  if (!f) return;
  if (f.size > 512 * 1024) { toast('Logo must be under 512 KB.', 'err'); input.value = ''; return; }
  const r = new FileReader();
  r.onload = () => { $('lgLogo').value = r.result; $('lgPrev').src = r.result; };
  r.readAsDataURL(f);
}

async function saveLogo() {
  const logo = $('lgLogo').value;
  if (!logo) { toast('Choose a logo image.', 'err'); return; }
  try {
    await apiPost('api/security.php', { action: 'logo_set', logo, name: $('lgName').value.trim(), tagline: $('lgTagline').value.trim() });
  } catch (e) { toast(esc(e.message), 'err'); return; }
  closeGen(); toast('Logo updated ✓ — reloading.', 'ok');
  setTimeout(() => location.reload(), 900);
}

async function resetLogo() {
  try { await apiPost('api/security.php', { action: 'logo_reset' }); } catch (e) { toast(esc(e.message), 'err'); return; }
  closeGen(); toast('Logo reset to default.', 'info');
  setTimeout(() => location.reload(), 900);
}


/* ================================================================
   STAFF PORTAL (MTAALAMU) — the worker's own desk
   ----------------------------------------------------------------
   A worker sees only their own work: the jobs assigned to them, the
   tools their job allows, and what those tools produced. The rule
   "no-one works outside their own job" is enforced by the API, not
   here — this code only draws what the API is willing to return.
================================================================ */
let sTab = 'home';

function staffTab(t) {
  sTab = t;
  document.querySelectorAll('#staffNav [data-stab]').forEach(b => {
    b.classList.toggle('active', b.dataset.stab === t);
    b.classList.toggle('text-white', b.dataset.stab === t);
    b.classList.toggle('text-white/70', b.dataset.stab !== t);
  });
  const titles = { home: T('My Desk', 'Dawati Langu'), tasks: T('My Tasks', 'Kazi Zangu'), tools: T('My Tools', 'Zana Zangu'), workspace: T('AI Workspace', 'Eneo la AI'), office: T('Online Office', 'Ofisi Mtandaoni') };
  if ($('staffTabTitle')) $('staffTabTitle').textContent = titles[t] || titles.home;
  staffLoad();
}

async function staffLoad() {
  const box = $('staffContent');
  if (!box) return;
  box.innerHTML = `<div class="text-white/40 text-sm"><i class="fa-solid fa-spinner fa-spin mr-2"></i>${T('Loading…', 'Inapakia…')}</div>`;
  try {
    if (sTab === 'home') return await staffHome(box);
    if (sTab === 'tasks') return await staffTasks(box);
    if (sTab === 'tools') return await staffTools(box);
    if (sTab === 'workspace') return await staffWorkspace(box);
    if (sTab === 'office') return staffOffice(box);
  } catch (e) {
    box.innerHTML = `<div class="rounded-2xl border border-red-400/25 bg-red-500/[.06] p-4 text-red-200 text-sm"><i class="fa-solid fa-circle-exclamation mr-2"></i>${esc(e.message)}</div>`;
  }
}

function paintStaffHeader(me) {
  if (!me) return;
  const e = me.employee || {};
  const initials = (e.fullName || 'MTS').split(' ').map(w => w[0]).join('').slice(0, 2).toUpperCase();
  if ($('staffAvatar')) {
    $('staffAvatar').innerHTML = e.photo ? `<img src="${esc(e.photo)}" class="w-full h-full object-cover rounded-2xl">` : esc(initials);
  }
  if ($('staffName')) $('staffName').textContent = e.fullName || 'Mtaalamu';
  if ($('staffProf')) $('staffProf').textContent = e.profession || '—';
}

async function staffHome(box) {
  const me = await apiGet('api/wafanyakazi.php', { action: 'me' });
  paintStaffHeader(me);
  const e = me.employee || {};
  const tools = (me.tools || []);
  box.innerHTML = `
    <div class="grid sm:grid-cols-3 gap-4">
      <div class="card3d pop-emerald p-5">
        <div class="text-[10.5px] text-white/50 font-bold uppercase tracking-wider">${T('Open tasks', 'Kazi zilizo hai')}</div>
        <div class="font-display font-bold text-3xl mt-1">${Number(me.openTasks) || 0}</div>
      </div>
      <div class="card3d pop-cyan p-5">
        <div class="text-[10.5px] text-white/50 font-bold uppercase tracking-wider">${T('Completed', 'Zilizokamilika')}</div>
        <div class="font-display font-bold text-3xl mt-1">${Number(me.completedTasks) || 0}</div>
      </div>
      <div class="card3d pop-violet p-5">
        <div class="text-[10.5px] text-white/50 font-bold uppercase tracking-wider">${T('Tools granted', 'Zana ulizopewa')}</div>
        <div class="font-display font-bold text-3xl mt-1">${tools.length}</div>
      </div>
    </div>
    <div class="glass rounded-3xl p-5 mt-5 border border-white/10">
      <div class="font-bold text-[15px] mb-2"><i class="fa-solid fa-id-badge text-emerald-300 mr-2"></i>${T('Your record', 'Taarifa zako')}</div>
      <div class="grid sm:grid-cols-2 gap-x-6 gap-y-2 text-[13px]">
        ${staffRow(T('Profession', 'Taaluma'), e.profession)}
        ${staffRow(T('Level', 'Ngazi'), e.level)}
        ${staffRow(T('Department', 'Idara'), e.department)}
        ${staffRow(T('Region', 'Mkoa'), e.region || 'Tanzania')}
        ${staffRow(T('Languages', 'Lugha'), e.languages)}
        ${staffRow(T('Experience', 'Uzoefu'), (Number(e.experienceYears) || 0) + ' ' + T('years', 'miaka'))}
      </div>
      ${(e.skills || []).length ? `<div class="flex flex-wrap gap-1.5 mt-4">${e.skills.map(s => `<span class="chip text-white/60 text-[11px]">${esc(s)}</span>`).join('')}</div>` : ''}
    </div>`;
}

function staffRow(k, v) {
  return `<div class="flex justify-between gap-3 border-b border-white/5 py-1.5"><span class="text-white/45">${esc(k)}</span><span class="font-semibold text-right">${esc(v || '—')}</span></div>`;
}

async function staffTasks(box) {
  const d = await apiGet('api/wafanyakazi.php', { action: 'my_tasks' });
  const tasks = d.tasks || [];
  if (!tasks.length) {
    box.innerHTML = `<div class="glass rounded-3xl p-8 text-center text-white/50">${T('No work has been assigned to you yet. When the CEO assigns a client request to you, it appears here.', 'Hakuna kazi iliyopewa bado. CEO akikupa ombi la mteja, litaonekana hapa.')}</div>`;
    return;
  }
  box.innerHTML = tasks.map(t => {
    const r = t.request || {};
    const st = r.status || 'Pending';
    const bar = Math.max(0, Math.min(100, Number(t.progress) || 0));
    return `<div class="glass rounded-3xl p-5 border border-white/10 mb-4">
      <div class="flex flex-wrap items-start justify-between gap-3">
        <div class="min-w-0">
          <div class="font-bold text-[15px]">${esc(r.title)}</div>
          <div class="text-[12px] text-white/45 mt-0.5">${esc(r.trackingId)} • ${esc(r.service)} • ${esc(r.name)}</div>
        </div>
        <span class="badge status-${esc(st.replace(/\s/g, ''))}">${esc(st.toUpperCase())}</span>
      </div>
      <div class="h-2 rounded-full bg-white/10 mt-3 overflow-hidden"><div class="h-full grad-btn" style="width:${bar}%"></div></div>
      <div class="text-[11.5px] text-white/45 mt-1">${bar}% ${T('complete', 'imekamilika')}</div>
      ${t.note ? `<div class="text-[12.5px] text-white/60 mt-2 italic">${esc(t.note)}</div>` : ''}
      <div class="flex flex-wrap gap-2 mt-4">
        ${t.canStart ? `<button onclick="staffSetStatus('${esc(r.id)}','In Progress')" class="grad-btn px-4 py-2 rounded-xl text-[13px] font-bold"><i class="fa-solid fa-play mr-1"></i>${T('Start work', 'Anza kazi')}</button>` : ''}
        ${t.canFinish ? `<button onclick="staffSetStatus('${esc(r.id)}','Completed')" class="px-4 py-2 rounded-xl text-[13px] font-bold border border-emerald-400/40 text-emerald-300 hover:bg-emerald-500/10"><i class="fa-solid fa-check mr-1"></i>${T('Mark finished', 'Maliza kazi')}</button>` : ''}
        <button onclick="staffTab('workspace')" class="px-4 py-2 rounded-xl text-[13px] font-bold border border-white/15 hover:bg-white/5"><i class="fa-solid fa-robot mr-1"></i>${T('Workspace', 'Eneo la kazi')}</button>
      </div>
    </div>`;
  }).join('');
}

async function staffSetStatus(requestId, status) {
  try {
    const r = await apiPost('api/wafanyakazi.php', { action: 'task_update', requestId, status });
    toast(status === 'In Progress' ? T('Work started ✓', 'Kazi imeanza ✓') : T('Marked finished — awaiting review ✓', 'Imekamilika — inasubiri ukaguzi ✓'), 'ok');
    staffLoad();
  } catch (e) { toast(esc(e.message), 'err'); }
}

async function staffTools(box) {
  const d = await apiGet('api/wafanyakazi.php', { action: 'my_tools' });
  const tools = d.tools || [];
  if (!tools.length) {
    box.innerHTML = `<div class="glass rounded-3xl p-8 text-center text-white/50">${T('No tools have been granted to your job yet. The CEO assigns the tools each job needs.', 'Hakuna zana iliyopewa kazi yako bado. CEO hupanga zana zinazohitajika kwa kila kazi.')}</div>`;
    return;
  }
  box.innerHTML = `<div class="text-[12.5px] text-white/50 mb-4"><i class="fa-solid fa-lock mr-1 text-emerald-300"></i>${T('These are the only tools your job may use. Anything outside this list is refused by the platform.', 'Hizi ni zana pekee ambazo kazi yako inaweza kutumia. Kitu chochote nje ya orodha hii kinakataliwa na mfumo.')}</div>
    <div class="grid sm:grid-cols-2 lg:grid-cols-3 gap-3">${tools.map(t => `
      <div class="card3d pop-cyan p-4">
        <div class="w-10 h-10 rounded-xl border border-white/10 flex items-center justify-center text-cyan-300"><i class="${esc(t.icon)}"></i></div>
        <div class="font-bold text-[13.5px] mt-2">${esc(t.name)}</div>
        <div class="text-[11.5px] text-white/45 mt-0.5">${esc(t.family)}</div>
        ${t.agentic ? `<span class="chip text-emerald-300 text-[10px] mt-2"><i class="fa-solid fa-bolt"></i>${T('Agentic', 'Inajifanya')}</span>` : ''}
      </div>`).join('')}</div>`;
}

async function staffWorkspace(box) {
  const d = await apiGet('api/wafanyakazi.php', { action: 'workspace' });
  const runs = d.runs || [];
  box.innerHTML = `
    <div class="glass rounded-3xl p-5 border border-white/10 mb-5">
      <div class="flex items-center gap-3">
        <div class="w-12 h-12 rounded-2xl grad-btn flex items-center justify-center text-xl"><i class="fa-solid fa-wand-magic-sparkles"></i></div>
        <div>
          <div class="font-bold text-[15px]">${T('AI Workspace', 'Eneo la Kazi la AI')}</div>
          <div class="text-[12px] text-white/50">${T('Give a granted tool a job and read the result — everything stays inside the platform.', 'Ipe zana uliyopewa kazi na usome matokeo — kila kitu kinabaki ndani ya mfumo.')}</div>
        </div>
      </div>
    </div>
    ${runs.length ? runs.map(r => `
      <div class="glass rounded-3xl p-5 border border-white/10 mb-3">
        <div class="flex flex-wrap items-center justify-between gap-2">
          <div class="font-bold text-[14px]"><i class="fa-solid fa-robot text-cyan-300 mr-2"></i>${esc(r.agentName || 'Agent')}</div>
          <span class="badge ${r.status === 'done' ? 'bg-emerald-500/15 text-emerald-300 border border-emerald-400/30' : r.status === 'error' ? 'bg-red-500/15 text-red-300 border border-red-400/30' : 'bg-amber-500/15 text-amber-300 border border-amber-400/30'}">${esc(String(r.status || '').toUpperCase())}</span>
        </div>
        <div class="text-[12.5px] text-white/60 mt-2">${esc(r.task)}</div>
        ${r.result ? `<div class="text-[12.5px] text-white/75 mt-3 whitespace-pre-wrap rounded-2xl bg-white/[.03] border border-white/10 p-3 max-h-72 overflow-auto">${esc(r.result)}</div>` : ''}
        <div class="text-[11px] text-white/35 mt-2">${esc(r.provider)} ${esc(r.model)} • ${Number(r.steps) || 0} ${T('steps', 'hatua')}</div>
      </div>`).join('') : `<div class="glass rounded-3xl p-8 text-center text-white/50">${T('No runs yet. When the CEO approves a request you are serving, the AI carries out the work and the result appears here.', 'Hakuna kazi iliyoendeshwa bado. CEO akikubali ombi unalolihudumia, AI hufanya kazi na matokeo huonekana hapa.')}</div>`}`;
}

function staffOffice(box) {
  box.innerHTML = `
    <div class="grid sm:grid-cols-2 gap-4">
      <div class="card3d pop-violet p-5">
        <div class="w-11 h-11 rounded-2xl border border-white/10 flex items-center justify-center text-violet-300 text-lg"><i class="fa-solid fa-video"></i></div>
        <div class="font-bold text-[14px] mt-3">${T('Live video call', 'Simu ya video')}</div>
        <div class="text-[12px] text-white/50 mt-1">${T('Meet a client or a colleague face to face inside the platform.', 'Kutana na mteja au mwenzako ana kwa ana ndani ya mfumo.')}</div>
        <button onclick="startCall && startCall()" class="grad-btn mt-3 px-4 py-2 rounded-xl text-[13px] font-bold">${T('Open call room', 'Fungua chumba cha simu')}</button>
      </div>
      <div class="card3d pop-emerald p-5">
        <div class="w-11 h-11 rounded-2xl border border-white/10 flex items-center justify-center text-emerald-300 text-lg"><i class="fa-solid fa-comment-sms"></i></div>
        <div class="font-bold text-[14px] mt-3">${T('Messages', 'Ujumbe')}</div>
        <div class="text-[12px] text-white/50 mt-1">${T('SMS to a client goes through the platform, and every message is logged.', 'SMS kwa mteja hupitia mfumo, na kila ujumbe hurekodiwa.')}</div>
        <button onclick="staffTab('tasks')" class="mt-3 px-4 py-2 rounded-xl text-[13px] font-bold border border-white/15 hover:bg-white/5">${T('Open tasks', 'Fungua kazi')}</button>
      </div>
    </div>`;
}


/* ================================================================
   SECURITY SANDBOX — the admin view of the real scanner
================================================================ */
async function loadSandbox() {
  const box = $('sandboxBox');
  if (!box) return;
  box.innerHTML = '<div class="text-white/40 text-[13px]"><i class="fa-solid fa-circle-notch fa-spin mr-2"></i>Loading…</div>';
  let sb;
  try { sb = await apiGet('api/security.php', { action: 'sandbox' }); }
  catch (e) { box.innerHTML = `<div class="text-red-300 text-[13px]">${esc(e.message)}</div>`; return; }
  const doms = sb.authorisedDomains || [];
  box.innerHTML = `
    <div class="grid sm:grid-cols-[1fr_auto] gap-3 items-end">
      <div>
        <label class="text-[11px] font-bold text-white/55 uppercase tracking-wider">Authorised domain (client's written permission required)</label>
        <input id="sbDomain" class="input mt-1 font-mono" placeholder="client-domain.co.tz">
      </div>
      <button onclick="authoriseDomain()" class="grad-btn px-5 py-3 rounded-xl font-bold text-[13px]"><i class="fa-solid fa-plus mr-1"></i>Authorise</button>
    </div>
    <div class="flex flex-wrap gap-1.5 mt-3">
      ${doms.map(d => `<span class="chip text-cyan-300">${esc(d)}<button onclick="deauthoriseDomain('${esc(d)}')" class="ml-1.5 text-white/40 hover:text-red-300">×</button></span>`).join('') || '<span class="text-[12.5px] text-white/40">No domain is authorised — nothing can be scanned.</span>'}
    </div>
    <div class="grid gap-1.5 mt-4 text-[12.5px] text-white/60">
      ${(sb.checks || []).map(c => `<div class="flex gap-2"><i class="fa-solid fa-check text-emerald-300 mt-0.5"></i><span>${esc(c)}</span></div>`).join('')}
    </div>
    <div class="rounded-xl border border-amber-400/25 bg-amber-500/[.05] p-3 mt-3 text-[12px] text-amber-200">
      <i class="fa-solid fa-scale-balanced mr-1"></i>${esc(sb.rule || '')}
    </div>
    <div class="grid sm:grid-cols-[170px_1fr_auto] gap-2 mt-4">
      <select id="sbKind" class="input">
        <option value="headers">Security headers</option>
        <option value="exposure">Exposure &amp; ports</option>
        <option value="dns">DNS posture</option>
        <option value="tls">TLS certificate</option>
        <option value="dependencies">Dependencies</option>
        <option value="code">Source code</option>
        <option value="malware">Malware</option>
      </select>
      <input id="sbTarget" class="input font-mono text-[12.5px]" placeholder="client-domain.co.tz">
      <button onclick="runScan()" class="px-5 py-3 rounded-xl font-bold text-[13px] border border-cyan-400/40 text-cyan-300 hover:bg-cyan-500/10"><i class="fa-solid fa-magnifying-glass mr-1"></i>Scan</button>
    </div>
    <pre id="sbOut" class="mt-3 p-3 rounded-xl bg-black/40 border border-white/10 text-[12px] text-white/80 max-h-80 overflow-auto hidden"></pre>`;
}

async function authoriseDomain() {
  const d = ($('sbDomain') || {}).value || '';
  if (!d.trim()) { toast('Give the domain.', 'err'); return; }
  try {
    await apiPost('api/security.php', { action: 'authorise_domain', domain: d.trim() });
    toast('Domain authorised ✓', 'ok');
    loadSandbox();
  } catch (e) { toast(esc(e.message), 'err'); }
}

async function deauthoriseDomain(d) {
  try {
    await apiPost('api/security.php', { action: 'deauthorise_domain', domain: d });
    toast('Domain removed.', 'info');
    loadSandbox();
  } catch (e) { toast(esc(e.message), 'err'); }
}

async function runScan() {
  const kind = ($('sbKind') || {}).value || 'headers';
  const target = ($('sbTarget') || {}).value || '';
  const out = $('sbOut');
  if (!out) return;
  if (!target.trim()) { toast('Give the domain to scan.', 'err'); return; }
  out.classList.remove('hidden');
  out.textContent = 'Scanning… (this makes real network requests)';
  try {
    const r = await apiPost('api/security.php', { action: 'scan', kind, target, authorised: 'the CEO' });
    out.textContent = r.evidence || '(no output)';
  } catch (e) { out.textContent = e.message; }
}

/* ================================================================
   RUNTIMES — run a snippet from the admin panel
================================================================ */
async function runSnippet() {
  const lang = ($('rtLang') || {}).value || 'python';
  const code = ($('rtCode') || {}).value || '';
  const out = $('rtOut');
  if (!out) return;
  if (!code.trim()) { toast('Give the code to run.', 'err'); return; }
  out.classList.remove('hidden');
  out.textContent = 'Compiling and running…';
  try {
    const r = await apiPost('api/runtime.php', { action: 'run', language: lang, code });
    out.textContent = `exit: ${r.ok ? 'success' : 'failure'} • ${r.ms} ms\n\n${r.output}`;
  } catch (e) { out.textContent = e.message; }
}
