<?php
require_once __DIR__ . '/config/app.php';
$pageTitle = 'Our Work — Production Systems We Have Shipped | Mbilinyi Tech Solutions';
$page = 'landing';
require __DIR__ . '/partials/head.php';
?>
<body class="antialiased">
<?php require __DIR__ . '/partials/header.php'; ?>

<!-- ============ WORK PAGE ============ -->
<div id="view-landing" class="view-section active">
  <section id="work" class="max-w-7xl mx-auto px-4 pt-8 pb-16">
    <div class="flex flex-wrap items-end justify-between gap-4"><div><div class="badge bg-emerald-500/10 text-emerald-300 border border-emerald-400/30">SELECTED WORK</div><h2 class="font-display font-bold text-3xl md:text-4xl mt-3">Production Systems <span class="grad-text">We've Shipped</span></h2></div><button onclick="openQuoteWizard()" class="px-5 py-3 rounded-xl border border-white/15 text-sm font-bold hover:bg-white/5">Become our next case study →</button></div>
    <div id="portfolioGrid" class="grid sm:grid-cols-2 lg:grid-cols-3 gap-4 mt-8"></div>
    <!-- Testimonials -->
    <div class="mt-8 glass rounded-3xl p-6 md:p-8 relative overflow-hidden">
      <div class="flex items-center justify-between"><div class="font-display font-bold text-xl"><i class="fa-solid fa-quote-left text-cyan-300 mr-2"></i>Client Voices</div><div class="flex gap-2"><button onclick="moveTesti(-1)" class="w-9 h-9 rounded-full border border-white/15 hover:bg-white/5"><i class="fa-solid fa-chevron-left text-xs"></i></button><button onclick="moveTesti(1)" class="w-9 h-9 rounded-full border border-white/15 hover:bg-white/5"><i class="fa-solid fa-chevron-right text-xs"></i></button></div></div>
      <div id="testiBox" class="mt-4"></div>
    </div>
  </section>
</div>

<?php require __DIR__ . '/partials/footer.php'; ?>
