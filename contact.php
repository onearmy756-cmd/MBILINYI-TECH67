<?php
require_once __DIR__ . '/config/app.php';
$pageTitle = 'Contact — Let\'s Build Something Extraordinary | Mbilinyi Tech Solutions';
$page = 'landing';
require __DIR__ . '/partials/head.php';
?>
<body class="antialiased">
<?php require __DIR__ . '/partials/header.php'; ?>

<!-- ============ CONTACT PAGE ============ -->
<div id="view-landing" class="view-section active">
  <section id="contact" class="max-w-7xl mx-auto px-4 pt-8 pb-16">
    <div class="grid lg:grid-cols-[1fr_1.1fr] gap-6">
      <div class="glass rounded-3xl p-6 md:p-8">
        <div class="badge bg-blue-500/10 text-blue-300 border border-blue-400/30"><span data-en="GET IN TOUCH" data-sw="WASILIANA NASI">GET IN TOUCH</span></div>
        <h2 class="font-display font-bold text-3xl mt-3"><span data-en="Let's Build Something" data-sw="Tujenge Kitu">Let's Build Something</span> <span class="grad-text" data-en="Extraordinary" data-sw="Cha Kipekee">Extraordinary</span></h2>
        <div class="grid gap-3 mt-6 text-[14px]">
          <a href="tel:+255796752645" class="flex items-center gap-4 p-4 rounded-2xl bg-white/[.03] border border-white/10 hover:border-green-400/40"><div class="w-11 h-11 rounded-xl bg-green-500/15 flex items-center justify-center shrink-0"><i class="fa-solid fa-phone text-green-300"></i></div><div><div class="font-bold">0796 752 645</div><div class="text-white/50 text-[12px]" data-en="Calls • WhatsApp • SMS — Mon–Sat 8:00–20:00 EAT" data-sw="Simu • WhatsApp • SMS — Jumatatu–Jumamosi 8:00–20:00 EAT">Calls • WhatsApp • SMS — Mon–Sat 8:00–20:00 EAT</div></div></a>
          <a href="mailto:mbilinyitech@gmail.com" class="flex items-center gap-4 p-4 rounded-2xl bg-white/[.03] border border-white/10 hover:border-blue-400/40"><div class="w-11 h-11 rounded-xl bg-blue-500/15 flex items-center justify-center shrink-0"><i class="fa-solid fa-envelope text-blue-300"></i></div><div><div class="font-bold">mbilinyitech@gmail.com</div><div class="text-white/50 text-[12px]" data-en="Replies within 4 business hours" data-sw="Hujibu ndani ya saa 4 za kazi">Replies within 4 business hours</div></div></a>
          <div class="flex items-center gap-4 p-4 rounded-2xl bg-white/[.03] border border-white/10"><div class="w-11 h-11 rounded-xl bg-pink-500/15 flex items-center justify-center shrink-0"><i class="fa-brands fa-instagram text-pink-300"></i></div><div><div class="font-bold">@mbilinyitech</div><div class="text-white/50 text-[12px]" data-en="Daily builds, tips & behind the scenes" data-sw="Ujenzi wa kila siku, vidokezo na nyuma ya pazia">Daily builds, tips & behind the scenes</div></div><a href="https://instagram.com/mbilinyitech" target="_blank" class="ml-auto text-xs font-bold px-4 py-2 rounded-full bg-pink-500/15 border border-pink-400/30 text-pink-200" data-en="Follow" data-sw="Fuatilia">Follow</a></div>
          <div class="grid grid-cols-2 gap-3">
            <div class="p-4 rounded-2xl bg-emerald-500/[.07] border border-emerald-400/25"><div class="text-[11px] font-bold text-emerald-300 uppercase tracking-wider" data-en="BRELA Reg No." data-sw="Usajili wa BRELA">BRELA Reg No.</div><div class="font-mono font-bold text-[13px]">706668</div><div class="text-[11px] text-white/50" data-en="Limited Company ✓" data-sw="Kampuni ya Ltd ✓">Limited Company ✓</div></div>
            <div class="p-4 rounded-2xl bg-cyan-500/[.07] border border-cyan-400/25"><div class="text-[11px] font-bold text-cyan-300 uppercase tracking-wider" data-en="TIN Number" data-sw="Namba ya TIN">TIN Number</div><div class="font-mono font-bold text-[13px]">192-147-522</div><div class="text-[11px] text-white/50" data-en="TRA Verified ✓" data-sw="Imethibitishwa TRA ✓">TRA Verified ✓</div></div>
          </div>
        </div>
      </div>
      <div class="glass rounded-3xl p-6 md:p-8">
        <div class="font-display font-bold text-xl" data-en="Send a Message / Open Support Ticket" data-sw="Tuma Ujumbe / Fungua Tiketi ya Msaada">Send a Message / Open Support Ticket</div>
        <p class="text-white/50 text-[13px]" data-en="Creates a ticket in your account. Our team responds fast." data-sw="Hutengeneza tiketi kwenye akaunti yako. Timu yetu hujibu haraka.">Creates a ticket in your account. Our team responds fast.</p>
        <div class="grid sm:grid-cols-2 gap-3 mt-5">
          <input id="cName" class="input" data-ph-en="Full name *" data-ph-sw="Jina kamili *" placeholder="Full name *"><input id="cPhone" class="input" data-ph-en="Phone / WhatsApp *" data-ph-sw="Simu / WhatsApp *" placeholder="Phone / WhatsApp *">
        </div>
        <input id="cEmail" class="input mt-3" data-ph-en="Email address *" data-ph-sw="Barua pepe *" placeholder="Email address *">
        <input id="cSubject" class="input mt-3" data-ph-en="Subject (e.g. Need school system)" data-ph-sw="Somo (mf. Nahitaji mfumo wa shule)" placeholder="Subject (e.g. Need school system)">
        <textarea id="cMsg" rows="4" class="input mt-3" data-ph-en="Describe your need..." data-ph-sw="Eleza hitaji lako..." placeholder="Describe your need..."></textarea>
        <div class="flex flex-wrap gap-3 mt-4">
          <button onclick="sendContactTicket()" class="grad-btn flex-1 px-6 py-3.5 rounded-xl font-bold"><i class="fa-solid fa-paper-plane mr-2"></i><span data-en="Send Message" data-sw="Tuma Ujumbe">Send Message</span></button>
          <a href="https://wa.me/255796752645?text=Hello%20Mbilinyi%20Tech!%20I%20need%20a%20service." target="_blank" class="px-6 py-3.5 rounded-xl font-bold bg-[#25D366]/15 border border-[#25D366]/40 text-[#4be584] flex items-center gap-2 hover:bg-[#25D366]/25"><i class="fa-brands fa-whatsapp text-lg"></i> WhatsApp</a>
        </div>
      </div>
    </div>
  </section>
</div>

<?php require __DIR__ . '/partials/footer.php'; ?>
