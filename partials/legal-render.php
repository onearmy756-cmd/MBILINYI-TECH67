<?php
declare(strict_types=1);

/**
 * Shared renderer for the legal / policy documents.
 *
 * Everything is emitted with data-en / data-sw attributes so the header language
 * toggle swaps the whole document client-side. The toggle uses textContent, so
 * the attributes must hold PLAIN TEXT only — no tags, no HTML entities.
 *
 * Usage:
 *   require __DIR__ . '/partials/legal-render.php';
 *   mts_legal_page([
 *     'badge'      => 'REFUND POLICY',
 *     'badgeColor' => 'rose',
 *     'titleEn'    => 'Refund Policy',
 *     'titleSw'    => 'Sera ya Kurudisha Pesa',
 *     'introEn'    => '...',
 *     'introSw'    => '...',
 *     'sections'   => [
 *       ['hEn' => '...', 'hSw' => '...', 'p' => [['en' => '...', 'sw' => '...']]],
 *     ],
 *   ]);
 */
function mts_e(string $s): string {
    /* plain-text safe for an HTML attribute (attributes are double-quoted) */
    return htmlspecialchars($s, ENT_QUOTES | ENT_SUBSTITUTE, 'UTF-8');
}

function mts_legal_page(array $doc): void {
    $badge      = $doc['badge'] ?? 'LEGAL';
    $badgeColor = $doc['badgeColor'] ?? 'cyan';
    $titleEn    = $doc['titleEn'] ?? '';
    $titleSw    = $doc['titleSw'] ?? '';
    $introEn    = $doc['introEn'] ?? '';
    $introSw    = $doc['introSw'] ?? '';
    $sections   = $doc['sections'] ?? [];
    $effective  = $doc['effective'] ?? ('1 January ' . date('Y'));
    $updated    = $doc['updated'] ?? date('d F Y');
    $tone       = match ($badgeColor) {
        'rose'   => ['bg-rose-500/10', 'text-rose-300', 'border-rose-400/30', 'text-rose-200'],
        'emerald'=> ['bg-emerald-500/10', 'text-emerald-300', 'border-emerald-400/30', 'text-emerald-200'],
        'violet' => ['bg-violet-500/10', 'text-violet-300', 'border-violet-400/30', 'text-violet-200'],
        default  => ['bg-cyan-500/10', 'text-cyan-300', 'border-cyan-400/30', 'text-cyan-200'],
    };
    ?>
    <div id="view-landing" class="view-section active">
      <section class="max-w-5xl mx-auto px-4 pt-8 pb-16">

        <!-- document header -->
        <div class="glass rounded-3xl p-6 md:p-9 relative overflow-hidden">
          <div class="absolute -top-24 -right-24 w-72 h-72 rounded-full blur-3xl opacity-20" style="background:radial-gradient(circle,#22d3ee,transparent 70%)"></div>
          <div class="badge <?php echo $tone[0]; ?> <?php echo $tone[1]; ?> border <?php echo $tone[2]; ?>">
            <i class="fa-solid fa-file-shield mr-1.5"></i><?php echo mts_e($badge); ?>
          </div>
          <h1 class="font-display font-bold text-3xl md:text-4xl mt-4 leading-tight">
            <span data-en="<?php echo mts_e($titleEn); ?>" data-sw="<?php echo mts_e($titleSw); ?>"><?php echo mts_e($titleEn); ?></span>
          </h1>
          <p class="text-white/65 text-[14.5px] leading-relaxed mt-4 max-w-3xl"
             data-en="<?php echo mts_e($introEn); ?>" data-sw="<?php echo mts_e($introSw); ?>"><?php echo mts_e($introEn); ?></p>

          <div class="grid sm:grid-cols-3 gap-3 mt-6 text-[12.5px]">
            <div class="rounded-xl bg-white/[.03] border border-white/10 p-3.5">
              <div class="text-[10px] font-bold uppercase tracking-widest text-cyan-300 mb-1">Effective / Kuanzia</div>
              <div class="font-semibold"><?php echo mts_e($effective); ?></div>
            </div>
            <div class="rounded-xl bg-white/[.03] border border-white/10 p-3.5">
              <div class="text-[10px] font-bold uppercase tracking-widest text-cyan-300 mb-1">Last updated / Imesasishwa</div>
              <div class="font-semibold"><?php echo mts_e($updated); ?></div>
            </div>
            <div class="rounded-xl bg-white/[.03] border border-white/10 p-3.5">
              <div class="text-[10px] font-bold uppercase tracking-widest text-cyan-300 mb-1">Jurisdiction / Sheria</div>
              <div class="font-semibold" data-en="United Republic of Tanzania" data-sw="Jamhuri ya Muungano wa Tanzania">United Republic of Tanzania</div>
            </div>
          </div>
        </div>

        <!-- contents -->
        <?php if (count($sections) > 1): ?>
        <div class="glass rounded-3xl p-6 md:p-7 mt-5">
          <div class="text-[11px] font-bold tracking-[.2em] text-cyan-300 mb-4">
            <i class="fa-solid fa-list-ol mr-2"></i><span data-en="CONTENTS" data-sw="YALIYOMO">CONTENTS</span>
          </div>
          <div class="grid sm:grid-cols-2 gap-2 text-[13.5px]">
            <?php foreach ($sections as $i => $s): ?>
              <a href="#<?php echo mts_e('sec' . $i); ?>" class="flex gap-2.5 rounded-xl bg-white/[.02] border border-white/10 px-3.5 py-2.5 hover:border-cyan-400/40 transition">
                <span class="font-mono text-cyan-300 shrink-0"><?php echo (string)($i + 1); ?>.</span>
                <span class="text-white/70" data-en="<?php echo mts_e($s['hEn'] ?? ''); ?>" data-sw="<?php echo mts_e($s['hSw'] ?? $s['hEn'] ?? ''); ?>"><?php echo mts_e($s['hEn'] ?? ''); ?></span>
              </a>
            <?php endforeach; ?>
          </div>
        </div>
        <?php endif; ?>

        <!-- sections -->
        <div class="grid gap-4 mt-5">
          <?php foreach ($sections as $i => $s): ?>
          <div id="<?php echo mts_e('sec' . $i); ?>" class="glass rounded-3xl p-6 md:p-7 scroll-mt-24">
            <div class="flex items-start gap-3">
              <span class="w-9 h-9 rounded-xl grad-btn flex items-center justify-center font-bold text-sm shrink-0"><?php echo (string)($i + 1); ?></span>
              <div>
                <h2 class="font-display font-bold text-xl leading-snug"
                    data-en="<?php echo mts_e($s['hEn'] ?? ''); ?>" data-sw="<?php echo mts_e($s['hSw'] ?? $s['hEn'] ?? ''); ?>"><?php echo mts_e($s['hEn'] ?? ''); ?></h2>
              </div>
            </div>
            <div class="grid gap-3 mt-4 text-[14px] leading-relaxed text-white/70">
              <?php foreach (($s['p'] ?? []) as $p): ?>
                <p data-en="<?php echo mts_e($p['en'] ?? ''); ?>" data-sw="<?php echo mts_e($p['sw'] ?? $p['en'] ?? ''); ?>"><?php echo mts_e($p['en'] ?? ''); ?></p>
              <?php endforeach; ?>
              <?php foreach (($s['ul'] ?? []) as $li): ?>
                <div class="flex gap-2.5 items-start">
                  <i class="fa-solid fa-check text-emerald-300 mt-1.5 text-xs shrink-0"></i>
                  <span data-en="<?php echo mts_e($li['en'] ?? ''); ?>" data-sw="<?php echo mts_e($li['sw'] ?? $li['en'] ?? ''); ?>"><?php echo mts_e($li['en'] ?? ''); ?></span>
                </div>
              <?php endforeach; ?>
            </div>
          </div>
          <?php endforeach; ?>
        </div>

        <!-- contact strip -->
        <div class="glass rounded-3xl p-6 md:p-7 mt-5 flex flex-wrap items-center gap-4 justify-between">
          <div>
            <div class="font-bold text-[15px]" data-en="Questions about this document?" data-sw="Una maswali kuhusu hati hii?">Questions about this document?</div>
            <div class="text-[13px] text-white/55" data-en="We reply within 4 business hours, Mon–Sat 08:00–20:00 EAT." data-sw="Tunajibu ndani ya saa 4 za kazi, Jumatatu–Jumamosi 08:00–20:00 EAT.">We reply within 4 business hours, Mon–Sat 08:00–20:00 EAT.</div>
          </div>
          <div class="flex flex-wrap gap-2.5">
            <a href="mailto:<?php echo mts_e(setting('company_email', 'info@mbilinyitech.co.tz')); ?>" class="px-5 py-3 rounded-xl font-bold text-sm grad-btn"><i class="fa-solid fa-envelope mr-2"></i><?php echo mts_e(setting('company_email', 'info@mbilinyitech.co.tz')); ?></a>
            <a href="https://wa.me/255796752645" target="_blank" class="px-5 py-3 rounded-xl font-bold text-sm bg-[#25D366]/15 border border-[#25D366]/40 text-[#4be584] hover:bg-[#25D366]/25"><i class="fa-brands fa-whatsapp mr-2"></i>WhatsApp</a>
            <a href="contact" class="px-5 py-3 rounded-xl font-bold text-sm border border-white/15 hover:bg-white/5"><i class="fa-solid fa-headset mr-2"></i><span data-en="Support" data-sw="Msaada">Support</span></a>
          </div>
        </div>

      </section>
    </div>
    <?php
}
