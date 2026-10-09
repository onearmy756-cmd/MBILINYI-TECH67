<?php
require_once __DIR__ . '/config/app.php';
$pageTitle = 'Mbilinyi Tech Solutions — World-Class Software, AI & Security | Jackson Mbilinyi, CEO';
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
          <span class="badge bg-emerald-500/15 text-emerald-300 border border-emerald-400/30"><i class="fa-solid fa-circle-check mr-1"></i> BRELA CERTIFIED</span>
          <span class="badge bg-cyan-500/10 text-cyan-300 border border-cyan-400/30">TIN 192-147-522 VERIFIED</span>
          <span class="badge bg-violet-500/15 text-violet-300 border border-violet-400/30">ISO-GRADE SECURITY</span>
        </div>
        <h1 class="font-display font-bold text-[38px] sm:text-[54px] lg:text-[62px] leading-[1.02]">
          <span data-en="We Engineer" data-sw="Tunaunda">We Engineer</span> <span class="grad-text">World-Class</span><br>
          <span data-en="Software That Scales." data-sw="Software ya Kisasa.">Software That Scales.</span>
        </h1>
        <div class="mt-4 inline-block px-4 py-2 rounded-2xl bg-gradient-to-r from-cyan-500/15 via-blue-500/15 to-violet-500/15 border border-cyan-400/30 font-mono text-[13px] tracking-[.15em] text-cyan-200/90">
          <span data-en="★ YOUR PROBLEM, OUR SOLUTION ★" data-sw="★ SHIDA YAKO, TATUZO LETU ★">★ YOUR PROBLEM, OUR SOLUTION ★</span>
        </div>
        <p class="mt-5 text-white/65 text-[15.5px] leading-relaxed max-w-xl" data-en="From custom systems, AI fine-tuning & chatbots to maintenance and military-grade security — Mbilinyi Tech Solutions, led by CEO Jackson Mbilinyi, delivers production-ready technology for startups, enterprises & government." data-sw="Kuanzia mifumo maalum, AI fine-tuning na chatbots hadi maintenance na usalama wa hali ya juu — Mbilinyi Tech Solutions, inayoongozwa na CEO Jackson Mbilinyi, inatoa teknolojia bora kwa startups, makampuni na serikali.">From custom systems, AI fine-tuning & chatbots to maintenance and military-grade security — Mbilinyi Tech Solutions, led by CEO Jackson Mbilinyi, delivers production-ready technology for startups, enterprises & government.</p>
        <div class="mt-7 flex flex-wrap gap-3">
          <button onclick="openQuoteWizard()" class="grad-btn px-7 py-4 rounded-2xl font-bold flex items-center gap-2"><i class="fa-solid fa-rocket"></i> <span data-en="Start Your Project — Free Quote" data-sw="Anza Mradi — Nukuu Bure">Start Your Project — Free Quote</span></button>
          <button onclick="location.href='track'" class="px-7 py-4 rounded-2xl font-bold border border-white/15 hover:bg-white/5 flex items-center gap-2"><i class="fa-solid fa-box-open text-gold"></i> <span data-en="Track Existing Request" data-sw="Fuatilia Ombi Lako">Track Existing Request</span></button>
        </div>
        <div class="mt-6 flex items-center gap-4 text-[13px] text-white/60">
          <div class="flex -space-x-3">
            <div class="w-9 h-9 rounded-full border-2 border-[#050914] bg-gradient-to-br from-cyan-500 to-blue-600 flex items-center justify-center font-bold text-xs">JM</div>
            <div class="w-9 h-9 rounded-full border-2 border-[#050914] bg-gradient-to-br from-violet-500 to-fuchsia-600 flex items-center justify-center font-bold text-xs">AK</div>
            <div class="w-9 h-9 rounded-full border-2 border-[#050914] bg-gradient-to-br from-emerald-500 to-teal-600 flex items-center justify-center font-bold text-xs">SN</div>
            <div class="w-9 h-9 rounded-full border-2 border-[#050914] bg-[#0E1933] flex items-center justify-center font-bold text-[10px] text-cyan-300">120+</div>
          </div>
          <div><div class="flex text-gold text-xs gap-0.5"><i class="fa-solid fa-star"></i><i class="fa-solid fa-star"></i><i class="fa-solid fa-star"></i><i class="fa-solid fa-star"></i><i class="fa-solid fa-star"></i> <span class="text-white ml-1 font-bold">4.9/5</span></div><div data-en="Trusted by 120+ clients in 8 countries" data-sw="Tunaaminiwa na wateja 120+ nchi 8">Trusted by 120+ clients in 8 countries</div></div>
        </div>
        <div class="mt-8 grid grid-cols-3 gap-3 max-w-lg">
          <div class="glass rounded-2xl p-4 text-center"><div class="font-display font-bold text-2xl grad-text"><span class="counter" data-target="120">0</span>+</div><div class="text-[11px] text-white/55 uppercase tracking-wider font-bold">Projects</div></div>
          <div class="glass rounded-2xl p-4 text-center"><div class="font-display font-bold text-2xl grad-text"><span class="counter" data-target="99">0</span>%</div><div class="text-[11px] text-white/55 uppercase tracking-wider font-bold">Uptime SLA</div></div>
          <div class="glass rounded-2xl p-4 text-center"><div class="font-display font-bold text-2xl grad-text"><span class="counter" data-target="24">0</span>/7</div><div class="text-[11px] text-white/55 uppercase tracking-wider font-bold">Support</div></div>
        </div>
      </div>
      <div class="relative">
        <div class="orbit-ring w-[420px] h-[420px] left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 hidden md:block"></div>
        <div class="orbit-ring w-[320px] h-[320px] left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 hidden md:block" style="animation-duration:16s"></div>
        <div class="glass rounded-3xl overflow-hidden shadow-2xl relative z-10">
          <div class="flex items-center justify-between px-4 py-3 border-b border-white/10 bg-white/[.02]">
            <div class="code-dots flex gap-1.5"><span class="bg-red-400"></span><span class="bg-yellow-400"></span><span class="bg-green-400"></span></div>
            <div class="font-mono text-[11px] text-white/50 flex items-center gap-2"><span class="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span> mbilinyi_core — production.ts • LIVE</div>
            <i class="fa-solid fa-lock text-emerald-300 text-xs"></i>
          </div>
          <div class="p-5 font-mono text-[12.5px] leading-[1.8] bg-[#070D1F]">
            <div><span class="text-fuchsia-400">import</span> <span class="text-white">{ MbilinyiCloud }</span> <span class="text-fuchsia-400">from</span> <span class="text-emerald-300">"@mbilinyi/sdk"</span>;</div>
            <div><span class="text-fuchsia-400">import</span> <span class="text-white">{ RustCore, ReactUI, PyAI }</span> <span class="text-fuchsia-400">from</span> <span class="text-emerald-300">"@mbilinyi/stack"</span>;</div>
            <div class="mt-2"><span class="text-slate-500">// CEO: Jackson Mbilinyi | BRELA ✓ | TIN 192-147-522</span></div>
            <div><span class="text-cyan-300">const</span> <span class="text-yellow-200">project</span> <span class="text-white">= await</span> <span class="text-cyan-300">MbilinyiCloud</span><span class="text-white">.deploy({</span></div>
            <div class="pl-5"><span class="text-white">stack:</span> <span class="text-emerald-300">[<span id="heroStackText">"react", "rust", "python"</span>]</span>,</div>
            <div class="pl-5"><span class="text-white">security:</span> <span class="text-emerald-300">"military-grade"</span>,</div>
            <div class="pl-5"><span class="text-white">ai:</span> <span class="text-emerald-300">"fine-tuned + chatbot"</span>,</div>
            <div class="pl-5"><span class="text-white">sla:</span> <span class="text-orange-300">99.99</span><span class="text-white">,</span></div>
            <div><span class="text-white">});</span> <span class="text-emerald-300 font-bold">→ status: "DEPLOYED ✓"</span><span class="animate-pulse text-cyan-300">▌</span></div>
          </div>
          <div class="px-4 py-3 border-t border-white/10 flex items-center justify-between text-[12px]">
            <span class="text-white/50 font-mono">build passed in 1.2s • 0 vulnerabilities</span>
            <span class="flex gap-2"><span class="badge bg-emerald-500/15 text-emerald-300 border border-emerald-400/30">SECURE</span><span class="badge bg-cyan-500/10 text-cyan-300 border border-cyan-400/30">SCALABLE</span></span>
          </div>
        </div>
        <div class="absolute -left-3 top-8 z-20 glass rounded-2xl px-4 py-3 float hidden sm:flex items-center gap-3"><div class="w-9 h-9 rounded-xl bg-emerald-500/20 flex items-center justify-center"><i class="fa-solid fa-shield-halved text-emerald-300"></i></div><div><div class="text-xs font-bold">Security Audit Passed</div><div class="text-[11px] text-white/50">OWASP • Zero CVEs</div></div></div>
        <div class="absolute -right-2 bottom-10 z-20 glass rounded-2xl px-4 py-3 float2 hidden sm:flex items-center gap-3"><div class="w-9 h-9 rounded-xl bg-violet-500/20 flex items-center justify-center"><i class="fa-solid fa-robot text-violet-300"></i></div><div><div class="text-xs font-bold">AI Chatbot Online</div><div class="text-[11px] text-white/50 flex items-center gap-1"><span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span> replies in ~2s</div></div></div>
      </div>
    </div>
    <div class="border-y border-white/10 bg-white/[.015] py-4 overflow-hidden relative">
      <div class="marquee-track font-mono text-[13px] text-white/60" id="techMarquee"></div>
    </div>
  </section>

  <!-- CEO -->
  <section class="max-w-7xl mx-auto px-4 py-14">
    <div class="glass rounded-3xl p-6 md:p-10 grid md:grid-cols-[auto_1fr_auto] gap-8 items-center relative overflow-hidden">
      <div class="absolute top-0 left-0 w-full h-1 bg-gradient-to-r from-cyan-400 via-blue-500 to-violet-500"></div>
      <div class="relative mx-auto">
        <div class="w-32 h-32 md:w-40 md:h-40 rounded-3xl bg-gradient-to-br from-cyan-500 via-blue-600 to-violet-600 p-[3px] rotate-3">
          <div class="w-full h-full rounded-3xl bg-[#0A1226] flex flex-col items-center justify-center -rotate-3"><span class="font-display font-bold text-4xl md:text-5xl grad-text">JM</span><span class="text-[10px] tracking-[.2em] text-white/50 mt-1 font-bold">CEO & FOUNDER</span></div>
        </div>
        <div class="absolute -bottom-3 -right-3 bg-emerald-500 text-[#050914] w-10 h-10 rounded-full flex items-center justify-center border-4 border-[#0E1933]"><i class="fa-solid fa-badge-check"></i></div>
      </div>
      <div>
        <div class="text-[11px] font-bold tracking-[.25em] text-cyan-300 uppercase">Leadership • Uongozi</div>
        <h2 class="font-display font-bold text-2xl md:text-3xl mt-1">Jackson Mbilinyi <span class="text-white/40 font-body font-normal text-base">— CEO, Software Developer, System Analyst & IT Consultant</span></h2>
        <p class="text-white/60 text-[14px] mt-3 leading-relaxed max-w-2xl">Registered enterprise under <strong class="text-white">BRELA</strong> • <strong class="text-white font-mono">TIN 192-147-522</strong>. I design, build, secure and maintain mission-critical systems in <span class="text-cyan-300 font-mono">Python • Rust • R • React • HTML/CSS/JS</span> and modern cloud stacks. Every project ships with a formal contract, transparent quotation, and lifetime support options.</p>
        <div class="mt-4 flex flex-wrap gap-2 text-[12px]">
          <span class="px-3 py-1.5 rounded-full bg-white/5 border border-white/10"><i class="fa-solid fa-code text-cyan-300 mr-1"></i> Software Development</span>
          <span class="px-3 py-1.5 rounded-full bg-white/5 border border-white/10"><i class="fa-solid fa-diagram-project text-violet-300 mr-1"></i> System Analysis</span>
          <span class="px-3 py-1.5 rounded-full bg-white/5 border border-white/10"><i class="fa-solid fa-briefcase text-gold mr-1"></i> IT Consultancy</span>
          <span class="px-3 py-1.5 rounded-full bg-white/5 border border-white/10"><i class="fa-solid fa-screwdriver-wrench text-emerald-300 mr-1"></i> Maintenance</span>
          <span class="px-3 py-1.5 rounded-full bg-white/5 border border-white/10"><i class="fa-solid fa-shield-halved text-red-300 mr-1"></i> Security</span>
        </div>
      </div>
      <div class="grid gap-3 min-w-[220px]">
        <a href="tel:+255796752645" class="glass rounded-2xl p-4 flex items-center gap-3 hover:border-cyan-400/40"><div class="w-10 h-10 rounded-xl bg-green-500/15 flex items-center justify-center"><i class="fa-solid fa-phone text-green-300"></i></div><div><div class="text-[11px] text-white/50 font-bold uppercase tracking-wider">Call / WhatsApp / SMS</div><div class="font-mono font-bold">0796 752 645</div></div></a>
        <a href="mailto:mbilinyitech@gmail.com" class="glass rounded-2xl p-4 flex items-center gap-3 hover:border-cyan-400/40"><div class="w-10 h-10 rounded-xl bg-blue-500/15 flex items-center justify-center"><i class="fa-solid fa-envelope text-blue-300"></i></div><div><div class="text-[11px] text-white/50 font-bold uppercase tracking-wider">Email</div><div class="font-bold text-[13px]">mbilinyitech@gmail.com</div></div></a>
        <a href="https://instagram.com/mbilinyitech" target="_blank" class="glass rounded-2xl p-4 flex items-center gap-3 hover:border-pink-400/40"><div class="w-10 h-10 rounded-xl bg-pink-500/15 flex items-center justify-center"><i class="fa-brands fa-instagram text-pink-300"></i></div><div><div class="text-[11px] text-white/50 font-bold uppercase tracking-wider">Instagram</div><div class="font-bold">@mbilinyitech</div></div></a>
      </div>
    </div>
  </section>
</div>

<?php require __DIR__ . '/partials/footer.php'; ?>
