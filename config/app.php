<?php
declare(strict_types=1);

define('APP_ROOT', dirname(__DIR__));

/* Load .env secrets first so getenv() works everywhere below (email, Ollama, DB). */
require_once __DIR__ . '/env.php';

/* Africa/Dar_es_Salaam (EAT, UTC+3) — Dodoma/Dar share the same national zone. */
if (function_exists('date_default_timezone_set')) {
    @date_default_timezone_set(getenv('APP_TIMEZONE') ?: 'Africa/Dar_es_Salaam');
}

/* ---------- warnings must never corrupt an API response ----------
 * A single PHP notice printed before the JSON would make every client fail to
 * parse the body. Details still land in the server log where we can read them. */
$__uri = $_SERVER['REQUEST_URI'] ?? '';
if (strncmp($__uri, '/api/', 5) === 0 || str_contains($__uri, '.php?') && str_contains($__uri, '/api/')) {
    @ini_set('display_errors', '0');
    @ini_set('log_errors', '1');
}
unset($__uri);

/* ---------- session (stored inside the project) ---------- */
$sesDir = APP_ROOT . '/storage/sessions';
if (!is_dir($sesDir)) { @mkdir($sesDir, 0775, true); }
if (session_status() !== PHP_SESSION_ACTIVE) {
    session_save_path($sesDir);
    session_set_cookie_params([
        'lifetime' => 0,
        'path' => '/',
        'httponly' => true,
        'samesite' => 'Lax',
    ]);
    session_name('mts_session');
    session_start();
}

require_once __DIR__ . '/db.php';
require_once __DIR__ . '/map.php';
require_once APP_ROOT . '/lib/email.php';

/* ---------- helpers ---------- */
function base_url(string $path = ''): string { return $path; }

function redirect(string $url): never { header('Location: ' . $url); exit; }

function new_id(string $prefix): string {
    return $prefix . strtoupper(substr(bin2hex(random_bytes(5)), 0, 6));
}

/* ---------- JSON API helpers ---------- */
function j_ok(array $data = [], int $code = 200): never {
    http_response_code($code);
    header('Content-Type: application/json; charset=utf-8');
    header('Cache-Control: no-store');
    echo json_encode(['ok' => true, 'data' => $data]);
    exit;
}

function j_err(string $msg, int $code = 400): never {
    http_response_code($code);
    header('Content-Type: application/json; charset=utf-8');
    header('Cache-Control: no-store');
    echo json_encode(['ok' => false, 'error' => $msg]);
    exit;
}

function req_body(): array {
    static $cache = null;
    if ($cache !== null) return $cache;
    $raw = file_get_contents('php://input');
    /* JSON_INVALID_UTF8_SUBSTITUTE: a malformed byte must degrade to a readable
     * body instead of null (which would silently drop the `action` key). */
    $decoded = ($raw !== false && $raw !== '')
        ? json_decode($raw, true, 512, JSON_INVALID_UTF8_SUBSTITUTE)
        : null;
    $cache = is_array($decoded) ? $decoded : [];
    return $cache;
}

function action_name(): string {
    $b = req_body();
    return (string)($_GET['action'] ?? $b['action'] ?? '');
}

function param(string $key, mixed $default = null): mixed {
    $b = req_body();
    if (array_key_exists($key, $b)) return $b[$key];
    if (array_key_exists($key, $_POST)) return $_POST[$key];
    if (array_key_exists($key, $_GET)) return $_GET[$key];
    return $default;
}

function str_param(string $key, string $default = ''): string {
    $v = param($key, $default);
    return is_scalar($v) ? trim((string)$v) : $default;
}

function require_csrf(): void {
    $sent = (string)($_SERVER['HTTP_X_CSRF_TOKEN'] ?? '');
    $have = (string)($_SESSION['csrf'] ?? '');
    if ($have === '' || $sent === '' || !hash_equals($have, $sent)) {
        j_err('Invalid or missing CSRF token. Reload the page and try again.', 403);
    }
}

function csrf_token(): string {
    if (empty($_SESSION['csrf'])) {
        $_SESSION['csrf'] = bin2hex(random_bytes(32));
    }
    return $_SESSION['csrf'];
}

/* ---------- auth ---------- */
function current_user(): ?array {
    $uid = $_SESSION['user_id'] ?? null;
    if (!$uid) return null;
    $st = db()->prepare('SELECT * FROM users WHERE id = ?');
    $st->execute([$uid]);
    $row = $st->fetch();
    return $row ? map_user($row) : null;
}

function require_user(): array {
    $u = current_user();
    if (!$u) j_err('You need to sign in first.', 401);
    if ($u['status'] !== 'Active') j_err('This account is suspended. Contact 0796 752 645.', 403);
    return $u;
}

function require_admin(): array {
    $u = require_user();
    if ($u['role'] !== 'admin') j_err('Admin access required.', 403);
    return $u;
}

function login_user(string $userId): void {
    session_regenerate_id(true);
    $_SESSION['user_id'] = $userId;
    $_SESSION['csrf'] = bin2hex(random_bytes(32));
}

/* ---------- page guards (HTML pages) ----------
 * All targets are extensionless — the server rewrites /pricing -> pricing.php,
 * so no .php ever appears in a Location header or the address bar. */
function client_page_user(): array {
    $u = current_user();
    if (!$u) redirect('auth?returnTo=client');
    if ($u['status'] !== 'Active') { session_destroy(); redirect('auth'); }
    if ($u['role'] === 'admin') redirect('admin');
    return $u;
}

function admin_page_user(): array {
    $u = current_user();
    if (!$u) redirect('auth?returnTo=admin');
    if ($u['status'] !== 'Active') { session_destroy(); redirect('auth'); }
    if ($u['role'] !== 'admin') redirect('client');
    return $u;
}

function json_out(mixed $data): never {
    echo json_encode($data, JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_SLASHES);
    exit;
}

/* ---------- settings (payment details, company info, editable by admin) ---------- */

/**
 * Read one setting. Cached per request so a page can ask for many keys cheaply.
 * Missing key → $default (never throws: settings are optional by design).
 */
function setting(string $key, string $default = ''): string {
    static $cache = null;
    if ($cache === null) {
        $cache = [];
        try {
            foreach (db_all('SELECT skey, svalue FROM settings') as $r) {
                $cache[(string)$r['skey']] = (string)$r['svalue'];
            }
        } catch (\Throwable $e) {
            error_log('[settings] read failed: ' . $e->getMessage());
        }
    }
    return array_key_exists($key, $cache) ? $cache[$key] : $default;
}

/** All settings as an assoc array. */
function settings_all(): array {
    $out = [];
    try {
        foreach (db_all('SELECT skey, svalue FROM settings') as $r) {
            $out[(string)$r['skey']] = (string)$r['svalue'];
        }
    } catch (\Throwable $e) {
        error_log('[settings] read failed: ' . $e->getMessage());
    }
    return $out;
}

function setting_set(string $key, string $value): void {
    db_run(
        'INSERT INTO settings (skey, svalue) VALUES (?, ?)
         ON DUPLICATE KEY UPDATE svalue = VALUES(svalue)',
        [$key, $value]
    );
}

/** Payment block shown to signed-in users (bank + mobile money). */
function payment_details(): array {
    return [
        'bank' => [
            'name'    => setting('pay_bank_name', 'NMB Bank'),
            'account' => setting('pay_bank_account', '51710099563'),
            'holder'  => setting('pay_bank_holder', 'JACKSON MBILINYI'),
            'branch'  => setting('pay_bank_branch', ''),
            'swift'   => setting('pay_bank_swift', ''),
        ],
        'mobile' => [
            'network' => setting('pay_mobile_network', 'Vodacom'),
            'number'  => setting('pay_mobile_number', '0796752645'),
            'holder'  => setting('pay_mobile_holder', 'JACKSON MBILINYI'),
        ],
        'contact'    => setting('pay_contact_person', 'Jackson Mbilinyi (CEO)'),
        'currency'   => setting('pay_currency', 'TZS'),
        'instructions' => setting('pay_instructions', ''),
    ];
}

/* ---------- service prices (admin-editable, shown on pricing + chatbot) ---------- */

/**
 * The catalogue of priced services. `price` is pulled live from the settings
 * table so the CEO can change any figure from Admin → Settings and both the
 * pricing page and the AI chatbot pick it up immediately.
 *
 * @return array<int, array{code:string, en:string, sw:string, price:float, period:string}>
 */
function service_price_list(): array {
    static $catalogue = [
        ['website_basic',   'Basic / starter website (up to 5 pages)',            'Tovuti ya msingi (kurasa hadi 5)',              '/project'],
        ['website_business','Business website (custom design, CMS)',              'Tovuti ya kampuni (muundo maalum, CMS)',        '/project'],
        ['ecommerce',       'Online shop / e-commerce with payments',             'Duka la mtandaoni lenye malipo',                '/project'],
        ['mobile_app',      'Mobile app (Android + iOS)',                         'Programu ya simu (Android + iOS)',              '/project'],
        ['custom_system',   'Custom business system (ERP / school / pharmacy)',   'Mfumo maalum wa biashara (ERP / shule / duka)', '/project'],
        ['whatsapp_bot',    'WhatsApp / SMS chatbot (Swahili + English)',         'Chatbot ya WhatsApp / SMS (Kiswahili + Kiingereza)', '/project'],
        ['ai_finetune',     'AI fine-tuning on your own data (RAG / LoRA)',       'Kurekebisha AI kwa data yako (RAG / LoRA)',     '/project'],
        ['security_audit',  'Cyber-security audit (OWASP Top-10)',                'Ukaguzi wa usalama (OWASP Top-10)',             '/one-time'],
        ['pentest',         'Penetration test (manual + automated)',              'Mtihani wa uvamizi (manual + automated)',       '/system'],
        ['maintenance',     'Software maintenance & SLA (Starter)',               'Matengenezo ya programu na SLA (Starter)',      '/month'],
        ['brela',           'BRELA business / company registration',              'Usajili wa biashara / kampuni ya BRELA',        '/service'],
        ['tra_tax',         'TRA tax services (TIN, VAT, PAYE, returns)',         'Huduma za kodi za TRA (TIN, VAT, PAYE)',        '/service'],
    ];

    $out = [];
    foreach ($catalogue as [$code, $en, $sw, $period]) {
        $out[] = [
            'code'   => $code,
            'en'     => $en,
            'sw'     => $sw,
            'price'  => (float)setting('price_' . $code, '0'),
            'period' => $period,
        ];
    }
    return $out;
}

/** Format a number as TZS 1,500,000. */
function tzs(float|int|string $n): string {
    return 'TZS ' . number_format((float)$n, 0, '.', ',');
}

/* ---------- Ollama Cloud config (Task 7) ---------- */
function ollama_config(): array {
    $key   = $_ENV['OLLAMA_CLOUD_API_KEY'] ?? getenv('OLLAMA_CLOUD_API_KEY') ?: '';
    $base  = rtrim($_ENV['OLLAMA_CLOUD_API_URL'] ?? getenv('OLLAMA_CLOUD_API_URL') ?: 'https://ollama.com/v1', '/');
    $model = $_ENV['OLLAMA_MODEL'] ?? getenv('OLLAMA_MODEL') ?: '';

    /* Prefer the small/fast "free tier" models first, then the larger ones. */
    $defaultCandidates = 'glm-5.3-flash,deepseek-v4.1-flash,gpt-oss:20b,nemotron-3-nano:30b,gemma4:31b,glm-5.3,minimax-m2.7,gpt-oss:120b';

    return [
        'enabled'  => !empty($key),
        'api_key'  => $key,
        'base_url' => $base,
        'model'    => $model,

        /**
         * Candidate models in priority order. The first one that answers is
         * used, so a renamed or rate-limited model never leaves the chatbot
         * dead — it just falls through to the next entry.
         *
         * Override with OLLAMA_MODEL_CANDIDATES=a,b,c in .env.
         */
        'candidates' => array_values(array_filter(array_map(
            'trim',
            explode(',', (string)($_ENV['OLLAMA_MODEL_CANDIDATES'] ?? getenv('OLLAMA_MODEL_CANDIDATES') ?: $defaultCandidates))
        ))),
    ];
}

/* ---------- Visitor tracking (Task 8) ---------- */
function record_visitor(): void {
    try {
        $requestUri = $_SERVER['REQUEST_URI'] ?? '';
        if (stripos($requestUri, '/api/') !== false || stripos($requestUri, 'sw.js') !== false || stripos($requestUri, 'manifest.json') !== false) {
            return;
        }
        $ipFull = $_SERVER['REMOTE_ADDR'] ?? '0.0.0.0';
        $userAgent = $_SERVER['HTTP_USER_AGENT'] ?? '';
        $sessionId = session_id() ?: bin2hex(random_bytes(8));
        $ipHash = hash('sha256', $ipFull . $sessionId);

        $page = parse_url($requestUri, PHP_URL_PATH) ?: '/';
        $page = basename($page) ?: 'index';

        $referer = $_SERVER['HTTP_REFERER'] ?? '';
        $u = current_user();
        $userId = $u ? (int)$u['id'] : null;

        $st = db()->prepare(
            'INSERT INTO visitor_logs (visit_date, ip_hash, user_agent, page_path, referer, user_id, session_id)
             VALUES (CURDATE(), ?, ?, ?, ?, ?, ?)'
        );
        $st->execute([$ipHash, substr($userAgent, 0, 255), substr($page, 0, 120), substr((string)$referer, 0, 255), $userId, $sessionId]);
    } catch (\Throwable $e) {
        error_log('[visitor] ' . $e->getMessage());
    }
}

record_visitor();

