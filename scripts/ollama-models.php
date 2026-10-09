<?php
/* List every model Ollama Cloud exposes for this API key. */
declare(strict_types=1);
require __DIR__ . '/../config/env.php';

$key  = getenv('OLLAMA_CLOUD_API_KEY') ?: '';
$base = getenv('OLLAMA_CLOUD_API_URL') ?: 'https://api.ollama.com/v1';

$ch = curl_init($base . '/models');
curl_setopt_array($ch, [
    CURLOPT_RETURNTRANSFER => true,
    CURLOPT_TIMEOUT        => 60,
    CURLOPT_FOLLOWLOCATION => true,
    CURLOPT_MAXREDIRS      => 5,
    CURLOPT_HTTPHEADER     => ['Authorization: Bearer ' . $key],
]);
$body = curl_exec($ch);
$code = (int)curl_getinfo($ch, CURLINFO_HTTP_CODE);
$err  = curl_error($ch);
curl_close($ch);

echo "HTTP {$code} {$err}\n";
$dec = json_decode((string)$body, true);
$models = $dec['data'] ?? [];
echo 'count=' . count($models) . "\n";
foreach ($models as $m) {
    $extra = [];
    if (isset($m['size']))     $extra[] = round($m['size'] / 1e9, 1) . 'GB';
    if (isset($m['expires_at']) && $m['expires_at']) $extra[] = 'exp=' . $m['expires_at'];
    if (isset($m['details']['parameter_size'])) $extra[] = $m['details']['parameter_size'];
    if (isset($m['created'])) $extra[] = date('Y-m-d', (int)$m['created']);
    printf("%-34s %s\n", $m['id'] ?? '?', implode(' ', $extra));
}
