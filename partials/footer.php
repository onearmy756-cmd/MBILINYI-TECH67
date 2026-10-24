</main>

<!-- FOOTER -->
<footer class="border-t border-white/10 bg-[#04070F] mt-6">
  <div class="max-w-7xl mx-auto px-4 py-12 grid md:grid-cols-4 gap-8 text-[13.5px]">
    <div><div class="flex items-center gap-3"><div class="w-10 h-10 rounded-xl grad-btn flex items-center justify-center font-bold">M</div><div class="font-display font-bold">MBILINYI TECH SOLUTIONS</div></div><p class="text-white/50 mt-3 leading-relaxed" data-en="BRELA-registered software house delivering custom systems, AI, maintenance & security — locally rooted, globally competitive." data-sw="Kampuni ya software iliyosajiliwa BRELA inayotoa mifumo maalum, AI, maintenance na usalama — yenye mizizi ya nyumbani, inashindana kimataifa.">BRELA-registered software house delivering custom systems, AI, maintenance & security — locally rooted, globally competitive.</p><div class="flex gap-2 mt-4"><a href="https://instagram.com/mbilinyitech" target="_blank" class="w-9 h-9 rounded-full bg-white/5 border border-white/10 flex items-center justify-center hover:border-pink-400/50"><i class="fa-brands fa-instagram text-pink-300"></i></a><a href="https://wa.me/255796752645" target="_blank" class="w-9 h-9 rounded-full bg-white/5 border border-white/10 flex items-center justify-center hover:border-green-400/50"><i class="fa-brands fa-whatsapp text-green-300"></i></a><a href="mailto:mbilinyitech@gmail.com" class="w-9 h-9 rounded-full bg-white/5 border border-white/10 flex items-center justify-center hover:border-blue-400/50"><i class="fa-solid fa-envelope text-blue-300"></i></a><a href="tel:+255796752645" class="w-9 h-9 rounded-full bg-white/5 border border-white/10 flex items-center justify-center hover:border-cyan-400/50"><i class="fa-solid fa-phone text-cyan-300"></i></a></div></div>
    <div><div class="font-bold mb-3" data-en="Services" data-sw="Huduma">Services</div><div class="grid gap-2 text-white/55">
      <span data-en="Custom Software (Mobile • Desktop • Web)" data-sw="Software Maalum (Simu • Kompyuta • Wavuti)">Custom Software (Mobile • Desktop • Web)</span>
      <span data-en="System Analysis & Design" data-sw="Uchambuzi na Ubunifu wa Mifumo">System Analysis & Design</span>
      <span data-en="IT Consultancy" data-sw="Ushauri wa IT">IT Consultancy</span>
      <span data-en="Software Maintenance & SLA" data-sw="Matengenezo ya Software na SLA">Software Maintenance & SLA</span>
      <?php if (service_visible('cybersecurity')): ?>
      <span data-en="Cybersecurity & Audits" data-sw="Usalama wa Mtandao na Ukaguzi">Cybersecurity & Audits</span>
      <?php endif; ?>
      <?php if (service_visible('forensics')): ?>
      <span data-en="Digital Forensics" data-sw="Uchunguzi wa Kidijitali">Digital Forensics</span>
      <?php endif; ?>
      <span data-en="Build Your Own AI & Chatbot" data-sw="Jenga AI na Chatbot Yako">Build Your Own AI & Chatbot</span>
    </div></div>
    <div><div class="font-bold mb-3" data-en="Company" data-sw="Kampuni">Company</div><div class="grid gap-2 text-white/55"><span>CEO: Jackson Mbilinyi</span><span>BRELA Reg: 706668</span><span class="font-mono">TIN: 192-147-522</span><span data-en="Dodoma, Tanzania • Work delivered online" data-sw="Dodoma, Tanzania • Kazi zote mtandaoni">Dodoma, Tanzania • Work delivered online</span><a href="auth" class="text-left text-cyan-300 font-bold" data-en="Client Login →" data-sw="Kuingia kwa Mteja →">Client Login →</a></div></div>
    <div><div class="font-bold mb-3" data-en="Contact" data-sw="Mawasiliano">Contact</div><div class="grid gap-2 text-white/55"><a href="tel:+255796752645" class="hover:text-white"><i class="fa-solid fa-phone mr-2 text-green-300"></i>0796 752 645</a><a href="mailto:mbilinyitech@gmail.com" class="hover:text-white"><i class="fa-solid fa-envelope mr-2 text-blue-300"></i>mbilinyitech@gmail.com</a><span><i class="fa-brands fa-instagram mr-2 text-pink-300"></i>@mbilinyitech</span><span data-en="Mon–Sat • 8:00–20:00 EAT" data-sw="Jumatatu–Jumamosi • 8:00–20:00 EAT">Mon–Sat • 8:00–20:00 EAT</span></div></div>
  </div>
  <div class="border-t border-white/10 py-4 text-center">
    <div class="text-[12px] font-semibold tracking-[.14em] text-cyan-300/70 mb-1.5" data-en="★ YOUR PROBLEM, OUR SOLUTION ★" data-sw="★ SHIDA YAKO, TATUZO LETU ★">★ YOUR PROBLEM, OUR SOLUTION ★</div>
    <div class="flex flex-wrap items-center justify-center gap-x-4 gap-y-1.5 text-[12px] text-white/55 mb-2">
      <a href="terms" class="hover:text-cyan-300 transition" data-en="Terms of Service" data-sw="Masharti ya Huduma">Terms of Service</a>
      <span class="text-white/20">•</span>
      <a href="refund" class="hover:text-cyan-300 transition" data-en="Refund Policy" data-sw="Sera ya Kurudisha Pesa">Refund Policy</a>
      <span class="text-white/20">•</span>
      <a href="privacy" class="hover:text-cyan-300 transition" data-en="Privacy Policy" data-sw="Sera ya Faragha">Privacy Policy</a>
      <span class="text-white/20">•</span>
      <a href="feedback" class="hover:text-cyan-300 transition" data-en="Feedback" data-sw="Maoni">Feedback</a>
      <span class="text-white/20">•</span>
      <a href="reports" class="hover:text-cyan-300 transition" data-en="Reports" data-sw="Ripoti">Reports</a>
      <span class="text-white/20">•</span>
      <a href="faq" class="hover:text-cyan-300 transition" data-en="FAQ" data-sw="Maswali">FAQ</a>
      <span class="text-white/20">•</span>
      <a href="community" class="hover:text-cyan-300 transition" data-en="Community" data-sw="Jamii">Community</a>
    </div>
    <div class="text-[12px] text-white/35">&copy; <?php
      /* Auto-rolls over every new year — no manual edit needed. */
      $founded = 2026;
      $now     = (int)date('Y');
      echo ($now > $founded) ? $founded . '–' . $now : $now;
    ?> <span data-en="Mbilinyi Tech Solutions • CEO Jackson Mbilinyi • Built to production standards • BRELA &amp; TIN Verified" data-sw="Mbilinyi Tech Solutions • CEO Jackson Mbilinyi • Imejengwa kwa viwango vya uzalishaji • Imethibitishwa BRELA na TIN">Mbilinyi Tech Solutions • CEO Jackson Mbilinyi • Built to production standards • BRELA &amp; TIN Verified</span></div>
  </div>
</footer>

<!-- WHATSAPP FLOAT -->
<a href="https://wa.me/255796752645?text=Hello%20Mbilinyi%20Tech%20Solutions!%20I%20need%20a%20service." target="_blank" class="fixed bottom-24 right-4 z-[200] w-14 h-14 rounded-full bg-[#25D366] flex items-center justify-center shadow-2xl hover:scale-110 transition" title="Chat on WhatsApp"><i class="fa-brands fa-whatsapp text-2xl text-white"></i><span class="absolute -top-1 -right-1 w-4 h-4 bg-red-500 rounded-full border-2 border-[#050914] animate-pulse"></span></a>

<!-- CHATBOT -->
<button onclick="toggleChat()" class="fixed bottom-6 right-4 z-[300] grad-btn w-14 h-14 rounded-full flex items-center justify-center text-xl shadow-2xl hover:scale-110 transition"><i class="fa-solid fa-robot"></i></button>
<div id="chatWindow" class="hidden-chat fixed bottom-[86px] right-4 z-[300] w-[380px] max-w-[94vw] glass-strong rounded-3xl overflow-hidden shadow-2xl flex flex-col" style="height:560px;max-height:72vh">
  <div class="p-4 bg-gradient-to-r from-cyan-600 via-blue-600 to-violet-600 flex items-center gap-3">
    <div class="w-11 h-11 rounded-2xl bg-white/15 flex items-center justify-center text-xl backdrop-blur"><i class="fa-solid fa-robot"></i></div>
    <div class="flex-1"><div class="font-bold text-[15px]"><span data-en="Mbilinyi AI Assistant" data-sw="Msaidizi wa AI wa Mbilinyi">Mbilinyi AI Assistant</span></div><div class="text-[11.5px] text-white/80 flex items-center gap-1.5"><span class="w-2 h-2 rounded-full bg-emerald-300 animate-pulse"></span> <span data-en="Online • speaks & listens" data-sw="Ipo hewani • husema na husikiliza">Online • speaks &amp; listens</span></div></div>
    <button id="voiceBtn" onclick="toggleVoice()" class="w-8 h-8 rounded-full bg-white/15" title="Speak replies / Sema majibu"><i class="fa-solid fa-volume-high text-sm"></i></button>
    <button onclick="toggleChat()" class="w-8 h-8 rounded-full bg-white/15"><i class="fa-solid fa-xmark text-sm"></i></button>
  </div>
  <div id="chatBody" class="flex-1 overflow-y-auto p-4 space-y-3 text-[13.5px]"></div>
  <div class="p-3 border-t border-white/10">
    <div class="flex flex-wrap gap-1.5 mb-2.5" id="chatQuick"></div>
    <div class="flex gap-2"><input id="chatInput" onkeydown="if(event.key==='Enter')sendChat()" class="input !py-2.5 text-[13px]" data-ph-en="Ask about services, prices, tracking..." data-ph-sw="Uliza kuhusu huduma, bei, ufuatiliaji..." placeholder="Ask about services, prices, tracking..."><button id="micBtn" onclick="toggleMic()" class="w-11 h-11 rounded-xl shrink-0 bg-white/5 border border-white/15" title="Speak your question / Ongea swali lako"><i class="fa-solid fa-microphone text-sm"></i></button><button onclick="sendChat()" class="grad-btn w-11 h-11 rounded-xl shrink-0"><i class="fa-solid fa-paper-plane text-sm"></i></button></div>
  </div>
</div>

<!-- QUOTE WIZARD MODAL -->
<div id="quoteModal" class="fixed inset-0 z-[400] hidden items-center justify-center p-4 bg-black/70 backdrop-blur-sm overflow-y-auto">
  <div class="glass-strong rounded-3xl w-full max-w-3xl my-6 overflow-hidden">
    <div class="p-5 md:p-6 bg-gradient-to-r from-cyan-600/20 to-violet-600/20 border-b border-white/10 flex items-center justify-between">
      <div><div class="text-[11px] font-bold tracking-[.2em] text-cyan-300" data-en="CLIENT REQUIREMENTS FORM" data-sw="FOMU YA MAHITAJI">CLIENT REQUIREMENTS FORM</div><div class="font-display font-bold text-xl" data-en="Request a Service" data-sw="Omba Huduma">Request a Service</div></div>
      <button onclick="closeQuoteWizard()" class="w-9 h-9 rounded-full bg-white/10 hover:bg-white/20"><i class="fa-solid fa-xmark"></i></button>
    </div>
    <div class="px-5 md:px-6 pt-5"><div class="flex items-center gap-2" id="wizProgress"></div></div>
    <div class="p-5 md:p-6">
      <!-- STEP 1 -->
      <div class="wizard-step active" data-step="1">
        <h3 class="font-bold text-lg"><i class="fa-solid fa-user text-cyan-300 mr-2"></i><span data-en="Step 1: Your Details" data-sw="Hatua 1: Taarifa Zako">Step 1: Your Details</span></h3>
        <div class="grid sm:grid-cols-2 gap-3 mt-4">
          <div><label class="text-[12px] font-bold text-white/60" data-en="FULL NAME *" data-sw="JINA KAMILI *">FULL NAME *</label><input id="qName" class="input mt-1" data-ph-en="Your full name" data-ph-sw="Jina lako kamili" placeholder="Your full name"></div>
          <div><label class="text-[12px] font-bold text-white/60" data-en="PHONE / WHATSAPP *" data-sw="SIMU / WHATSAPP *">PHONE / WHATSAPP *</label><input id="qPhone" class="input mt-1" placeholder="07XX XXX XXX"></div>
        </div>
        <div class="grid sm:grid-cols-2 gap-3 mt-3">
          <div><label class="text-[12px] font-bold text-white/60" data-en="EMAIL *" data-sw="BARUA PEPE *">EMAIL *</label><input id="qEmail" class="input mt-1" placeholder="you@email.com"></div>
          <div><label class="text-[12px] font-bold text-white/60" data-en="COMPANY / ORGANIZATION" data-sw="KAMPUNI / TAASISI">COMPANY / ORGANIZATION</label><input id="qCompany" class="input mt-1" data-ph-en="Optional" data-ph-sw="Si lazima" placeholder="Optional"></div>
        </div>
      </div>
      <!-- STEP 2 -->
      <div class="wizard-step" data-step="2">
        <h3 class="font-bold text-lg"><i class="fa-solid fa-briefcase text-violet-300 mr-2"></i><span data-en="Step 2: Service & Project" data-sw="Hatua 2: Huduma na Mradi">Step 2: Service & Project</span></h3>
        <label class="text-[12px] font-bold text-white/60 mt-4 block" data-en="SELECT SERVICE *" data-sw="CHAGUA HUDUMA *">SELECT SERVICE *</label>
        <div class="grid sm:grid-cols-2 gap-2 mt-2" id="qServiceGrid"></div>
        <!-- Service-specific fields (each service has its own form) -->
        <div id="qServiceFields" class="mt-4"></div>
        <label class="text-[12px] font-bold text-white/60 mt-4 block" id="qTitleLabel" data-en="PROJECT TITLE *" data-sw="KICHWA CHA MRADI *">PROJECT TITLE *</label><input id="qTitle" class="input mt-1" data-ph-en="e.g. Pharmacy stock + sales system with M-Pesa" data-ph-sw="mf. Mfumo wa pharmacy wa stock na mauzo na M-Pesa" placeholder="e.g. Pharmacy stock + sales system with M-Pesa">
        <label class="text-[12px] font-bold text-white/60 mt-3 block" id="qDescLabel" data-en="DETAILED REQUIREMENTS *" data-sw="MAHITAJI KWA UNDANI *">DETAILED REQUIREMENTS *</label><textarea id="qDesc" rows="4" class="input mt-1" data-ph-en="Describe features, users, integrations (M-Pesa, SMS), reports you need..." data-ph-sw="Eleza vipengele, watumiaji, miunganisho (M-Pesa, SMS), ripoti unazohitaji..." placeholder="Describe features, users, integrations (M-Pesa, SMS), reports you need..."></textarea>
      </div>
      <!-- STEP 3 (rendered per service -- no two services show the same fields) -->
      <div class="wizard-step" data-step="3">
        <h3 class="font-bold text-lg" id="wizStep3Title"><i class="fa-solid fa-code text-emerald-300 mr-2"></i><span data-en="Step 3: Technology Preference" data-sw="Hatua 3: Mapendeleo ya Teknolojia">Step 3: Technology Preference</span></h3>
        <p class="text-[13px] text-white/50" id="wizStep3Hint" data-en="Our System Analyst will advise the best stack if you are unsure." data-sw="Mtaalamu wetu wa Mifumo atashauri teknolojia bora ikiwa hujui.">Our System Analyst will advise the best stack if you are unsure.</p>
        <div id="qTechWrap">
          <label class="text-[12px] font-bold text-white/60 mt-4 block" id="qTechLabel" data-en="PREFERRED LANGUAGES (optional)" data-sw="LUGHA UNAZOPENDA (si lazima)">PREFERRED LANGUAGES (optional)</label>
          <div class="flex flex-wrap gap-2 mt-2" id="qTechGrid"></div>
        </div>
        <div class="grid sm:grid-cols-2 gap-3 mt-4">
          <div><label class="text-[12px] font-bold text-white/60" id="qPlatformLabel" data-en="PLATFORM" data-sw="JUKWAA">PLATFORM</label><select id="qPlatform" class="input mt-1"></select></div>
          <div><label class="text-[12px] font-bold text-white/60" data-en="PRIORITY" data-sw="KIPAUMBELE">PRIORITY</label><select id="qPriority" class="input mt-1"><option value="Normal" data-en="Normal" data-sw="Kawaida">Normal</option><option value="Urgent" data-en="Urgent" data-sw="Haraka">Urgent</option><option value="Critical / ASAP" data-en="Critical / ASAP" data-sw="Muhimu Sana / Haraka iwezekanavyo">Critical / ASAP</option></select></div>
        </div>
      </div>
      <!-- STEP 4 -->
      <div class="wizard-step" data-step="4">
        <h3 class="font-bold text-lg"><i class="fa-solid fa-coins text-gold mr-2"></i><span data-en="Step 4: Budget & Timeline" data-sw="Hatua 4: Bajeti na Muda">Step 4: Budget & Timeline</span></h3>
        <label class="text-[12px] font-bold text-white/60 mt-3 block" data-en="BUDGET RANGE (TZS)" data-sw="KIWANGO CHA BAJETI (TZS)">BUDGET RANGE (TZS)</label>
        <select id="qBudget" class="input mt-1"><option data-en="Under TZS 500,000 (< $200)" data-sw="Chini ya TZS 500,000 (< $200)">Under TZS 500,000 (< $200)</option><option data-en="TZS 500K – 1.5M ($200 – $600)" data-sw="TZS 500K – 1.5M">TZS 500K – 1.5M ($200 – $600)</option><option data-en="TZS 1.5M – 5M ($600 – $2,000)" data-sw="TZS 1.5M – 5M">TZS 1.5M – 5M ($600 – $2,000)</option><option data-en="TZS 5M – 15M ($2,000 – $6,000)" data-sw="TZS 5M – 15M">TZS 5M – 15M ($2,000 – $6,000)</option><option data-en="TZS 15M+ (> $6,000)" data-sw="Zaidi ya TZS 15M">TZS 15M+ (> $6,000)</option><option data-en="Need professional advice" data-sw="Nahitaji ushauri wa kitaalamu">Need professional advice</option></select>
        <label class="text-[12px] font-bold text-white/60 mt-3 block" data-en="EXPECTED DEADLINE" data-sw="MUDA UNAOTARAJIWA">EXPECTED DEADLINE</label><input id="qDeadline" type="date" class="input mt-1">
        <label class="text-[12px] font-bold text-white/60 mt-3 block" data-en="YOUR REGION (optional — Tanzania)" data-sw="MKOA WAKO (si lazima — Tanzania)">YOUR REGION (optional — Tanzania)</label>
        <select id="qRegion" class="input mt-1"><option value="">—</option></select>
        <label class="text-[12px] font-bold text-white/60 mt-3 block" data-en="HOW DID YOU HEAR ABOUT US?" data-sw="ULISIKIA KUHUSU SISI VIPI?">HOW DID YOU HEAR ABOUT US?</label><select id="qSource" class="input mt-1"><option data-en="Instagram @mbilinyitech" data-sw="Instagram @mbilinyitech">Instagram @mbilinyitech</option><option data-en="WhatsApp / Referral" data-sw="WhatsApp / Rafiki">WhatsApp / Referral</option><option data-en="Google Search" data-sw="Utafutaji wa Google">Google Search</option><option data-en="Previous Client" data-sw="Mteja wa Awali">Previous Client</option><option data-en="Other" data-sw="Nyingine">Other</option></select>
        <div class="mt-4 rounded-2xl bg-cyan-500/[.06] border border-cyan-400/25 p-4 text-[13px] text-white/70"><i class="fa-solid fa-circle-info text-cyan-300 mr-1"></i> <span data-en="After submitting, CEO Jackson Mbilinyi reviews your request and sends an official quotation with prices to your portal, email & WhatsApp within 24 hours. A formal BRELA contract follows approval." data-sw="Baada ya kutuma, CEO Jackson Mbilinyi hupitia ombi lako na kutuma nukuu rasmi yenye bei kwenye portal yako, email na WhatsApp ndani ya saa 24. Mkataba rasmi wa BRELA unafuata baada ya kukubali.">After submitting, CEO Jackson Mbilinyi reviews your request and sends an official quotation with prices to your portal, email & WhatsApp within 24 hours. A formal BRELA contract follows approval.</span></div>
      </div>
      <!-- STEP 5 review -->
      <div class="wizard-step" data-step="5">
        <h3 class="font-bold text-lg"><i class="fa-solid fa-clipboard-check text-emerald-300 mr-2"></i><span data-en="Step 5: Review & Submit" data-sw="Hatua 5: Hakiki na Tuma">Step 5: Review & Submit</span></h3>
        <div id="qReview" class="mt-4 rounded-2xl border border-white/10 bg-white/[.02] p-5 text-[13.5px] grid gap-2"></div>
        <label class="flex items-start gap-2.5 mt-4 text-[13px] text-white/65 cursor-pointer"><input type="checkbox" id="qAgree" class="mt-1 w-4 h-4 accent-cyan-400"><span data-en="I agree to be contacted by Mbilinyi Tech Solutions (BRELA Reg, TIN 192-147-522) regarding this request via call, WhatsApp, SMS or email." data-sw="Nakubali kuwasiliana nami na Mbilinyi Tech Solutions (BRELA Reg, TIN 192-147-522) kuhusu ombi hili kwa simu, WhatsApp, SMS au email.">I agree to be contacted by Mbilinyi Tech Solutions (BRELA Reg, TIN 192-147-522) regarding this request via call, WhatsApp, SMS or email.</span></label>
      </div>
      <div class="flex items-center justify-between mt-6 gap-3">
        <button id="wizBack" onclick="wizMove(-1)" class="px-5 py-3 rounded-xl border border-white/15 font-bold text-sm invisible"><span data-en="← Back" data-sw="← Rudi">← Back</span></button>
        <div class="text-[12px] text-white/40 font-mono" id="wizLabel">Step 1 of 5</div>
        <button id="wizNext" onclick="wizMove(1)" class="grad-btn px-6 py-3 rounded-xl font-bold text-sm"><span data-en="Continue →" data-sw="Endelea →">Continue →</span></button>
      </div>
    </div>
  </div>
</div>

<!-- GENERIC MODAL -->
<div id="genModal" class="fixed inset-0 z-[450] hidden items-center justify-center p-4 bg-black/70 backdrop-blur-sm overflow-y-auto">
  <div class="glass-strong rounded-3xl w-full max-w-3xl my-6 overflow-hidden">
    <div class="p-5 border-b border-white/10 flex items-center justify-between bg-white/[.02]"><div id="genTitle" class="font-display font-bold text-lg">Details</div><button onclick="closeGen()" class="w-9 h-9 rounded-full bg-white/10 hover:bg-white/20"><i class="fa-solid fa-xmark"></i></button></div>
    <div id="genBody" class="p-5 md:p-6 max-h-[75vh] overflow-y-auto"></div>
  </div>
</div>

<!-- PRINT AREA -->
<div id="printArea"></div>

<script>
window.__CSRF__ = <?= json_encode(csrf_token()) ?>;
window.__ME__   = <?= json_encode(current_user()) ?>;
window.__PAGE__ = <?= json_encode($page ?? '') ?>;
window.__SERVICES_ON__ = <?= json_encode(service_visibility()) ?>;
/* Set by the CEO in Admin → Settings. Empty means "not configured yet". */
window.__PLATFORM__ = <?= json_encode([
    'url'  => setting('platform_url'),
    'name' => setting('platform_name', 'MbilinyiTech Platform'),
], JSON_UNESCAPED_SLASHES) ?>;
</script>
<?php /* Versioned so a returning visitor never runs a stale script. */ ?>
<script src="assets/app.js?v=<?= (int)@filemtime(__DIR__ . '/../assets/app.js') ?>"></script>
<script>
/* PWA: Register service worker safely (no errors if unsupported) */
(function () {
  if ('serviceWorker' in navigator) {
    window.addEventListener('load', function () {
      navigator.serviceWorker.register('sw.js').catch(function () { /* ignore */ });
    });
  }
})();
</script>
</body>
</html>
