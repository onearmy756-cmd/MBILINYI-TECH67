<?php
require_once __DIR__ . '/config/app.php';
$pageTitle = 'Our Experts — The People Who Serve You | Mbilinyi Tech Solutions';
$page = 'landing';
require __DIR__ . '/partials/head.php';
?>
<body class="antialiased">
<?php require __DIR__ . '/partials/header.php'; ?>

<!-- ============ STAFF / WATAALAMU WETU ============ -->
<section class="max-w-7xl mx-auto px-4 pt-10">
  <div class="glass rounded-3xl p-6 md:p-9 relative overflow-hidden">
    <div class="orb w-[340px] h-[340px] bg-emerald-500/25 -bottom-24 -left-16"></div>
    <div class="relative">
      <div class="text-[11px] font-bold tracking-[.24em] text-emerald-300">WATAALAMU WETU • OUR EXPERTS</div>
      <h1 class="font-display font-bold text-3xl md:text-5xl mt-2">The People Who Serve You</h1>
      <p class="text-white/60 mt-3 max-w-3xl leading-relaxed">
        Every service is delivered by a named expert with a recorded profession and level.
        When you send a request, it goes to the expert who serves that service — so you always know who is working for you,
        and you can watch the progress of your job inside the platform.
      </p>
      <div class="flex flex-wrap gap-2 mt-5">
        <span class="chip text-emerald-300"><i class="fa-solid fa-user-check"></i>A named expert per request</span>
        <span class="chip text-cyan-300"><i class="fa-solid fa-chart-line"></i>Visible progress</span>
        <span class="chip text-violet-300"><i class="fa-solid fa-shield-halved"></i>Signed contracts</span>
      </div>
    </div>
  </div>
</section>

<section class="max-w-7xl mx-auto px-4 mt-8 pb-6">
  <div class="flex flex-wrap items-center justify-between gap-3">
    <h2 class="font-display font-bold text-2xl"><i class="fa-solid fa-people-group text-emerald-300 mr-2"></i>Our Experts</h2>
    <div class="text-[13px] text-white/50"><span id="staffCount">0</span> expert(s)</div>
  </div>
  <div id="staffList" class="grid sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4 mt-5"></div>
</section>

<script>
(function () {
  const esc = s => String(s == null ? '' : s).replace(/[&<>"']/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#039;' }[c]));
  const lvlColor = { junior: 'text-sky-300', mid: 'text-cyan-300', senior: 'text-emerald-300', lead: 'text-amber-300', principal: 'text-fuchsia-300' };
  const lvlPop = { junior: 'pop-cyan', mid: 'pop-emerald', senior: 'pop-violet', lead: 'pop-amber', principal: 'pop-rose' };

  (async function loadStaff() {
    const box = document.getElementById('staffList');
    const cnt = document.getElementById('staffCount');
    let staff = [];
    try {
      const res = await fetch('api/wafanyakazi.php?action=public', { headers: { 'X-Requested-With': 'fetch' } });
      const j = await res.json();
      staff = (j.data && j.data.employees) || [];
    } catch (e) { staff = []; }
    cnt.textContent = staff.length;
    if (!staff.length) {
      box.innerHTML = '<div class="sm:col-span-2 lg:col-span-3 xl:col-span-4 glass rounded-3xl p-8 text-center text-white/50">Our expert roster is being assembled. Send a request and an expert is assigned to you.</div>';
      return;
    }
    box.innerHTML = staff.map(m => {
      const initials = (m.fullName || 'MTS').split(' ').map(w => w[0]).join('').slice(0, 2).toUpperCase();
      return `<div class="card3d ${lvlPop[m.level] || 'pop-cyan'} p-5 text-center">
        ${m.photo ? `<img src="${esc(m.photo)}" class="w-20 h-20 rounded-2xl object-cover border border-white/15 mx-auto">` : `<div class="w-20 h-20 rounded-2xl grad-btn flex items-center justify-center font-bold text-xl mx-auto">${esc(initials)}</div>`}
        <div class="font-bold text-[15px] mt-3">${esc(m.fullName)}</div>
        <div class="text-[12.5px] text-emerald-300 mt-0.5">${esc(m.profession)}</div>
        <div class="flex flex-wrap justify-center gap-1.5 mt-2">
          <span class="chip ${lvlColor[m.level] || 'text-white/60'}"><i class="fa-solid fa-layer-group"></i>${esc(m.level)}</span>
          <span class="chip text-white/55">${esc(m.department)}</span>
        </div>
        <div class="text-[11.5px] text-white/45 mt-2.5">${esc(m.region || 'Tanzania')} • ${Number(m.experienceYears) || 0} yrs${m.languages ? ' • ' + esc(m.languages) : ''}</div>
        ${(m.skills || []).length ? `<div class="flex flex-wrap justify-center gap-1 mt-2.5">${m.skills.slice(0, 4).map(s => `<span class="chip text-[10px] text-white/60">${esc(s)}</span>`).join('')}</div>` : ''}
      </div>`;
    }).join('');
  })();
})();
</script>

<?php require __DIR__ . '/partials/footer.php'; ?>
