<?php
require_once __DIR__ . '/config/app.php';
$pageTitle = 'Services — Every Service We Offer | Mbilinyi Tech Solutions';
$page = 'landing';
require __DIR__ . '/partials/head.php';
?>
<body class="antialiased">
<?php require __DIR__ . '/partials/header.php'; ?>

<!-- ============ SERVICES PAGE ============ -->
<div id="view-landing" class="view-section active">
  <section id="services" class="max-w-7xl mx-auto px-4 pt-8 pb-16">
    <div class="text-center max-w-2xl mx-auto">
      <div class="badge bg-cyan-500/10 text-cyan-300 border border-cyan-400/30 inline-block"><span data-en="MBILINYITECH SERVICES" data-sw="HUDUMA ZA MBILINYITECH">MBILINYITECH SERVICES</span></div>
      <h1 class="font-display font-bold text-3xl md:text-5xl mt-4"><span data-en="Every Service." data-sw="Kila Huduma.">Every Service.</span> <span class="grad-text" data-en="One Partner." data-sw="Mshirika Mmoja.">One Partner.</span></h1>
      <p class="text-white/55 mt-3 text-[14.5px]" data-en="All the services Mbilinyi Tech Solutions offers — software, AI &amp; chatbots, cybersecurity, forensics, graphics, government e-services, email, repairs and more. Pick any one and request it directly." data-sw="Huduma zote zinazotolewa na Mbilinyi Tech Solutions — software, AI na chatbots, usalama wa mtandao, uchunguzi wa kidijitali, ubunifu wa picha, huduma za serikali, email, matengenezo na zaidi. Chagua yoyote na uiombe moja kwa moja.">All the services Mbilinyi Tech Solutions offers — software, AI &amp; chatbots, cybersecurity, forensics, graphics, government e-services, email, repairs and more. Pick any one and request it directly.</p>
      <div class="flex flex-wrap justify-center gap-3 mt-6">
        <button onclick="openQuoteWizard()" class="grad-btn px-6 py-3.5 rounded-2xl font-bold flex items-center gap-2"><i class="fa-solid fa-file-pen"></i> <span data-en="Request a Service" data-sw="Omba Huduma">Request a Service</span></button>
        <a href="pricing" class="px-6 py-3.5 rounded-2xl font-bold border border-white/15 hover:bg-white/5 flex items-center gap-2"><i class="fa-solid fa-tags text-gold"></i> <span data-en="See Pricing" data-sw="Ona Bei">See Pricing</span></a>
        <a href="track" class="px-6 py-3.5 rounded-2xl font-bold border border-white/15 hover:bg-white/5 flex items-center gap-2"><i class="fa-solid fa-magnifying-glass text-cyan-300"></i> <span data-en="Track a Request" data-sw="Fuatilia Ombi">Track a Request</span></a>
      </div>
    </div>

    <div class="flex flex-wrap items-center gap-3 mt-12 mb-2">
      <div class="badge bg-violet-500/15 text-violet-300 border border-violet-400/30"><span data-en="ALL SERVICES" data-sw="HUDUMA ZOTE">ALL SERVICES</span></div>
      <div class="text-white/45 text-[13px]"><span data-en="Choose one and request it — you get an official quotation within 24 hours." data-sw="Chagua moja na uiombe — unapata nukuu rasmi ndani ya saa 24.">Choose one and request it — you get an official quotation within 24 hours.</span></div>
    </div>
    <div id="servicesGrid" class="grid sm:grid-cols-2 lg:grid-cols-4 gap-4 mt-4"></div>
  </section>

  <!-- HOW REQUESTING WORKS -->
  <section class="max-w-7xl mx-auto px-4 pb-14">
    <div class="glass rounded-3xl p-6 md:p-10 relative overflow-hidden">
      <div class="absolute top-0 left-0 w-full h-1 bg-gradient-to-r from-cyan-400 via-blue-500 to-violet-500"></div>
      <div class="relative">
        <div class="badge bg-emerald-500/15 text-emerald-300 border border-emerald-400/30" data-en="HOW REQUESTING WORKS" data-sw="UNAVYOMBA HUDUMA">HOW REQUESTING WORKS</div>
        <h2 class="font-display font-bold text-2xl md:text-3xl mt-3"><span data-en="From Request to" data-sw="Kutoka Ombi hadi">From Request to</span> <span class="grad-text" data-en="Delivered System" data-sw="Mfumo Uliokabidhiwa">Delivered System</span></h2>
        <div class="grid sm:grid-cols-2 lg:grid-cols-4 gap-4 mt-7">
          <div class="glass rounded-2xl p-5"><div class="w-10 h-10 rounded-xl bg-cyan-500/15 flex items-center justify-center mb-3 font-display font-bold text-cyan-300">1</div><div class="font-bold text-[15px]" data-en="Request" data-sw="Omba">Request</div><p class="text-white/50 text-[12.5px] mt-1.5" data-en="Pick a service and send your requirement. You get a tracking ID immediately." data-sw="Chagua huduma na tuma mahitaji yako. Unapata namba ya kufuatilia mara moja.">Pick a service and send your requirement. You get a tracking ID immediately.</p></div>
          <div class="glass rounded-2xl p-5"><div class="w-10 h-10 rounded-xl bg-violet-500/15 flex items-center justify-center mb-3 font-display font-bold text-violet-300">2</div><div class="font-bold text-[15px]" data-en="Quotation" data-sw="Nukuu">Quotation</div><p class="text-white/50 text-[12.5px] mt-1.5" data-en="We reply with an itemised official quotation within 24 hours." data-sw="Tunajibu kwa nukuu rasmi yenye vitu vilivyoorodheshwa ndani ya saa 24.">We reply with an itemised official quotation within 24 hours.</p></div>
          <div class="glass rounded-2xl p-5"><div class="w-10 h-10 rounded-xl bg-gold/15 flex items-center justify-center mb-3 font-display font-bold text-gold">3</div><div class="font-bold text-[15px]" data-en="Contract &amp; Deposit" data-sw="Mkataba na Amana">Contract &amp; Deposit</div><p class="text-white/50 text-[12.5px] mt-1.5" data-en="You e-sign a BRELA contract and pay the agreed 50% deposit to start." data-sw="Unasaini mkataba wa BRELA kidijitali na kulipa amana ya 50% iliyokubaliwa kuanza.">You e-sign a BRELA contract and pay the agreed 50% deposit to start.</p></div>
          <div class="glass rounded-2xl p-5"><div class="w-10 h-10 rounded-xl bg-emerald-500/15 flex items-center justify-center mb-3 font-display font-bold text-emerald-300">4</div><div class="font-bold text-[15px]" data-en="Delivery" data-sw="Ukabidhishaji">Delivery</div><p class="text-white/50 text-[12.5px] mt-1.5" data-en="We build, test and hand over the working system — with the source code." data-sw="Tunajenga, tunapima na tunakabidhi mfumo unaofanya kazi — pamoja na source code.">We build, test and hand over the working system — with the source code.</p></div>
        </div>
        <div class="mt-6 flex flex-wrap gap-3">
          <button onclick="openQuoteWizard()" class="grad-btn px-6 py-3.5 rounded-2xl font-bold flex items-center gap-2"><i class="fa-solid fa-rocket"></i> <span data-en="Start Now — Free Quote" data-sw="Anza Sasa — Nukuu Bure">Start Now — Free Quote</span></button>
          <a href="process" class="px-6 py-3.5 rounded-2xl font-bold border border-white/15 hover:bg-white/5 flex items-center gap-2"><i class="fa-solid fa-circle-info text-cyan-300"></i> <span data-en="Full Process Detail" data-sw="Maelezo Kamili ya Mchakato">Full Process Detail</span></a>
        </div>
      </div>
    </div>
  </section>

  <!-- NOT SURE WHICH SERVICE -->
  <section class="max-w-7xl mx-auto px-4 pb-16">
    <div class="glass rounded-3xl p-6 md:p-10 relative overflow-hidden">
      <div class="orb w-[360px] h-[360px] bg-violet-500/20 -top-24 -right-24"></div>
      <div class="relative grid md:grid-cols-[1fr_auto] gap-8 items-center">
        <div>
          <div class="badge bg-gold/15 text-gold border border-gold/30" data-en="NOT SURE WHICH ONE?" data-sw="HUJUI IPI?">NOT SURE WHICH ONE?</div>
          <h2 class="font-display font-bold text-3xl md:text-4xl mt-3"><span data-en="Tell Us the Problem," data-sw="Tuambie Shida,">Tell Us the Problem,</span> <span class="grad-text" data-en="We Pick the Service." data-sw="Tunachagua Huduma.">We Pick the Service.</span></h2>
          <p class="text-white/60 text-[14.5px] mt-3 leading-relaxed max-w-2xl" data-en="Describe what you need in your own words — in English or Kiswahili. Our System Analyst reads it and comes back with the right service, the right stack and a fixed price." data-sw="Eleza unachohitaji kwa maneno yako mwenyewe — kwa Kiingereza au Kiswahili. Mchambuzi wetu wa Mifumo anasoma na kurudi na huduma sahihi, teknolojia sahihi na bei isiyobadilika.">Describe what you need in your own words — in English or Kiswahili. Our System Analyst reads it and comes back with the right service, the right stack and a fixed price.</p>
          <div class="flex flex-wrap gap-2 mt-5 text-[12px]">
            <span class="px-3 py-1.5 rounded-full bg-white/5 border border-white/10"><i class="fa-solid fa-comments text-cyan-300 mr-1"></i> <span data-en="English &amp; Kiswahili" data-sw="Kiingereza na Kiswahili">English &amp; Kiswahili</span></span>
            <span class="px-3 py-1.5 rounded-full bg-white/5 border border-white/10"><i class="fa-solid fa-clock text-violet-300 mr-1"></i> <span data-en="24-hour reply target" data-sw="Jibu ndani ya saa 24">24-hour reply target</span></span>
            <span class="px-3 py-1.5 rounded-full bg-white/5 border border-white/10"><i class="fa-solid fa-shield-halved text-emerald-300 mr-1"></i> <span data-en="Contract &amp; SLA on every job" data-sw="Mkataba na SLA kwa kila kazi">Contract &amp; SLA on every job</span></span>
          </div>
        </div>
        <div class="grid gap-3 min-w-[240px]">
          <button onclick="openQuoteWizard()" class="grad-btn px-7 py-4 rounded-2xl font-bold flex items-center justify-center gap-2"><i class="fa-solid fa-file-pen"></i> <span data-en="Request a Custom Quote" data-sw="Omba Nukuu Maalum">Request a Custom Quote</span></button>
          <a href="contact" class="px-7 py-4 rounded-2xl font-bold border border-white/15 hover:bg-white/5 flex items-center justify-center gap-2"><i class="fa-solid fa-phone text-green-300"></i> <span data-en="Talk to the CEO" data-sw="Ongea na CEO">Talk to the CEO</span></a>
          <a href="work" class="px-7 py-4 rounded-2xl font-bold border border-white/15 hover:bg-white/5 flex items-center justify-center gap-2"><i class="fa-solid fa-briefcase text-gold"></i> <span data-en="See Our Work" data-sw="Ona Kazi Zetu">See Our Work</span></a>
        </div>
      </div>
    </div>
  </section>
</div>

<?php require __DIR__ . '/partials/footer.php'; ?>
