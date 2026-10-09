<?php
require_once __DIR__ . '/config/app.php';
$pageTitle = 'How It Works — From Request to Contract in 5 Steps | Mbilinyi Tech Solutions';
$page = 'landing';
require __DIR__ . '/partials/head.php';
?>
<body class="antialiased">
<?php require __DIR__ . '/partials/header.php'; ?>

<!-- ============ PROCESS PAGE ============ -->
<div id="view-landing" class="view-section active">
  <section id="process" class="max-w-7xl mx-auto px-4 pt-8 pb-16">
    <div class="text-center"><div class="badge bg-blue-500/10 text-blue-300 border border-blue-400/30 inline-block">HOW IT WORKS</div><h2 class="font-display font-bold text-3xl md:text-4xl mt-3">From Request to Contract <span class="grad-text">in 5 Steps</span></h2></div>
    <div class="grid md:grid-cols-5 gap-3 mt-8">
      <div class="glass rounded-2xl p-5 text-center relative"><div class="w-11 h-11 mx-auto rounded-2xl grad-btn flex items-center justify-center font-bold">1</div><div class="font-bold mt-3 text-[14px]">Submit Requirements</div><div class="text-[12px] text-white/55 mt-1">Fill the smart form — specs, tech, budget, deadline.</div></div>
      <div class="glass rounded-2xl p-5 text-center"><div class="w-11 h-11 mx-auto rounded-2xl bg-violet-600 flex items-center justify-center font-bold">2</div><div class="font-bold mt-3 text-[14px]">Admin Review & Price</div><div class="text-[12px] text-white/55 mt-1">CEO reviews & issues official quotation.</div></div>
      <div class="glass rounded-2xl p-5 text-center"><div class="w-11 h-11 mx-auto rounded-2xl bg-cyan-600 flex items-center justify-center font-bold">3</div><div class="font-bold mt-3 text-[14px]">Approve & Contract</div><div class="text-[12px] text-white/55 mt-1">You approve → e-sign BRELA contract.</div></div>
      <div class="glass rounded-2xl p-5 text-center"><div class="w-11 h-11 mx-auto rounded-2xl bg-emerald-600 flex items-center justify-center font-bold">4</div><div class="font-bold mt-3 text-[14px]">Build & Track</div><div class="text-[12px] text-white/55 mt-1">Live progress, milestones & testing.</div></div>
      <div class="glass rounded-2xl p-5 text-center"><div class="w-11 h-11 mx-auto rounded-2xl gold-btn flex items-center justify-center font-bold">5</div><div class="font-bold mt-3 text-[14px]">Deploy & Maintain</div><div class="text-[12px] text-white/55 mt-1">Launch, security hardening & SLA support.</div></div>
    </div>
    <div class="mt-8 max-w-3xl mx-auto glass rounded-3xl p-6 text-[13.5px] text-white/70">
      <div class="font-display font-bold text-lg text-white mb-2"><i class="fa-solid fa-circle-info text-cyan-300 mr-2"></i>Payment Rule — Kanuni ya Malipo</div>
      Mradi unaanza rasmi baada ya mkataba kusainiwa na <strong class="text-white">50% ya bei kulipwa (deposit)</strong>. Nusu iliyobaki unalipia baada ya kukabidhi project. Deposit invoice inaonekana kwenye portal yako — bila deposit, project haiwezi kuanza.
    </div>
  </section>
</div>

<?php require __DIR__ . '/partials/footer.php'; ?>
