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
    <div class="text-center"><div class="badge bg-blue-500/10 text-blue-300 border border-blue-400/30 inline-block"><span data-en="HOW IT WORKS" data-sw="INAVYOFANYA KAZI">HOW IT WORKS</span></div><h2 class="font-display font-bold text-3xl md:text-4xl mt-3"><span data-en="From Request to Contract" data-sw="Kutoka Ombi hadi Mkataba">From Request to Contract</span> <span class="grad-text" data-en="in 5 Steps" data-sw="kwa Hatua 5">in 5 Steps</span></h2></div>
    <div class="grid md:grid-cols-5 gap-3 mt-8">
      <div class="glass rounded-2xl p-5 text-center relative"><div class="w-11 h-11 mx-auto rounded-2xl grad-btn flex items-center justify-center font-bold">1</div><div class="font-bold mt-3 text-[14px]" data-en="Submit Requirements" data-sw="Tuma Mahitaji">Submit Requirements</div><div class="text-[12px] text-white/55 mt-1" data-en="Fill the smart form — specs, tech, budget, deadline." data-sw="Jaza fomu — mahitaji, teknolojia, bajeti na muda.">Fill the smart form — specs, tech, budget, deadline.</div></div>
      <div class="glass rounded-2xl p-5 text-center"><div class="w-11 h-11 mx-auto rounded-2xl bg-violet-600 flex items-center justify-center font-bold">2</div><div class="font-bold mt-3 text-[14px]" data-en="Review &amp; Price" data-sw="Uhakiki na Bei">Review &amp; Price</div><div class="text-[12px] text-white/55 mt-1" data-en="CEO reviews and issues an official quotation." data-sw="CEO hupitia na kutoa nukuu rasmi.">CEO reviews and issues an official quotation.</div></div>
      <div class="glass rounded-2xl p-5 text-center"><div class="w-11 h-11 mx-auto rounded-2xl bg-cyan-600 flex items-center justify-center font-bold">3</div><div class="font-bold mt-3 text-[14px]" data-en="Approve &amp; Contract" data-sw="Kubali na Mkataba">Approve &amp; Contract</div><div class="text-[12px] text-white/55 mt-1" data-en="You approve, then e-sign the BRELA contract." data-sw="Unakubali, kisha unasaini mkataba wa BRELA kidijitali.">You approve, then e-sign the BRELA contract.</div></div>
      <div class="glass rounded-2xl p-5 text-center"><div class="w-11 h-11 mx-auto rounded-2xl bg-emerald-600 flex items-center justify-center font-bold">4</div><div class="font-bold mt-3 text-[14px]" data-en="Build &amp; Track" data-sw="Ujenzi na Ufuatiliaji">Build &amp; Track</div><div class="text-[12px] text-white/55 mt-1" data-en="Live progress, milestones &amp; testing." data-sw="Maendeleo ya moja kwa moja, hatua na upimaji.">Live progress, milestones &amp; testing.</div></div>
      <div class="glass rounded-2xl p-5 text-center"><div class="w-11 h-11 mx-auto rounded-2xl gold-btn flex items-center justify-center font-bold">5</div><div class="font-bold mt-3 text-[14px]" data-en="Deploy &amp; Maintain" data-sw="Kupeleka na Matengenezo">Deploy &amp; Maintain</div><div class="text-[12px] text-white/55 mt-1" data-en="Launch, security hardening &amp; SLA support." data-sw="Kuzindua, kuimarisha usalama na msaada wa SLA.">Launch, security hardening &amp; SLA support.</div></div>
    </div>
    <div class="mt-8 max-w-3xl mx-auto glass rounded-3xl p-6 text-[13.5px] text-white/70">
      <div class="font-display font-bold text-lg text-white mb-2"><i class="fa-solid fa-circle-info text-cyan-300 mr-2"></i><span data-en="Payment Rule" data-sw="Kanuni ya Malipo">Payment Rule</span></div>
      <span data-en="A project starts officially once the contract is signed and 50% of the price is paid as a deposit. The remaining half is paid at mid-project, before final delivery. Your deposit invoice appears in your portal — without the deposit the project cannot start." data-sw="Mradi unaanza rasmi baada ya mkataba kusainiwa na 50% ya bei kulipwa kama amana. Nusu iliyobaki unalipia katikati ya mradi, kabla ya kukabidhi kazi. Invoice ya amana inaonekana kwenye portal yako — bila amana, mradi hauwezi kuanza.">A project starts officially once the contract is signed and 50% of the price is paid as a deposit. The remaining half is paid at mid-project, before final delivery. Your deposit invoice appears in your portal — without the deposit the project cannot start.</span>
    </div>
  </section>
</div>

<?php require __DIR__ . '/partials/footer.php'; ?>
