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

    /* SQLite paths do not survive parse_url ("sqlite:////abs/path" fails), so
     * they are handled before the generic URL parsing. */
    if (preg_match('#^sqlite3?:#i', $url)) {
        /* Accepted spellings, mirroring PDO's own DSN:
         *   sqlite:/abs/path    -> absolute
         *   sqlite:///abs/path  -> absolute
         *   sqlite:rel/path     -> relative to the project root
         *   sqlite://rel/path   -> relative (the empty authority is dropped)
         */
        $file = substr($url, strpos($url, ':') + 1);
        $slashes = strspn($file, '/');
        $rest = substr($file, $slashes);
        if ($rest === '') throw new RuntimeException('SQLite URL has no file path.');
        // An even, non-zero slash count means the host part was left empty.
        $absolute = $slashes === 1 || $slashes > 2;
        if ($absolute) {
            $file = '/' . $rest;
        } else {
            // Relative paths resolve against the project root, never the process
            // CWD, which differs between the web server and api/*.php.
            $file = APP_ROOT . '/' . ltrim($rest, '/');
        }
        return ['dsn' => 'sqlite:' . $file, 'user' => null, 'pass' => null];
    }

    if (!preg_match('#^[a-z][a-z0-9+.\-]*://#i', $url)) {
        /* cPanel hands out the connection as four separate values. Accept the
         * pasted shorthand "host|database|user|password" as well. */
        if (str_contains($url, '|')) {
            $parts = array_pad(explode('|', $url, 4), 4, '');
            [$host, $dbname, $user, $pass] = array_map('trim', $parts);
            if ($host === '' || $dbname === '') {
                throw new RuntimeException('Database shorthand needs at least host|database.');
            }
            $dsn = 'mysql:host=' . $host . ';dbname=' . $dbname . ';charset=utf8mb4';
            return ['dsn' => $dsn, 'user' => $user ?: null, 'pass' => $pass ?: null];
        }
        $url = 'mysql://' . $url;
    }

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

/**
 * Database candidates, most specific first.
 *
 * 1. DATABASE_URL / MYSQL_URL / DB_URL — an explicit choice always wins.
 * 2. Local MariaDB used in development (only when the driver exists).
 * 3. SQLite in storage/mts.sqlite — the zero-config floor, so a fresh clone
 *    on a host without MySQL still boots with no setup at all.
 */
function db_candidates(): array {
    $out = [];
    foreach (['DATABASE_URL', 'MYSQL_URL', 'DB_URL'] as $key) {
        $value = getenv($key);
        if (!$value) continue;
        try {
            $out[] = dsn_from_url($value);
        } catch (Throwable $e) {
            error_log('[db] ' . $key . ' unusable: ' . $e->getMessage());
        }
    }
    if (in_array('mysql', PDO::getAvailableDrivers(), true)) {
        $out[] = dsn_from_url('mysql://mts:mts_local_dev@127.0.0.1:3306/mbilinyi');
    }
    $out[] = dsn_from_url('sqlite:storage/mts.sqlite');
    return $out;
}

function db(): PDO {
    static $pdo = null;
    if ($pdo instanceof PDO) return $pdo;

    $opts = [
        PDO::ATTR_ERRMODE => PDO::ERRMODE_EXCEPTION,
        PDO::ATTR_DEFAULT_FETCH_MODE => PDO::FETCH_ASSOC,
        PDO::ATTR_EMULATE_PREPARES => false,
    ];

    $last = null;
    foreach (db_candidates() as $cfg) {
        $isSqlite = str_starts_with((string)$cfg['dsn'], 'sqlite:');
        if ($isSqlite) {
            require_once __DIR__ . '/sql-map.php';
            $dir = dirname(substr($cfg['dsn'], strlen('sqlite:')));
            if (!is_dir($dir)) @mkdir($dir, 0775, true);
        }
        try {
            $pdo = $isSqlite
                ? new MtsSqlitePdo($cfg['dsn'], null, null, $opts)
                : new PDO($cfg['dsn'], $cfg['user'], $cfg['pass'], $opts);
        } catch (PDOException $e) {
            /* A cPanel database that has not been created yet: make it and
             * reconnect. Anything else moves on to the next candidate. */
            $dbname = null;
            if (preg_match('/dbname=([^;]+)/', (string)$cfg['dsn'], $m)) $dbname = $m[1];
            if ($dbname !== null && mts_is_missing_db($e)) {
                try {
                    $tmp = new PDO(preg_replace('/;dbname=[^;]+/', '', $cfg['dsn']), $cfg['user'], $cfg['pass'], $opts);
                    $tmp->exec("CREATE DATABASE IF NOT EXISTS `$dbname` CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci");
                    $pdo = new PDO($cfg['dsn'], $cfg['user'], $cfg['pass'], $opts);
                } catch (Throwable $inner) {
                    $last = $inner;
                    error_log('[db] ' . $cfg['dsn'] . ' unusable: ' . $inner->getMessage());
                    $pdo = null;
                    continue;
                }
            } else {
                $last = $e;
                error_log('[db] ' . $cfg['dsn'] . ' unusable: ' . $e->getMessage());
                continue;
            }
        }

        try {
            ensure_schema($pdo);
            return $pdo;
        } catch (Throwable $e) {
            /* e.g. MySQL reachable but the schema cannot be created — keep the
             * fallback chain alive rather than failing the whole request. */
            $last = $e;
            error_log('[db] schema on ' . $cfg['dsn'] . ' failed: ' . $e->getMessage());
            $pdo = null;
        }
    }

    throw new RuntimeException('No usable database. Last error: ' . ($last?->getMessage() ?? 'unknown'), 0, $last);
}

/** True when the failure means "database does not exist yet". */
function mts_is_missing_db(PDOException $e): bool {
    return in_array((string)$e->getCode(), ['3D000', '42000'], true)
        || stripos($e->getMessage(), 'Unknown database') !== false
        || stripos($e->getMessage(), '1049') !== false;
}

/**
 * PDO subclass that lets the application keep writing MySQL while the file
 * under it is SQLite. Every statement is translated by sql-map.php before it
 * reaches the driver; MySQL connections use plain PDO and are untouched.
 */
class MtsSqlitePdo extends PDO {
    public function __construct(string $dsn, ?string $user = null, ?string $pass = null, ?array $options = null) {
        parent::__construct($dsn, $user, $pass, $options ?? []);
        require_once __DIR__ . '/sql-map.php';
        mts_sqlite_init($this);
    }
    #[\ReturnTypeWillChange]
    public function prepare(string $query, array $options = []): PDOStatement|false {
        return parent::prepare(mts_sqlite_sql($query), $options);
    }
    #[\ReturnTypeWillChange]
    public function query(string $query, ?int $fetchMode = null, ...$fetchModeArgs) {
        return parent::query(mts_sqlite_sql($query), $fetchMode, ...$fetchModeArgs);
    }
    #[\ReturnTypeWillChange]
    public function exec(string $statement): int|false {
        return parent::exec(mts_sqlite_sql($statement));
    }
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
    /* Idempotent per connection: db() may try several candidates, and a failed
     * candidate must not poison the one that finally works. */
    static $done = [];
    $key = spl_object_id($pdo);
    if (isset($done[$key])) return;

    require_once __DIR__ . '/schema.php';   // defines mts_schema_sql()
    $sqlite = $pdo->getAttribute(PDO::ATTR_DRIVER_NAME) === 'sqlite';
    $ddl = mts_schema_sql();
    if ($sqlite) {
        require_once __DIR__ . '/sql-map.php';
        $ddl = mts_sqlite_schema_sql($ddl);
    }
    $pdo->exec($ddl);
    mts_migrate_visitor_logs($pdo);   // top up visitor_logs on pre-existing installs
    mts_migrate_requests_service_details($pdo); // per-service request form answers
    mts_migrate_users_pin($pdo);      // account-verification PIN on pre-existing installs
    mts_migrate_team_i18n($pdo);      // bilingual role/bio on team profiles

    $count = (int)$pdo->query('SELECT count(*) FROM users')->fetchColumn();
    if ($count === 0) {
        require_once __DIR__ . '/seed.php'; // defines mts_seed(PDO $pdo)
        mts_seed($pdo);
    }

    /* payment details + price list (only fills blanks — admin edits survive) */
    require_once __DIR__ . '/settings-seed.php';
    mts_seed_settings($pdo);

    $done[$key] = true;
}
