<?php
require_once __DIR__ . '/config/app.php';
$pageTitle = 'Community — Builders, Learners & Partners | Mbilinyi Tech Solutions';
$page = 'landing';
require __DIR__ . '/partials/head.php';
?>
<body class="antialiased">
<?php require __DIR__ . '/partials/header.php'; ?>

<!-- ============ COMMUNITY PAGE ============ -->
<div id="view-landing" class="view-section active">
  <section id="community" class="max-w-7xl mx-auto px-4 pt-8 pb-16">
    <div class="text-center">
      <div class="badge bg-violet-500/10 text-violet-300 border border-violet-400/30 inline-block"><span data-en="COMMUNITY" data-sw="JAMII">COMMUNITY</span></div>
      <h2 class="font-display font-bold text-3xl md:text-4xl mt-3"><span data-en="Builders, Learners &amp;" data-sw="Wajenzi, Wanafunzi na">Builders, Learners &amp;</span> <span class="grad-text" data-en="Partners" data-sw="Washirika">Partners</span></h2>
      <p class="text-white/55 mt-3 text-[14px] max-w-2xl mx-auto" data-en="Mbilinyi Tech Solutions grows with the people around it — students learning to code, engineers trading skills, and businesses sharing what they built. Join in." data-sw="Mbilinyi Tech Solutions hukua na watu walio karibu nayo — wanafunzi wanaojifunza kuandika programu, wahandisi wakibadilishana ujuzi, na biashara zikishirikiana kile walichojenga. Jiunge nasi.">Mbilinyi Tech Solutions grows with the people around it — students learning to code, engineers trading skills, and businesses sharing what they built. Join in.</p>
    </div>

    <div class="grid md:grid-cols-3 gap-4 mt-10">
      <div class="card3d pop-violet p-6">
        <div class="w-12 h-12 rounded-2xl bg-violet-500/20 border border-violet-400/30 flex items-center justify-center text-xl"><i class="fa-solid fa-graduation-cap text-violet-300"></i></div>
        <div class="font-display font-bold text-lg mt-3" data-en="Learn With Us" data-sw="Jifunze Nasi">Learn With Us</div>
        <p class="text-[13px] text-white/60 mt-2 leading-relaxed" data-en="Free tutorials in Kiswahili and English on web development, AI, cyber security and the tools we use daily. Ask a question and an engineer answers in the open forum." data-sw="Mafunzo ya bure kwa Kiswahili na Kiingereza kuhusu uundaji wa web, AI, usalama wa mtandao na zana tunazotumia kila siku. Uliza swali na mhandisi hujibu kwenye jukwaa la wazi.">Free tutorials in Kiswahili and English on web development, AI, cyber security and the tools we use daily. Ask a question and an engineer answers in the open forum.</p>
      </div>
      <div class="card3d pop-cyan p-6">
        <div class="w-12 h-12 rounded-2xl bg-cyan-500/20 border border-cyan-400/30 flex items-center justify-center text-xl"><i class="fa-solid fa-people-group text-cyan-300"></i></div>
        <div class="font-display font-bold text-lg mt-3" data-en="Work With Us" data-sw="Fanya Kazi Nasi">Work With Us</div>
        <p class="text-[13px] text-white/60 mt-2 leading-relaxed" data-en="We hire engineers, designers and analysts, and every applicant is assessed practically and theoretically before a role is offered. See the open roles and apply from the careers page." data-sw="Tunaajiri wahandisi, wabunifu na wachambuzi, na kila mwombaji hupimwa kivitendo na kinadharia kabla ya nafasi kutolewa. Angalia nafasi zilizo wazi na omba kwenye ukurasa wa ajira.">We hire engineers, designers and analysts, and every applicant is assessed practically and theoretically before a role is offered. See the open roles and apply from the careers page.</p>
        <a href="/careers" class="inline-block mt-3 text-cyan-300 font-bold text-[13px] hover:underline" data-en="See open roles →" data-sw="Angalia nafasi →">See open roles →</a>
      </div>
      <div class="card3d pop-emerald p-6">
        <div class="w-12 h-12 rounded-2xl bg-emerald-500/20 border border-emerald-400/30 flex items-center justify-center text-xl"><i class="fa-solid fa-handshake text-emerald-300"></i></div>
        <div class="font-display font-bold text-lg mt-3" data-en="Partner With Us" data-sw="Shirikiana Nasi">Partner With Us</div>
        <p class="text-[13px] text-white/60 mt-2 leading-relaxed" data-en="Agencies, schools and businesses that need technology delivered at scale can partner with us. Bring the client, we bring the engineers, the AI and the security." data-sw="Wakala, shule na biashara zinazohitaji teknolojia kwa kiwango kikubwa zinaweza kushirikiana nasi. Lete mteja, sisi tunaleta wahandisi, AI na usalama.">Agencies, schools and businesses that need technology delivered at scale can partner with us. Bring the client, we bring the engineers, the AI and the security.</p>
      </div>
    </div>

    <div class="mt-10 grid lg:grid-cols-2 gap-4">
      <div class="glass rounded-3xl p-6">
        <div class="font-display font-bold text-lg mb-3"><i class="fa-solid fa-comments text-violet-300 mr-2"></i><span data-en="Open Forum" data-sw="Jukwaa Huria">Open Forum</span></div>
        <p class="text-[13px] text-white/60" data-en="Ask technical questions and get answers from our engineers and other builders. The best questions become tutorials." data-sw="Uliza maswali ya kiufundi na upate majibu kutoka kwa wahandisi wetu na wajenzi wengine. Maswali bora huwa mafunzo.">Ask technical questions and get answers from our engineers and other builders. The best questions become tutorials.</p>
        <a href="/contact" class="inline-block mt-4 px-5 py-2.5 rounded-xl border border-violet-400/40 text-violet-200 font-bold text-[13px] hover:bg-violet-500/10" data-en="Ask a Question" data-sw="Uliza Swali">Ask a Question</a>
      </div>
      <div class="glass rounded-3xl p-6">
        <div class="font-display font-bold text-lg mb-3"><i class="fa-solid fa-bullhorn text-amber-300 mr-2"></i><span data-en="Events &amp; Workshops" data-sw="Matukio na Warsha">Events &amp; Workshops</span></div>
        <p class="text-[13px] text-white/60" data-en="Hands-on sessions on AI fine-tuning, cyber security and shipping software, held online inside the platform and in person across Tanzania." data-sw="Vikao vya vitendo kuhusu urekebishaji wa AI, usalama wa mtandao na kuwasilisha programu, hufanyika mtandaoni ndani ya mfumo na ana kwa ana kote Tanzania.">Hands-on sessions on AI fine-tuning, cyber security and shipping software, held online inside the platform and in person across Tanzania.</p>
        <a href="/news" class="inline-block mt-4 px-5 py-2.5 rounded-xl border border-amber-400/40 text-amber-200 font-bold text-[13px] hover:bg-amber-500/10" data-en="See News &amp; Updates" data-sw="Angalia Habari">See News &amp; Updates</a>
      </div>
    </div>
  </section>
</div>

<?php require __DIR__ . '/partials/footer.php'; ?>
