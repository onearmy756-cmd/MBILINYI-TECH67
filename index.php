<?php
require_once __DIR__ . '/config/app.php';
$pageTitle = 'Mbilinyi Tech Solutions — Custom Software, AI & Security | Jackson Mbilinyi, CEO';
$page = 'landing';
$me = current_user();
require __DIR__ . '/partials/head.php';
?>
<body class="antialiased">
<?php require __DIR__ . '/partials/header.php'; ?>

<!-- ============ LANDING VIEW ============ -->
<div id="view-landing" class="view-section active">

  <!-- HERO -->
  <section class="relative overflow-hidden">
    <div class="absolute inset-0 grid-bg"></div>
    <div class="orb w-[520px] h-[520px] bg-cyan-500/40 -top-40 -left-40"></div>
    <div class="orb w-[520px] h-[520px] bg-violet-600/40 top-10 -right-40"></div>
    <div class="orb w-[300px] h-[300px] bg-emerald-500/20 bottom-0 left-1/3"></div>
    <div class="max-w-7xl mx-auto px-4 pt-12 pb-16 lg:pt-20 lg:pb-24 grid lg:grid-cols-2 gap-12 items-center relative">
      <div>
        <div class="flex flex-wrap gap-2 mb-5">
          <span class="badge bg-emerald-500/15 text-emerald-300 border border-emerald-400/30"><i class="fa-solid fa-circle-check mr-1"></i><span data-en="BRELA CERTIFIED" data-sw="IMESAJILIWA BRELA">BRELA CERTIFIED</span></span>
          <span class="badge bg-cyan-500/10 text-cyan-300 border border-cyan-400/30"><span data-en="TIN 192-147-522 VERIFIED" data-sw="TIN 192-147-522 IMETHIBITISHWA">TIN 192-147-522 VERIFIED</span></span>
          <span class="badge bg-violet-500/15 text-violet-300 border border-violet-400/30"><span data-en="SECURITY-FIRST ENGINEERING" data-sw="Uhandisi UNAOTANGULIZA USALAMA">SECURITY-FIRST ENGINEERING</span></span>
        </div>
        <h1 class="font-display font-bold text-[38px] sm:text-[54px] lg:text-[62px] leading-[1.02]">
          <span data-en="We Engineer" data-sw="Tunaunda">We Engineer</span> <span class="grad-text" data-en="Software" data-sw="Software">Software</span><br>
          <span data-en="That Scales." data-sw="Inayokua.">That Scales.</span>
        </h1>
        <div class="mt-4 inline-block px-4 py-2 rounded-2xl bg-gradient-to-r from-cyan-500/15 via-blue-500/15 to-violet-500/15 border border-cyan-400/30 font-mono text-[13px] tracking-[.15em] text-cyan-200/90">
          <span data-en="★ YOUR PROBLEM, OUR SOLUTION ★" data-sw="★ SHIDA YAKO, TATUZO LETU ★">★ YOUR PROBLEM, OUR SOLUTION ★</span>
        </div>
        <p class="mt-5 text-white/65 text-[15.5px] leading-relaxed max-w-xl" data-en="From custom systems, AI fine-tuning & chatbots to maintenance and security hardening — Mbilinyi Tech Solutions, led by CEO Jackson Mbilinyi, builds secure, well-tested technology for startups, enterprises & government." data-sw="Kuanzia mifumo maalum, AI fine-tuning na chatbots hadi matengenezo na kuimarisha usalama — Mbilinyi Tech Solutions, inayoongozwa na CEO Jackson Mbilinyi, hujenga teknolojia salama na iliyopimwa vizuri kwa startups, makampuni na serikali.">From custom systems, AI fine-tuning & chatbots to maintenance and security hardening — Mbilinyi Tech Solutions, led by CEO Jackson Mbilinyi, builds secure, well-tested technology for startups, enterprises & government.</p>
        <div class="mt-7 flex flex-wrap gap-3">
          <button onclick="openQuoteWizard()" class="grad-btn px-7 py-4 rounded-2xl font-bold flex items-center gap-2"><i class="fa-solid fa-rocket"></i> <span data-en="Start Your Project — Free Quote" data-sw="Anza Mradi — Nukuu Bure">Start Your Project — Free Quote</span></button>
          <button onclick="location.href='track'" class="px-7 py-4 rounded-2xl font-bold border border-white/15 hover:bg-white/5 flex items-center gap-2"><i class="fa-solid fa-box-open text-gold"></i> <span data-en="Track Existing Request" data-sw="Fuatilia Ombi Lako">Track Existing Request</span></button>
        </div>
        <div class="mt-6 flex items-center gap-4 text-[13px] text-white/60">
          <div class="flex -space-x-3">
            <div class="w-9 h-9 rounded-full border-2 border-[#050914] bg-gradient-to-br from-cyan-500 to-blue-600 flex items-center justify-center font-bold text-xs" title="Jackson Mbilinyi — CEO">JM</div>
          </div>
          <div><div class="text-gold text-xs font-bold" data-en="Founder-led • You work directly with the CEO" data-sw="Inaongozwa na mwanzilishi • Unafanya kazi moja kwa moja na CEO">Founder-led • You work directly with the CEO</div><div class="text-white/45" data-en="Registered under BRELA • TIN 192-147-522" data-sw="Imesajiliwa BRELA • TIN 192-147-522">Registered under BRELA • TIN 192-147-522</div></div>
        </div>
        <div class="mt-8 grid grid-cols-3 gap-3 max-w-lg">
          <div class="glass rounded-2xl p-4 text-center"><div class="font-display font-bold text-2xl grad-text"><span class="counter" data-target="100">0</span>%</div><div class="text-[11px] text-white/55 uppercase tracking-wider font-bold" data-en="Source code handed over" data-sw="Source code unakabidhiwa">Source code handed over</div></div>
          <div class="glass rounded-2xl p-4 text-center"><div class="font-display font-bold text-2xl grad-text"><span class="counter" data-target="24">0</span>h</div><div class="text-[11px] text-white/55 uppercase tracking-wider font-bold" data-en="Quote reply target" data-sw="Lengo la kujibu nukuu">Quote reply target</div></div>
          <div class="glass rounded-2xl p-4 text-center"><div class="font-display font-bold text-2xl grad-text"><span class="counter" id="svcCount" data-target="0">0</span></div><div class="text-[11px] text-white/55 uppercase tracking-wider font-bold" data-en="Services" data-sw="Huduma">Services</div></div>
        </div>
      </div>
      <div class="relative">
        <div class="orbit-ring w-[420px] h-[420px] left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 hidden md:block"></div>
        <div class="orbit-ring w-[320px] h-[320px] left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 hidden md:block" style="animation-duration:16s"></div>
        <div class="glass rounded-3xl overflow-hidden shadow-2xl relative z-10">
          <div class="flex items-center justify-between px-4 py-3 border-b border-white/10 bg-white/[.02]">
            <div class="code-dots flex gap-1.5"><span class="bg-red-400"></span><span class="bg-yellow-400"></span><span class="bg-green-400"></span></div>
            <div class="font-mono text-[11px] text-white/50 flex items-center gap-2"><span class="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span> mbilinyi_core — sample.ts • DEMO</div>
            <i class="fa-solid fa-lock text-emerald-300 text-xs"></i>
          </div>
          <div class="p-5 font-mono text-[12.5px] leading-[1.8] bg-[#070D1F]">
            <div><span class="text-fuchsia-400">import</span> <span class="text-white">{ MbilinyiCloud }</span> <span class="text-fuchsia-400">from</span> <span class="text-emerald-300">"@mbilinyi/sdk"</span>;</div>
            <div><span class="text-fuchsia-400">import</span> <span class="text-white">{ RustCore, ReactUI, PyAI }</span> <span class="text-fuchsia-400">from</span> <span class="text-emerald-300">"@mbilinyi/stack"</span>;</div>
            <div class="mt-2"><span class="text-slate-500">// CEO: Jackson Mbilinyi | BRELA ✓ | TIN 192-147-522</span></div>
            <div><span class="text-cyan-300">const</span> <span class="text-yellow-200">project</span> <span class="text-white">= await</span> <span class="text-cyan-300">MbilinyiCloud</span><span class="text-white">.deploy({</span></div>
            <div class="pl-5"><span class="text-white">stack:</span> <span class="text-emerald-300">[<span id="heroStackText">"react", "rust", "python"</span>]</span>,</div>
            <div class="pl-5"><span class="text-white">security:</span> <span class="text-emerald-300">"hardened + audited"</span>,</div>
            <div class="pl-5"><span class="text-white">ai:</span> <span class="text-emerald-300">"fine-tuned + chatbot"</span>,</div>
            <div class="pl-5"><span class="text-white">sla:</span> <span class="text-orange-300">"as agreed"</span><span class="text-white">,</span></div>
            <div><span class="text-white">});</span> <span class="text-emerald-300 font-bold">→ status: "DEPLOYED ✓"</span><span class="animate-pulse text-cyan-300">▌</span></div>
          </div>
          <div class="px-4 py-3 border-t border-white/10 flex items-center justify-between text-[12px]">
            <span class="text-white/50 font-mono" data-en="typed • reviewed • tested before handover" data-sw="imeandikwa • imepitiwa • imepimwa kabla ya kukabidhi">typed • reviewed • tested before handover</span>
            <span class="flex gap-2"><span class="badge bg-emerald-500/15 text-emerald-300 border border-emerald-400/30" data-en="SECURE" data-sw="SALAMA">SECURE</span><span class="badge bg-cyan-500/10 text-cyan-300 border border-cyan-400/30" data-en="SCALABLE" data-sw="INAKUA">SCALABLE</span></span>
          </div>
        </div>
        <div class="absolute -left-3 top-8 z-20 glass rounded-2xl px-4 py-3 float hidden sm:flex items-center gap-3"><div class="w-9 h-9 rounded-xl bg-emerald-500/20 flex items-center justify-center"><i class="fa-solid fa-code text-emerald-300"></i></div><div><div class="text-xs font-bold" data-en="Code You Own" data-sw="Code Unamiliki">Code You Own</div><div class="text-[11px] text-white/50" data-en="Full source + docs" data-sw="Source kamili + nyaraka">Full source + docs</div></div></div>
        <div class="absolute -right-2 bottom-10 z-20 glass rounded-2xl px-4 py-3 float2 hidden sm:flex items-center gap-3"><div class="w-9 h-9 rounded-xl bg-violet-500/20 flex items-center justify-center"><i class="fa-solid fa-robot text-violet-300"></i></div><div><div class="text-xs font-bold" data-en="AI Chatbot Online" data-sw="Chatbot ya AI Ipo Hewani">AI Chatbot Online</div><div class="text-[11px] text-white/50 flex items-center gap-1"><span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span> <span data-en="replies in ~2s" data-sw="hujibu kwa sekunde ~2">replies in ~2s</span></div></div></div>
      </div>
    </div>
    <div class="border-y border-white/10 bg-white/[.015] py-4 overflow-hidden relative">
      <div class="marquee-track font-mono text-[13px] text-white/60" id="techMarquee"></div>
    </div>
  </section>

  <!-- EXPLORE — cards to the Company and Services pages -->
  <section class="max-w-7xl mx-auto px-4 pb-14">
    <div class="flex flex-wrap items-end justify-between gap-4 mb-6">
      <div>
        <div class="badge bg-cyan-500/15 text-cyan-300 border border-cyan-400/30" data-en="EXPLORE" data-sw="CHUNGUZA">EXPLORE</div>
        <h2 class="font-display font-bold text-3xl md:text-4xl mt-3"><span data-en="Everything About" data-sw="Kila Kitu Kuhusu">Everything About</span> <span class="grad-text" data-en="MbilinyiTech" data-sw="MbilinyiTech">MbilinyiTech</span></h2>
      </div>
    </div>
    <div class="grid sm:grid-cols-2 lg:grid-cols-3 gap-4">
      <a href="company" class="glass rounded-3xl p-6 hover:border-cyan-400/40 transition group">
        <div class="w-12 h-12 rounded-2xl bg-cyan-500/15 flex items-center justify-center mb-4"><i class="fa-solid fa-building-columns text-cyan-300 text-lg"></i></div>
        <div class="badge bg-cyan-500/15 text-cyan-300 border border-cyan-400/30 inline-block" data-en="MBILINYITECH COMPANY" data-sw="KAMPUNI YA MBILINYITECH">MBILINYITECH COMPANY</div>
        <div class="font-display font-bold text-lg mt-3" data-en="About the Company" data-sw="Kuhusu Kampuni">About the Company</div>
        <p class="text-white/55 text-[13.5px] mt-2" data-en="BRELA 706668 • TIN 192-147-522, our leadership, the principles we work by and the people behind your project." data-sw="BRELA 706668 • TIN 192-147-522, uongozi wetu, kanuni tunazofuata na watu walio nyuma ya mradi wako.">BRELA 706668 • TIN 192-147-522, our leadership, the principles we work by and the people behind your project.</p>
        <span class="inline-block mt-3 text-cyan-300 font-bold text-[13px] group-hover:translate-x-1 transition-transform" data-en="Visit company →" data-sw="Tembelea kampuni →">Visit company →</span>
      </a>
      <a href="services" class="glass rounded-3xl p-6 hover:border-violet-400/40 transition group">
        <div class="w-12 h-12 rounded-2xl bg-violet-500/15 flex items-center justify-center mb-4"><i class="fa-solid fa-layer-group text-violet-300 text-lg"></i></div>
        <div class="badge bg-violet-500/15 text-violet-300 border border-violet-400/30 inline-block" data-en="MBILINYITECH SERVICES" data-sw="HUDUMA ZA MBILINYITECH">MBILINYITECH SERVICES</div>
        <div class="font-display font-bold text-lg mt-3" data-en="Every Service We Offer" data-sw="Kila Huduma Tunayotoa">Every Service We Offer</div>
        <p class="text-white/55 text-[13.5px] mt-2" data-en="Software, AI & chatbots, cybersecurity, forensics, graphics, government e-services, repairs and more — request any of them directly." data-sw="Software, AI na chatbots, usalama wa mtandao, forensics, graphics, huduma za serikali, ukarabati na zaidi — omba yoyote moja kwa moja.">Software, AI &amp; chatbots, cybersecurity, forensics, graphics, government e-services, repairs and more — request any of them directly.</p>
        <span class="inline-block mt-3 text-violet-300 font-bold text-[13px] group-hover:translate-x-1 transition-transform" data-en="Browse services →" data-sw="Angalia huduma →">Browse services →</span>
      </a>
      <a href="stack" class="glass rounded-3xl p-6 hover:border-blue-400/40 transition group">
        <div class="w-12 h-12 rounded-2xl bg-blue-500/15 flex items-center justify-center mb-4"><i class="fa-solid fa-code text-blue-300 text-lg"></i></div>
        <div class="badge bg-blue-500/15 text-blue-300 border border-blue-400/30 inline-block" data-en="MBILINYITECH STACK" data-sw="TEKNOLOJIA ZA MBILINYITECH">MBILINYITECH STACK</div>
        <div class="font-display font-bold text-lg mt-3" data-en="Tech Stack" data-sw="Teknolojia">Tech Stack</div>
        <p class="text-white/55 text-[13.5px] mt-2" data-en="PHP, Laravel, Python, Rust, React, R, Node.js, Flutter, PostgreSQL and the cloud tools we build with every day." data-sw="PHP, Laravel, Python, Rust, React, R, Node.js, Flutter, PostgreSQL na zana za cloud tunazotumia kila siku.">PHP, Laravel, Python, Rust, React, R, Node.js, Flutter, PostgreSQL and the cloud tools we build with every day.</p>
        <span class="inline-block mt-3 text-blue-300 font-bold text-[13px] group-hover:translate-x-1 transition-transform" data-en="See the stack →" data-sw="Ona teknolojia →">See the stack →</span>
      </a>
      <a href="ai" class="glass rounded-3xl p-6 hover:border-fuchsia-400/40 transition group">
        <div class="w-12 h-12 rounded-2xl bg-fuchsia-500/15 flex items-center justify-center mb-4"><i class="fa-solid fa-robot text-fuchsia-300 text-lg"></i></div>
        <div class="badge bg-fuchsia-500/15 text-fuchsia-300 border border-fuchsia-400/30 inline-block" data-en="MBILINYITECH AI" data-sw="AI YA MBILINYITECH">MBILINYITECH AI</div>
        <div class="font-display font-bold text-lg mt-3" data-en="AI &amp; Fine-Tuning" data-sw="AI na Fine-Tuning">AI &amp; Fine-Tuning</div>
        <p class="text-white/55 text-[13.5px] mt-2" data-en="Build your own AI & chatbot — we fine-tune models on your data and ship website and WhatsApp bots in English and Kiswahili." data-sw="Jenga AI na chatbot yako — tunafine-tune modeli kwa data yako na kupeleka bots za tovuti na WhatsApp kwa Kiingereza na Kiswahili.">Build your own AI &amp; chatbot — we fine-tune models on your data and ship website and WhatsApp bots in English and Kiswahili.</p>
        <span class="inline-block mt-3 text-fuchsia-300 font-bold text-[13px] group-hover:translate-x-1 transition-transform" data-en="Explore AI →" data-sw="Chunguza AI →">Explore AI →</span>
      </a>
      <a href="pricing" class="glass rounded-3xl p-6 hover:border-gold/40 transition group">
        <div class="w-12 h-12 rounded-2xl bg-gold/15 flex items-center justify-center mb-4"><i class="fa-solid fa-tags text-gold text-lg"></i></div>
        <div class="badge bg-gold/15 text-gold border border-gold/30 inline-block" data-en="MBILINYITECH PRICING" data-sw="BEI ZA MBILINYITECH">MBILINYITECH PRICING</div>
        <div class="font-display font-bold text-lg mt-3" data-en="Transparent Pricing" data-sw="Bei Iliyo Wazi">Transparent Pricing</div>
        <p class="text-white/55 text-[13.5px] mt-2" data-en="Published rates, itemised quotations and the 50/50 payment terms — deposit to start, balance at delivery." data-sw="Bei zilizochapishwa, nukuu zenye vitu vilivyoorodheshwa na masharti ya 50/50 — deposit kuanza, salio wakati wa kukabidhi.">Published rates, itemised quotations and the 50/50 payment terms — deposit to start, balance at delivery.</p>
        <span class="inline-block mt-3 text-gold font-bold text-[13px] group-hover:translate-x-1 transition-transform" data-en="See pricing →" data-sw="Ona bei →">See pricing →</span>
      </a>
      <a href="work" class="glass rounded-3xl p-6 hover:border-emerald-400/40 transition group">
        <div class="w-12 h-12 rounded-2xl bg-emerald-500/15 flex items-center justify-center mb-4"><i class="fa-solid fa-briefcase text-emerald-300 text-lg"></i></div>
        <div class="badge bg-emerald-500/15 text-emerald-300 border border-emerald-400/30 inline-block" data-en="MBILINYITECH WORK" data-sw="KAZI ZA MBILINYITECH">MBILINYITECH WORK</div>
        <div class="font-display font-bold text-lg mt-3" data-en="Our Work" data-sw="Kazi Zetu">Our Work</div>
        <p class="text-white/55 text-[13.5px] mt-2" data-en="Systems, platforms and tools we have designed, built and delivered — with the stack and the live status of each." data-sw="Mifumo, majukwaa na zana tulizobuni, tukajenga na tukakabidhi — pamoja na teknolojia na hali ya kila moja.">Systems, platforms and tools we have designed, built and delivered — with the stack and the live status of each.</p>
        <span class="inline-block mt-3 text-emerald-300 font-bold text-[13px] group-hover:translate-x-1 transition-transform" data-en="See our work →" data-sw="Ona kazi zetu →">See our work →</span>
      </a>
    </div>
  </section>

  <!-- CEO -->
  <section class="max-w-7xl mx-auto px-4 py-14">
    <div class="glass rounded-3xl p-6 md:p-10 grid md:grid-cols-[auto_1fr_auto] gap-8 items-center relative overflow-hidden">
      <div class="absolute top-0 left-0 w-full h-1 bg-gradient-to-r from-cyan-400 via-blue-500 to-violet-500"></div>
      <div class="relative mx-auto">
        <div class="w-32 h-32 md:w-40 md:h-40 rounded-3xl bg-gradient-to-br from-cyan-500 via-blue-600 to-violet-600 p-[3px] rotate-3">
          <div class="w-full h-full rounded-3xl bg-[#0A1226] flex flex-col items-center justify-center -rotate-3"><span class="font-display font-bold text-4xl md:text-5xl grad-text">JM</span><span class="text-[10px] tracking-[.2em] text-white/50 mt-1 font-bold" data-en="CEO &amp; FOUNDER" data-sw="MWANZILISHI">CEO &amp; FOUNDER</span></div>
        </div>
        <div class="absolute -bottom-3 -right-3 bg-emerald-500 text-[#050914] w-10 h-10 rounded-full flex items-center justify-center border-4 border-[#0E1933]"><i class="fa-solid fa-badge-check"></i></div>
      </div>
      <div>
        <div class="text-[11px] font-bold tracking-[.25em] text-cyan-300 uppercase"><span data-en="Leadership" data-sw="Uongozi">Leadership</span></div>
        <h2 class="font-display font-bold text-2xl md:text-3xl mt-1">Jackson Mbilinyi <span class="text-white/40 font-body font-normal text-base" data-en="— CEO, Software Developer, System Analyst & IT Consultant" data-sw="— CEO, Mtengenezaji wa Software, Mchambuzi wa Mifumo na Mshauri wa IT">— CEO, Software Developer, System Analyst & IT Consultant</span></h2>
        <p class="text-white/60 text-[14px] mt-3 leading-relaxed max-w-2xl" data-en="Registered enterprise under BRELA • TIN 192-147-522. I design, build, secure and maintain mission-critical systems in Python • Rust • R • React • HTML/CSS/JS and modern cloud stacks. Every project ships with a formal contract, transparent quotation, and lifetime support options." data-sw="Kampuni iliyosajiliwa chini ya BRELA • TIN 192-147-522. Nabiuni, najaenga, nalinda na natengeneza mifumo muhimu kwa Python • Rust • R • React • HTML/CSS/JS na teknolojia za kisasa za cloud. Kila mradi huja na mkataba rasmi, nukuu iliyo wazi, na chaguzi za msaada wa maisha yote.">Registered enterprise under BRELA • TIN 192-147-522. I design, build, secure and maintain mission-critical systems in Python • Rust • R • React • HTML/CSS/JS and modern cloud stacks. Every project ships with a formal contract, transparent quotation, and lifetime support options.</p>
        <div class="mt-4 flex flex-wrap gap-2 text-[12px]">
          <span class="px-3 py-1.5 rounded-full bg-white/5 border border-white/10"><i class="fa-solid fa-code text-cyan-300 mr-1"></i> <span data-en="Software Development" data-sw="Uundaji wa Software">Software Development</span></span>
          <span class="px-3 py-1.5 rounded-full bg-white/5 border border-white/10"><i class="fa-solid fa-diagram-project text-violet-300 mr-1"></i> <span data-en="System Analysis" data-sw="Uchambuzi wa Mifumo">System Analysis</span></span>
          <span class="px-3 py-1.5 rounded-full bg-white/5 border border-white/10"><i class="fa-solid fa-briefcase text-gold mr-1"></i> <span data-en="IT Consultancy" data-sw="Ushauri wa IT">IT Consultancy</span></span>
          <span class="px-3 py-1.5 rounded-full bg-white/5 border border-white/10"><i class="fa-solid fa-screwdriver-wrench text-emerald-300 mr-1"></i> <span data-en="Maintenance" data-sw="Matengenezo">Maintenance</span></span>
          <span class="px-3 py-1.5 rounded-full bg-white/5 border border-white/10"><i class="fa-solid fa-shield-halved text-red-300 mr-1"></i> <span data-en="Security" data-sw="Usalama">Security</span></span>
        </div>
      </div>
      <div class="grid gap-3 min-w-[220px]">
        <a href="tel:+255796752645" class="glass rounded-2xl p-4 flex items-center gap-3 hover:border-cyan-400/40"><div class="w-10 h-10 rounded-xl bg-green-500/15 flex items-center justify-center"><i class="fa-solid fa-phone text-green-300"></i></div><div><div class="text-[11px] text-white/50 font-bold uppercase tracking-wider" data-en="Call / WhatsApp / SMS" data-sw="Simu / WhatsApp / SMS">Call / WhatsApp / SMS</div><div class="font-mono font-bold">0796 752 645</div></div></a>
        <a href="mailto:mbilinyitech@gmail.com" class="glass rounded-2xl p-4 flex items-center gap-3 hover:border-cyan-400/40"><div class="w-10 h-10 rounded-xl bg-blue-500/15 flex items-center justify-center"><i class="fa-solid fa-envelope text-blue-300"></i></div><div><div class="text-[11px] text-white/50 font-bold uppercase tracking-wider" data-en="Email" data-sw="Barua pepe">Email</div><div class="font-bold text-[13px]">mbilinyitech@gmail.com</div></div></a>
        <a href="https://instagram.com/mbilinyitech" target="_blank" class="glass rounded-2xl p-4 flex items-center gap-3 hover:border-pink-400/40"><div class="w-10 h-10 rounded-xl bg-pink-500/15 flex items-center justify-center"><i class="fa-brands fa-instagram text-pink-300"></i></div><div><div class="text-[11px] text-white/50 font-bold uppercase tracking-wider" data-en="Instagram" data-sw="Instagram">Instagram</div><div class="font-bold">@mbilinyitech</div></div></a>
      </div>
    </div>
  </section>

  <!-- OUR TEAM -->
  <section class="max-w-7xl mx-auto px-4 py-14">
    <div class="flex flex-wrap items-end justify-between gap-4">
      <div>
        <div class="badge bg-violet-500/15 text-violet-300 border border-violet-400/30" data-en="OUR TEAM" data-sw="TIMU YETU">OUR TEAM</div>
        <h2 class="font-display font-bold text-3xl md:text-4xl mt-3"><span data-en="The People Behind" data-sw="Watu Nyuma ya">The People Behind</span> <span class="grad-text" data-en="Your Project" data-sw="Mradi Wako">Your Project</span></h2>
      </div>
      <a href="team" class="px-5 py-3 rounded-xl border border-white/15 text-sm font-bold hover:bg-white/5" data-en="Meet the full team →" data-sw="Kutana na timu kamili →">Meet the full team →</a>
    </div>
    <div id="teamGrid" class="grid sm:grid-cols-2 lg:grid-cols-3 gap-4 mt-8"></div>
  </section>

  <!-- SIGN IN / SIGN UP — right on the homepage -->
  <?php if (!$me): ?>
  <section id="homeAuth" class="max-w-7xl mx-auto px-4 py-14">
    <div class="max-w-5xl mx-auto">
      <div class="text-center mb-8">
        <div class="badge bg-gold/15 text-gold border border-gold/30" data-en="CLIENT PORTAL" data-sw="PORTAL YA MTEJA">CLIENT PORTAL</div>
        <h2 class="font-display font-bold text-3xl md:text-4xl mt-3">
          <span data-en="Sign In or" data-sw="Ingia au">Sign In or</span>
          <span class="grad-text" data-en="Create Your Account" data-sw="Fungua Akaunti Yako">Create Your Account</span>
        </h2>
        <p class="text-white/55 text-[14.5px] mt-3 max-w-2xl mx-auto" data-en="One secure login for everything: submit requirements, receive official quotations, e-sign contracts, pay invoices and chat with support." data-sw="Kuingia moja salama kwa kila kitu: tuma mahitaji, pokea nukuu rasmi, saini mikataba kidijitali, lipa ankara na zungumza na msaada.">One secure login for everything: submit requirements, receive official quotations, e-sign contracts, pay invoices and chat with support.</p>
      </div>
      <div class="grid md:grid-cols-2 gap-6 items-stretch">
        <div class="rounded-3xl p-[1.5px] bg-gradient-to-br from-cyan-400 to-violet-600">
          <div class="rounded-3xl bg-[#070D1F] p-8 h-full flex flex-col justify-between relative overflow-hidden">
            <div class="orb w-[300px] h-[300px] bg-violet-600/30 -top-20 -right-20"></div>
            <div class="relative">
              <div class="w-14 h-14 rounded-2xl grad-btn flex items-center justify-center font-display font-bold text-2xl">M</div>
              <h3 class="font-display font-bold text-2xl mt-5"><span data-en="Why sign in?" data-sw="Kwa nini kuingia?">Why sign in?</span></h3>
              <div class="grid gap-2.5 mt-5 text-[13px]">
                <div class="flex items-center gap-3 glass rounded-xl px-4 py-3"><i class="fa-solid fa-file-pen text-cyan-300"></i> <span data-en="Requirement forms with tracking IDs" data-sw="Fomu za mahitaji zenye namba za kufuatilia">Requirement forms with tracking IDs</span></div>
                <div class="flex items-center gap-3 glass rounded-xl px-4 py-3"><i class="fa-solid fa-file-invoice-dollar text-gold"></i> <span data-en="Official quotations & transparent pricing" data-sw="Nukuu rasmi na bei iliyo wazi">Official quotations &amp; transparent pricing</span></div>
                <div class="flex items-center gap-3 glass rounded-xl px-4 py-3"><i class="fa-solid fa-file-contract text-violet-300"></i> <span data-en="BRELA contracts with e-signature" data-sw="Mikataba ya BRELA yenye sahihi ya kidijitali">BRELA contracts with e-signature</span></div>
                <div class="flex items-center gap-3 glass rounded-xl px-4 py-3"><i class="fa-solid fa-users-gear text-emerald-300"></i> <span data-en="Full account management" data-sw="Usimamizi kamili wa akaunti">Full account management</span></div>
              </div>
            </div>
            <div class="relative mt-6 rounded-2xl bg-cyan-500/[.06] border border-cyan-400/25 p-4 text-[12px] text-white/70">
              <div class="font-bold text-cyan-300 mb-1"><i class="fa-solid fa-shield-halved mr-1"></i> <span data-en="VERIFIED SIGN-IN" data-sw="KUINGIA KUTHIBITISHWA">VERIFIED SIGN-IN</span></div>
              <span data-en="After your password we ask for your PIN (or the last 4 digits of your phone) so nobody else can enter your portal." data-sw="Baada ya neno la siri tunakuomba PIN yako (au tarakimu 4 za mwisho za simu yako) ili mtu mwingine asipate portal yako.">After your password we ask for your PIN (or the last 4 digits of your phone) so nobody else can enter your portal.</span>
            </div>
          </div>
        </div>
        <?php require __DIR__ . '/partials/auth-panel.php'; ?>
      </div>
    </div>
  </section>
  <?php endif; ?>

  <!-- OUR PROJECTS → the MbilinyiTech Platform -->
  <section class="max-w-7xl mx-auto px-4 pb-16">
    <div class="glass rounded-3xl p-6 md:p-10 relative overflow-hidden">
      <div class="absolute top-0 left-0 w-full h-1 bg-gradient-to-r from-cyan-400 via-blue-500 to-violet-500"></div>
      <div class="orb w-[360px] h-[360px] bg-cyan-500/20 -top-24 -right-24"></div>
      <div class="relative grid md:grid-cols-[1fr_auto] gap-8 items-center">
        <div>
          <div class="badge bg-cyan-500/15 text-cyan-300 border border-cyan-400/30" data-en="OUR PROJECTS" data-sw="MIRADI YETU">OUR PROJECTS</div>
          <h2 class="font-display font-bold text-3xl md:text-4xl mt-3">
            <span data-en="See Everything We Have" data-sw="Ona Kila Kitu Tulichojenga">See Everything We Have</span>
            <span class="grad-text" data-en="Built" data-sw="Kujenga">Built</span>
          </h2>
          <p class="text-white/60 text-[14.5px] mt-3 leading-relaxed max-w-2xl" data-en="Systems, platforms and tools we have designed, built and delivered — with the full stack, the problem each one solves and its live status. Browse the complete catalogue on the MbilinyiTech Platform." data-sw="Mifumo, majukwaa na zana tulizobuni, tukajenga na tukakabidhi — pamoja na teknolojia, tatizo ambalo kila moja hutatua na hali yake. Angalia orodha kamili kwenye MbilinyiTech Platform.">Systems, platforms and tools we have designed, built and delivered — with the full stack, the problem each one solves and its live status. Browse the complete catalogue on the MbilinyiTech Platform.</p>
          <div class="flex flex-wrap gap-2 mt-5 text-[12px]">
            <span class="px-3 py-1.5 rounded-full bg-white/5 border border-white/10"><i class="fa-solid fa-diagram-project text-cyan-300 mr-1"></i> <span data-en="Every project, in full detail" data-sw="Kila mradi, kwa maelezo kamili">Every project, in full detail</span></span>
            <span class="px-3 py-1.5 rounded-full bg-white/5 border border-white/10"><i class="fa-solid fa-layer-group text-violet-300 mr-1"></i> <span data-en="Stack & technologies used" data-sw="Teknolojia zilizotumika">Stack &amp; technologies used</span></span>
            <span class="px-3 py-1.5 rounded-full bg-white/5 border border-white/10"><i class="fa-solid fa-circle-check text-emerald-300 mr-1"></i> <span data-en="Live status of each build" data-sw="Hali ya kila mradi">Live status of each build</span></span>
          </div>
        </div>
        <div class="grid gap-3 min-w-[240px]">
          <a id="platformCta" href="#" target="_blank" rel="noopener" class="grad-btn px-7 py-4 rounded-2xl font-bold flex items-center justify-center gap-2 text-center">
            <i class="fa-solid fa-arrow-up-right-from-square"></i>
            <span data-en="Visit MbilinyiTech Platform" data-sw="Tembelea MbilinyiTech Platform">Visit MbilinyiTech Platform</span>
          </a>
          <a href="work" class="px-7 py-4 rounded-2xl font-bold border border-white/15 hover:bg-white/5 flex items-center justify-center gap-2">
            <i class="fa-solid fa-briefcase text-gold"></i>
            <span data-en="See projects on this site" data-sw="Ona miradi kwenye tovuti hii">See projects on this site</span>
          </a>
        </div>
      </div>
    </div>
  </section>
</div>

<?php require __DIR__ . '/partials/footer.php'; ?>
