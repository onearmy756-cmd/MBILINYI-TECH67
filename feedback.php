<?php
require_once __DIR__ . '/config/app.php';
$pageTitle = 'Feedback — Maoni Yako | Mbilinyi Tech Solutions';
$page = 'legal';
require __DIR__ . '/partials/head.php';
?>
<body class="antialiased">
<?php require __DIR__ . '/partials/header.php'; ?>

<div id="view-landing" class="view-section active">
  <section class="max-w-5xl mx-auto px-4 pt-8 pb-16">

    <div class="glass rounded-3xl p-6 md:p-9 relative overflow-hidden">
      <div class="absolute -top-24 -right-24 w-72 h-72 rounded-full blur-3xl opacity-20" style="background:radial-gradient(circle,#a78bfa,transparent 70%)"></div>
      <div class="badge bg-violet-500/10 text-violet-300 border border-violet-400/30"><i class="fa-solid fa-comment-dots mr-1.5"></i>FEEDBACK • MAONI</div>
      <h1 class="font-display font-bold text-3xl md:text-4xl mt-4 leading-tight">
        <span data-en="Tell Us What You Think" data-sw="Tusiambie Unaonaje">Tell Us What You Think</span>
      </h1>
      <p class="text-white/65 text-[14.5px] leading-relaxed mt-4 max-w-3xl"
         data-en="Every message goes straight to CEO Jackson Mbilinyi. Praise, complaints, feature ideas — all of it is read and answered. This is how we keep improving."
         data-sw="Kila ujumbe unaenda moja kwa moja kwa CEO Jackson Mbilinyi. Sifa, malalamiko, mawazo ya kipengele — yote yanaguswa na kujibiwa. Ndivyo tunavyoendelea kuboresha.">Every message goes straight to CEO Jackson Mbilinyi. Praise, complaints, feature ideas — all of it is read and answered.</p>

      <div class="grid sm:grid-cols-3 gap-3 mt-6 text-[13px]">
        <div class="rounded-xl bg-white/[.03] border border-white/10 p-4">
          <div class="text-[10px] font-bold uppercase tracking-widest text-violet-300 mb-1"><i class="fa-solid fa-bolt mr-1.5"></i>Response time</div>
          <div data-en="Within 1 business day" data-sw="Ndani ya siku 1 ya kazi">Within 1 business day</div>
        </div>
        <div class="rounded-xl bg-white/[.03] border border-white/10 p-4">
          <div class="text-[10px] font-bold uppercase tracking-widest text-violet-300 mb-1"><i class="fa-solid fa-lock mr-1.5"></i>Privacy</div>
          <div data-en="Used only internally" data-sw="Inatumika ndani tu">Used only internally</div>
        </div>
        <div class="rounded-xl bg-white/[.03] border border-white/10 p-4">
          <div class="text-[10px] font-bold uppercase tracking-widest text-violet-300 mb-1"><i class="fa-solid fa-language mr-1.5"></i>Language</div>
          <div data-en="Kiswahili or English" data-sw="Kiswahili au Kiingereza">Kiswahili or English</div>
        </div>
      </div>
    </div>

    <div class="grid lg:grid-cols-[1.15fr_.85fr] gap-5 mt-5">

      <!-- FEEDBACK FORM -->
      <div class="glass rounded-3xl p-6 md:p-7">
        <div class="font-display font-bold text-xl"><span data-en="Send Feedback" data-sw="Tuma Maoni">Send Feedback</span></div>
        <p class="text-white/55 text-[13px] mt-1"><span data-en="Fields marked * are required." data-sw="Sehemu zilizo na * ni lazima.">Fields marked * are required.</span></p>

        <div class="grid sm:grid-cols-2 gap-3 mt-5">
          <input id="fbName" class="input" placeholder="Your name / Jina lako *">
          <input id="fbEmail" class="input" type="email" placeholder="Email / Barua pepe">
        </div>

        <input id="fbSubject" class="input mt-3" placeholder="Subject / Somo (optional)">

        <div class="grid sm:grid-cols-2 gap-3 mt-3">
          <select id="fbCategory" class="input">
            <option value="general">General feedback / Maoni ya jumla</option>
            <option value="praise">Praise / Sifa</option>
            <option value="complaint">Complaint / Malalamiko</option>
            <option value="feature">Feature idea / Wazo la kipengele</option>
            <option value="bug">Bug report / Ripoti ya hitilafu</option>
            <option value="service">Service quality / Ubora wa huduma</option>
            <option value="price">Pricing / Bei</option>
          </select>

          <div>
            <div class="flex items-center gap-2">
              <span class="text-[12px] font-bold text-white/60"><span data-en="Rating" data-sw="Alama">Rating</span></span>
              <div id="fbStars" class="flex gap-1"></div>
              <span id="fbRatingVal" class="text-[12px] text-cyan-300 font-mono">0/5</span>
            </div>
          </div>
        </div>

        <textarea id="fbMessage" rows="6" class="input mt-3" placeholder="Write your feedback here... / Andika maoni yako hapa... *"></textarea>

        <button onclick="submitFeedback()" id="fbBtn" class="grad-btn w-full mt-4 px-6 py-3.5 rounded-xl font-bold">
          <i class="fa-solid fa-paper-plane mr-2"></i><span data-en="Send Feedback" data-sw="Tuma Maoni">Send Feedback</span>
        </button>

        <div id="fbResult" class="hidden mt-4 rounded-xl p-4 text-[13.5px]"></div>
      </div>

      <!-- SIDE -->
      <div class="grid gap-5">
        <div class="glass rounded-3xl p-6">
          <div class="text-[11px] font-bold tracking-[.2em] text-cyan-300 mb-4"><i class="fa-solid fa-phone mr-2"></i><span data-en="OTHER WAYS TO REACH US" data-sw="NJIA ZA KUFIKIWA">OTHER WAYS TO REACH US</span></div>
          <div class="grid gap-2.5 text-[13.5px]">
            <a href="tel:+255796752645" class="flex items-center gap-3 p-3.5 rounded-xl bg-white/[.03] border border-white/10 hover:border-green-400/40">
              <div class="w-10 h-10 rounded-xl bg-green-500/15 flex items-center justify-center shrink-0"><i class="fa-solid fa-phone text-green-300"></i></div>
              <div><div class="font-bold">0796 752 645</div><div class="text-white/50 text-[12px]">Call • WhatsApp • SMS</div></div>
            </a>
            <a href="mailto:<?php echo htmlspecialchars(setting('company_email', 'info@mbilinyitech.co.tz'), ENT_QUOTES); ?>" class="flex items-center gap-3 p-3.5 rounded-xl bg-white/[.03] border border-white/10 hover:border-blue-400/40">
              <div class="w-10 h-10 rounded-xl bg-blue-500/15 flex items-center justify-center shrink-0"><i class="fa-solid fa-envelope text-blue-300"></i></div>
              <div><div class="font-bold break-all"><?php echo htmlspecialchars(setting('company_email', 'info@mbilinyitech.co.tz'), ENT_QUOTES); ?></div><div class="text-white/50 text-[12px]">Replies within 4 business hours</div></div>
            </a>
            <a href="contact" class="flex items-center gap-3 p-3.5 rounded-xl bg-white/[.03] border border-white/10 hover:border-violet-400/40">
              <div class="w-10 h-10 rounded-xl bg-violet-500/15 flex items-center justify-center shrink-0"><i class="fa-solid fa-ticket text-violet-300"></i></div>
              <div><div class="font-bold"><span data-en="Open a support ticket" data-sw="Fungua tiketi ya msaada">Open a support ticket</span></div><div class="text-white/50 text-[12px]">Tracked in your portal</div></div>
            </a>
          </div>
        </div>

        <div class="glass rounded-3xl p-6">
          <div class="text-[11px] font-bold tracking-[.2em] text-emerald-300 mb-4"><i class="fa-solid fa-shield-heart mr-2"></i><span data-en="YOUR DATA" data-sw="DATA YAKO">YOUR DATA</span></div>
          <p class="text-[13.5px] text-white/65 leading-relaxed"
             data-en="Feedback is used only to improve our service. It is never sold or shared for marketing. Read the Privacy Policy for full detail."
             data-sw="Maoni yanatumika kuboresha huduma yetu tu. Hayauzwi wala kushirikishwa kwa matangazo. Soma Sera ya Faragha kwa maelezo kamili.">Feedback is used only to improve our service.</p>
          <a href="privacy" class="inline-flex items-center gap-2 mt-3 text-[13px] font-bold text-cyan-300 hover:text-white transition">
            <span data-en="Read our Privacy Policy" data-sw="Soma Sera yetu ya Faragha">Read our Privacy Policy</span> <i class="fa-solid fa-arrow-right text-xs"></i>
          </a>
        </div>

        <div class="glass rounded-3xl p-6">
          <div class="text-[11px] font-bold tracking-[.2em] text-gold mb-3"><i class="fa-solid fa-award mr-2"></i><span data-en="REGISTERED BUSINESS" data-sw="BIASHARA IMESAJILIWA">REGISTERED BUSINESS</span></div>
          <div class="grid gap-1.5 text-[13px] text-white/65">
            <div class="flex justify-between gap-3"><span>BRELA Reg</span><strong class="font-mono text-emerald-300">Z-418822-77-TZ</strong></div>
            <div class="flex justify-between gap-3"><span>TIN</span><strong class="font-mono text-emerald-300">192-147-522</strong></div>
            <div class="flex justify-between gap-3"><span data-en="Based in" data-sw="Makao makuu">Based in</span><strong><?php echo htmlspecialchars(setting('company_location', 'Dodoma, Tanzania'), ENT_QUOTES); ?></strong></div>
          </div>
        </div>
      </div>

    </div>
  </section>
</div>

<?php require __DIR__ . '/partials/footer.php'; ?>
