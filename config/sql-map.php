<?php
declare(strict_types=1);

/**
 * SQLite compatibility layer.
 *
 * The application speaks MySQL: its DDL is MySQL, a handful of queries call
 * MySQL date functions, and the boot-time migrations read `SHOW COLUMNS`.
 * Rather than fork every query, a single translation shim sits between the
 * query helpers in db.php and PDO: when the connection is SQLite the SQL is
 * rewritten in place and MySQL-only expressions are registered as user
 * functions. MySQL connections pass through untouched, so a cPanel host that
 * does have MySQL keeps byte-for-byte the query it had before.
 *
 * Zero-config goal: a fresh clone that finds no MySQL URL simply creates
 * storage/mts.sqlite and runs.
 */

function mts_is_sqlite(PDO $pdo): bool {
    return $pdo->getAttribute(PDO::ATTR_DRIVER_NAME) === 'sqlite';
}

/* ------------------------------------------------------------------ *
 *  MySQL date functions emulated as SQLite UDFs so the query text does
 *  not have to change. Registering them also means standard MySQL
 *  snippets keep working (a timezone-independent "now" is good enough
 *  for a single-server deployment; PHP still writes explicit timestamps
 *  for created_at via dt()).
 * ------------------------------------------------------------------ */
function mts_sqlite_functions(PDO $pdo): void {
    $pdo->sqliteCreateFunction('now',            fn() => date('Y-m-d H:i:s'), 0);
    $pdo->sqliteCreateFunction('curdate',        fn() => date('Y-m-d'), 0);
    $pdo->sqliteCreateFunction('date_add',       fn($dt, $mod) => mts_sqlite_shift($dt, $mod, +1), 2);
    $pdo->sqliteCreateFunction('date_sub',       fn($dt, $mod) => mts_sqlite_shift($dt, $mod, -1), 2);
    $pdo->sqliteCreateFunction('date_format',    fn($dt, $fmt) => mts_sqlite_format($dt, $fmt), 2);
    $pdo->sqliteCreateFunction('if',             fn($c, $a, $b) => $c ? $a : $b, 3);
    $pdo->sqliteCreateFunction('database',       fn() => 'main', 0);
}

/** '+N minute' / '-N day' style modifier used by sqlite's date() function. */
function mts_sqlite_shift(?string $datetime, ?string $modifier, int $sign): string {
    $datetime = (string)$datetime;
    $modifier = trim((string)$modifier);
    if ($modifier === '') return $datetime;
    if ($modifier[0] !== '+' && $modifier[0] !== '-') $modifier = ($sign >= 0 ? '+' : '-') . $modifier;
    // SQLite understands "1 minute" but not "1 MINUTE" case-insensitively — it does; keep as-is.
    return date('Y-m-d H:i:s', strtotime($datetime . ' ' . $modifier) ?: time());
}

/** MySQL DATE_FORMAT -> PHP date() for the formats the app actually uses. */
function mts_sqlite_format(?string $datetime, ?string $fmt): string {
    $map = ['%Y' => 'Y', '%m' => 'm', '%d' => 'd', '%H' => 'H', '%i' => 'i', '%s' => 's', '%M' => 'F', '%D' => 'j', '%y' => 'y'];
    $php = strtr((string)$fmt, $map);
    return date($php, strtotime((string)$datetime) ?: time());
}

/* ------------------------------------------------------------------ *
 *  Query translation
 * ------------------------------------------------------------------ */

/** Rewrite a single MySQL statement into SQLite dialect. */
function mts_sqlite_sql(string $sql): string {
    // SHOW COLUMNS FROM t  ->  PRAGMA table_info(t) aliased to MySQL keys.
    if (preg_match('/^\s*SHOW\s+COLUMNS\s+FROM\s+`?([A-Za-z0-9_]+)`?\s*;?\s*$/i', $sql, $m)) {
        return "SELECT name AS \"Field\", type AS \"Type\", CASE WHEN \"notnull\" = 0 THEN 'YES' ELSE 'NO' END AS \"Null\", dflt_value AS \"Default\", CASE WHEN pk = 1 THEN 'PRI' ELSE '' END AS \"Key\" FROM pragma_table_info('{$m[1]}')";
    }

    // information_schema.statistics index existence check -> sqlite_master.
    if (stripos($sql, 'information_schema.statistics') !== false) {
        preg_match("/table_name\s*=\s*'([^']+)'/i", $sql, $t);
        preg_match("/index_name\s*=\s*'([^']+)'/i", $sql, $i);
        $table = $t[1] ?? '';
        $index = $i[1] ?? '';
        return "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND tbl_name = '{$table}' AND name = '{$index}'";
    }

    // ALTER TABLE t ADD KEY idx (col)  ->  CREATE INDEX IF NOT EXISTS ...
    if (preg_match('/^\s*ALTER\s+TABLE\s+`?([A-Za-z0-9_]+)`?\s+ADD\s+(?:KEY|INDEX)\s+`?([A-Za-z0-9_]+)`?\s*\(([^)]+)\)/i', $sql, $m)) {
        return "CREATE INDEX IF NOT EXISTS {$m[2]} ON {$m[1]} (" . mts_sqlite_cols($m[3]) . ")";
    }

    // ALTER TABLE t ADD COLUMN def  ->  SQLite flavour of the same column.
    if (preg_match('/^\s*ALTER\s+TABLE\s+`?([A-Za-z0-9_]+)`?\s+ADD\s+COLUMN\s+(.+)$/is', $sql, $m)) {
        $def = mts_sqlite_column(trim(rtrim($m[2], ';')));
        $def = preg_replace('/\s+AFTER\s+`?[A-Za-z0-9_]+`?/i', '', $def); // SQLite has no AFTER
        return "ALTER TABLE {$m[1]} ADD COLUMN {$def}";
    }

    // INSERT IGNORE  ->  INSERT OR IGNORE
    $sql = preg_replace('/\bINSERT\s+IGNORE\b/i', 'INSERT OR IGNORE', $sql);

    // ON DUPLICATE KEY UPDATE col = VALUES(col)  ->  SQLite upsert.
    $sql = mts_sqlite_rewrite_upsert($sql);

    // datetime functions that SQLite has no parser for. Registered as UDFs,
    // but DATE_ADD(x, INTERVAL 10 MINUTE) hides its INTERVAL keyword which a
    // UDF argument cannot carry, so rewrite that form explicitly.
    $sql = preg_replace_callback(
        '/DATE_(ADD|SUB)\s*\(\s*([^,]+?)\s*,\s*INTERVAL\s+([0-9]+)\s+([A-Za-z]+)\s*\)/i',
        function ($m) {
            $unit = strtolower($m[4]);
            if ($unit === 'minutes' || $unit === 'minute') $unit = 'minute';
            elseif ($unit === 'hours' || $unit === 'hour') $unit = 'hour';
            elseif ($unit === 'days' || $unit === 'day') $unit = 'day';
            $sign = strtoupper($m[1]) === 'SUB' ? '-' : '+';
            return "datetime(" . trim($m[2]) . ", '{$sign}{$m[3]} {$unit}')";
        },
        $sql
    );

    return $sql;
}

/** `INSERT ... ON DUPLICATE KEY UPDATE a = VALUES(a), b = b` -> `INSERT ... ON CONFLICT DO UPDATE SET a = excluded.a, b = b`. */
function mts_sqlite_rewrite_upsert(string $sql): string {
    $pos = stripos($sql, 'ON DUPLICATE KEY UPDATE');
    if ($pos === false) return $sql;

    $head = substr($sql, 0, $pos);
    $assign = trim(substr($sql, $pos + strlen('ON DUPLICATE KEY UPDATE')), " \t\n\r;");

    // `a = VALUES(a)` -> `a = excluded.a`
    $assign = preg_replace('/\bVALUES\s*\(\s*([A-Za-z0-9_]+)\s*\)/i', 'excluded.$1', $assign);

    // The conflict target is the primary key; SQLite accepts a bare
    // ON CONFLICT when the INSERT itself would violate a UNIQUE/PK index.
    return $head . ' ON CONFLICT DO UPDATE SET ' . $assign;
}

/** Ensure a boot-bookkeeping table exists in the sqlite database. */
function mts_sqlite_init(PDO $pdo): void {
    mts_sqlite_functions($pdo);
    $pdo->exec('PRAGMA journal_mode = WAL');
    $pdo->exec('PRAGMA foreign_keys = ON');
    $pdo->exec('PRAGMA busy_timeout = 5000');
}

/* ------------------------------------------------------------------ *
 *  DDL generation (sqlite flavour of config/schema.php)
 * ------------------------------------------------------------------ */

/** Translate the MySQL schema DDL into SQLite. */
function mts_sqlite_schema_sql(string $mysqlDdl): string {
    $out = [];
    foreach (preg_split('/;\s*(?:\r?\n|$)/', $mysqlDdl) as $stmt) {
        $stmt = trim($stmt);
        if ($stmt === '') continue;
        if (stripos($stmt, 'CREATE TABLE') !== 0) continue;

        // Drop the MySQL table options tail.
        $stmt = preg_replace('/\)\s*ENGINE=[^;]*$/i', ')', $stmt);

        $open = strpos($stmt, '(');
        $body = substr($stmt, $open + 1);
        $body = preg_replace('/\)\s*$/', '', $body); // drop the table-closing paren
        $lines = preg_split('/\r?\n/', $body);
        $cols = [];
        foreach ($lines as $line) {
            $line = trim($line, " \t,");
            if ($line === '') continue;
            // MySQL index lines: keep the uniqueness, drop the rest.
            if (preg_match('/^UNIQUE\s+KEY\s+`?[A-Za-z0-9_]+`?\s*\(([^)]+)\)/i', $line, $m)) {
                $cols[] = 'UNIQUE (' . mts_sqlite_cols($m[1]) . ')';
                continue;
            }
            if (preg_match('/^(KEY|INDEX|CONSTRAINT|FOREIGN\s+KEY|FULLTEXT|SPATIAL|PRIMARY\s+KEY)/i', $line)) {
                continue;
            }
            $cols[] = mts_sqlite_column($line);
        }

        // Preserve the primary key: SQLite keeps it as a column constraint.
        // Skip it when a column is already INTEGER PRIMARY KEY AUTOINCREMENT,
        // which SQLite rejects as a second primary key.
        $hasAutoPk = (bool)preg_grep('/AUTOINCREMENT/i', $cols);
        if (!$hasAutoPk) {
            foreach ($lines as $line) {
                $line = trim($line, " \t,");
                if (preg_match('/^PRIMARY\s+KEY\s*\(\s*`?([A-Za-z0-9_]+)`?\s*\)/i', $line, $m)) {
                    $cols[] = "PRIMARY KEY ({$m[1]})";
                }
            }
        }

        $name = null;
        if (preg_match('/CREATE\s+TABLE\s+IF\s+NOT\s+EXISTS\s+`?([A-Za-z0-9_]+)`?/i', $stmt, $m)) {
            $name = $m[1];
        }
        if ($name === null) continue;

        $out[] = "CREATE TABLE IF NOT EXISTS {$name} (\n  " . implode(",\n  ", $cols) . "\n)";
    }
    return implode(";\n\n", $out) . ';';
}

/** `a, b(11)` -> "a, b" for a UNIQUE(...) clause (strip index prefix lengths). */
function mts_sqlite_cols(string $list): string {
    $parts = array_map(fn($c) => trim(preg_replace('/\(\d+\)?/', '', trim($c)), ' `'), explode(',', $list));
    return implode(', ', array_filter($parts, fn($c) => $c !== ''));
}

/** Translate one column definition line. */
function mts_sqlite_column(string $line): string {
    if (stripos($line, 'AUTO_INCREMENT') !== false) {
        $name = preg_split('/\s+/', $line)[0];
        return trim($name, '`') . ' INTEGER PRIMARY KEY AUTOINCREMENT';
    }
    // TINYINT / BIGINT / INT -> INTEGER ; JSON -> TEXT
    $line = preg_replace('/\b(TINYINT|BIGINT|SMALLINT|MEDIUMINT)\b(\s*\(\d+\))?/i', 'INTEGER', $line);
    $line = preg_replace('/\bJSON\b/i', 'TEXT', $line);
    // SQLite ignores VARCHAR length but tolerates it.
    // Drop MySQL's ON UPDATE CURRENT_TIMESTAMP.
    $line = preg_replace('/\s+ON\s+UPDATE\s+CURRENT_TIMESTAMP/i', '', $line);
    // DEFAULT CURRENT_TIMESTAMP is valid in SQLite.
    return $line;
}

/** Add the SQLite-only DEFAULT for ON UPDATE columns (none required) — placeholder for future use. */
function mts_sqlite_ddl_ready(): bool { return true; }