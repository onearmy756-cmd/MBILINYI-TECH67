<?php
require_once __DIR__ . '/config/app.php';
$pageTitle = 'Pricing — Maintenance & Security Plans (TZS & USD) | Mbilinyi Tech Solutions';
$page = 'landing';
require __DIR__ . '/partials/head.php';
?>
<body class="antialiased">
<?php require __DIR__ . '/partials/header.php'; ?>

<!-- ============ PRICING PAGE ============ -->
<div id="view-landing" class="view-section active">
  <section id="pricing" class="max-w-7xl mx-auto px-4 pt-8 pb-16">
    <div class="text-center"><div class="badge bg-gold/15 text-gold border border-yellow-500/30 inline-block">TRANSPARENT PRICING • TZS</div>
      <h2 class="font-display font-bold text-3xl md:text-4xl mt-3">Our <span class="grad-text">Service Prices</span></h2>
      <p class="text-white/60 text-[14px] mt-2 max-w-2xl mx-auto" data-en="Starting prices in Tanzanian Shillings. Every project is confirmed with a formal quotation within 24 hours." data-sw="Bei za kuanzia kwa Shilingi za Tanzania. Kila mradi uthibitishwa na nukuu rasmi ndani ya saa 24.">Starting prices in Tanzanian Shillings. Every project is confirmed with a formal quotation within 24 hours.</p>
    </div>

    <div class="grid sm:grid-cols-2 lg:grid-cols-3 gap-4 mt-8">
      <?php foreach (service_price_list() as $sp): if ($sp['price'] <= 0) continue; ?>
      <div class="glass rounded-3xl p-5 relative overflow-hidden group hover:border-cyan-400/40 transition">
        <div class="absolute -top-16 -right-16 w-40 h-40 rounded-full blur-3xl opacity-25 group-hover:opacity-45 transition" style="background:radial-gradient(circle,#22d3ee,transparent 70%)"></div>
        <div class="text-[13.5px] font-bold leading-snug min-h-[38px]"><?php echo htmlspecialchars($sp['en'], ENT_QUOTES); ?></div>
        <div class="text-[12px] text-white/45 mb-3"><?php echo htmlspecialchars($sp['sw'], ENT_QUOTES); ?></div>
        <div class="flex items-end gap-1.5">
          <span class="font-display font-bold text-2xl grad-text"><?php echo htmlspecialchars(tzs($sp['price']), ENT_QUOTES); ?></span>
        </div>
        <div class="text-[11.5px] text-white/45 mt-0.5"><?php echo htmlspecialchars($sp['period'], ENT_QUOTES); ?></div>
        <button onclick="openQuoteWizard()" class="w-full mt-4 py-2.5 rounded-xl font-bold text-[13px] border border-white/15 hover:bg-white/5 transition">
          <span data-en="Get an exact quote →" data-sw="Pata nukuu kamili →">Get an exact quote →</span>
        </button>
      </div>
      <?php endforeach; ?>
    </div>

    <div class="text-center mt-12"><div class="badge bg-emerald-500/15 text-emerald-300 border border-emerald-400/30 inline-block">MAINTENANCE • SECURITY</div>
      <h2 class="font-display font-bold text-3xl md:text-4xl mt-3">Maintenance & Security <span class="grad-text">Plans</span></h2></div>
    <div class="grid md:grid-cols-3 gap-4 mt-8" id="maintPlans"></div>
    <div class="grid md:grid-cols-3 gap-4 mt-4" id="secPlans"></div>
    <div class="mt-8 max-w-3xl mx-auto glass rounded-3xl p-6 text-[13.5px] text-white/70">
      <div class="font-display font-bold text-lg text-white mb-2"><i class="fa-solid fa-coins text-gold mr-2"></i><span data-en="Payment Terms — 50 / 50 for Software Projects" data-sw="Masharti ya Malipo — 50 / 50 kwa Miradi ya Software">Payment Terms — 50 / 50 for Software Projects</span></div>
      <p class="mb-2"><span data-en="This 50/50 schedule applies to SOFTWARE projects only. Every other service (consulting, training, BRELA/TRA filings, audits) follows the terms both parties agree in writing first." data-sw="Ratiba hii ya 50/50 inatumika kwa miradi ya SOFTWARE pekee. Huduma zingine zote (ushauri, mafunzo, usajili wa BRELA/TRA, ukaguzi) zinafuata masharti ambayo pande mbili zimekubaliana kimaandishi kwanza.">This 50/50 schedule applies to SOFTWARE projects only. Every other service follows the terms both parties agree in writing first.</span></p>
      <p class="mb-2"><span data-en="Software projects are quoted per project via the" data-sw="Miradi ya software inapewa bei kwa kila mradi kupitia">Software projects are quoted per project via the</span> <button onclick="openQuoteWizard()" class="text-cyan-300 underline font-bold"><span data-en="Request Service Form" data-sw="Fomu ya Ombi la Huduma">Request Service Form</span></button> <span data-en="— CEO responds with an official BRELA-compliant quotation within 24hrs." data-sw="— CEO anajibu na nukuu rasmi inayofuata sheria za BRELA ndani ya masaa 24.">— CEO responds with an official quotation within 24hrs.</span></p>
      <div class="grid sm:grid-cols-2 gap-3 my-4 text-[12.5px]">
        <div class="rounded-2xl border border-cyan-400/30 bg-cyan-500/[.06] p-4">
          <div class="text-[10px] font-bold text-cyan-300 uppercase tracking-widest"><span data-en="① Installment 1 — BEFORE START" data-sw="① Malipo 1 — KABLA YA KUANZA">① Installment 1 — BEFORE START</span></div>
          <div class="font-display font-bold text-xl grad-text mt-1">50% <span class="text-white/60 text-[13px] font-normal">/ <span data-en="Deposit" data-sw="Amana">Deposit</span></span></div>
          <div class="text-white/60 mt-1.5 leading-relaxed"><span data-en="Mradi HAUTAANZISHI hadi hela hii iwe lipewe na kuthibitishwa na CEO Jackson Mbilinyi. Hii ndio inayoruhusu kusainiwa kwa mkataba." data-sw="Mradi HAUTAANZISHI hadi hela hii iwe lipewe na kuthibitishwa na CEO Jackson Mbilinyi. Hii ndio inayoruhusu kusainiwa kwa mkataba.">Mradi HAUTAANZISHI hadi hela hii iwe lipewe na kuthibitishwa na CEO Jackson Mbilinyi. Hii ndio inayoruhusu kusainiwa kwa mkataba.</span></div>
        </div>
        <div class="rounded-2xl border border-violet-400/30 bg-violet-500/[.06] p-4">
          <div class="text-[10px] font-bold text-violet-300 uppercase tracking-widest"><span data-en="② Installment 2 — MID-PROJECT (NOT Delivery)" data-sw="② Malipo 2 — KATI YA MRADI (SI Delivery)">② Installment 2 — MID-PROJECT (NOT Delivery)</span></div>
          <div class="font-display font-bold text-xl grad-text mt-1">50% <span class="text-white/60 text-[13px] font-normal">/ <span data-en="Mid-Project" data-sw="Kati ya Mradi">Mid-Project</span></span></div>
          <div class="text-white/60 mt-1.5 leading-relaxed"><span data-en="Inalipwa wakati timeline imekamilika 50% (kati ya mradi), si baada ya handover. Final delivery, training na source code hazinapelekwi mpaka installment hii ikamilike." data-sw="Inalipwa wakati timeline imekamilika 50% (kati ya mradi), si baada ya handover. Final delivery, training na source code hazinapelekwi mpaka installment hii ikamilike.">Inalipwa wakati timeline imekamilika 50% (kati ya mradi), si baada ya handover. Final delivery, training na source code hazinapelekwi mpaka installment hii ikamilike.</span></div>
        </div>
      </div>
      <p class="text-[12px] text-white/60 leading-relaxed"><i class="fa-solid fa-circle-exclamation text-amber-300 mr-1.5"></i><span data-en="NOT Deposit 50% + Delivery 50%. It is Deposit 50% + Mid-Project 50%. The deposit is non-refundable once work begins. Late payments carry a 2% per week penalty. See the Refund Policy for full detail." data-sw="SIYO Deposit 50% + Delivery 50%. Ni Deposit 50% + Mid-Project 50%. Amana hairudishwi baada ya kazi kuanza. Malipo ya kuchelewa yana penalty ya 2% kwa wiki. Angalia Sera ya Kurudisha Pesa kwa maelezo kamili.">NOT Deposit 50% + Delivery 50%. It is Deposit 50% + Mid-Project 50%. The deposit is non-refundable once work begins. Late payments carry a 2% per week penalty. See the Refund Policy for full detail.</span> <a href="refund" class="text-rose-300 underline font-bold whitespace-nowrap" data-en="Read it →" data-sw="Soma →">Read it →</a></p>
    </div>
  </section>
</div>

<?php require __DIR__ . '/partials/footer.php'; ?>
