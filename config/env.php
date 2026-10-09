<?php
declare(strict_types=1);

/**
 * Tiny .env loader.
 *
 * Values are exported with putenv() + $_ENV so that every existing
 * getenv('RESEND_API_KEY') / getenv('OLLAMA_CLOUD_API_KEY') call in the codebase
 * starts working without touching those files. Real process environment always
 * wins over the file, so a hosting panel can still override anything.
 *
 * File format (one per line, # comments):
 *   KEY=value
 *   KEY="value with spaces"
 */
function mts_load_env(?string $file = null): void {
    static $loaded = false;
    if ($loaded) return;
    $loaded = true;

    $file = $file ?? dirname(__DIR__) . '/.env';
    if (!is_file($file) || !is_readable($file)) return;

    $lines = file($file, FILE_IGNORE_NEW_LINES);
    if ($lines === false) return;

    foreach ($lines as $line) {
        $line = trim($line);
        if ($line === '' || $line[0] === '#') continue;

        $eq = strpos($line, '=');
        if ($eq === false) continue;

        $key = trim(substr($line, 0, $eq));
        if ($key === '' || !preg_match('/^[A-Za-z_][A-Za-z0-9_]*$/', $key)) continue;

        $value = trim(substr($line, $eq + 1));

        // strip matching quotes + inline comment after an unquoted value
        if (strlen($value) > 1 && ($value[0] === '"' || $value[0] === "'")) {
            $q = $value[0];
            $end = strrpos($value, $q);
            $value = ($end > 0) ? substr($value, 1, $end - 1) : substr($value, 1);
        } else {
            $hash = strpos($value, ' #');
            if ($hash !== false) $value = rtrim(substr($value, 0, $hash));
        }

        // the real environment always wins
        $existing = getenv($key);
        if ($existing !== false && $existing !== '') continue;

        putenv($key . '=' . $value);
        $_ENV[$key] = $value;
        $_SERVER[$key] = $value;
    }
}

mts_load_env();
