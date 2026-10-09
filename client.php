<?php
require_once __DIR__ . '/config/app.php';
$client = client_page_user();
$pageTitle = 'Mbilinyi Tech Solutions — Client Portal';
$page = 'client';
require __DIR__ . '/partials/head.php';
?>
<body class="antialiased">
<?php require __DIR__ . '/partials/header.php'; ?>

<!-- ============ CLIENT VIEW ============ -->
<div id="view-client" class="view-section active">
  <div class="max-w-7xl mx-auto px-4 py-8 grid lg:grid-cols-[260px_1fr] gap-5">
    <aside class="glass rounded-3xl p-4 h-fit lg:sticky lg:top-24">
      <div class="flex items-center gap-3 p-3 rounded-2xl bg-white/[.03] border border-white/10">
        <div id="clientAvatar" class="w-12 h-12 rounded-2xl grad-btn flex items-center justify-center font-bold text-lg">C</div>
        <div class="min-w-0"><div id="clientName" class="font-bold text-[14px] truncate">Client</div><div id="clientEmail" class="text-[12px] text-white/50 truncate">—</div><span class="badge bg-cyan-500/10 text-cyan-300 border border-cyan-400/30 mt-1 inline-block">CLIENT</span></div>
      </div>
      <nav class="mt-4 grid gap-1 text-[13.5px] font-semibold" id="clientNav">
        <button data-ctab="overview" onclick="clientTab('overview')" class="sidebar-link active text-left px-4 py-3 rounded-xl text-white/70 hover:bg-white/5"><i class="fa-solid fa-grid-2 mr-2 text-cyan-300"></i>Overview</button>
        <button data-ctab="requests" onclick="clientTab('requests')" class="sidebar-link text-left px-4 py-3 rounded-xl text-white/70 hover:bg-white/5"><i class="fa-solid fa-folder-open mr-2 text-violet-300"></i>My Requests</button>
        <button data-ctab="quotes" onclick="clientTab('quotes')" class="sidebar-link text-left px-4 py-3 rounded-xl text-white/70 hover:bg-white/5"><i class="fa-solid fa-file-invoice-dollar mr-2 text-gold"></i>Quotations</button>
        <button data-ctab="contracts" onclick="clientTab('contracts')" class="sidebar-link text-left px-4 py-3 rounded-xl text-white/70 hover:bg-white/5"><i class="fa-solid fa-file-contract mr-2 text-emerald-300"></i>Contracts</button>
        <button data-ctab="invoices" onclick="clientTab('invoices')" class="sidebar-link text-left px-4 py-3 rounded-xl text-white/70 hover:bg-white/5"><i class="fa-solid fa-receipt mr-2 text-blue-300"></i>Invoices & Pay</button>
        <button data-ctab="support" onclick="clientTab('support')" class="sidebar-link text-left px-4 py-3 rounded-xl text-white/70 hover:bg-white/5"><i class="fa-solid fa-headset mr-2 text-pink-300"></i>Support</button>
      </nav>
      <button onclick="openQuoteWizard()" class="grad-btn w-full mt-4 py-3 rounded-xl font-bold text-sm"><i class="fa-solid fa-plus mr-1"></i> New Request</button>
      <button onclick="logout()" class="w-full mt-2 py-3 rounded-xl font-bold text-sm border border-red-400/30 text-red-300 hover:bg-red-500/10"><i class="fa-solid fa-right-from-bracket mr-1"></i> Logout</button>
    </aside>
    <div class="min-w-0">
      <div class="glass rounded-3xl p-5 md:p-7">
        <div class="flex flex-wrap items-center justify-between gap-3">
          <div><div class="text-[11px] font-bold tracking-[.2em] text-cyan-300">CLIENT PORTAL</div><h2 id="clientTabTitle" class="font-display font-bold text-2xl">Overview</h2></div>
          <div class="flex gap-2"><span class="badge bg-emerald-500/15 text-emerald-300 border border-emerald-400/30"><i class="fa-solid fa-circle-check mr-1"></i> BRELA Partner</span><button onclick="showView('landing')" class="text-[12px] px-4 py-2 rounded-full border border-white/15 hover:bg-white/5">← Website</button></div>
        </div>
        <div id="clientContent" class="mt-6"></div>
      </div>
    </div>
  </div>
</div>

<?php require __DIR__ . '/partials/footer.php'; ?>
