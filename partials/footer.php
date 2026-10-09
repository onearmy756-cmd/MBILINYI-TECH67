</main>

<!-- FOOTER -->
<footer class="border-t border-white/10 bg-[#04070F] mt-6">
  <div class="max-w-7xl mx-auto px-4 py-12 grid md:grid-cols-4 gap-8 text-[13.5px]">
    <div><div class="flex items-center gap-3"><div class="w-10 h-10 rounded-xl grad-btn flex items-center justify-center font-bold">M</div><div class="font-display font-bold">MBILINYI TECH SOLUTIONS</div></div><p class="text-white/50 mt-3 leading-relaxed">BRELA-registered software house delivering custom systems, AI, maintenance & security — locally rooted, globally competitive.</p><div class="flex gap-2 mt-4"><a href="https://instagram.com/mbilinyitech" target="_blank" class="w-9 h-9 rounded-full bg-white/5 border border-white/10 flex items-center justify-center hover:border-pink-400/50"><i class="fa-brands fa-instagram text-pink-300"></i></a><a href="https://wa.me/255796752645" target="_blank" class="w-9 h-9 rounded-full bg-white/5 border border-white/10 flex items-center justify-center hover:border-green-400/50"><i class="fa-brands fa-whatsapp text-green-300"></i></a><a href="mailto:mbilinyitech@gmail.com" class="w-9 h-9 rounded-full bg-white/5 border border-white/10 flex items-center justify-center hover:border-blue-400/50"><i class="fa-solid fa-envelope text-blue-300"></i></a><a href="tel:+255796752645" class="w-9 h-9 rounded-full bg-white/5 border border-white/10 flex items-center justify-center hover:border-cyan-400/50"><i class="fa-solid fa-phone text-cyan-300"></i></a></div></div>
    <div><div class="font-bold mb-3">Services</div><div class="grid gap-2 text-white/55"><span>Custom Software Development</span><span>System Analysis & Design</span><span>IT Consultancy</span><span>Software Maintenance & SLA</span><span>Cybersecurity & Audits</span><span>AI Fine-Tuning & Chatbots</span></div></div>
    <div><div class="font-bold mb-3">Company</div><div class="grid gap-2 text-white/55"><span>CEO: Jackson Mbilinyi</span><span>BRELA Reg: Z-418822-77-TZ</span><span class="font-mono">TIN: 192-147-522</span><span data-en="Dodoma, Tanzania • Work delivered online" data-sw="Dodoma, Tanzania • Kazi zote mtandaoni">Dodoma, Tanzania • Work delivered online</span><a href="auth" class="text-left text-cyan-300 font-bold">Client / Admin Login →</a></div></div>
    <div><div class="font-bold mb-3">Contact — Mawasiliano</div><div class="grid gap-2 text-white/55"><a href="tel:+255796752645" class="hover:text-white"><i class="fa-solid fa-phone mr-2 text-green-300"></i>0796 752 645 (Call/WhatsApp/SMS)</a><a href="mailto:mbilinyitech@gmail.com" class="hover:text-white"><i class="fa-solid fa-envelope mr-2 text-blue-300"></i>mbilinyitech@gmail.com</a><span><i class="fa-brands fa-instagram mr-2 text-pink-300"></i>@mbilinyitech</span><span><i class="fa-solid fa-clock mr-2 text-gold"></i>Mon–Sat • 8:00–20:00 EAT</span></div></div>
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
    </div>
    <div class="text-[12px] text-white/35">&copy; <?php
      /* Auto-rolls over every new year — no manual edit needed. */
      $founded = 2026;
      $now     = (int)date('Y');
      echo ($now > $founded) ? $founded . '–' . $now : $now;
    ?> Mbilinyi Tech Solutions • CEO Jackson Mbilinyi • All systems production-grade • BRELA & TIN Verified</div>
  </div>
</footer>

<!-- WHATSAPP FLOAT -->
<a href="https://wa.me/255796752645?text=Hello%20Mbilinyi%20Tech%20Solutions!%20I%20need%20a%20service." target="_blank" class="fixed bottom-24 right-4 z-[200] w-14 h-14 rounded-full bg-[#25D366] flex items-center justify-center shadow-2xl hover:scale-110 transition" title="Chat on WhatsApp"><i class="fa-brands fa-whatsapp text-2xl text-white"></i><span class="absolute -top-1 -right-1 w-4 h-4 bg-red-500 rounded-full border-2 border-[#050914] animate-pulse"></span></a>

<!-- CHATBOT -->
<button onclick="toggleChat()" class="fixed bottom-6 right-4 z-[300] grad-btn w-14 h-14 rounded-full flex items-center justify-center text-xl shadow-2xl hover:scale-110 transition"><i class="fa-solid fa-robot"></i></button>
<div id="chatWindow" class="hidden-chat fixed bottom-[86px] right-4 z-[300] w-[380px] max-w-[94vw] glass-strong rounded-3xl overflow-hidden shadow-2xl flex flex-col" style="height:560px;max-height:72vh">
  <div class="p-4 bg-gradient-to-r from-cyan-600 via-blue-600 to-violet-600 flex items-center gap-3">
    <div class="w-11 h-11 rounded-2xl bg-white/15 flex items-center justify-center text-xl backdrop-blur"><i class="fa-solid fa-robot"></i></div>
    <div class="flex-1"><div class="font-bold text-[15px]">Mbilinyi AI Assistant</div><div class="text-[11.5px] text-white/80 flex items-center gap-1.5"><span class="w-2 h-2 rounded-full bg-emerald-300 animate-pulse"></span> Online • understands English & Kiswahili</div></div>
    <button onclick="toggleChat()" class="w-8 h-8 rounded-full bg-white/15"><i class="fa-solid fa-xmark text-sm"></i></button>
  </div>
  <div id="chatBody" class="flex-1 overflow-y-auto p-4 space-y-3 text-[13.5px]"></div>
  <div class="p-3 border-t border-white/10">
    <div class="flex flex-wrap gap-1.5 mb-2.5" id="chatQuick"></div>
    <div class="flex gap-2"><input id="chatInput" onkeydown="if(event.key==='Enter')sendChat()" class="input !py-2.5 text-[13px]" placeholder="Ask about services, prices, tracking..."><button onclick="sendChat()" class="grad-btn w-11 h-11 rounded-xl shrink-0"><i class="fa-solid fa-paper-plane text-sm"></i></button></div>
  </div>
</div>

<!-- QUOTE WIZARD MODAL -->
<div id="quoteModal" class="fixed inset-0 z-[400] hidden items-center justify-center p-4 bg-black/70 backdrop-blur-sm overflow-y-auto">
  <div class="glass-strong rounded-3xl w-full max-w-3xl my-6 overflow-hidden">
    <div class="p-5 md:p-6 bg-gradient-to-r from-cyan-600/20 to-violet-600/20 border-b border-white/10 flex items-center justify-between">
      <div><div class="text-[11px] font-bold tracking-[.2em] text-cyan-300">CLIENT REQUIREMENTS FORM • FOMU YA MAHITAJI</div><div class="font-display font-bold text-xl">Request a Service — Omba Huduma</div></div>
      <button onclick="closeQuoteWizard()" class="w-9 h-9 rounded-full bg-white/10 hover:bg-white/20"><i class="fa-solid fa-xmark"></i></button>
    </div>
    <div class="px-5 md:px-6 pt-5"><div class="flex items-center gap-2" id="wizProgress"></div></div>
    <div class="p-5 md:p-6">
      <!-- STEP 1 -->
      <div class="wizard-step active" data-step="1">
        <h3 class="font-bold text-lg"><i class="fa-solid fa-user text-cyan-300 mr-2"></i>Step 1: Your Details</h3>
        <div class="grid sm:grid-cols-2 gap-3 mt-4">
          <div><label class="text-[12px] font-bold text-white/60">FULL NAME *</label><input id="qName" class="input mt-1" placeholder="Jina kamili"></div>
          <div><label class="text-[12px] font-bold text-white/60">PHONE / WHATSAPP *</label><input id="qPhone" class="input mt-1" placeholder="07XX XXX XXX"></div>
        </div>
        <div class="grid sm:grid-cols-2 gap-3 mt-3">
          <div><label class="text-[12px] font-bold text-white/60">EMAIL *</label><input id="qEmail" class="input mt-1" placeholder="you@email.com"></div>
          <div><label class="text-[12px] font-bold text-white/60">COMPANY / ORGANIZATION</label><input id="qCompany" class="input mt-1" placeholder="Optional"></div>
        </div>
      </div>
      <!-- STEP 2 -->
      <div class="wizard-step" data-step="2">
        <h3 class="font-bold text-lg"><i class="fa-solid fa-briefcase text-violet-300 mr-2"></i>Step 2: Service & Project</h3>
        <label class="text-[12px] font-bold text-white/60 mt-4 block">SELECT SERVICE *</label>
        <div class="grid sm:grid-cols-2 gap-2 mt-2" id="qServiceGrid"></div>
        <label class="text-[12px] font-bold text-white/60 mt-4 block">PROJECT TITLE *</label><input id="qTitle" class="input mt-1" placeholder="e.g. Pharmacy stock + sales system with M-Pesa">
        <label class="text-[12px] font-bold text-white/60 mt-3 block">DETAILED REQUIREMENTS * (Mahitaji kwa undani)</label><textarea id="qDesc" rows="4" class="input mt-1" placeholder="Describe features, users, integrations (M-Pesa, SMS), reports you need..."></textarea>
      </div>
      <!-- STEP 3 -->
      <div class="wizard-step" data-step="3">
        <h3 class="font-bold text-lg"><i class="fa-solid fa-code text-emerald-300 mr-2"></i>Step 3: Technology Preference</h3>
        <p class="text-[13px] text-white/50">Pick preferred languages (optional — our System Analyst will advise the best stack).</p>
        <div class="flex flex-wrap gap-2 mt-3" id="qTechGrid"></div>
        <div class="grid sm:grid-cols-2 gap-3 mt-4">
          <div><label class="text-[12px] font-bold text-white/60">PLATFORM</label><select id="qPlatform" class="input mt-1"><option>Web Application</option><option>Mobile App (Android/iOS)</option><option>Desktop System</option><option>Web + Mobile</option><option>AI / Chatbot Solution</option><option>Full System (All)</option></select></div>
          <div><label class="text-[12px] font-bold text-white/60">PRIORITY</label><select id="qPriority" class="input mt-1"><option>Normal</option><option>Urgent</option><option>Critical / ASAP</option></select></div>
        </div>
      </div>
      <!-- STEP 4 -->
      <div class="wizard-step" data-step="4">
        <h3 class="font-bold text-lg"><i class="fa-solid fa-coins text-gold mr-2"></i>Step 4: Budget & Timeline</h3>
        <label class="text-[12px] font-bold text-white/60 mt-3 block">BUDGET RANGE (TZS)</label>
        <select id="qBudget" class="input mt-1"><option>Under TZS 500,000 (< $200)</option><option>TZS 500K – 1.5M ($200 – $600)</option><option>TZS 1.5M – 5M ($600 – $2,000)</option><option>TZS 5M – 15M ($2,000 – $6,000)</option><option>TZS 15M+ (> $6,000)</option><option>Need professional advice</option></select>
        <label class="text-[12px] font-bold text-white/60 mt-3 block">EXPECTED DEADLINE</label><input id="qDeadline" type="date" class="input mt-1">
        <label class="text-[12px] font-bold text-white/60 mt-3 block">HOW DID YOU HEAR ABOUT US?</label><select id="qSource" class="input mt-1"><option>Instagram @mbilinyitech</option><option>WhatsApp / Referral</option><option>Google Search</option><option>Previous Client</option><option>Other</option></select>
        <div class="mt-4 rounded-2xl bg-cyan-500/[.06] border border-cyan-400/25 p-4 text-[13px] text-white/70"><i class="fa-solid fa-circle-info text-cyan-300 mr-1"></i> After submitting, Admin (CEO Jackson Mbilinyi) will review and send an <strong class="text-white">official quotation with prices</strong> to your portal, email & WhatsApp within 24 hours. A formal <strong class="text-white">BRELA contract</strong> follows approval.</div>
      </div>
      <!-- STEP 5 review -->
      <div class="wizard-step" data-step="5">
        <h3 class="font-bold text-lg"><i class="fa-solid fa-clipboard-check text-emerald-300 mr-2"></i>Step 5: Review & Submit</h3>
        <div id="qReview" class="mt-4 rounded-2xl border border-white/10 bg-white/[.02] p-5 text-[13.5px] grid gap-2"></div>
        <label class="flex items-start gap-2.5 mt-4 text-[13px] text-white/65 cursor-pointer"><input type="checkbox" id="qAgree" class="mt-1 w-4 h-4 accent-cyan-400"><span>I agree to be contacted by Mbilinyi Tech Solutions (BRELA Reg, TIN 192-147-522) regarding this request via call, WhatsApp, SMS or email. / Nakubali kuwasiliana nami.</span></label>
      </div>
      <div class="flex items-center justify-between mt-6 gap-3">
        <button id="wizBack" onclick="wizMove(-1)" class="px-5 py-3 rounded-xl border border-white/15 font-bold text-sm invisible">← Back</button>
        <div class="text-[12px] text-white/40 font-mono" id="wizLabel">Step 1 of 5</div>
        <button id="wizNext" onclick="wizMove(1)" class="grad-btn px-6 py-3 rounded-xl font-bold text-sm">Continue →</button>
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
</script>
<script src="assets/app.js"></script>
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
