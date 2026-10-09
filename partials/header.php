<!-- TOASTS -->
<div id="toasts" class="fixed top-20 right-4 z-[9999] flex flex-col gap-3 w-[340px] max-w-[92vw]"></div>

<!-- TOP COMPLIANCE BAR -->
<div class="bg-gradient-to-r from-cyan-950 via-blue-950 to-violet-950 border-b border-white/10 text-[12px]">
  <div class="max-w-7xl mx-auto px-4 py-2 flex flex-wrap items-center justify-between gap-2">
    <div class="flex items-center gap-4 flex-wrap">
      <span class="flex items-center gap-1.5 text-emerald-300"><i class="fa-solid fa-shield-halved"></i> BRELA Registered <span class="text-white/50">•</span> <span class="font-mono">TIN 192-147-522</span></span>
      <span class="hidden md:inline text-white/40">|</span>
      <span class="hidden md:flex items-center gap-1.5 text-white/70"><i class="fa-solid fa-location-dot text-cyan-300"></i> <span data-en="Dodoma, Tanzania • Serving Worldwide Online" data-sw="Dodoma, Tanzania • Tunahudumia Dunia Mtandaoni">Dodoma, Tanzania • Serving Worldwide Online</span></span>
    </div>
    <div class="flex items-center gap-3">
      <button onclick="setLang('en')" id="langEnBtn" class="px-2.5 py-1 rounded-full bg-white/10 font-bold text-white">EN</button>
      <button onclick="setLang('sw')" id="langSwBtn" class="px-2.5 py-1 rounded-full text-white/60 font-bold hover:bg-white/10">SW</button>
      <span class="w-px h-4 bg-white/15"></span>
      <a href="tel:+255796752645" class="flex items-center gap-1.5 hover:text-cyan-300"><i class="fa-solid fa-phone text-cyan-300"></i> 0796 752 645</a>
      <a href="mailto:mbilinyitech@gmail.com" class="hidden sm:flex items-center gap-1.5 hover:text-cyan-300"><i class="fa-solid fa-envelope text-cyan-300"></i> mbilinyitech@gmail.com</a>
    </div>
  </div>
</div>

<!-- NAVBAR -->
<header class="sticky top-0 z-[100] glass-strong border-b border-white/10">
  <div class="max-w-7xl mx-auto px-4 py-3 flex items-center justify-between gap-3">
    <a href="/" class="flex items-center gap-3 text-left">
      <div class="w-11 h-11 rounded-2xl grad-btn flex items-center justify-center font-display font-bold text-xl relative overflow-hidden">M<span class="absolute inset-0 shimmer"></span></div>
      <div>
        <div class="font-display font-bold leading-none text-[16px]">MBILINYI <span class="grad-text">TECH</span> SOLUTIONS</div>
        <div class="text-[10.5px] tracking-[.22em] text-cyan-300/80 font-semibold">INNOVATE • SECURE • SCALE</div>
        <div class="text-[9.5px] tracking-[.18em] text-white/60 font-semibold mt-0.5" data-en="YOUR PROBLEM, OUR SOLUTION" data-sw="SHIDA YAKO, TATUZO LETU">YOUR PROBLEM, OUR SOLUTION</div>
      </div>
    </a>
    <nav class="hidden lg:flex items-center gap-6 text-[13.5px] font-medium text-white/70">
      <a href="/" class="hover:text-white" data-en="Home" data-sw="Mwanzo">Home</a>
      <a href="services" class="hover:text-white" data-en="Services" data-sw="Huduma">Services</a>
      <a href="stack" class="hover:text-white" data-en="Tech Stack" data-sw="Teknolojia">Tech Stack</a>
      <a href="ai" class="hover:text-white">AI & Fine-Tuning</a>
      <a href="pricing" class="hover:text-white" data-en="Pricing" data-sw="Bei">Pricing</a>
      <a href="work" class="hover:text-white" data-en="Work" data-sw="Kazi">Work</a>
      <a href="track" class="hover:text-white flex items-center gap-1"><i class="fa-solid fa-magnifying-glass text-cyan-300 text-xs"></i> <span data-en="Track Order" data-sw="Fuatilia Oda">Track Order</span></a>
      <a href="contact" class="hover:text-white" data-en="Contact" data-sw="Mawasiliano">Contact</a>
    </nav>
    <div class="flex items-center gap-2">
      <button id="navAuthBtn" onclick="location.href='auth'" class="hidden sm:flex items-center gap-2 px-4 py-2.5 rounded-xl border border-white/15 hover:border-cyan-400/50 hover:bg-white/5 text-sm font-semibold"><i class="fa-solid fa-right-to-bracket text-cyan-300"></i> <span data-en="Sign In" data-sw="Ingia">Sign In</span></button>
      <button onclick="openQuoteWizard()" class="grad-btn px-4 sm:px-5 py-2.5 rounded-xl text-sm font-bold flex items-center gap-2"><i class="fa-solid fa-file-pen"></i> <span data-en="Request Service" data-sw="Omba Huduma">Request Service</span></button>
      <button onclick="toggleMobileMenu()" class="lg:hidden w-10 h-10 rounded-xl border border-white/15 flex items-center justify-center"><i class="fa-solid fa-bars"></i></button>
    </div>
  </div>
  <div id="mobileMenu" class="hidden lg:hidden border-t border-white/10 px-4 py-4 grid gap-2 text-sm bg-[#0A1226]">
    <a href="/" onclick="toggleMobileMenu()" class="p-3 rounded-xl hover:bg-white/5">Home / Mwanzo</a>
    <a href="services" onclick="toggleMobileMenu()" class="p-3 rounded-xl hover:bg-white/5">Services / Huduma</a>
    <a href="stack" onclick="toggleMobileMenu()" class="p-3 rounded-xl hover:bg-white/5">Tech Stack</a>
    <a href="ai" onclick="toggleMobileMenu()" class="p-3 rounded-xl hover:bg-white/5">AI & Fine-Tuning</a>
    <a href="pricing" onclick="toggleMobileMenu()" class="p-3 rounded-xl hover:bg-white/5">Pricing / Bei</a>
    <a href="work" onclick="toggleMobileMenu()" class="p-3 rounded-xl hover:bg-white/5">Work / Kazi</a>
    <a href="process" onclick="toggleMobileMenu()" class="p-3 rounded-xl hover:bg-white/5">How It Works</a>
    <a href="track" onclick="toggleMobileMenu()" class="p-3 rounded-xl hover:bg-white/5">Track Order</a>
    <a href="contact" onclick="toggleMobileMenu()" class="p-3 rounded-xl hover:bg-white/5">Contact</a>
    <button onclick="location.href='auth'" class="p-3 rounded-xl bg-white/5 text-left"><i class="fa-solid fa-right-to-bracket mr-2 text-cyan-300"></i> Sign In / Client & Admin Portal</button>
  </div>
</header>

<main>
