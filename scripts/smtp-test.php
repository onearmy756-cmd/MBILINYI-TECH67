<?php
/* Send one test email over the SMTP account stored in Supabase. */
declare(strict_types=1);
require __DIR__ . '/../config/env.php';
require __DIR__ . '/../lib/smtp.php';

$to = $argv[1] ?? 'mbilinyitech@gmail.com';

$c = smtp_config();
echo "host={$c['host']}:{$c['port']} user={$c['user']} from={$c['from']} sender={$c['sender_name']}\n";
echo 'configured: ' . (smtp_configured() ? 'YES' : 'NO') . "\n";
echo "to: {$to}\n\n";

$html = '<div style="font-family:Arial,sans-serif;background:#050914;color:#e6edf7;padding:32px;border-radius:16px">'
      . '<h2 style="color:#22d3ee;margin-top:0">✅ Mbilinyi Tech Solutions — SMTP test</h2>'
      . '<p>Habari — hii ni barua pepe ya majaribio kutoka mfumo wako wa Mbilinyi Tech Solutions.</p>'
      . '<p>This is a delivery test from the Mbilinyi Tech Solutions mailer.</p>'
      . '<p style="color:#8EA0BF;font-size:13px">YOUR PROBLEM, OUR SOLUTION • SHIDA YAKO, TATUZO LETU<br>'
      . date('Y-m-d H:i:s T') . '</p></div>';

$ok = smtp_send([$to], 'MTS — SMTP delivery test', $html);
echo $ok ? "\nRESULT: ACCEPTED by server (check the inbox)\n"
         : "\nRESULT: FAILED — see the [smtp] lines above\n";
