<?php
require_once __DIR__ . '/config/app.php';
$pageTitle = 'Track Your Request in Real Time | Mbilinyi Tech Solutions';
$page = 'landing';
require __DIR__ . '/partials/head.php';
?>
<body class="antialiased">
<?php require __DIR__ . '/partials/header.php'; ?>

<!-- ============ TRACK PAGE ============ -->
<div id="view-landing" class="view-section active">
  <section id="track" class="max-w-7xl mx-auto px-4 pt-8 pb-16">
    <div class="glass rounded-3xl p-6 md:p-10 grid lg:grid-cols-2 gap-8">
      <div>
        <div class="badge bg-cyan-500/10 text-cyan-300 border border-cyan-400/30"><span data-en="LIVE TRACKING" data-sw="UFUATILIAJI WA MOJA KWA MOJA">LIVE TRACKING</span></div>
        <h2 class="font-display font-bold text-3xl mt-3"><span data-en="Track Your Request" data-sw="Fuatilia Ombi Lako">Track Your Request</span> <span class="grad-text" data-en="in Real Time" data-sw="papo hapo">in Real Time</span></h2>
        <p class="text-white/55 text-[14px] mt-2" data-en="Enter your Tracking ID (e.g. MTS-2026-XXXX) — it was sent to your email &amp; WhatsApp after you submitted the form." data-sw="Ingiza Tracking ID yako (mf. MTS-2026-XXXX) — ulitumwa kwenye email na WhatsApp yako baada ya kuwasilisha fomu.">Enter your Tracking ID (e.g. MTS-2026-XXXX) — it was sent to your email &amp; WhatsApp after you submitted the form.</p>
        <div class="flex gap-2 mt-5"><input id="trackInput" class="input font-mono uppercase" placeholder="MTS-2026-XXXX"><button onclick="trackOrder()" class="grad-btn px-6 rounded-xl font-bold whitespace-nowrap"><i class="fa-solid fa-magnifying-glass mr-1"></i> <span data-en="Track" data-sw="Fuatilia">Track</span></button></div>
        <div class="mt-4 flex flex-wrap gap-2 text-[12px]"><span class="text-white/40" data-en="Try demo IDs:" data-sw="Jaribu ID za mfano:">Try demo IDs:</span><button onclick="fillTrack('MTS-2026-4810')" class="px-3 py-1 rounded-full bg-white/5 border border-white/10 font-mono hover:border-cyan-400/40">MTS-2026-4810</button><button onclick="fillTrack('MTS-2026-5523')" class="px-3 py-1 rounded-full bg-white/5 border border-white/10 font-mono hover:border-cyan-400/40">MTS-2026-5523</button></div>
      </div>
      <div id="trackResult" class="glass rounded-2xl p-6 min-h-[280px] flex items-center justify-center text-center text-white/40 text-[14px]" data-en="Your project timeline, quotation status &amp; contract will appear here." data-sw="Ratiba ya mradi wako, hali ya nukuu na mkataba vitaonekana hapa.">Your project timeline, quotation status &amp; contract will appear here.</div>
    </div>
  </section>
</div>

<?php require __DIR__ . '/partials/footer.php'; ?>
