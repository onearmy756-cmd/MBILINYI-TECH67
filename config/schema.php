<?php
declare(strict_types=1);

/**
 * MySQL (InnoDB, utf8mb4) schema — idempotent, safe to run on every boot.
 *
 * Keep every statement starting with CREATE TABLE and add no comments inside
 * the heredoc: mts_sqlite_schema_sql() walks the text statement by statement
 * and skips anything that does not begin with CREATE TABLE.
 *
 * `webauthn_credentials` holds one passkey per row so a user can keep several
 * (phone, laptop, security key) and revoke one without touching the rest.
 * `public_key` is the PEM of the COSE key the authenticator handed us and
 * `sign_count` guards against cloned authenticators.
 */
function mts_schema_sql(): string {
    return <<<'SQL'
CREATE TABLE IF NOT EXISTS users (
  id            VARCHAR(64)  NOT NULL,
  name          VARCHAR(160) NOT NULL,
  email         VARCHAR(190) NOT NULL,
  phone         VARCHAR(60)  NOT NULL DEFAULT '',
  company       VARCHAR(160) NOT NULL DEFAULT '',
  password_hash VARCHAR(255) NOT NULL,
  pin_hash      VARCHAR(255) NULL,
  pin_fails     INT          NOT NULL DEFAULT 0,
  pin_locked_until DATETIME NULL,
  role          VARCHAR(20)  NOT NULL DEFAULT 'client',
  status        VARCHAR(20)  NOT NULL DEFAULT 'Active',
  created_at    DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id),
  UNIQUE KEY uq_users_email (email)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS requests (
  id          VARCHAR(64)   NOT NULL,
  tracking_id VARCHAR(32)   NOT NULL,
  user_id     VARCHAR(64)   NOT NULL DEFAULT '',
  name        VARCHAR(160)  NOT NULL DEFAULT '',
  email       VARCHAR(190)  NOT NULL DEFAULT '',
  phone       VARCHAR(60)   NOT NULL DEFAULT '',
  company     VARCHAR(160)  NOT NULL DEFAULT '',
  service     VARCHAR(160)  NOT NULL DEFAULT '',
  title       VARCHAR(255)  NOT NULL DEFAULT '',
  description TEXT          NOT NULL,
  tech        JSON          NULL,
  service_details JSON       NULL,
  platform    VARCHAR(120)  NOT NULL DEFAULT '',
  budget      VARCHAR(120)  NOT NULL DEFAULT '',
  deadline    VARCHAR(40)   NOT NULL DEFAULT '',
  priority    VARCHAR(40)   NOT NULL DEFAULT 'Normal',
  status      VARCHAR(40)   NOT NULL DEFAULT 'Pending',
  admin_note  TEXT          NOT NULL,
  created_at  DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id),
  UNIQUE KEY uq_requests_tracking (tracking_id),
  KEY idx_requests_user (user_id),
  KEY idx_requests_email (email)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS quotes (
  id            VARCHAR(64)   NOT NULL,
  request_id    VARCHAR(64)   NOT NULL,
  tracking_id   VARCHAR(32)   NOT NULL DEFAULT '',
  user_id       VARCHAR(64)   NOT NULL DEFAULT '',
  items         JSON          NULL,
  subtotal      DECIMAL(14,2) NOT NULL DEFAULT 0,
  vat           DECIMAL(14,2) NOT NULL DEFAULT 0,
  total         DECIMAL(14,2) NOT NULL DEFAULT 0,
  currency      VARCHAR(16)   NOT NULL DEFAULT 'TZS',
  valid_until   DATETIME      NULL,
  status        VARCHAR(30)   NOT NULL DEFAULT 'Sent',
  admin_message TEXT          NOT NULL,
  created_at    DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id),
  KEY idx_quotes_request (request_id),
  KEY idx_quotes_user (user_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS contracts (
  id            VARCHAR(64)   NOT NULL,
  request_id    VARCHAR(64)   NOT NULL,
  quote_id      VARCHAR(64)   NOT NULL DEFAULT '',
  tracking_id   VARCHAR(32)   NOT NULL DEFAULT '',
  client_name   VARCHAR(160)  NOT NULL DEFAULT '',
  project_title VARCHAR(255)  NOT NULL DEFAULT '',
  scope         TEXT          NOT NULL,
  duration      VARCHAR(60)   NOT NULL DEFAULT '45 days',
  total         DECIMAL(14,2) NOT NULL DEFAULT 0,
  status        VARCHAR(30)   NOT NULL DEFAULT 'Sent',
  client_sig    TEXT          NULL,
  client_signed_at DATETIME   NULL,
  admin_sig     TEXT          NULL,
  admin_signed_at  DATETIME   NULL,
  created_at    DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id),
  KEY idx_contract_req (request_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS invoices (
  id          VARCHAR(64)   NOT NULL,
  tracking_id VARCHAR(32)   NOT NULL DEFAULT '',
  user_id     VARCHAR(64)   NOT NULL DEFAULT '',
  title       VARCHAR(255)  NOT NULL DEFAULT '',
  amount      DECIMAL(14,2) NOT NULL DEFAULT 0,
  status      VARCHAR(30)   NOT NULL DEFAULT 'Unpaid',
  method      VARCHAR(160)  NOT NULL DEFAULT 'M-Pesa / Bank',
  due_date    DATETIME      NULL,
  created_at  DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id),
  KEY idx_invoices_user (user_id),
  KEY idx_invoices_track (tracking_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS tickets (
  id         VARCHAR(64) NOT NULL,
  user_id    VARCHAR(64) NOT NULL DEFAULT '',
  name       VARCHAR(160) NOT NULL DEFAULT '',
  email      VARCHAR(190) NOT NULL DEFAULT '',
  subject    VARCHAR(255) NOT NULL DEFAULT '',
  message    TEXT        NOT NULL,
  status     VARCHAR(30) NOT NULL DEFAULT 'Open',
  created_at DATETIME    NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id),
  KEY idx_tickets_user (user_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS ticket_replies (
  id         VARCHAR(64) NOT NULL,
  ticket_id  VARCHAR(64) NOT NULL,
  by_name    VARCHAR(160) NOT NULL DEFAULT '',
  body       TEXT        NOT NULL,
  created_at DATETIME    NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id),
  KEY idx_replies_ticket (ticket_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS otp_codes (
  id         VARCHAR(64)  NOT NULL,
  user_id    VARCHAR(64)  NOT NULL DEFAULT '',
  email      VARCHAR(190) NOT NULL DEFAULT '',
  purpose    VARCHAR(30)  NOT NULL DEFAULT 'login',
  code_hash  VARCHAR(255) NOT NULL,
  expires_at DATETIME     NOT NULL,
  used_at    DATETIME     NULL,
  created_at DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id),
  KEY idx_otp_user (user_id),
  KEY idx_otp_email (email),
  KEY idx_otp_expires (expires_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS password_resets (
  id         VARCHAR(64)  NOT NULL,
  email      VARCHAR(190) NOT NULL,
  token_hash VARCHAR(255) NOT NULL,
  expires_at DATETIME     NOT NULL,
  used_at    DATETIME     NULL,
  created_at DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id),
  KEY idx_pr_email (email),
  KEY idx_pr_expires (expires_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS portfolio_projects (
  id          VARCHAR(64)   NOT NULL,
  title       VARCHAR(255)  NOT NULL,
  category    VARCHAR(160)  NOT NULL DEFAULT '',
  description VARCHAR(500)  NOT NULL DEFAULT '',
  image_url   VARCHAR(500)  NOT NULL DEFAULT '',
  stat_label  VARCHAR(100)  NOT NULL DEFAULT '',
  link_url    VARCHAR(500)  NOT NULL DEFAULT '',
  sort_order  INT           NOT NULL DEFAULT 0,
  is_active   TINYINT(1)    NOT NULL DEFAULT 1,
  created_at  DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at  DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (id),
  KEY idx_pp_active (is_active),
  KEY idx_pp_sort (sort_order)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS testimonials (
  id         VARCHAR(64)   NOT NULL,
  name       VARCHAR(160)  NOT NULL,
  role       VARCHAR(255)  NOT NULL DEFAULT '',
  text       TEXT          NOT NULL,
  stars      INT           NOT NULL DEFAULT 5,
  sort_order INT           NOT NULL DEFAULT 0,
  is_active  TINYINT(1)    NOT NULL DEFAULT 1,
  created_at DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (id),
  KEY idx_tm_active (is_active),
  KEY idx_tm_sort (sort_order)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS settings (
  skey       VARCHAR(80)   NOT NULL,
  svalue     TEXT          NOT NULL,
  updated_at DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (skey)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS news_posts (
  id         VARCHAR(64)   NOT NULL,
  title      VARCHAR(255)  NOT NULL,
  category   VARCHAR(80)   NOT NULL DEFAULT 'Announcement',
  excerpt    VARCHAR(500)  NOT NULL DEFAULT '',
  body       TEXT          NOT NULL,
  image_url  VARCHAR(500)  NOT NULL DEFAULT '',
  is_active  TINYINT(1)    NOT NULL DEFAULT 1,
  created_at DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (id),
  KEY idx_news_active (is_active, created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS team_members (
  id         VARCHAR(64)   NOT NULL,
  name       VARCHAR(160)  NOT NULL,
  role       VARCHAR(200)  NOT NULL DEFAULT '',
  role_sw    VARCHAR(200)  NOT NULL DEFAULT '',
  bio        VARCHAR(500)  NOT NULL DEFAULT '',
  bio_sw     VARCHAR(500)  NOT NULL DEFAULT '',
  image_url  VARCHAR(500)  NOT NULL DEFAULT '',
  email      VARCHAR(190)  NOT NULL DEFAULT '',
  phone      VARCHAR(60)   NOT NULL DEFAULT '',
  linkedin   VARCHAR(500)  NOT NULL DEFAULT '',
  sort_order INT           NOT NULL DEFAULT 0,
  is_active  TINYINT(1)    NOT NULL DEFAULT 1,
  created_at DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (id),
  KEY idx_team_active (is_active, sort_order)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS pricing_plans (
  id         VARCHAR(64)   NOT NULL,
  code       VARCHAR(64)   NOT NULL,
  group_name VARCHAR(40)   NOT NULL DEFAULT 'maint',
  name       VARCHAR(160)  NOT NULL,
  price      DECIMAL(14,2) NOT NULL DEFAULT 0,
  currency   VARCHAR(16)   NOT NULL DEFAULT 'TZS',
  period     VARCHAR(40)   NOT NULL DEFAULT '',
  features   TEXT          NULL,
  is_active  TINYINT(1)    NOT NULL DEFAULT 1,
  sort_order INT           NOT NULL DEFAULT 0,
  created_at DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (id),
  UNIQUE KEY uq_pp_code (code),
  KEY idx_pp_group (group_name, is_active)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS feedback (
  id         VARCHAR(64)  NOT NULL,
  name       VARCHAR(160) NOT NULL DEFAULT '',
  email      VARCHAR(190) NOT NULL DEFAULT '',
  category   VARCHAR(60)  NOT NULL DEFAULT 'general',
  subject    VARCHAR(255) NOT NULL DEFAULT '',
  message    TEXT         NOT NULL,
  rating     TINYINT      NOT NULL DEFAULT 0,
  status     VARCHAR(30)  NOT NULL DEFAULT 'New',
  created_at DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id),
  KEY idx_fb_status (status),
  KEY idx_fb_email (email)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS visitor_logs (
  id         BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
  visit_date DATE            NOT NULL,
  ip_hash    VARCHAR(128)    NOT NULL DEFAULT '',
  user_agent VARCHAR(500)    NOT NULL DEFAULT '',
  page_path  VARCHAR(500)    NOT NULL DEFAULT '',
  page_url   VARCHAR(500)    NOT NULL DEFAULT '',
  referer    VARCHAR(500)    NOT NULL DEFAULT '',
  user_id    BIGINT UNSIGNED NULL,
  session_id VARCHAR(120)    NOT NULL DEFAULT '',
  visited_at DATETIME        NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id),
  KEY idx_vl_visit_date (visit_date),
  KEY idx_vl_visited (visited_at),
  KEY idx_vl_session (session_id),
  KEY idx_vl_page (page_path(191))
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS webauthn_credentials (
  id            VARCHAR(64)   NOT NULL,
  user_id       VARCHAR(64)   NOT NULL,
  credential_id VARCHAR(512)  NOT NULL,
  public_key    TEXT          NOT NULL,
  sign_count    BIGINT        NOT NULL DEFAULT 0,
  transports    VARCHAR(120)  NOT NULL DEFAULT '',
  label         VARCHAR(120)  NOT NULL DEFAULT '',
  aaguid        VARCHAR(64)   NOT NULL DEFAULT '',
  created_at    DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  last_used_at  DATETIME      NULL,
  PRIMARY KEY (id),
  UNIQUE KEY uq_wac_cred (credential_id(190)),
  KEY idx_wac_user (user_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
SQL;
}

/**
 * Idempotent migration for `visitor_logs`.
 *
 * The reporting code (record_visitor / dashboard visitor_stats) reads
 * `visit_date` + `page_path` + `user_id`, while the original table shipped
 * with `page_url` + `visited_at`. Existing installs created before those
 * columns existed would silently lose every page view, so we top the table up
 * with whichever columns are still missing instead of dropping data.
 */
function mts_migrate_visitor_logs(PDO $pdo): void {
    static $done = false;
    if ($done) return;
    $done = true;

    try {
        $have = [];
        foreach ($pdo->query('SHOW COLUMNS FROM visitor_logs')->fetchAll(PDO::FETCH_ASSOC) as $r) {
            $have[strtolower((string)$r['Field'])] = true;
        }
    } catch (\Throwable $e) {
        error_log('[schema] visitor_logs unreadable: ' . $e->getMessage());
        return;
    }

    $add = [
        'visit_date' => "ALTER TABLE visitor_logs ADD COLUMN visit_date DATE NOT NULL DEFAULT '1970-01-01'",
        'page_path'  => "ALTER TABLE visitor_logs ADD COLUMN page_path VARCHAR(500) NOT NULL DEFAULT ''",
        'user_id'    => 'ALTER TABLE visitor_logs ADD COLUMN user_id BIGINT UNSIGNED NULL',
        'page_url'   => "ALTER TABLE visitor_logs ADD COLUMN page_url VARCHAR(500) NOT NULL DEFAULT ''",
        'visited_at' => 'ALTER TABLE visitor_logs ADD COLUMN visited_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP',
        'ip_hash'    => "ALTER TABLE visitor_logs ADD COLUMN ip_hash VARCHAR(128) NOT NULL DEFAULT ''",
        'user_agent' => "ALTER TABLE visitor_logs ADD COLUMN user_agent VARCHAR(500) NOT NULL DEFAULT ''",
        'referer'    => "ALTER TABLE visitor_logs ADD COLUMN referer VARCHAR(500) NOT NULL DEFAULT ''",
        'session_id' => "ALTER TABLE visitor_logs ADD COLUMN session_id VARCHAR(120) NOT NULL DEFAULT ''",
    ];
    foreach ($add as $col => $sql) {
        if (isset($have[$col])) continue;
        try { $pdo->exec($sql); } catch (\Throwable $e) { error_log('[schema] add ' . $col . ': ' . $e->getMessage()); }
    }

    try {
        $n = (int)$pdo->query("SELECT COUNT(*) FROM information_schema.statistics
            WHERE table_schema = DATABASE() AND table_name = 'visitor_logs' AND index_name = 'idx_vl_visit_date'")->fetchColumn();
        if ($n === 0) $pdo->exec('ALTER TABLE visitor_logs ADD KEY idx_vl_visit_date (visit_date)');
    } catch (\Throwable $e) {
        error_log('[schema] visitor_logs index: ' . $e->getMessage());
    }
}

/**
 * Idempotent migration for `requests.service_details`.
 *
 * Per-service request forms submit their own key/value answers; we store them
 * as JSON on the existing row so pre-migration installs keep working (the
 * column is simply absent until this runs).
 */
function mts_migrate_requests_service_details(PDO $pdo): void {
    static $done = false;
    if ($done) return;
    $done = true;

    try {
        $have = [];
        foreach ($pdo->query('SHOW COLUMNS FROM requests')->fetchAll(PDO::FETCH_ASSOC) as $r) {
            $have[strtolower((string)$r['Field'])] = true;
        }
    } catch (\Throwable $e) {
        error_log('[schema] requests unreadable: ' . $e->getMessage());
        return;
    }

    if (!isset($have['service_details'])) {
        try { $pdo->exec('ALTER TABLE requests ADD COLUMN service_details JSON NULL AFTER tech'); }
        catch (\Throwable $e) { error_log('[schema] add service_details: ' . $e->getMessage()); }
    }
}

/**
 * Idempotent migration for `users.pin_hash`.
 *
 * Account verification uses a short PIN (or the phone-derived pattern) as a
 * second factor. Pre-existing installs created before the column existed must
 * gain it, otherwise verification would fail on an unknown column.
 */
function mts_migrate_users_pin(PDO $pdo): void {
    static $done = false;
    if ($done) return;
    $done = true;

    try {
        $have = [];
        foreach ($pdo->query('SHOW COLUMNS FROM users')->fetchAll(PDO::FETCH_ASSOC) as $r) {
            $have[strtolower((string)$r['Field'])] = true;
        }
    } catch (\Throwable $e) {
        error_log('[schema] users unreadable: ' . $e->getMessage());
        return;
    }

    $want = [
        'pin_hash'         => 'ALTER TABLE users ADD COLUMN pin_hash VARCHAR(255) NULL AFTER password_hash',
        'pin_fails'        => 'ALTER TABLE users ADD COLUMN pin_fails INT NOT NULL DEFAULT 0 AFTER pin_hash',
        'pin_locked_until' => 'ALTER TABLE users ADD COLUMN pin_locked_until DATETIME NULL AFTER pin_fails',
    ];
    foreach ($want as $col => $sql) {
        if (isset($have[$col])) continue;
        try { $pdo->exec($sql); }
        catch (\Throwable $e) { error_log('[schema] add users.' . $col . ': ' . $e->getMessage()); }
    }
}

/**
 * Idempotent migration for bilingual team profiles.
 *
 * A profile's role and bio are shown on both the English and the Kiswahili
 * page, so each needs its own column; the English column is the fallback when
 * no translation was supplied.
 */
function mts_migrate_team_i18n(PDO $pdo): void {
    static $done = false;
    if ($done) return;
    $done = true;

    try {
        $have = [];
        foreach ($pdo->query('SHOW COLUMNS FROM team_members')->fetchAll(PDO::FETCH_ASSOC) as $r) {
            $have[strtolower((string)$r['Field'])] = true;
        }
    } catch (\Throwable $e) {
        error_log('[schema] team_members unreadable: ' . $e->getMessage());
        return;
    }

    $want = [
        'role_sw' => "ALTER TABLE team_members ADD COLUMN role_sw VARCHAR(200) NOT NULL DEFAULT '' AFTER role",
        'bio_sw'  => "ALTER TABLE team_members ADD COLUMN bio_sw VARCHAR(500) NOT NULL DEFAULT '' AFTER bio",
    ];
    foreach ($want as $col => $sql) {
        if (isset($have[$col])) continue;
        try { $pdo->exec($sql); }
        catch (\Throwable $e) { error_log('[schema] add team_members.' . $col . ': ' . $e->getMessage()); }
    }
}
