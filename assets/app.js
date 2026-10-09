/* ================================================================
   MBILINYI TECH SOLUTIONS — FRONTEND DATA LAYER (PHP API-backed)
   Same UI, same function names — persistence now lives in Postgres
   through the PHP endpoints in /api.
================================================================ */

/* ---------- helpers ---------- */
const $ = id => document.getElementById(id);
const fmtTZS = n => 'TZS ' + Number(n || 0).toLocaleString('en-TZ');
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

/* ---------- STATIC DATA ---------- */
const SERVICES = [
  { icon: 'fa-code', c: 'cyan', t: 'Custom Software Development', d: 'Web, mobile & desktop systems in React, Python, Rust, R & HTML — built production-ready, tested & documented.', tags: ['React', 'Python', 'Rust'] },
  { icon: 'fa-diagram-project', c: 'violet', t: 'System Analysis & Design', d: 'Professional SRS, database design, UML, workflows & feasibility studies before a single line of code.', tags: ['SRS', 'UML', 'ERD'] },
  { icon: 'fa-briefcase', c: 'gold', t: 'IT Consultancy', d: 'Digital strategy, system audits, tech hiring advice & CTO-as-a-service for businesses & NGOs.', tags: ['Strategy', 'Audit'] },
  { icon: 'fa-screwdriver-wrench', c: 'emerald', t: 'Software Maintenance', d: 'Bug fixes, updates, backups, uptime monitoring & SLA support — your system never sleeps.', tags: ['SLA 99.9%', '24/7'] },
  { icon: 'fa-shield-halved', c: 'red', t: 'Cybersecurity Services', d: 'Penetration testing, OWASP audits, malware removal, SSL, firewall & staff security training.', tags: ['Pentest', 'OWASP'] },
  { icon: 'fa-robot', c: 'fuchsia', t: 'AI Chatbots', d: 'Website & WhatsApp chatbots in English & Kiswahili that answer, sell & book — 24/7.', tags: ['WhatsApp', 'Web'] },
  { icon: 'fa-brain', c: 'blue', t: 'AI Fine-Tuning', d: 'We fine-tune LLMs on YOUR data — price lists, policies, FAQs — for accurate, on-brand answers.', tags: ['LoRA', 'RAG'] },
  { icon: 'fa-cloud', c: 'teal', t: 'Cloud & DevOps', d: 'Deployment on AWS/VPS, CI/CD, Docker, domains, business emails & blazing performance.', tags: ['AWS', 'Docker'] },
  { icon: 'fa-building', c: 'cyan', t: 'BRELA & Business Registration', d: 'Usajili wa makampuni, Biashara, SME na vyeti vya BRELA — kila hatua kiotomatiki kwa utulivu.', tags: ['BRELA', 'SME'] },
  { icon: 'fa-file-invoice-dollar', c: 'gold', t: 'TRA & Tax Services', d: 'TIN registration, File Tax Returns, VAT, PAYE na msaada kamili wa mifumo ya TRA kwa Biashara.', tags: ['TIN', 'TRA', 'VAT'] },
  { icon: 'fa-screwdriver-wrench', c: 'emerald', t: 'Computer Troubleshooting', d: 'Kutatua matatizo ya software, malware removal, slow PC, crashes, data recovery, drivers na updates.', tags: ['Repair', 'Malware'] },
  { icon: 'fa-windows', c: 'blue', t: 'Windows & OS Installation', d: 'Windows 10/11, Linux, macOS + drivers, updates, activation, backup na data transfer salama.', tags: ['Windows', 'Drivers'] },
  { icon: 'fa-envelope-open-text', c: 'fuchsia', t: 'Email & Communication Setup', d: 'Kutengeneza email (G-Suite, Zoho, cPanel), SMTP, SPF/DKIM, na kutatua matatizo yote ya email.', tags: ['SMTP', 'G-Suite'] },
  { icon: 'fa-landmark', c: 'violet', t: 'Government e-Services', d: 'NIDA, NHIF, NSSF, eCitizen, Huduma Number na huduma zingine za serikali — kila hatua tutakupitia.', tags: ['NIDA', 'NHIF'] },
  { icon: 'fa-box', c: 'teal', t: 'Software Installation', d: 'Kuinstall MS Office, Adobe, IDEs, Antivirus, Design Apps, na customize settings kwa utulivu.', tags: ['Office', 'Apps'] },
];
const STACK = {
  'Python': { level: 98, desc: 'Backend APIs (Django/FastAPI), AI/ML, chatbots, automation & data pipelines. Our #1 backend.', use: 'APIs • AI • Automation' },
  'React': { level: 97, desc: 'Blazing dashboards, portals & mobile-friendly SPAs with modern UI/UX and reusable components.', use: 'Web Apps • Dashboards' },
  'Rust': { level: 90, desc: 'Memory-safe, ultra-fast microservices & system tools where performance and security are critical.', use: 'High-perf services' },
  'PHP': { level: 96, desc: 'Server-side rendering, portals, CMS, WordPress, payment gateways & robust REST APIs.', use: 'Websites • Portals' },
  'Laravel': { level: 94, desc: 'Enterprise-grade PHP framework: auth, queues, billing, ORM, admin panels & secure APIs.', use: 'Enterprise Apps' },
  'R': { level: 88, desc: 'Statistical engines, research dashboards, reports & data visualization for institutions & NGOs.', use: 'Analytics • Research' },
  'HTML/CSS/JS': { level: 99, desc: 'Pixel-perfect, responsive, SEO-ready websites with Tailwind, animations & accessibility.', use: 'Websites • Landing' },
  'Node.js': { level: 92, desc: 'Realtime apps, M-Pesa integrations, websockets & scalable REST/GraphQL backends.', use: 'Realtime • M-Pesa' },
  'Flutter': { level: 85, desc: 'One codebase Android + iOS apps — perfect for shops, schools & delivery businesses.', use: 'Mobile Apps' },
  'SQL & Cloud': { level: 94, desc: 'PostgreSQL/MySQL design, AWS/VPS deployment, Docker, backups & monitoring.', use: 'Data • Deploy' },
};
const MAINT = [
  { n: 'Starter Care', p: 'TZS 150,000', per: '/month', f: ['Monthly health check & updates', 'Bug fixes (up to 5 hrs)', 'Weekly backups', 'Email support (48hr)', 'Uptime monitoring'], hot: false },
  { n: 'Business SLA', p: 'TZS 450,000', per: '/month', f: ['Everything in Starter', 'Priority fixes (20 hrs) + 24hr response', 'Security patches & SSL', 'Monthly performance report', 'WhatsApp priority line', 'Small feature tweaks'], hot: true },
  { n: 'Enterprise 24/7', p: 'TZS 1,200,000', per: '/month', f: ['Dedicated engineer', 'Unlimited critical fixes', '99.9% uptime guarantee', 'Daily backups + DR', 'On-call 24/7', 'Quarterly security audit'], hot: false },
];
const SEC = [
  { n: 'Security Audit', p: 'TZS 800,000', per: 'one-time', f: ['Full OWASP Top-10 scan', 'Vulnerability report + fixes plan', 'SSL & headers hardening', '1 staff training session'] },
  { n: 'Penetration Test', p: 'TZS 2,500,000', per: 'per system', f: ['Manual + automated pentest', 'Exploit verification', 'Executive + technical reports', 'Free re-test after fixes'] },
  { n: 'Managed Security', p: 'TZS 900,000', per: '/month', f: ['Firewall + WAF management', 'Malware monitoring & removal', 'Monthly scans & patching', 'Incident response 24/7'] },
];
const PORTFOLIO = [
  { t: 'PharmaPlus Management Suite', c: 'Pharmacy • React + Python + M-Pesa', img: 'https://images.unsplash.com/photo-1576091160399-112ba8d25d1d?w=800&q=80', d: 'Stock, expiry alerts, sales & M-Pesa receipts for 12 branches.', s: '+38% revenue tracked' },
  { t: 'ShulePay Fees Portal', c: 'Education • React + Node + SMS', img: 'https://images.unsplash.com/photo-1503676260728-1c00da094a0b?w=800&q=80', d: 'Fee tracking, receipts & parent SMS for 3,000+ students.', s: 'Zero fee leakage' },
  { t: 'KilimoFresh WhatsApp Bot', c: 'AI • Python • Swahili NLP', img: 'https://images.unsplash.com/photo-1464226184884-fa280b87c399?w=800&q=80', d: 'Swahili ordering bot handling 500+ orders/week.', s: '4.9★ customer rating' },
  { t: 'RustPay Microservice', c: 'Fintech • Rust + PostgreSQL', img: 'https://images.unsplash.com/photo-1551288049-bebda4e38f71?w=800&q=80', d: '12ms transaction engine processing 1M+ records daily.', s: '12ms avg latency' },
  { t: 'Hotel Booking + R Analytics', c: 'Hospitality • R + React', img: 'https://images.unsplash.com/photo-1566073771259-6a8506099945?w=800&q=80', d: 'Bookings, occupancy forecasts & revenue dashboards.', s: '+27% occupancy' },
  { t: 'SaccoGuard Security Hardening', c: 'Security • Audit + WAF', img: 'https://images.unsplash.com/photo-1550751827-4bd374c3f58b?w=800&q=80', d: 'Full pentest & hardening for a 8,000-member SACCO.', s: '0 breaches since' },
];
const TESTIS = [
  { n: 'Amina Juma', r: 'Director, Amina Pharmacy — Dar es Salaam', t: 'Jackson built our entire pharmacy system with M-Pesa. Official quotation, BRELA contract, training — everything was professional. Stock losses dropped to almost zero.', s: 5 },
  { n: 'Eng. Baraka Mwansa', r: 'CEO, Kilimo Fresh', t: 'The Swahili WhatsApp bot takes orders while we sleep. Clients think we hired 5 new staff! Best investment we made this year.', s: 5 },
  { n: 'Sarah Nkya', r: 'Headmistress, Moshi Academy', t: 'Fee collection used to be chaos. Now parents get instant SMS receipts and we see every shilling. The maintenance plan is worth every cent.', s: 5 },
  { n: 'David Mwakalinga', r: 'CTO, Fintech Startup — Nairobi', t: 'Their Rust microservice handles our peak load effortlessly. International quality, African pricing. The security audit alone saved us from a major breach.', s: 5 },
];
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

/* ---------- NAV / PAGES ---------- */
/* Kila sehemu ya landing ni ukurasa wake halisi — kubonyeza inakupeleka ukurasa mpya.
 * URLs are extensionless: the server rewrites /pricing -> pricing.php. */
const PAGES = {
  landing: '', services: 'services', stack: 'stack', ai: 'ai',
  pricing: 'pricing', process: 'process', work: 'work', track: 'track',
  contact: 'contact', auth: 'auth', client: 'client', admin: 'admin'
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
  if ($('langEnBtn')) $('langEnBtn').className = l === 'en' ? 'px-2.5 py-1 rounded-full bg-white/10 font-bold text-white' : 'px-2.5 py-1 rounded-full text-white/60 font-bold hover:bg-white/10';
  if ($('langSwBtn')) $('langSwBtn').className = l === 'sw' ? 'px-2.5 py-1 rounded-full bg-white/10 font-bold text-white' : 'px-2.5 py-1 rounded-full text-white/60 font-bold hover:bg-white/10';
  document.querySelectorAll('[data-en]').forEach(el => { el.textContent = l === 'sw' ? el.dataset.sw : el.dataset.en; });
}
function setLang(l) {
  try { localStorage.setItem('mts_lang', l); } catch (e) {}
  applyLang(l);
  toast(l === 'sw' ? 'Lugha imebadilishwa: Kiswahili ✓' : 'Language switched: English ✓', 'info');
}
try { LANG = localStorage.getItem('mts_lang') === 'sw' ? 'sw' : 'en'; } catch (e) {}
applyLang(LANG);

/* ---------- LANDING RENDER ---------- */
/* Kila sehemu inarenderiwa kwenye ukurasa wake — guards za kila section hapa chini. */
function renderServices() {
  if (!$('servicesGrid')) return;
  const cc = { cyan: 'text-cyan-300 bg-cyan-500/10 border-cyan-400/25', violet: 'text-violet-300 bg-violet-500/10 border-violet-400/25', gold: 'text-yellow-300 bg-yellow-500/10 border-yellow-400/25', emerald: 'text-emerald-300 bg-emerald-500/10 border-emerald-400/25', red: 'text-red-300 bg-red-500/10 border-red-400/25', fuchsia: 'text-fuchsia-300 bg-fuchsia-500/10 border-fuchsia-400/25', blue: 'text-blue-300 bg-blue-500/10 border-blue-400/25', teal: 'text-teal-300 bg-teal-500/10 border-teal-400/25' };
  $('servicesGrid').innerHTML = SERVICES.map(s => `
    <div class="service-card glass rounded-3xl p-6 border border-white/10">
      <div class="w-12 h-12 rounded-2xl border flex items-center justify-center text-lg ${cc[s.c]}"><i class="fa-solid ${s.icon}"></i></div>
      <div class="font-bold mt-4 text-[15px]">${s.t}</div>
      <p class="text-white/55 text-[12.5px] mt-2 leading-relaxed">${s.d}</p>
      <div class="flex flex-wrap gap-1.5 mt-3">${s.tags.map(t => `<span class="text-[10.5px] font-mono px-2 py-0.5 rounded-full bg-white/5 border border-white/10 text-white/60">${t}</span>`).join('')}</div>
      <button onclick="openQuoteWizard('${s.t}')" class="mt-4 text-[13px] font-bold text-cyan-300 hover:text-white">Request this →</button>
    </div>`).join('');
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
  if ($('secPlans')) $('secPlans').innerHTML = `<div class="col-span-full text-[12px] font-bold tracking-[.2em] text-red-300 mt-6"><i class="fa-solid fa-shield-halved mr-2"></i>SECURITY SERVICES</div>` + SEC.map(m => `
    <div class="glass rounded-3xl p-6">
      <div class="font-display font-bold text-lg">${m.n}</div>
      <div class="mt-2"><span class="font-display font-bold text-3xl text-red-200">${m.p}</span><span class="text-white/40 text-sm"> ${m.per}</span></div>
      <div class="grid gap-2 mt-4 text-[13px] text-white/65">${m.f.map(f => `<div class="flex gap-2"><i class="fa-solid fa-check text-red-300 mt-1 text-xs"></i>${f}</div>`).join('')}</div>
      <button onclick="openQuoteWizard('Cybersecurity Services')" class="w-full mt-5 py-3 rounded-xl font-bold text-sm border border-red-400/30 text-red-200 hover:bg-red-500/10">Secure My System →</button>
    </div>`).join('');
}
let PORTFOLIO_DB = PORTFOLIO.slice();
let TESTIS_DB = TESTIS.slice();

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
    }).join('') || '<div class="text-white/40 text-[13px] glass rounded-3xl p-8 col-span-full text-center">Portfolio projects will appear here after admin adds them. Coming soon!</div>';
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
  if (!$('testiBox') || !TESTIS_DB.length) return;
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
  $('aiDemoAnswer').innerHTML = `<span class="text-white/50 font-mono text-[12px]">Q: "${esc(q)}"</span><br><br>${ans[aiBrain]}<br><br><span class="badge bg-emerald-500/15 text-emerald-300 border border-emerald-400/30">TRAINED ON YOUR DATA • SW/EN • 1.8s</span>`;
  toast('Fine-tuned answer generated — imagine this trained on YOUR business!', 'info');
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
      ${r.adminNote ? `<div class="mt-4 rounded-xl bg-violet-500/[.08] border border-violet-400/25 p-3.5 text-[13px]"><span class="font-bold text-violet-300"><i class="fa-solid fa-user-tie mr-1"></i> Update from CEO:</span> ${esc(r.adminNote)}</div>` : ''}
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
  if (!n || !p || !e || !m) { toast('Please fill name, phone, email and message. / Jaza sehemu zote muhimu.', 'err'); return; }
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
  if (!n || !m) { toast('Jaza jina na ujumbe. / Please enter your name and message.', 'err'); return; }
  if (e && !/^[^@\s]+@[^@\s]+\.[^@\s]+$/.test(e)) { toast('Email si sahihi / Invalid email address.', 'err'); return; }
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
    toast('Asante kwa maoni yako! / Thanks for your feedback!', 'ok');
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
function togglePw(id) { const el = $(id); el.type = el.type === 'password' ? 'text' : 'password'; }
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

function openOtpModal(email, debugCode = null) {
  __OTP_EMAIL = email;
  const hint = debugCode
    ? `<div class="text-[11px] font-mono text-white/35 mt-2 rounded-lg bg-white/[.03] p-2 border border-white/10"><span class="text-white/50 font-bold">DEV debug</span> — your test OTP is: <span class="text-emerald-300 font-bold text-lg">${esc(debugCode)}</span></div>`
    : `<div class="text-[11px] text-white/35 mt-2"><i class="fa-solid fa-envelope-open-text mr-1 text-cyan-300"></i><span data-en="Check your inbox (and spam/junk folder) for a 6-digit code from mbilinyitech@gmail.com. Expires in 10 minutes." data-sw="Tafuta kwenye email yako (na spam/junk folder) code ya 6 tarakimu kutoka mbilinyitech@gmail.com. Itaisha baada ya dakika 10.">Check your inbox (and spam/junk folder) for a 6-digit code from mbilinyitech@gmail.com. Expires in 10 minutes.</span></div>`;
  openGen('<i class="fa-solid fa-shield-halved mr-1.5 text-cyan-300"></i>Verify Login', `
    <div class="text-[13px] text-white/65"><span data-en="We sent a verification code to" data-sw="Tumetuma code ya uthibitisho kwa">We sent a verification code to</span> <span class="font-bold text-white">${esc(email)}</span></div>
    ${hint}
    <label class="text-[12px] font-bold text-white/60 mt-4 block"><span data-en="ENTER 6-DIGIT CODE" data-sw="INGIZA CODE YA 6 TARAKIMU">ENTER 6-DIGIT CODE</span></label>
    <div class="grid grid-cols-6 gap-2 mt-2">
      ${[0,1,2,3,4,5].map(i => `<input id="otp${i}" inputmode="numeric" maxlength="1" class="input !text-center !text-xl !font-mono !tracking-[0.3em]" oninput="onOtpInput(${i})" onkeydown="onOtpKey(${i},event)">`).join('')}
    </div>
    <div class="flex flex-col sm:flex-row gap-2 mt-6">
      <button onclick="doVerifyOTP()" id="verifyBtn" class="grad-btn flex-1 py-3.5 rounded-xl font-bold text-sm"><i class="fa-solid fa-shield-check mr-2"></i><span data-en="Verify & Sign In" data-sw="Thibitisha na Uingie">Verify & Sign In</span></button>
      <button onclick="resendLoginCode()" id="resendBtn" class="px-4 py-3.5 rounded-xl border border-cyan-400/50 text-cyan-300 hover:text-white hover:bg-cyan-500/10 text-sm font-bold"><i class="fa-solid fa-rotate-right mr-1.5"></i><span data-en="Resend code" data-sw="Tuma tena">Resend code</span></button>
    </div>
    <button onclick="closeGen(); stopResendTimer();" class="w-full mt-3 text-[12px] text-white/40 hover:text-white/80"><span data-en="← Cancel — go back to Sign In" data-sw="← Ghairi — rudi kwenye Kuingia">← Cancel — go back to Sign In</span></button>
  `);
  startResendTimer(60);
  setTimeout(() => $('otp0') && $('otp0').focus(), 50);
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
  if (!__OTP_EMAIL) { toast('Enter email first / Andika email kwanza.', 'err'); return; }
  try {
    const d = await apiPost('api/auth.php', { action: 'resend-login-otp', email: __OTP_EMAIL });
    toast('A new 6-digit code has been emailed. / Code mpya imetumwa kwa email ✓', 'ok');
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
  if (!reset_token || !email) { toast('Missing reset token or email. / Token hakipo.', 'err'); return; }
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
      openOtpModal(d.email, d.otp_debug_only ?? null);
      return;
    }
    if (d && d.user) {
      window.__ME__ = d.user;
      toast(`Welcome back, ${esc(d.user.name)}! ✓`, 'ok');
      location.href = returnTarget(d.user.role);
    }
  } catch (err) { toast(esc(err.message), 'err'); }
}

async function doRegister() {
  const n = $('regName').value.trim(), ph = $('regPhone').value.trim(), e = $('regEmail').value.trim(), c = $('regCompany').value.trim(), p = $('regPass').value, p2 = $('regPass2').value;
  if (!n || !ph || !e) { toast('Fill all required fields.', 'err'); return; }
  if (p.length < 6) { toast('Password must be 6+ characters.', 'err'); return; }
  if (p !== p2) { toast('Passwords do not match.', 'err'); return; }
  try {
    const d = await apiPost('api/auth.php', { action: 'register', name: n, phone: ph, email: e, company: c, password: p, password2: p2 });
    window.__ME__ = d.user;
    toast(`Welcome, ${esc(n)}! Account created ✓`, 'ok');
    location.href = 'client';
  } catch (err) { toast(esc(err.message), 'err'); }
}

async function logout() {
  try { await apiPost('api/auth.php', { action: 'logout' }); } catch (e) { /* ignore */ }
  window.__ME__ = null;
  toast('Signed out securely.', 'info');
  location.href = '/';
}

/* ---------- QUOTE WIZARD ---------- */
let wizStep = 1, wizService = '';
const QTECH = ['Python', 'React', 'Rust', 'R', 'HTML/CSS/JS', 'Node.js', 'Flutter', 'PostgreSQL', 'Docker', 'AWS'];
let qTechSel = new Set();

function openQuoteWizard(preset = '') {
  wizStep = 1; qTechSel = new Set(); if (preset) wizService = preset;
  const u = currentUser();
  if (u) { $('qName').value = u.name; $('qEmail').value = u.email; $('qPhone').value = u.phone || ''; $('qCompany').value = u.company || ''; }
  $('qServiceGrid').innerHTML = SERVICES.map(s => `<button onclick="pickService('${s.t}',this)" class="qsvc text-left px-3.5 py-3 rounded-xl border text-[12.5px] font-bold ${wizService === s.t ? 'tab-active border-transparent' : 'border-white/10 text-white/70 hover:border-cyan-400/40'}"><i class="fa-solid ${s.icon} mr-1.5"></i>${s.t}</button>`).join('');
  $('qTechGrid').innerHTML = QTECH.map(t => `<button onclick="toggleQTech('${t}',this)" class="px-3.5 py-2 rounded-full border border-white/10 text-[12.5px] font-mono hover:border-cyan-400/40 text-white/60">${t}</button>`).join('');
  paintWiz(); $('quoteModal').classList.remove('hidden'); $('quoteModal').classList.add('flex'); document.body.style.overflow = 'hidden';
}
function pickService(t, btn) { wizService = t; document.querySelectorAll('.qsvc').forEach(b => { b.classList.remove('tab-active', 'border-transparent'); b.classList.add('border-white/10', 'text-white/70'); }); btn.classList.add('tab-active'); btn.classList.remove('border-white/10', 'text-white/70'); }
function toggleQTech(t, btn) { if (qTechSel.has(t)) { qTechSel.delete(t); btn.classList.remove('tab-active'); } else { qTechSel.add(t); btn.classList.add('tab-active'); } }
function closeQuoteWizard() { $('quoteModal').classList.add('hidden'); $('quoteModal').classList.remove('flex'); document.body.style.overflow = ''; }

function paintWiz() {
  document.querySelectorAll('.wizard-step').forEach(s => s.classList.toggle('active', +s.dataset.step === wizStep));
  $('wizProgress').innerHTML = [1, 2, 3, 4, 5].map(i => `<div class="flex-1 h-2 rounded-full ${i <= wizStep ? 'bg-gradient-to-r from-cyan-400 to-violet-500' : 'bg-white/10'}"></div>`).join('');
  $('wizLabel').textContent = `Step ${wizStep} of 5`;
  $('wizBack').classList.toggle('invisible', wizStep === 1);
  $('wizNext').innerHTML = wizStep === 5 ? '<i class="fa-solid fa-paper-plane mr-2"></i>Submit Request' : 'Continue →';
  if (wizStep === 5) {
    $('qReview').innerHTML = `
    <div><span class="text-white/45">Name:</span> <strong>${esc($('qName').value || '—')}</strong> • ${esc($('qPhone').value || '')} • ${esc($('qEmail').value || '')}</div>
    <div><span class="text-white/45">Service:</span> <strong class="text-cyan-300">${esc(wizService || '—')}</strong></div>
    <div><span class="text-white/45">Project:</span> <strong>${esc($('qTitle').value || '—')}</strong></div>
    <div><span class="text-white/45">Details:</span> ${esc(($('qDesc').value || '—').slice(0, 220))}</div>
    <div><span class="text-white/45">Tech:</span> <span class="font-mono text-cyan-200">${[...qTechSel].join(', ') || 'To be advised by analyst'}</span> • ${esc($('qPlatform').value)} • ${esc($('qPriority').value)}</div>
    <div><span class="text-white/45">Budget:</span> ${esc($('qBudget').value)} • <span class="text-white/45">Deadline:</span> ${esc($('qDeadline').value || 'Flexible')}</div>`;
  }
}

function wizMove(d) {
  if (d === 1) {
    if (wizStep === 1) {
      if (!$('qName').value.trim() || !$('qPhone').value.trim() || !$('qEmail').value.trim()) { toast('Fill name, phone & email.', 'err'); return; }
      if (!/^[^@\s]+@[^@\s]+\.[^@\s]+$/.test($('qEmail').value.trim())) { toast('Invalid email.', 'err'); return; }
    }
    if (wizStep === 2) {
      if (!wizService) { toast('Select a service. / Chagua huduma.', 'err'); return; }
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
      tech: [...qTechSel], platform: $('qPlatform').value, budget: $('qBudget').value,
      deadline: $('qDeadline').value, priority: $('qPriority').value,
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
  const valid = ['overview', 'requests', 'quotes', 'contracts', 'invoices', 'support'];
  if (!valid.includes(t)) t = 'overview';
  cTab = t;
  document.querySelectorAll('#clientNav .sidebar-link').forEach(b => b.classList.toggle('active', b.dataset.ctab === t));
  const u = currentUser(); if (!u) { showView('auth'); return; }
  const titles = { overview: 'Overview', requests: 'My Requests', quotes: 'Quotations — Nukuu za Bei', contracts: 'Contracts — Mikataba', invoices: 'Invoices & Payments', support: 'Support Center' };
  $('clientTabTitle').textContent = titles[t];
  $('clientContent').innerHTML = '<div class="text-white/45 text-[13.5px] flex items-center gap-2"><i class="fa-solid fa-circle-notch fa-spin"></i> Loading your data…</div>';

  let myReq, myQ, myC, myI, myT;
  try {
    [myReq, myQ, myC, myI, myT] = await Promise.all([
      apiGet('api/requests.php', { action: 'mine' }).then(d => d.requests),
      apiGet('api/quotes.php', { action: 'mine' }).then(d => d.quotes),
      apiGet('api/contracts.php', { action: 'mine' }).then(d => d.contracts),
      apiGet('api/invoices.php', { action: 'mine' }).then(d => d.invoices),
      apiGet('api/tickets.php', { action: 'mine' }).then(d => d.tickets),
    ]);
  } catch (e) { toast(esc(e.message), 'err'); $('clientContent').innerHTML = ''; return; }

  let h = '';
  if (t === 'overview') {
    const pay = await apiGet('api/settings.php', { action: 'list' }).catch(() => null);
    const P = pay && pay.payment ? pay.payment : null;
    const payCard = P ? `
    <div class="mt-5 rounded-2xl border border-gold/30 overflow-hidden" style="background:linear-gradient(135deg,rgba(245,179,1,.07),rgba(34,211,238,.05))">
      <div class="px-5 py-3.5 border-b border-white/10 flex flex-wrap items-center gap-2 justify-between" style="border-color:rgba(245,179,1,.25)">
        <div class="font-bold text-[14px]"><i class="fa-solid fa-wallet text-gold mr-2"></i><span data-en="Payment Details — Malipo Yako" data-sw="Maelezo ya Malipo — Malipo Yako">Payment Details — Malipo Yako</span></div>
        <span class="badge" style="background:rgba(245,179,1,.15);border:1px solid rgba(245,179,1,.4);color:#F5B301">50% Deposit • 50% Mid-Project</span>
      </div>
      <div class="p-5 grid gap-3 sm:grid-cols-2">
        <div class="rounded-xl bg-white/[.03] border border-white/10 p-4">
          <div class="text-[10.5px] font-bold uppercase tracking-widest text-cyan-300 mb-2"><i class="fa-solid fa-building-columns mr-1.5"></i>Bank Transfer</div>
          <div class="text-[13px] grid gap-1.5">
            <div class="flex justify-between gap-3"><span class="text-white/50">Bank</span><strong>${esc(P.bank.name)}</strong></div>
            <div class="flex justify-between gap-3"><span class="text-white/50">Account</span><strong class="font-mono text-gold">${esc(P.bank.account)}</strong></div>
            <div class="flex justify-between gap-3"><span class="text-white/50">Name</span><strong>${esc(P.bank.holder)}</strong></div>
            ${P.bank.swift ? `<div class="flex justify-between gap-3"><span class="text-white/50">SWIFT</span><strong class="font-mono">${esc(P.bank.swift)}</strong></div>` : ''}
          </div>
        </div>
        <div class="rounded-xl bg-white/[.03] border border-white/10 p-4">
          <div class="text-[10.5px] font-bold uppercase tracking-widest text-emerald-300 mb-2"><i class="fa-solid fa-mobile-screen-button mr-1.5"></i>Mobile Money</div>
          <div class="text-[13px] grid gap-1.5">
            <div class="flex justify-between gap-3"><span class="text-white/50">Network</span><strong>${esc(P.mobile.network)}</strong></div>
            <div class="flex justify-between gap-3"><span class="text-white/50">Number</span><strong class="font-mono text-gold">${esc(P.mobile.number)}</strong></div>
            <div class="flex justify-between gap-3"><span class="text-white/50">Name</span><strong>${esc(P.mobile.holder)}</strong></div>
          </div>
        </div>
        <div class="sm:col-span-2 text-[12.5px] text-white/65 rounded-xl bg-black/20 border border-white/10 p-3.5">
          <i class="fa-solid fa-circle-info text-cyan-300 mr-1.5"></i>${esc(P.instructions)}
        </div>
      </div>
    </div>` : '';
    h = `<div class="grid sm:grid-cols-4 gap-3">
      ${[['folder-open', 'Requests', myReq.length, 'violet'], ['file-invoice-dollar', 'Quotations', myQ.length, 'gold'], ['file-contract', 'Contracts', myC.length, 'emerald'], ['receipt', 'Invoices', myI.length, 'blue']].map(([i, l, n, c]) => `<div class="rounded-2xl bg-white/[.03] border border-white/10 p-4 text-center"><i class="fa-solid fa-${i} text-${c === 'gold' ? 'yellow-400' : c + '-300'} text-xl"></i><div class="font-display font-bold text-2xl mt-1">${n}</div><div class="text-[11px] text-white/50 font-bold uppercase tracking-wider">${l}</div></div>`).join('')}
    </div>
    <div class="mt-5 rounded-2xl border border-cyan-400/25 bg-cyan-500/[.05] p-5 flex flex-wrap items-center gap-4"><div class="flex-1 min-w-[220px]"><div class="font-bold">Need a new system, maintenance or security audit?</div><div class="text-[13px] text-white/55">Submit requirements — get an official priced quotation within 24hrs.</div></div><button onclick="openQuoteWizard()" class="grad-btn px-6 py-3 rounded-xl font-bold text-sm">+ New Request</button></div>
    <div class="font-bold mt-6 mb-3">Recent Activity</div>
    <div class="grid gap-2">${myReq.slice(0, 4).map(r => `<div class="flex items-center gap-3 p-3.5 rounded-xl bg-white/[.02] border border-white/10 text-[13px]"><span class="font-mono text-cyan-300 text-[12px]">${r.trackingId}</span><span class="flex-1 truncate">${esc(r.title)}</span><span class="badge status-${r.status.replace(' ', '')}">${r.status}</span></div>`).join('') || '<div class="text-white/40 text-sm">No requests yet — click New Request above.</div>'}</div>
    ${payCard}`;
  }
  if (t === 'requests') {
    h = `<div class="grid gap-3">${myReq.map(r => { const q = myQ.find(x => x.requestId === r.id); return `<div class="rounded-2xl border border-white/10 bg-white/[.02] p-5">
      <div class="flex flex-wrap items-center gap-2 justify-between"><div><span class="font-mono text-cyan-300 text-[12px] font-bold">${r.trackingId}</span><div class="font-bold text-[15px]">${esc(r.title)}</div><div class="text-[12px] text-white/45">${esc(r.service)} • ${fmtDate(r.createdAt)} • Priority: ${esc(r.priority)}</div></div><span class="badge status-${r.status.replace(' ', '')}">${r.status.toUpperCase()}</span></div>
      <p class="text-[13px] text-white/60 mt-2">${esc(r.description.slice(0, 180))}...</p>
      ${r.adminNote ? `<div class="mt-2 text-[13px] rounded-xl bg-violet-500/[.08] border border-violet-400/25 p-3"><strong class="text-violet-300">CEO Update:</strong> ${esc(r.adminNote)}</div>` : ''}
      <div class="flex flex-wrap gap-2 mt-3 text-[12px] font-bold"><button onclick="viewRequestDetail('${r.id}')" class="px-4 py-2 rounded-full bg-white/5 border border-white/10 hover:border-cyan-400/40">View Details</button>${q ? `<button onclick="clientTab('quotes')" class="px-4 py-2 rounded-full grad-btn">View Quotation ${fmtTZS(q.total)}</button>` : ''}<a href="https://wa.me/255796752645?text=Hi!%20About%20${r.trackingId}" target="_blank" class="px-4 py-2 rounded-full bg-[#25D366]/10 border border-[#25D366]/30 text-[#4be584]">WhatsApp Follow-up</a></div>
    </div>`; }).join('') || '<div class="text-white/40">No requests. <button onclick="openQuoteWizard()" class="text-cyan-300 font-bold underline">Submit your first request</button></div>'}</div>`;
  }
  if (t === 'quotes') {
    h = `<div class="grid gap-3">${myQ.map(q => { const r = myReq.find(x => x.id === q.requestId); return `<div class="rounded-2xl border ${q.status === 'Sent' ? 'border-yellow-400/30' : 'border-white/10'} bg-white/[.02] p-5">
      <div class="flex flex-wrap justify-between gap-2 items-start"><div><div class="font-bold">${q.id} • ${esc(r ? r.title : '')}</div><div class="text-[12px] text-white/45 font-mono">${q.trackingId} • Valid till ${fmtDate(q.validUntil)} • Status: <strong class="text-cyan-300">${q.status}</strong></div><span class="badge mt-1.5 inline-block" style="background:rgba(245,179,1,.12);border:1px solid rgba(245,179,1,.4);color:#F5B301"><i class="fa-solid fa-coins mr-1"></i>50/50 Terms</span></div><div class="text-right"><div class="font-display font-bold text-2xl grad-text">${fmtTZS(q.total)}</div><div class="text-[11px] text-white/40">incl. 18% VAT</div><div class="text-[11px] text-gold font-bold mt-0.5">Deposit ${fmtTZS(q.total / 2)} + Mid ${fmtTZS(q.total / 2)}</div></div></div>
      <div class="mt-3 rounded-xl border border-gold/30 p-3 text-[12.5px] text-white/70" style="background:rgba(245,179,1,.06)"><strong class="text-gold"><i class="fa-solid fa-circle-info mr-1"></i>Payment Terms — Masharti ya Malipo:</strong> <strong class="text-gold">50% deposit inalipwa kabla mradi kuanza</strong>, na <strong class="text-gold">50% mid-project inalipwa mradi ukifika nusu</strong> ya timeline (kabla ya delivery). Si Deposit + Delivery.</div>
      ${q.adminMessage ? `<div class="text-[13px] mt-2 p-3 rounded-xl bg-white/[.03] border border-white/10 whitespace-pre-line"><strong>From CEO:</strong> ${esc(q.adminMessage)}</div>` : ''}
      <div class="flex flex-wrap gap-2 mt-3"><button onclick="viewQuote('${q.id}')" class="px-4 py-2 rounded-full bg-white/5 border border-white/10 text-[12px] font-bold hover:border-cyan-400/40">View Breakdown</button>
      ${q.status === 'Sent' ? `<button onclick="respondQuote('${q.id}','Approved')" class="px-4 py-2 rounded-full bg-emerald-500/15 border border-emerald-400/40 text-emerald-200 text-[12px] font-bold">✓ Approve & Generate Contract</button><button onclick="respondQuote('${q.id}','Rejected')" class="px-4 py-2 rounded-full bg-red-500/10 border border-red-400/30 text-red-200 text-[12px] font-bold">Decline</button>` : q.status === 'Approved' ? `<span class="badge status-Approved">APPROVED ✓ — contract generated</span>` : ''}</div>
    </div>`; }).join('') || '<div class="text-white/40 text-sm">No quotations yet. Admin sends priced quotations here within 24hrs of your request.</div>'}</div>`;
  }
  if (t === 'contracts') {
    h = `<div class="grid gap-3">${myC.map(c => `<div class="rounded-2xl border border-white/10 bg-white/[.02] p-5 flex flex-wrap items-center gap-3 justify-between"><div><div class="font-bold">${c.id} — ${esc(c.projectTitle)}</div><div class="text-[12px] text-white/45 font-mono">${c.trackingId} • ${fmtTZS(c.total)} • ${c.duration}</div><span class="badge ${c.status === 'Signed' ? 'status-Completed' : c.status === 'Client Signed' ? 'status-Approved' : 'status-Quoted'} mt-1 inline-block">${c.status === 'Client Signed' ? 'CLIENT SIGNED — CEO PENDING' : c.status.toUpperCase()}</span></div><div class="flex gap-2"><button onclick="viewContract('${c.id}')" class="px-4 py-2 rounded-full grad-btn text-[12px] font-bold"><i class="fa-solid fa-file-contract mr-1"></i> ${c.status === 'Signed' ? 'View Signed' : c.status === 'Client Signed' ? 'Track CEO Signature' : 'Review & E-Sign'}</button></div></div>`).join('') || '<div class="text-white/40 text-sm">Contracts appear here after you approve a quotation.</div>'}</div>
    <div class="mt-3 rounded-2xl border border-cyan-400/25 bg-cyan-500/[.05] p-4 text-[12.5px] text-white/70"><strong class="text-white"><i class="fa-solid fa-coins text-gold mr-1"></i>Kanuni ya malipo (50/50):</strong> <strong class="text-gold">50% ya bei inalipwa kabla project kuanza</strong>; nusu iliyobaki inalipwa baada ya kukabidhi project. Deposit invoice yako ipo kwenye <button onclick="clientTab('invoices')" class="text-cyan-300 underline font-bold">Invoices & Pay</button>.</div>`;
  }
  if (t === 'invoices') {
    h = `<div class="grid gap-3">${myI.map(v => `<div class="rounded-2xl border border-white/10 bg-white/[.02] p-5 flex flex-wrap items-center justify-between gap-3"><div><div class="font-bold">${v.id} — ${esc(v.title)}</div><div class="text-[12px] text-white/45">Due ${fmtDate(v.dueDate)} • ${v.method}</div><span class="badge ${v.status === 'Paid' ? 'status-Completed' : 'status-Pending'} mt-1 inline-block">${v.status.toUpperCase()}</span></div><div class="text-right"><div class="font-display font-bold text-xl">${fmtTZS(v.amount)}</div>${v.status !== 'Paid' ? `<button onclick="payInvoice('${v.id}')" class="mt-2 px-5 py-2.5 rounded-xl gold-btn text-[13px] font-bold text-[#2a1500]"><i class="fa-solid fa-mobile-screen mr-1"></i> Pay via M-Pesa</button>` : '<div class="text-emerald-300 text-[13px] font-bold">✓ Paid — Asante!</div>'}</div></div>`).join('') || '<div class="text-white/40 text-sm">No invoices yet.</div>'}</div>
    <div class="mt-4 rounded-2xl border border-cyan-400/25 bg-cyan-500/[.05] p-4 text-[12.5px] text-white/70"><strong class="text-white"><i class="fa-solid fa-coins text-gold mr-1"></i>Kanuni ya malipo — 50/50:</strong> <strong class="text-gold">Lipa 50% (deposit) kabla project kuanza</strong> — mradi unanza rasmi baada ya deposit na saini ya CEO; nusu iliyobaki inalipwa baada ya kukabidhi project. <strong class="text-white">Payment:</strong> M-Pesa 0796 752 645 (Jackson Mbilinyi) • Bank details on invoice. Send receipt on WhatsApp after paying.</div>`;
  }
  if (t === 'support') {
    h = `<div class="flex flex-wrap gap-2"><input id="tkSub" class="input !w-auto flex-1 min-w-[200px]" placeholder="Subject"><button onclick="clientNewTicket()" class="grad-btn px-5 py-3 rounded-xl font-bold text-sm">+ New Ticket</button></div>
    <textarea id="tkMsg" rows="2" class="input mt-2" placeholder="Describe your issue..."></textarea>
    <div class="grid gap-3 mt-4">${myT.map(x => `<div class="rounded-2xl border border-white/10 bg-white/[.02] p-4"><div class="flex justify-between gap-2 flex-wrap"><div class="font-bold text-[14px]">${esc(x.subject)}</div><span class="badge ${x.status === 'Open' ? 'status-Pending' : 'status-Completed'}">${x.status.toUpperCase()}</span></div><div class="text-[13px] text-white/60 mt-1 whitespace-pre-line">${esc(x.message)}</div>${(x.replies || []).map(r => `<div class="mt-2 ml-4 p-3 rounded-xl bg-cyan-500/[.06] border border-cyan-400/20 text-[13px]"><strong class="text-cyan-300">${esc(r.by)}:</strong> ${esc(r.text)}</div>`).join('')}<div class="flex gap-2 mt-2"><input id="rep-${x.id}" class="input !py-2 text-[13px]" placeholder="Reply..."><button onclick="clientReply('${x.id}')" class="px-4 rounded-xl bg-white/5 border border-white/10 text-[12px] font-bold">Send</button></div></div>`).join('') || '<div class="text-white/40 text-sm">No tickets yet.</div>'}</div>`;
  }
  $('clientContent').innerHTML = h;
}

async function viewRequestDetail(id) {
  let r;
  try { r = (await apiGet('api/requests.php', { action: 'get', id })).request; } catch (e) { toast(esc(e.message), 'err'); return; }
  openGen(`Request ${r.trackingId}`, `<div class="grid gap-3 text-[14px]">
    <div class="flex justify-between flex-wrap gap-2"><span class="badge status-${r.status.replace(' ', '')}">${r.status}</span><span class="text-white/40 text-[12px]">${fmtDate(r.createdAt)} • ${esc(r.priority)}</span></div>
    <div><div class="text-[11px] font-bold text-white/40">SERVICE</div><div class="font-bold text-cyan-300">${esc(r.service)}</div></div>
    <div><div class="text-[11px] font-bold text-white/40">PROJECT TITLE</div><div class="font-bold">${esc(r.title)}</div></div>
    <div><div class="text-[11px] font-bold text-white/40">FULL REQUIREMENTS</div><div class="p-4 rounded-xl bg-white/[.03] border border-white/10 whitespace-pre-line text-white/75">${esc(r.description)}</div></div>
    <div class="grid sm:grid-cols-3 gap-2 text-[13px]"><div class="p-3 rounded-xl bg-white/[.03] border border-white/10"><div class="text-white/40 text-[11px] font-bold">TECH</div><span class="font-mono">${(r.tech || []).join(', ') || '—'}</span></div><div class="p-3 rounded-xl bg-white/[.03] border border-white/10"><div class="text-white/40 text-[11px] font-bold">BUDGET</div>${esc(r.budget)}</div><div class="p-3 rounded-xl bg-white/[.03] border border-white/10"><div class="text-white/40 text-[11px] font-bold">DEADLINE</div>${esc(r.deadline || 'Flexible')}</div></div>
    ${r.adminNote ? `<div class="p-3 rounded-xl bg-violet-500/[.08] border border-violet-400/25 text-[13px]"><strong class="text-violet-300">CEO note:</strong> ${esc(r.adminNote)}</div>` : ''}
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
  else toast('Quotation declined. Admin has been notified.', 'info');
  clientTab('quotes');
}

async function payInvoice(id) {
  openGen(`Pay ${id} — invoice`, `
    <div class="text-center"><div class="w-16 h-16 mx-auto rounded-2xl bg-[#25D366]/15 flex items-center justify-center text-3xl"><i class="fa-brands fa-whatsapp text-[#4be584]"></i></div>
    <div class="font-bold text-lg mt-3">M-Pesa Payment</div>
    <p class="text-white/55 text-[13px]">Send your payment to <strong class="font-mono text-white">0796 752 645</strong> (Jackson Mbilinyi), then click confirm.</p>
    <div class="mt-2 rounded-xl bg-cyan-500/[.06] border border-cyan-400/25 p-3 text-[12px] text-white/70 text-left"><i class="fa-solid fa-coins text-gold mr-1"></i>Deposit ya <strong class="text-white">50%</strong> inaruhusu CEO kusaini na kuanza project. Nusu iliyobaki inalipwa baada ya kukabidhi project.</div>
    <div class="grid grid-cols-3 gap-2 mt-4 text-[12px]"><button class="py-2.5 rounded-xl border border-white/10 font-bold">M-Pesa</button><button class="py-2.5 rounded-xl border border-white/10 font-bold">Tigo Pesa</button><button class="py-2.5 rounded-xl border border-white/10 font-bold">Bank</button></div>
    <input id="payRef" class="input mt-3 font-mono" placeholder="Enter M-Pesa transaction code (e.g. TX98412...)">
    <button onclick="confirmPay('${id}')" class="gold-btn w-full mt-3 py-3.5 rounded-xl font-bold text-[#2a1500]">I Have Paid — Confirm</button></div>`);
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
      <div><div class="font-bold text-lg tracking-tight">MBILINYI TECH SOLUTIONS</div><div class="text-[11px] text-slate-500">BRELA Registered (Z-418822-77-TZ) • TIN 192-147-522 • Dodoma, Tanzania<br>0796752645 • mbilinyitech@gmail.com • @mbilinyitech</div></div>
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

async function adminTab(t) {
  aTab = t;
  document.querySelectorAll('#adminNav .sidebar-link').forEach(b => b.classList.toggle('active', b.dataset.atab === t));
  const titles = { overview: 'Business Dashboard', requests: 'Client Requests Inbox', quotes: 'Quotations & Pricing (Admin Sets Prices)', contracts: 'Contracts Manager', portfolio: 'Portfolio Projects (Admin CRUD)', testimonials: 'Client Testimonials (Admin CRUD)', users: 'Manage All Accounts', invoices: 'Invoices & Revenue', tickets: 'Support Tickets', settings: 'Settings — Payments, Prices & Company Info', feedback: 'Client Feedback & Reviews', email: 'Email / SMTP Configuration' };
  $('adminTabTitle').textContent = titles[t];
  $('adminContent').innerHTML = '<div class="text-white/45 text-[13.5px] flex items-center gap-2"><i class="fa-solid fa-circle-notch fa-spin"></i> Loading…</div>';

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
        <p class="text-[13px] text-white/65 mt-1.5 whitespace-pre-line">${esc(r.description)}</p>
        <div class="flex flex-wrap gap-2 mt-3">
          <select onchange="setReqStatus('${r.id}',this.value)" class="input !w-auto !py-2 text-[12px] font-bold">${['Pending', 'Under Review', 'Quoted', 'Approved', 'In Progress', 'Completed', 'Rejected'].map(s => `<option ${r.status === s ? 'selected' : ''}>${s}</option>`).join('')}</select>
          <button onclick="openRequestEditor('${r.id}')" class="px-4 py-2 rounded-xl bg-white/5 border border-white/10 text-[12px] font-bold hover:border-cyan-400/40"><i class="fa-solid fa-pen mr-1"></i>Edit</button>
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

    if (t === 'settings') {
      const [sd, pl] = await Promise.all([
        apiGet('api/settings.php', { action: 'list' }),
        apiGet('api/settings.php', { action: 'plans' }),
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

      <div class="mt-4"><button onclick="saveAllSettings()" class="grad-btn px-7 py-3.5 rounded-xl font-bold text-sm"><i class="fa-solid fa-floppy-disk mr-2"></i>Save Settings</button></div>

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
  document.querySelectorAll('[data-setkey]').forEach(i => { settings[i.dataset.setkey] = i.value; });
  try {
    await apiPost('api/settings.php', { action: 'save_all', settings });
    toast(`Saved ${Object.keys(settings).length} settings ✓ — clients see them immediately.`, 'ok');
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
      <div><label class="text-[12px] font-bold text-white/60">STAT / RESULT TAG</label><input id="pfStat" class="input mt-1" value="${esc(p.stat)}" placeholder="e.g. +27% revenue, Zero fee leakage"></div>
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

/* full edit of a request (admin) */
async function openRequestEditor(id) {
  let r;
  try { r = (await apiGet('api/requests.php', { action: 'get', id })).request; } catch (e) { toast(esc(e.message), 'err'); return; }
  const services = SERVICES.map(s => `<option ${r.service === s.t ? 'selected' : ''}>${s.t}</option>`).join('');
  const extra = SERVICES.some(s => s.t === r.service) ? '' : `<option selected>${esc(r.service)}</option>`;
  openGen(`Edit Request — ${r.trackingId}`, `
    <label class="text-[12px] font-bold text-white/60">SERVICE</label><select id="reService" class="input mt-1">${extra}${services}</select>
    <label class="text-[12px] font-bold text-white/60 mt-3 block">PROJECT TITLE</label><input id="reTitle" class="input mt-1" value="${esc(r.title)}">
    <label class="text-[12px] font-bold text-white/60 mt-3 block">REQUIREMENTS</label><textarea id="reDesc" rows="4" class="input mt-1">${esc(r.description)}</textarea>
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
  toast('Mkataba hauwezi kufutwa — ni rekodi ya kisheria inayohifadhiwa milele kwa mteja na kampuni. / Contracts are permanent and cannot be deleted.', 'info');
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

/* ---------- CHATBOT ---------- */
let chatOpened = false;
function toggleChat() {
  const w = $('chatWindow'); w.classList.toggle('hidden-chat');
  if (!chatOpened) {
    chatOpened = true;
    botSay(`Habari! 👋 I'm <strong>Mbilinyi AI</strong> — assistant wa <strong>Mbilinyi Tech Solutions</strong> (CEO Jackson Mbilinyi).<br><br>Naweza kukusaidia na: <strong>services, prices/bei, tracking, maintenance, security, AI fine-tuning</strong> — in English au Kiswahili. Unahitaji nini leo?`);
    renderQuick(['💰 Prices / Bei', '🛠 Request Service', '📦 Track Order', '🔒 Security', '🤖 AI & Chatbots', '📞 Contact']);
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
      const reply = `📦 <strong>${r.trackingId}</strong> — <strong>${esc(r.title)}</strong><br>Status: <strong>${r.status}</strong>${q ? `<br>Quotation: <strong>${fmtTZS(q.total)}</strong> (${q.status})` : ''}<br>${r.adminNote ? `CEO update: ${esc(r.adminNote)}<br>` : ''}<a href="track" class="text-cyan-300 underline font-bold">Open full timeline →</a>`;
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
      return `📦 <strong>${r.trackingId}</strong> — <strong>${esc(r.title)}</strong><br>Status: <strong>${r.status}</strong>${q ? `<br>Quotation: <strong>${fmtTZS(q.total)}</strong> (${q.status})` : ''}<br>${r.adminNote ? `CEO update: ${esc(r.adminNote)}<br>` : ''}<a href="track" class="text-cyan-300 underline font-bold">Open full timeline →</a>`;
    } catch (e) {
      return `Sikupata <strong>${trackMatch[0]}</strong>. Hakikisha ID ni sahihi au WhatsApp 0796752645. Track IDs look like <span class="font-mono">MTS-2026-XXXX</span>.`;
    }
  }
  if (/track|fuatilia|status|oda|order|wapi/.test(t)) return `Kufuatilia oda yako, niandikie <strong>Tracking ID</strong> yako (mfano <span class="font-mono">MTS-2026-4810</span>) — au <a href="track" class="text-cyan-300 underline font-bold">fungua ukurasa wa Tracking →</a>`;
  if (/bei|price|cost|gharama|pesa|how much|quotation|nukuu/.test(t)) return `💰 <strong>Pricing guide (TZS):</strong><br>• Websites: 500K+ • Business systems: 1.5M–5M • Mobile apps: 2M+ • Chatbots: 600K+ • Maintenance: 150K/mo+ • Security audit: 800K+<br><br>Exact price = official quotation from CEO within 24hrs after you submit the <a href="#" onclick="openQuoteWizard();return false;" class="text-cyan-300 underline font-bold">Requirements Form →</a>`;
  if (/maint|matengenezo|support|update|bug|warranty/.test(t)) return `🛠 <strong>Software Maintenance:</strong> Starter 150K/mo • Business SLA 450K/mo (popular) • Enterprise 24/7 1.2M/mo.<br>Includes updates, backups, bug fixes & monitoring. <a href="#" onclick="openQuoteWizard('Software Maintenance');return false;" class="text-cyan-300 underline font-bold">Subscribe →</a>`;
  if (/secur|hack|virus|pentest|audit|usalama/.test(t)) return `🔒 <strong>Security Services:</strong> Audit 800K • Pentest 2.5M • Managed 900K/mo.<br>OWASP Top-10, malware removal, WAF, SSL & staff training. Zero breaches to date. <a href="#" onclick="openQuoteWizard('Cybersecurity Services');return false;" class="text-cyan-300 underline font-bold">Secure my system →</a>`;
  if (/fine|tuning|ai|chatbot|llm|robot|bot/.test(t)) return `🤖 <strong>AI Services:</strong> WhatsApp & website chatbots (Swahili + English) + LLM fine-tuning on YOUR data (price lists, FAQs, policies). Try the <a href="ai" class="text-cyan-300 underline font-bold">live AI demo →</a> or <a href="#" onclick="openQuoteWizard('AI Solutions & Fine-Tuning');return false;" class="text-cyan-300 underline font-bold">request AI project →</a>`;
  if (/service|huduma|offer|fanya|mnatoa/.test(t)) return `We offer <strong>15</strong> service lines:<br><strong>1.</strong> Custom Software<br><strong>2.</strong> System Analysis<br><strong>3.</strong> IT Consultancy<br><strong>4.</strong> Maintenance<br><strong>5.</strong> Cybersecurity<br><strong>6.</strong> AI Chatbots<br><strong>7.</strong> AI Fine-Tuning<br><strong>8.</strong> Cloud & DevOps<br><strong>9.</strong> BRELA & Business Registration<br><strong>10.</strong> TRA & Tax Services<br><strong>11.</strong> Computer Troubleshooting<br><strong>12.</strong> Windows & OS Installation<br><strong>13.</strong> Email Setup & Repair<br><strong>14.</strong> Government e-Services<br><strong>15.</strong> Software Installation<br><a href="services" class="text-cyan-300 underline font-bold">View details →</a> or <a href="#" onclick="openQuoteWizard();return false;" class="text-cyan-300 underline font-bold">Start a request →</a>`;
  if (/brela|tin|register|sajili|license|certificate/.test(t)) return `✅ <strong>100% Legitimate:</strong> BRELA Registered (Z-418822-77-TZ) • TRA TIN <span class="font-mono">192-147-522</span>. We also DO BRELA + TRA registrations for YOUR business. Every project includes a stamped service contract, e-signature & official receipts.`;
  if (/contact|mawasiliano|number|namba|email|instagram|whatsapp|call|ceo|jackson|mbilinyi/.test(t)) return `📞 <strong>CEO Jackson Mbilinyi</strong><br>• Phone/WhatsApp/SMS: <a href="tel:+255796752645" class="text-cyan-300 font-bold">0796 752 645</a><br>• Email: <a href="mailto:mbilinyitech@gmail.com" class="text-cyan-300 font-bold">mbilinyitech@gmail.com</a><br>• Instagram: <strong>@mbilinyitech</strong><br>• Hours: Mon–Sat 8:00–20:00 EAT`;
  if (/\br\b|rust|react|python|html|php|laravel|language|stack|teknolojia/.test(t)) return `💻 <strong>Our stack:</strong> PHP 96% • Laravel 94% • Python 98% • React 97% • Rust 90% • R 88% • HTML/CSS/JS 99% • Node 92% • Flutter 85% • SQL & Cloud 94%. We choose the right tool per project — speed, security & scale guaranteed.`;
  if (/contract|mkataba|sign|saini|agreement/.test(t)) return `📄 <strong>Contracts:</strong> after you approve a quotation, a BRELA-compliant contract with strict 50/50 payment terms is generated. You e-sign with finger/mouse in your portal FIRST, then CEO signs. Legally binding under Tanzanian law — archived on both portals & emailed on completion.`;
  if (/pay|lipia|mpesa|invoice|bill|deposit|malipo/.test(t)) return `💳 <strong>Payments — 50/50 STRICT (NOT deposit + delivery):</strong><br>① <strong>50% Deposit</strong> → payable BEFORE project start (verified by CEO; project does NOT start until paid).<br>② <strong>50% Mid-Project</strong> → at 50% of timeline (NOT at final delivery). Final source code, training & handover only AFTER second installment paid.<br><br>M-Pesa: <strong>0796 752 645</strong> (Jackson Mbilinyi). Bank on request. Late payments: 2% per week penalty.`;
  if (/hi|hello|hey|habari|mambo|niaje|salut|jambo/.test(t)) return `Habari! 👋 Karibu <strong>Mbilinyi Tech Solutions</strong>. <span class="grad-text font-bold">"Your Problem, Our Solution"</span> — <em>"Shida Yako, Tatizo Letu."</em> Mimi ni AI assistant. Uliza kuhusu bei, huduma, tracking, contracts — au andika <strong>"request"</strong> kuanza ombi. How can I help?`;
  if (/request|omba|form|quote|start|anza|need|nataka|nahitaji/.test(t)) return `Great! 🚀 Click <a href="#" onclick="openQuoteWizard();return false;" class="text-cyan-300 underline font-bold">HERE to open the Requirements Form</a> — it takes 2 minutes, and CEO Jackson responds with official prices within 24hrs.`;
  if (/thank|asante|shukran/.test(t)) return `Karibu sana! 🙏 Tunathamini. Anything else — tracking, bei, au msaada? We're here 24/7.`;
  if (/human|agent|mtu|ceo|call me|nipigie/.test(t)) return `Nitakupigia! 📲 CEO Jackson is available on <a href="https://wa.me/255796752645" target="_blank" class="text-emerald-300 font-bold underline">WhatsApp 0796752645</a> — usually replies within minutes (8:00–20:00). Meanwhile, unaweza kuacha ticket hapa: eleza tatizo lako.`;
  return `Nimekuelewa — "<em>${esc(raw).slice(0, 80)}</em>". 🤔<br>Let me connect the dots: for <strong>prices</strong> type <em>"bei"</em>, for <strong>tracking</strong> paste your <em>MTS ID</em>, for a <strong>new project</strong> type <em>"request"</em>, or chat with a human on <a href="https://wa.me/255796752645" target="_blank" class="text-emerald-300 font-bold underline">WhatsApp →</a>`;
}

/* ---------- INIT ---------- */
document.addEventListener('DOMContentLoaded', () => {
  const u = currentUser();
  const btn = $('navAuthBtn');
  if (btn && u) {
    btn.innerHTML = `<i class="fa-solid fa-gauge text-cyan-300"></i> ${u.role === 'admin' ? 'Admin Panel' : 'My Portal'}`;
  }

  renderHero();
  renderServices();
  renderStack();
  renderPricing();
  loadPublicPrices();
  renderWork();
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

  if (typeof gsap !== 'undefined' && typeof ScrollTrigger !== 'undefined') {
    gsap.registerPlugin(ScrollTrigger);
  }

  if ($('view-landing')) {
    setTimeout(() => { toast('👋 Karibu Mbilinyi Tech Solutions! Click the robot for AI help, or Request Service to start.', 'info'); }, 1200);
  }

  // Auto-open password reset panel if URL has ?reset_token=...&email=...
  if ($('authLogin') || $('authReset')) {
    const q = getUrlParams();
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
});

document.addEventListener('keydown', e => { if (e.key === 'Escape') { closeQuoteWizard(); closeGen(); } });
