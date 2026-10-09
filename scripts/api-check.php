<?php
/* One-off connectivity check for the Resend + Ollama credentials in .env. */
declare(strict_types=1);
require __DIR__ . '/../config/env.php';

function call_api(string $url, string $key, ?array $body = null): array {
    $ch = curl_init($url);
    $headers = ['Authorization: Bearer ' . $key, 'Content-Type: application/json'];
    curl_setopt_array($ch, [
        CURLOPT_RETURNTRANSFER => true,
        CURLOPT_TIMEOUT        => 45,
        CURLOPT_HTTPHEADER     => $headers,
    ]);
    if ($body !== null) {
        curl_setopt($ch, CURLOPT_POST, true);
        curl_setopt($ch, CURLOPT_POSTFIELDS, json_encode($body, JSON_UNESCAPED_UNICODE));
    }
    $res  = curl_exec($ch);
    $code = (int)curl_getinfo($ch, CURLINFO_HTTP_CODE);
    $err  = curl_error($ch);
    curl_close($ch);
    return ['code' => $code, 'err' => $err, 'body' => (string)$res];
}

/* ---------------- Ollama ---------------- */
$ok = getenv('OLLAMA_CLOUD_API_KEY') ?: '';
echo "== OLLAMA ==\nkey: " . ($ok ? 'present' : 'MISSING') . "\n";
if ($ok) {
    $r = call_api('https://api.ollama.com/v1/models', $ok);
    echo "GET /v1/models => HTTP {$r['code']} {$r['err']}\n";
    echo substr($r['body'], 0, 1200) . "\n\n";
}

/* ---------------- Resend ---------------- */
$rk = getenv('RESEND_API_KEY') ?: '';
echo "== RESEND ==\nkey: " . ($rk ? 'present' : 'MISSING') . "\n";
if ($rk) {
    $r = call_api('https://api.resend.com/domains', $rk);
    echo "GET /domains => HTTP {$r['code']}\n";
    echo substr($r['body'], 0, 2500) . "\n";
}
