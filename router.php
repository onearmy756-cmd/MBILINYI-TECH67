<?php
/**
 * router.php — clean URLs for the PHP built-in development server.
 *
 *   php -c config\php.ini -S 127.0.0.1:8090 -t . router.php
 *
 * Mirrors .htaccess exactly:
 *     /pricing          ->  pricing.php        (address bar stays clean)
 *     /pricing.php      ->  301 /pricing       (GET/HEAD only)
 *     /api/settings     ->  api/settings.php
 *     /storage|config|scripts  ->  403
 *
 * Returning false hands the request back to the built-in server, which
 * then serves the real file (assets, images, sw.js ...).
 */

$rawPath  = urldecode((string)parse_url($_SERVER['REQUEST_URI'] ?? '/', PHP_URL_PATH));
$hadSlash = ($rawPath !== '/' && substr($rawPath, -1) === '/');
$path  = '/' . ltrim($rawPath, '/');
if (strlen($path) > 1) {
    $path = rtrim($path, '/');
}
$method = $_SERVER['REQUEST_METHOD'] ?? 'GET';
$qs     = (string)($_SERVER['QUERY_STRING'] ?? '');

/* ---- 1. protected folders / dotfiles: never downloadable ----------- */
if (preg_match('#^/(storage|config|scripts)(/|$)#', $path) || preg_match('#(^|/)\.#', $path)) {
    http_response_code(403);
    header('Content-Type: text/plain; charset=utf-8');
    echo '403 Forbidden';
    return true;
}

/* ---- 2. Canonicalise a trailing slash: /pricing/ -> /pricing --------
 * Without this a page loaded from /pricing/ would resolve its relative
 * links against /pricing/ and break them (/pricing/services).          */
if ($hadSlash) {
    $target = $path;
    if ($qs !== '') {
        $target .= '?' . $qs;
    }
    header('Location: ' . $target, true, 301);
    return true;
}

/* ---- 3. /pricing.php  ->  /pricing  (GET/HEAD only) -----------------
 * A 301 on POST would silently become a GET and drop the body, which
 * would break every API call. The query string is carried over.        */
if ($method === 'GET' || $method === 'HEAD') {
    if ($path !== '/' && preg_match('#^(.+)\.php$#', $path, $m)) {
        $target = ($m[1] === 'index') ? '/' : $m[1];
        if ($qs !== '') {
            $target .= '?' . $qs;
        }
        header('Location: ' . $target, true, 301);
        return true;
    }
}

/* ---- 4. a real file is served untouched ---------------------------- */
if ($path !== '/' && is_file(__DIR__ . $path)) {
    return false;
}

/* ---- 5. /pricing  ->  pricing.php ---------------------------------- */
$target = __DIR__ . ($path === '/' ? '/index.php' : $path . '.php');
if (is_file($target)) {
    /* keep SCRIPT_NAME honest for anything that reads it */
    $_SERVER['SCRIPT_NAME'] = $path;
    $_SERVER['PHP_SELF']   = $path;
    require $target;
    return true;
}

/* ---- 6. nothing matched ------------------------------------------- */
http_response_code(404);
header('Content-Type: text/plain; charset=utf-8');
echo '404 Not Found';
return true;
