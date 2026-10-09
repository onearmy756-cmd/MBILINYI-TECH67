<?php
declare(strict_types=1);

/**
 * PDO connection. Primary source is DATABASE_URL / MYSQL_URL in the classic
 * mysql://user:pass@host:port/dbname form. When no usable URL is present in
 * the environment (fresh sandbox), it falls back to the local MariaDB instance
 * created for development (127.0.0.1 / mbilinyi).
 */
function dsn_from_url(string $url): array {
    $url = trim($url, " \t\n\r\0\x0B\"'");
    if ($url === '') throw new RuntimeException('Database URL is empty.');
    if (!preg_match('#^[a-z][a-z0-9+.\-]*://#i', $url)) $url = 'mysql://' . $url;

    $p = parse_url($url);
    if ($p === false || empty($p['host'])) throw new RuntimeException('Database URL is not a valid MySQL URL.');

    $scheme = strtolower($p['scheme'] ?? 'mysql');
    $dbname = ltrim($p['path'] ?? '/', '/');
    $user = isset($p['user']) ? rawurldecode($p['user']) : '';
    $pass = isset($p['pass']) ? rawurldecode($p['pass']) : '';
    $port = isset($p['port']) ? (int)$p['port'] : 0;

    $dsn = '';
    if (in_array($scheme, ['mysql', 'mysqli', 'mariadb'], true)) {
        if ($dbname === '') throw new RuntimeException('Database URL has no database name.');
        $dsn = 'mysql:host=' . $p['host'] . ($port ? ';port=' . $port : '') . ';dbname=' . $dbname . ';charset=utf8mb4';
    } elseif (in_array($scheme, ['postgres', 'postgresql', 'pgsql'], true)) {
        $dsn = 'pgsql:host=' . $p['host'] . ($port ? ';port=' . $port : '') . ($dbname ? ';dbname=' . $dbname : '');
        if (!empty($p['query'])) {
            parse_str($p['query'], $q);
            foreach ($q as $k => $v) {
                if (in_array($k, ['sslmode', 'connect_timeout', 'application_name'], true)) $dsn .= ';' . $k . '=' . $v;
            }
        }
    } else {
        throw new RuntimeException('Unsupported database scheme: ' . $scheme);
    }

    return ['dsn' => $dsn, 'user' => $user ?: null, 'pass' => $pass ?: null];
}

function db_config(): array {
    foreach (['DATABASE_URL', 'MYSQL_URL', 'DB_URL'] as $key) {
        $value = getenv($key);
        if (!$value) continue;
        try {
            return dsn_from_url($value);
        } catch (Throwable $e) {
            error_log('[db] ' . $key . ' unusable: ' . $e->getMessage());
        }
    }
    error_log('[db] No usable MySQL URL in environment — falling back to local MariaDB (mbilinyi).');
    return dsn_from_url('mysql://mts:mts_local_dev@127.0.0.1:3306/mbilinyi');
}

function db(): PDO {
    static $pdo = null;
    if ($pdo instanceof PDO) return $pdo;

    $cfg = db_config();
    
    /* ---- SILENT AUTO-CREATE DATABASE IF MISSING ----
     * No errors, no logs, just works. */
    try {
        $pdo = new PDO($cfg['dsn'], $cfg['user'], $cfg['pass'], [
            PDO::ATTR_ERRMODE => PDO::ERRMODE_EXCEPTION,
            PDO::ATTR_DEFAULT_FETCH_MODE => PDO::FETCH_ASSOC,
            PDO::ATTR_EMULATE_PREPARES => false,
        ]);
    } catch (PDOException $e) {
        // If database doesn't exist (SQLSTATE 3D000 / 1049), create it silently
        $isMissingDb = in_array($e->getCode(), ['3D000', '42000'], true) ||
            stripos($e->getMessage(), 'Unknown database') !== false ||
            stripos($e->getMessage(), '1049') !== false;
        
        if ($isMissingDb && preg_match('/dbname=([^;]+)/', $cfg['dsn'], $m)) {
            $dbname = $m[1];
            $dsnNoDb = preg_replace('/;dbname=[^;]+/', '', $cfg['dsn']);
            $tmp = new PDO($dsnNoDb, $cfg['user'], $cfg['pass'], [
                PDO::ATTR_ERRMODE => PDO::ERRMODE_EXCEPTION,
            ]);
            $tmp->exec("CREATE DATABASE IF NOT EXISTS `$dbname` CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci");
            // Reconnect with dbname - no error logging, no exceptions
            $pdo = new PDO($cfg['dsn'], $cfg['user'], $cfg['pass'], [
                PDO::ATTR_ERRMODE => PDO::ERRMODE_EXCEPTION,
                PDO::ATTR_DEFAULT_FETCH_MODE => PDO::FETCH_ASSOC,
                PDO::ATTR_EMULATE_PREPARES => false,
            ]);
        } else {
            throw $e;
        }
    }
    
    ensure_schema($pdo);
    return $pdo;
}

function db_run(string $sql, array $params = []): PDOStatement {
    $st = db()->prepare($sql);
    $st->execute($params);
    return $st;
}

function db_one(string $sql, array $params = []): ?array {
    $row = db_run($sql, $params)->fetch();
    return $row === false ? null : $row;
}

function db_all(string $sql, array $params = []): array {
    return db_run($sql, $params)->fetchAll();
}

/** Store a timestamp in MySQL DATETIME format. */
function dt(mixed $when = null): string {
    $ts = is_numeric($when) ? (int)$when : (is_string($when) && $when !== '' ? strtotime($when) : time());
    return date('Y-m-d H:i:s', $ts ?: time());
}

function ensure_schema(PDO $pdo): void {
    static $done = false;
    if ($done) return;
    $done = true;

    require __DIR__ . '/schema.php';   // defines mts_schema_sql()
    $pdo->exec(mts_schema_sql());
    mts_migrate_visitor_logs($pdo);   // top up visitor_logs on pre-existing installs

    $count = (int)$pdo->query('SELECT count(*) FROM users')->fetchColumn();
    if ($count === 0) {
        require __DIR__ . '/seed.php'; // defines mts_seed(PDO $pdo)
        mts_seed($pdo);
    }

    /* payment details + price list (only fills blanks — admin edits survive) */
    require_once __DIR__ . '/settings-seed.php';
    mts_seed_settings($pdo);
}
