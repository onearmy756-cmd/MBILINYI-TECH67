<?php
require_once __DIR__ . '/config/app.php';
$pageTitle = 'Services — Custom Software, System Analysis, IT Consultancy, Security & AI | Mbilinyi Tech Solutions';
$page = 'landing';
require __DIR__ . '/partials/head.php';
?>
<body class="antialiased">
<?php require __DIR__ . '/partials/header.php'; ?>

<!-- ============ SERVICES PAGE ============ -->
<div id="view-landing" class="view-section active">
  <section id="services" class="max-w-7xl mx-auto px-4 pt-8 pb-16">
    <div class="text-center max-w-2xl mx-auto">
      <div class="badge bg-cyan-500/10 text-cyan-300 border border-cyan-400/30 inline-block">ENTERPRISE SERVICES</div>
      <h2 class="font-display font-bold text-3xl md:text-5xl mt-4">Everything Your Business Needs <span class="grad-text">to Dominate Digitally</span></h2>
      <p class="text-white/55 mt-3 text-[14.5px]">Eight production-grade service lines. One accountable partner. Formal contracts & SLAs on every engagement.</p>
    </div>
    <div id="servicesGrid" class="grid sm:grid-cols-2 lg:grid-cols-4 gap-4 mt-10"></div>
  </section>
</div>

<?php require __DIR__ . '/partials/footer.php'; ?>
