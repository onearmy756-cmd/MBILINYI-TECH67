<?php
/* Smoke-test the chatbot endpoint the way the site calls it (CSRF + session). */
declare(strict_types=1);
require __DIR__ . '/../config/app.php';

$base = 'http://127.0.0.1:8090';

function post_json(string $url, array $payload, string $cookieJar): array {
    global $csrf;
    $ch = curl_init($url);
    curl_setopt_array($ch, [
        CURLOPT_RETURNTRANSFER => true,
        CURLOPT_TIMEOUT        => 90,
        CURLOPT_POST           => true,
        CURLOPT_POSTFIELDS     => json_encode($payload),
        CURLOPT_HTTPHEADER     => ['Content-Type: application/json', 'X-CSRF-Token: ' . $csrf],
        CURLOPT_COOKIEJAR      => $cookieJar,
        CURLOPT_COOKIEFILE     => $cookieJar,
        CURLOPT_HEADER         => false,
    ]);
    $body = curl_exec($ch);
    $code = (int)curl_getinfo($ch, CURLINFO_HTTP_CODE);
    $err  = curl_error($ch);
    curl_close($ch);
    return ['code' => $code, 'err' => $err, 'body' => (string)$body];
}

function get_csrf(string $url, string $cookieJar): string {
    $ch = curl_init($url);
    curl_setopt_array($ch, [
        CURLOPT_RETURNTRANSFER => true, CURLOPT_TIMEOUT => 40,
        CURLOPT_COOKIEJAR => $cookieJar, CURLOPT_COOKIEFILE => $cookieJar,
    ]);
    $body = (string)curl_exec($ch);
    curl_close($ch);
    return preg_match('/window\.__CSRF__\s*=\s*"([a-f0-9]+)"/', $body, $m) ? $m[1] : '';
}

$jar = tempnam(sys_get_temp_dir(), 'mts-cookies');
$csrf = get_csrf($base . '/index.php', $jar);
echo 'csrf: ' . substr($csrf, 0, 12) . "...\n";

$questions = [
    'Je, mnatengeneza TIN na BRELA?',
    'How much is a business website?',
    'Nisaidie na refund policy yenu.',
];

foreach ($questions as $q) {
    $r = post_json($base . '/api/ollama.php', [
        'action'    => 'chat',
        'prompt'    => $q,
        'messages'  => [['role' => 'user', 'content' => $q]],
        'csrf'      => $csrf,
    ], $jar);
    echo "\nQ: {$q}\nHTTP {$r['code']} {$r['err']}\n";
    $j = json_decode($r['body'], true);
    if (!is_array($j)) {
        echo 'BODY: ' . substr($r['body'], 0, 400) . "\n";
        continue;
    }
    $msg = $j['data']['message'] ?? $j['data']['reply'] ?? $j['data']['content'] ?? null;
    if ($msg === null) { echo 'KEYS: ' . implode(',', array_keys($j['data'] ?? $j)) . "\n"; echo substr($r['body'], 0, 500) . "\n"; continue; }
    echo 'A: ' . mb_substr((string)$msg, 0, 700) . "\n";
}

@unlink($jar);
