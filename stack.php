<?php
require_once __DIR__ . '/config/app.php';
$pageTitle = 'Tech Stack — Python, Rust, R, React & More | Mbilinyi Tech Solutions';
$page = 'landing';
require __DIR__ . '/partials/head.php';
?>
<body class="antialiased">
<?php require __DIR__ . '/partials/header.php'; ?>

<!-- ============ TECH STACK PAGE ============ -->
<div id="view-landing" class="view-section active">
  <section id="stack" class="max-w-7xl mx-auto px-4 pt-8 pb-16">
    <div class="glass rounded-3xl p-6 md:p-10 relative overflow-hidden">
      <div class="orb w-[400px] h-[400px] bg-blue-600/20 -top-40 right-0"></div>
      <div class="grid lg:grid-cols-2 gap-10 relative">
        <div>
          <div class="badge bg-violet-500/15 text-violet-300 border border-violet-400/30">TECH ARSENAL • R • RUST • REACT • PYTHON • HTML</div>
          <h2 class="font-display font-bold text-3xl md:text-4xl mt-4">Mastered Languages. <span class="grad-text">Production Discipline.</span></h2>
          <p class="text-white/55 text-[14px] mt-3">We pick the right tool for the job — from blazing-fast Rust microservices to R statistical engines and pixel-perfect React frontends.</p>
          <div class="flex flex-wrap gap-2 mt-5" id="stackTabs"></div>
          <div id="stackDetail" class="mt-5 glass rounded-2xl p-5"></div>
        </div>
        <div>
          <div class="text-[12px] font-bold tracking-widest text-white/50 uppercase mb-3">Proficiency Matrix</div>
          <div id="skillBars" class="grid gap-3"></div>
          <div class="mt-5 grid grid-cols-3 gap-3 text-center">
            <div class="rounded-2xl bg-white/[.03] border border-white/10 p-4"><div class="font-display font-bold text-xl text-cyan-300">15+</div><div class="text-[11px] text-white/50">Languages & Frameworks</div></div>
            <div class="rounded-2xl bg-white/[.03] border border-white/10 p-4"><div class="font-display font-bold text-xl text-violet-300">40+</div><div class="text-[11px] text-white/50">Cloud Deployments</div></div>
            <div class="rounded-2xl bg-white/[.03] border border-white/10 p-4"><div class="font-display font-bold text-xl text-emerald-300">0</div><div class="text-[11px] text-white/50">Breaches to Date</div></div>
          </div>
        </div>
      </div>
    </div>
  </section>
</div>

<?php require __DIR__ . '/partials/footer.php'; ?>
