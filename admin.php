<?php
require_once __DIR__ . '/config/app.php';
$admin = admin_page_user();
$pageTitle = 'Mbilinyi Tech Solutions — Admin Control Center';
$page = 'admin';
require __DIR__ . '/partials/head.php';
?>
<body class="antialiased">
<?php require __DIR__ . '/partials/header.php'; ?>

<!-- ============ ADMIN VIEW ============ -->
<div id="view-admin" class="view-section active">
  <div class="max-w-[1400px] mx-auto px-4 py-8 grid lg:grid-cols-[270px_1fr] gap-5">
    <aside class="glass rounded-3xl p-4 h-fit lg:sticky lg:top-24">
      <div class="flex items-center gap-3 p-3 rounded-2xl bg-gradient-to-r from-red-500/15 to-violet-500/15 border border-red-400/20">
        <div class="w-12 h-12 rounded-2xl bg-gradient-to-br from-red-500 to-violet-600 flex items-center justify-center font-bold text-lg">JM</div>
        <div><div class="font-bold text-[14px]">Jackson Mbilinyi</div><div class="text-[12px] text-white/50">CEO • Administrator</div><span class="badge bg-red-500/15 text-red-300 border border-red-400/30 mt-1 inline-block">ADMIN</span></div>
      </div>
      <nav class="mt-4 grid gap-1 text-[13.5px] font-semibold" id="adminNav">
        <button data-atab="overview" onclick="adminTab('overview')" class="sidebar-link active text-left px-4 py-3 rounded-xl text-white/70 hover:bg-white/5"><i class="fa-solid fa-chart-line mr-2 text-cyan-300"></i>Dashboard</button>
        <button data-atab="requests" onclick="adminTab('requests')" class="sidebar-link text-left px-4 py-3 rounded-xl text-white/70 hover:bg-white/5"><i class="fa-solid fa-inbox mr-2 text-violet-300"></i>Requests <span id="adminReqCount" class="ml-auto text-[11px] bg-red-500 text-white px-2 py-0.5 rounded-full font-bold">0</span></button>
        <button data-atab="quotes" onclick="adminTab('quotes')" class="sidebar-link text-left px-4 py-3 rounded-xl text-white/70 hover:bg-white/5"><i class="fa-solid fa-file-invoice-dollar mr-2 text-gold"></i>Quotations & Pricing</button>
        <button data-atab="contracts" onclick="adminTab('contracts')" class="sidebar-link text-left px-4 py-3 rounded-xl text-white/70 hover:bg-white/5"><i class="fa-solid fa-file-contract mr-2 text-emerald-300"></i>Contracts</button>
        <button data-atab="portfolio" onclick="adminTab('portfolio')" class="sidebar-link text-left px-4 py-3 rounded-xl text-white/70 hover:bg-white/5"><i class="fa-solid fa-briefcase mr-2 text-fuchsia-300"></i>Portfolio Projects</button>
        <button data-atab="testimonials" onclick="adminTab('testimonials')" class="sidebar-link text-left px-4 py-3 rounded-xl text-white/70 hover:bg-white/5"><i class="fa-solid fa-comments mr-2 text-emerald-300"></i>Testimonials</button>
        <button data-atab="users" onclick="adminTab('users')" class="sidebar-link text-left px-4 py-3 rounded-xl text-white/70 hover:bg-white/5"><i class="fa-solid fa-users-gear mr-2 text-blue-300"></i>Manage Accounts</button>
        <button data-atab="invoices" onclick="adminTab('invoices')" class="sidebar-link text-left px-4 py-3 rounded-xl text-white/70 hover:bg-white/5"><i class="fa-solid fa-receipt mr-2 text-pink-300"></i>Invoices</button>
        <button data-atab="tickets" onclick="adminTab('tickets')" class="sidebar-link text-left px-4 py-3 rounded-xl text-white/70 hover:bg-white/5"><i class="fa-solid fa-headset mr-2 text-orange-300"></i>Support Tickets</button>
        <button data-atab="feedback" onclick="adminTab('feedback')" class="sidebar-link text-left px-4 py-3 rounded-xl text-white/70 hover:bg-white/5"><i class="fa-solid fa-comment-dots mr-2 text-violet-300"></i>Feedback</button>
        <button data-atab="settings" onclick="adminTab('settings')" class="sidebar-link text-left px-4 py-3 rounded-xl text-white/70 hover:bg-white/5"><i class="fa-solid fa-sliders mr-2 text-cyan-300"></i>Settings & Prices</button>
        <button data-atab="email" onclick="adminTab('email')" class="sidebar-link text-left px-4 py-3 rounded-xl text-white/70 hover:bg-white/5"><i class="fa-solid fa-envelope-gear mr-2 text-teal-300"></i>Email Setup</button>
      </nav>
      <button onclick="logout()" class="w-full mt-4 py-3 rounded-xl font-bold text-sm border border-red-400/30 text-red-300 hover:bg-red-500/10"><i class="fa-solid fa-right-from-bracket mr-1"></i> Logout Admin</button>
    </aside>
    <div class="min-w-0">
      <div class="glass rounded-3xl p-5 md:p-7">
        <div class="flex flex-wrap items-center justify-between gap-3">
          <div><div class="text-[11px] font-bold tracking-[.2em] text-red-300">ADMIN CONTROL CENTER • CEO ONLY</div><h2 id="adminTabTitle" class="font-display font-bold text-2xl">Dashboard</h2></div>
          <div class="flex gap-2"><button onclick="adminTab('requests')" class="grad-btn px-4 py-2.5 rounded-xl text-[13px] font-bold"><i class="fa-solid fa-plus mr-1"></i> New Quotation</button><button onclick="showView('landing')" class="text-[12px] px-4 py-2 rounded-full border border-white/15 hover:bg-white/5">← Website</button></div>
        </div>
        <div id="adminContent" class="mt-6"></div>
      </div>
    </div>
  </div>
</div>

<?php require __DIR__ . '/partials/footer.php'; ?>
