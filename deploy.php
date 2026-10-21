<?php
/**
 * /deploy — deployment health check.
 *
 * Answers one question: "is this clone ready to serve?" It shows the PHP
 * version, which database answered, whether the schema is present, and which
 * extensions are missing. It never prints a password, key or connection
 * string — only the driver name and host class.
 */
declare(strict_types=1);
require_once __DIR__ . '/config/app.php';

$checks = [];
$add = function (string $label, bool $ok, string $detail, string $hint = '', bool $optional = false) use (&$checks): void {
    $checks[] = ['label' => $label, 'ok' => $ok, 'detail' => $detail, 'hint' => $hint, 'optional' => $optional];
};

/* ---- PHP ---- */
$phpOk = version_compare(PHP_VERSION, '8.1.0', '>=');
$add('PHP version', $phpOk, PHP_VERSION, $phpOk ? '' : 'PHP 8.1 or newer is required.');

/* ---- extensions ---- */
foreach (['pdo' => 'Database access', 'json' => 'API responses', 'mbstring' => 'Swahili/UTF-8 text', 'session' => 'Login sessions'] as $ext => $why) {
    $add('Extension: ' . $ext, extension_loaded($ext), extension_loaded($ext) ? 'loaded' : 'missing', extension_loaded($ext) ? '' : $why . ' will not work without it.');
}
$hasSqlite = extension_loaded('pdo_sqlite');
$hasMysql = extension_loaded('pdo_mysql');
$add(
    'A database driver',
    $hasSqlite || $hasMysql,
    trim(($hasSqlite ? 'pdo_sqlite ' : '') . ($hasMysql ? 'pdo_mysql' : '')) ?: 'none',
    ($hasSqlite || $hasMysql) ? '' : 'Enable pdo_sqlite or pdo_mysql in the hosting panel.'
);

/* ---- database ---- */
$driver = 'unavailable';
$dbOk = false;
$dbDetail = 'could not connect';
$tables = 0;
try {
    $pdo = db();
    $driver = (string)$pdo->getAttribute(PDO::ATTR_DRIVER_NAME);
    $dbOk = true;
    $tables = (int)$pdo->query("SELECT count(*) FROM sqlite_master WHERE type = 'table'")->fetchColumn();
    if ($driver !== 'sqlite') {
        $tables = (int)$pdo->query('SELECT count(*) FROM information_schema.tables WHERE table_schema = DATABASE()')->fetchColumn();
    }
    $dbDetail = $driver . ' — ' . $tables . ' tables';
} catch (Throwable $e) {
    $dbDetail = $e->getMessage();
}

$add('Database', $dbOk, $dbDetail, $dbOk ? '' : 'Check DATABASE_URL in .env, or let the app fall back to SQLite.');
if ($dbOk) {
    $add(
        'Database engine in use',
        true,
        $driver === 'sqlite' ? 'SQLite (file in storage/)' : 'MySQL / MariaDB',
        $driver === 'sqlite' ? 'Fine for a small site. For heavy traffic, set DATABASE_URL to MySQL.' : ''
    );
}

/* ---- writable folders ---- */
foreach (['storage' => 'sessions' . ($driver === 'sqlite' ? ' and the database' : ''), 'storage/sessions' => 'login sessions'] as $dir => $why) {
    $path = APP_ROOT . '/' . $dir;
    if (!is_dir($path)) @mkdir($path, 0775, true);
    $add('Writable: ' . $dir, is_writable($path), is_writable($path) ? 'writable' : 'not writable', 'Run: chmod 755 ' . $dir . ' — used for ' . $why . '.');
}

/* ---- clean URLs ---- */
$add('Clean URLs (.htaccess)', true, 'Apache reads .htaccess automatically', 'On Nginx copy the server block from the bottom of .htaccess.');

/* ---- email (optional) ---- */
$smtp = smtp_configured();
$add('Email (SMTP)', $smtp, $smtp ? 'configured' : 'not configured — optional', $smtp ? '' : 'Optional. Without it, mail is written to the error log. Set SMTP_* in .env or open /setup.', true);

$failures = array_filter($checks, fn($c) => !$c['ok'] && !$c['optional']);
$ready = $failures === [];

$pageTitle = 'Deployment check | Mbilinyi Tech Solutions';
$page = 'landing';
require __DIR__ . '/partials/head.php';
?>
<body class="antialiased">
<?php require __DIR__ . '/partials/header.php'; ?>

<div class="view-section active">
  <section class="max-w-4xl mx-auto px-4 pt-8 pb-16">
    <div class="glass rounded-3xl p-6 md:p-10">
      <div class="badge <?= $ready ? 'bg-emerald-500/10 text-emerald-300 border border-emerald-400/30' : 'bg-amber-500/10 text-amber-300 border border-amber-400/30' ?>">
        <span data-en="<?= $ready ? 'READY' : 'NEEDS ATTENTION' ?>" data-sw="<?= $ready ? 'TAYARI' : 'INAHITAJI MAREKEBISHO' ?>"><?= $ready ? 'READY' : 'NEEDS ATTENTION' ?></span>
      </div>
      <h2 class="font-display font-bold text-3xl mt-3"><span data-en="Deployment" data-sw="Usanidi">Deployment</span> <span class="grad-text" data-en="check" data-sw="ukaguzi">check</span></h2>
      <p class="text-white/55 text-[14px] mt-2" data-en="Everything this site needs to run, in one list. No passwords are shown." data-sw="Kila kitu tovuti hii inahitaji ili kufanya kazi, kwenye orodha moja. Nenosiri hazionyeshwi.">Everything this site needs to run, in one list. No passwords are shown.</p>

      <div class="mt-6 divide-y divide-white/10 rounded-2xl border border-white/10 overflow-hidden">
        <?php foreach ($checks as $c): ?>
        <?php $icon = $c['ok'] ? 'circle-check text-emerald-400' : ($c['optional'] ? 'circle-info text-cyan-400' : 'circle-exclamation text-amber-400'); ?>
        <div class="flex items-start gap-3 p-4 bg-white/[.02]">
          <i class="fa-solid fa-<?= $icon ?> mt-1"></i>
          <div class="min-w-0 flex-1">
            <div class="font-bold text-[14px]"><?= htmlspecialchars($c['label']) ?>
              <?php if ($c['optional']): ?><span class="text-white/40 font-normal text-[12px]">— <span data-en="optional" data-sw="si lazima">optional</span></span><?php endif; ?>
            </div>
            <div class="text-white/50 text-[13px] font-mono break-words"><?= htmlspecialchars($c['detail']) ?></div>
            <?php if (!$c['ok'] && !$c['optional'] && $c['hint'] !== ''): ?>
            <div class="text-amber-300/90 text-[13px] mt-1"><?= htmlspecialchars($c['hint']) ?></div>
            <?php endif; ?>
          </div>
        </div>
        <?php endforeach; ?>
      </div>

      <div class="mt-6 flex flex-wrap gap-3 text-sm">
        <a href="/" class="grad-btn px-5 py-3 rounded-xl font-bold"><i class="fa-solid fa-house mr-1"></i> <span data-en="Home" data-sw="Nyumbani">Home</span></a>
        <a href="/setup" class="glass px-5 py-3 rounded-xl font-bold"><i class="fa-solid fa-envelope mr-1"></i> <span data-en="Email setup" data-sw="Usanidi wa barua pepe">Email setup</span></a>
      </div>
    </div>
  </section>
</div>

<?php require __DIR__ . '/partials/footer.php'; ?>
