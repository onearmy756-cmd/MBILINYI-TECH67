<?php
/**
 * Mbilinyi Tech Solutions — One-Click Email Configuration
 *
 * Enter your Gmail App Password once → this script:
 *   1. Updates .env with SMTP credentials
 *   2. Pushes them to Supabase (Management API) so Auth uses YOUR SMTP
 *   3. Sends a test email to verify everything works
 *
 * No manual file editing, no Supabase dashboard clicks.
 */
declare(strict_types=1);
require_once __DIR__ . '/config/env.php';   // loads .env into getenv()
require_once __DIR__ . '/config/app.php';   // db, helpers, csrf_token()

/* ---- CSRF for the form ---- */
$csrf = csrf_token();

/* ---- Handle POST ---- */
$msg = '';
$msgType = 'info';
if ($_SERVER['REQUEST_METHOD'] === 'POST' && isset($_POST['app_password'])) {
    if (!hash_equals((string)($_SESSION['csrf'] ?? ''), (string)($_POST['csrf'] ?? ''))) {
        $msg = 'CSRF token invalid. Refresh and try again.';
        $msgType = 'err';
    } else {
        $raw = trim((string)$_POST['app_password']);
        /* Gmail App Password: 16 chars, usually shown as 4 groups of 4.
         * Accept with or without spaces. */
        $pass = str_replace(' ', '', $raw);
        if (strlen($pass) !== 16 || !ctype_alnum($pass)) {
            $msg = 'App Password must be 16 alphanumeric characters (spaces allowed).';
            $msgType = 'err';
        } else {
            /* 1. Update .env */
            $envPath = __DIR__ . '/.env';
            $env = file_exists($envPath) ? file_get_contents($envPath) : '';
            $lines = [];
            foreach (explode("\n", $env) as $ln) {
                $ln = rtrim($ln);
                if (str_starts_with($ln, 'SMTP_PASS=')) continue;
                if (str_starts_with($ln, 'SMTP_HOST=')) continue;
                if (str_starts_with($ln, 'SMTP_PORT=')) continue;
                if (str_starts_with($ln, 'SMTP_USER=')) continue;
                if (str_starts_with($ln, 'SMTP_FROM=')) continue;
                if (str_starts_with($ln, 'SMTP_SENDER_NAME=')) continue;
                $lines[] = $ln;
            }
            $lines[] = 'SMTP_HOST=smtp.gmail.com';
            $lines[] = 'SMTP_PORT=587';
            $lines[] = 'SMTP_USER=mbilinyitech@gmail.com';
            $lines[] = 'SMTP_PASS=' . $pass;
            $lines[] = 'SMTP_FROM=mbilinyitech@gmail.com';
            $lines[] = 'SMTP_SENDER_NAME=Mbilinyi Tech Solutions';
            file_put_contents($envPath, implode("\n", $lines) . "\n");

            /* Reload so getenv() sees new values immediately */
            foreach (['SMTP_HOST','SMTP_PORT','SMTP_USER','SMTP_PASS','SMTP_FROM','SMTP_SENDER_NAME'] as $k) {
                putenv("$k=" . getenv($k));
            }

            /* 2. Push to Supabase Management API */
            $supabaseRef = 'zpflwwbqagmgvbmxkpbl';
            $pat = getenv('SUPABASE_PAT');
            if (!$pat) { throw new RuntimeException('SUPABASE_PAT not set in .env'); }
            $apiUrl = "https://api.supabase.com/v1/projects/$supabaseRef/config/auth";

            $payload = json_encode([
                'smtp_host'        => 'smtp.gmail.com',
                'smtp_port'        => 587,
                'smtp_user'        => 'mbilinyitech@gmail.com',
                'smtp_pass'        => $pass,
                'smtp_admin_email' => 'mbilinyitech@gmail.com',
                'smtp_sender_name' => 'Mbilinyi Tech Solutions',
            ], JSON_UNESCAPED_UNICODE);

            $ch = curl_init($apiUrl);
            curl_setopt_array($ch, [
                CURLOPT_CUSTOMREQUEST => 'PATCH',
                CURLOPT_POSTFIELDS    => $payload,
                CURLOPT_RETURNTRANSFER => true,
                CURLOPT_TIMEOUT       => 30,
                CURLOPT_HTTPHEADER    => [
                    'Authorization: Bearer ' . $pat,
                    'Content-Type: application/json',
                ],
            ]);
            $resp = curl_exec($ch);
            $http = curl_getinfo($ch, CURLINFO_HTTP_CODE);
            $cerr = curl_error($ch);
            curl_close($ch);

            if ($cerr !== '' || $http >= 400) {
                $msg = "Supabase update failed (HTTP $http): " . ($cerr ?: $resp);
                $msgType = 'err';
            } else {
                /* 3. Send a test email via our SMTP client (lib/smtp.php) */
                require_once __DIR__ . '/lib/smtp.php';
                $html = "<p>If you receive this, your Gmail App Password is working.</p>"
                    . "<p>Timestamp: " . date('Y-m-d H:i:s') . "</p>"
                    . "<p>Server: " . htmlspecialchars($_SERVER['SERVER_ADDR'] ?? 'cli') . "</p>";
                $testOk = smtp_send(
                    ['mbilinyitech@gmail.com'],
                    'Mbilinyi Tech — SMTP Test ✅',
                    $html
                );
                if ($testOk) {
                    $msg = '✅ SUCCESS! .env updated, Supabase configured, TEST EMAIL SENT to mbilinyitech@gmail.com. Check your inbox (and spam).';
                    $msgType = 'ok';
                } else {
                    $msg = '⚠️ .env + Supabase updated, but test email failed. Check error log (storage/logs or PHP error_log).';
                    $msgType = 'warn';
                }
            }
        }
    }
}

/* ---- Current status for display ---- */
$smtpConfigured = smtp_configured();
$supabaseSmtp = null;
/* Quick probe: ask Supabase what SMTP host it currently has (read-only, safe) */
$probePat = getenv('SUPABASE_PAT');
$supabaseSmtp = null;
if ($probePat) {
    $ch = curl_init("https://api.supabase.com/v1/projects/zpflwwbqagmgvbmxkpbl/config/auth");
    curl_setopt_array($ch, [
        CURLOPT_RETURNTRANSFER => true,
        CURLOPT_TIMEOUT => 15,
        CURLOPT_HTTPHEADER => ['Authorization: Bearer ' . $probePat],
    ]);
    $r = curl_exec($ch);
    curl_close($ch);
    $j = json_decode($r, true);
    if (is_array($j)) $supabaseSmtp = $j;
}
?>
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Email Setup — Mbilinyi Tech Solutions</title>
<script src="https://cdn.tailwindcss.com"></script>
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=Space+Grotesk:wght@500;600;700&display=swap" rel="stylesheet">
<style>
  *{scroll-behavior:smooth}
  body{background:#050914;color:#E6ECF5;font-family:Inter,sans-serif}
  .glass{background:rgba(255,255,255,.04);backdrop-filter:blur(18px);border:1px solid rgba(255,255,255,.08)}
  .grad-btn{background:linear-gradient(135deg,#06b6d4,#3b82f6 50%,#8b5cf6);box-shadow:0 10px 30px -8px rgba(59,130,246,.6)}
  .grad-btn:hover{transform:translateY(-2px);box-shadow:0 18px 40px -8px rgba(59,130,246,.7)}
  .input{background:rgba(255,255,255,.05);border:1px solid rgba(255,255,255,.1);border-radius:12px;padding:14px 16px;width:100%;color:white;outline:none;font-family:inherit;font-size:15px}
  .input:focus{border-color:#22d3ee;box-shadow:0 0 0 3px rgba(34,211,238,.15);background:rgba(255,255,255,.07)}
  .status-ok{background:rgba(34,197,94,.15);border:1px solid rgba(34,197,94,.3);color:#4ade80}
  .status-warn{background:rgba(245,179,1,.15);border:1px solid rgba(245,179,1,.3);color:#f5b301}
  .status-err{background:rgba(239,68,68,.15);border:1px solid rgba(239,68,68,.3);color:#fca5a5}
  .mono{font-family:'JetBrains Mono',monospace;font-size:13px}
</style>
</head>
<body class="min-h-screen flex items-center justify-center p-6">
<div class="w-full max-w-2xl">
  <div class="text-center mb-10">
    <div class="w-16 h-16 mx-auto mb-4 rounded-2xl grad-btn flex items-center justify-center text-2xl font-bold">M</div>
    <h1 class="font-display font-bold text-3xl">Email Configuration</h1>
    <p class="text-white/60 mt-2">Enter your Gmail App Password once — everything else is automatic.</p>
  </div>

  <div class="glass rounded-3xl p-8 space-y-6">
    <!-- Current Status -->
    <div class="rounded-2xl p-5 <?= $smtpConfigured ? 'status-ok' : 'status-warn' ?>">
      <div class="flex items-center gap-3 mb-2">
        <i class="fa-solid fa-<?= $smtpConfigured ? 'circle-check text-emerald-300' : 'circle-exclamation text-amber-300' ?> text-xl"></i>
        <span class="font-bold text-lg">SMTP Status</span>
      </div>
      <div class="grid sm:grid-cols-2 gap-3 text-sm mono">
        <div><span class="text-white/50">.env SMTP_PASS:</span> <?= $smtpConfigured ? '<span class="text-emerald-300">SET (16 chars)</span>' : '<span class="text-rose-300">NOT SET</span>' ?></div>
        <div><span class="text-white/50">Supabase SMTP:</span> <?= $supabaseSmtp && !empty($supabaseSmtp['smtp_host']) ? '<span class="text-emerald-300">CONFIGURED</span>' : '<span class="text-amber-300">Using Built-in (rate-limited)</span>' ?></div>
        <?php if ($supabaseSmtp && isset($supabaseSmtp['smtp_host'])): ?>
        <div class="sm:col-span-2"><span class="text-white/50">Supabase Host:</span> <?= htmlspecialchars($supabaseSmtp['smtp_host']) ?>:<?= htmlspecialchars($supabaseSmtp['smtp_port'] ?? '') ?></div>
        <div class="sm:col-span-2"><span class="text-white/50">Supabase User:</span> <?= htmlspecialchars($supabaseSmtp['smtp_user'] ?? '') ?></div>
        <?php endif; ?>
      </div>
    </div>

    <!-- Form -->
    <form method="post" class="space-y-5" id="setupForm">
      <input type="hidden" name="csrf" value="<?= htmlspecialchars($csrf) ?>">

      <div>
        <label class="block text-sm font-bold text-white/70 mb-2">Gmail App Password <span class="text-rose-300">*</span></label>
        <input type="password" name="app_password" id="appPass" class="input mono" placeholder="abcd efgh ijkl mnop" autocomplete="off" required>
        <p class="text-[12px] text-white/40 mt-1.5">Get it at: <a href="https://myaccount.google.com/apppasswords" target="_blank" class="text-cyan-300 underline hover:text-cyan-200">myaccount.google.com/apppasswords</a> → Select "Mail" → "Other" → name it "Mbilinyi Tech" → Copy the 16-char code.</p>
      </div>

      <div class="rounded-2xl bg-white/[.03] border border-white/10 p-4 text-sm text-white/60">
        <strong class="text-white">What this does:</strong>
        <ol class="list-decimal list-inside mt-2 space-y-1">
          <li>Writes <code class="mono">SMTP_HOST/PORT/USER/PASS/FROM/SENDER_NAME</code> to <code class="mono">.env</code></li>
          <li>Calls Supabase Management API (PATCH /config/auth) so <strong>Auth uses YOUR Gmail SMTP</strong> — no more built-in rate limits</li>
          <li>Sends a test email to <code class="mono">mbilinyitech@gmail.com</code> immediately</li>
          <li>After this, <strong>OTP login, contract emails, password resets, notifications all work</strong></li>
        </ol>
      </div>

      <button type="submit" class="grad-btn w-full py-4 rounded-xl font-bold text-lg flex items-center justify-center gap-2">
        <i class="fa-solid fa-paper-plane"></i> Configure & Send Test Email
      </button>
    </form>

    <!-- Message -->
    <?php if ($msg): ?>
    <div class="rounded-2xl p-4 <?= $msgType === 'ok' ? 'status-ok' : ($msgType === 'warn' ? 'status-warn' : 'status-err') ?> animate-fade-in">
      <?= nl2br(htmlspecialchars($msg)) ?>
    </div>
    <?php endif; ?>

    <!-- Links -->
    <div class="pt-4 border-t border-white/10 flex flex-wrap justify-center gap-4 text-sm">
      <a href="/" class="text-white/60 hover:text-cyan-300"><i class="fa-solid fa-house mr-1"></i> Home</a>
      <a href="/auth" class="text-white/60 hover:text-cyan-300"><i class="fa-solid fa-right-to-bracket mr-1"></i> Sign In</a>
      <a href="/admin" class="text-white/60 hover:text-cyan-300"><i class="fa-solid fa-gear mr-1"></i> Admin</a>
    </div>
  </div>

  <p class="text-center text-[11px] text-white/30 mt-6">Mbilinyi Tech Solutions — BRELA Reg • TIN 192-147-522 • Dodoma, Tanzania</p>
</div>

<style>
@keyframes fadeIn{from{opacity:0;transform:translateY(8px)}to{opacity:1;transform:translateY(0)}}
.animate-fade-in{animation:fadeIn .35s ease-out}
</style>
</body>
</html>