<?php
require_once __DIR__ . '/config/app.php';
$pageTitle = 'Our Team — Mbilinyi Tech Solutions';
$page = 'landing';
require __DIR__ . '/partials/head.php';
?>
<body class="antialiased">
<?php require __DIR__ . '/partials/header.php'; ?>

<!-- ============ TEAM PAGE ============ -->
<div id="view-landing" class="view-section active">
  <section class="max-w-7xl mx-auto px-4 pt-8 pb-16">
    <div class="badge bg-violet-500/15 text-violet-300 border border-violet-400/30"><span data-en="OUR TEAM" data-sw="TIMU YETU">OUR TEAM</span></div>
    <h2 class="font-display font-bold text-3xl md:text-4xl mt-3"><span data-en="The People Behind" data-sw="Watu Nyuma ya">The People Behind</span> <span class="grad-text" data-en="Your Project" data-sw="Mradi Wako">Your Project</span></h2>
    <p class="text-white/55 text-[14px] mt-3 max-w-2xl" data-en="A focused team of engineers, analysts and consultants led by CEO Jackson Mbilinyi." data-sw="Timu madhubuti ya wahandisi, wachambuzi na washauri inayoongozwa na CEO Jackson Mbilinyi.">A focused team of engineers, analysts and consultants led by CEO Jackson Mbilinyi.</p>
    <div id="teamGrid" class="grid sm:grid-cols-2 lg:grid-cols-3 gap-4 mt-8"></div>
  </section>
</div>

<?php require __DIR__ . '/partials/footer.php'; ?>
