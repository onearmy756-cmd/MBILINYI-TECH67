// Generated from the live SQLite schema — the canonical shape the PHP
// backend created, so a Rust port reads and writes identical rows.
// Every statement is idempotent, so starting against an existing database
// is safe.
pub const SCHEMA_SQL: &str = r#"CREATE TABLE IF NOT EXISTS contracts (
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
  PRIMARY KEY (id)
);
CREATE TABLE IF NOT EXISTS feedback (
  id         VARCHAR(64)  NOT NULL,
  name       VARCHAR(160) NOT NULL DEFAULT '',
  email      VARCHAR(190) NOT NULL DEFAULT '',
  category   VARCHAR(60)  NOT NULL DEFAULT 'general',
  subject    VARCHAR(255) NOT NULL DEFAULT '',
  message    TEXT         NOT NULL,
  rating     INTEGER      NOT NULL DEFAULT 0,
  status     VARCHAR(30)  NOT NULL DEFAULT 'New',
  created_at DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id)
);
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
  PRIMARY KEY (id)
);
CREATE TABLE IF NOT EXISTS news_posts (
  id         VARCHAR(64)   NOT NULL,
  title      VARCHAR(255)  NOT NULL,
  category   VARCHAR(80)   NOT NULL DEFAULT 'Announcement',
  excerpt    VARCHAR(500)  NOT NULL DEFAULT '',
  body       TEXT          NOT NULL,
  image_url  VARCHAR(500)  NOT NULL DEFAULT '',
  is_active  INTEGER    NOT NULL DEFAULT 1,
  created_at DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id)
);
CREATE TABLE IF NOT EXISTS otp_codes (
  id         VARCHAR(64)  NOT NULL,
  user_id    VARCHAR(64)  NOT NULL DEFAULT '',
  email      VARCHAR(190) NOT NULL DEFAULT '',
  purpose    VARCHAR(30)  NOT NULL DEFAULT 'login',
  code_hash  VARCHAR(255) NOT NULL,
  expires_at DATETIME     NOT NULL,
  used_at    DATETIME     NULL,
  created_at DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id)
);
CREATE TABLE IF NOT EXISTS password_resets (
  id         VARCHAR(64)  NOT NULL,
  email      VARCHAR(190) NOT NULL,
  token_hash VARCHAR(255) NOT NULL,
  expires_at DATETIME     NOT NULL,
  used_at    DATETIME     NULL,
  created_at DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id)
);
CREATE TABLE IF NOT EXISTS portfolio_projects (
  id          VARCHAR(64)   NOT NULL,
  title       VARCHAR(255)  NOT NULL,
  category    VARCHAR(160)  NOT NULL DEFAULT '',
  description VARCHAR(500)  NOT NULL DEFAULT '',
  image_url   VARCHAR(500)  NOT NULL DEFAULT '',
  stat_label  VARCHAR(100)  NOT NULL DEFAULT '',
  link_url    VARCHAR(500)  NOT NULL DEFAULT '',
  sort_order  INT           NOT NULL DEFAULT 0,
  is_active   INTEGER    NOT NULL DEFAULT 1,
  created_at  DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at  DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id)
);
CREATE TABLE IF NOT EXISTS pricing_plans (
  id         VARCHAR(64)   NOT NULL,
  code       VARCHAR(64)   NOT NULL,
  group_name VARCHAR(40)   NOT NULL DEFAULT 'maint',
  name       VARCHAR(160)  NOT NULL,
  price      DECIMAL(14,2) NOT NULL DEFAULT 0,
  currency   VARCHAR(16)   NOT NULL DEFAULT 'TZS',
  period     VARCHAR(40)   NOT NULL DEFAULT '',
  features   TEXT          NULL,
  is_active  INTEGER    NOT NULL DEFAULT 1,
  sort_order INT           NOT NULL DEFAULT 0,
  created_at DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (code),
  PRIMARY KEY (id)
);
CREATE TABLE IF NOT EXISTS quotes (
  id            VARCHAR(64)   NOT NULL,
  request_id    VARCHAR(64)   NOT NULL,
  tracking_id   VARCHAR(32)   NOT NULL DEFAULT '',
  user_id       VARCHAR(64)   NOT NULL DEFAULT '',
  items         TEXT          NULL,
  subtotal      DECIMAL(14,2) NOT NULL DEFAULT 0,
  vat           DECIMAL(14,2) NOT NULL DEFAULT 0,
  total         DECIMAL(14,2) NOT NULL DEFAULT 0,
  currency      VARCHAR(16)   NOT NULL DEFAULT 'TZS',
  valid_until   DATETIME      NULL,
  status        VARCHAR(30)   NOT NULL DEFAULT 'Sent',
  admin_message TEXT          NOT NULL,
  created_at    DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id)
);
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
  tech        TEXT          NULL,
  service_details TEXT       NULL,
  platform    VARCHAR(120)  NOT NULL DEFAULT '',
  budget      VARCHAR(120)  NOT NULL DEFAULT '',
  deadline    VARCHAR(40)   NOT NULL DEFAULT '',
  priority    VARCHAR(40)   NOT NULL DEFAULT 'Normal',
  status      VARCHAR(40)   NOT NULL DEFAULT 'Pending',
  admin_note  TEXT          NOT NULL,
  created_at  DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE (tracking_id),
  PRIMARY KEY (id)
);
CREATE TABLE IF NOT EXISTS settings (
  skey       VARCHAR(80)   NOT NULL,
  svalue     TEXT          NOT NULL,
  updated_at DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (skey)
);
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
  is_active  INTEGER    NOT NULL DEFAULT 1,
  created_at DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id)
);
CREATE TABLE IF NOT EXISTS testimonials (
  id         VARCHAR(64)   NOT NULL,
  name       VARCHAR(160)  NOT NULL,
  role       VARCHAR(255)  NOT NULL DEFAULT '',
  text       TEXT          NOT NULL,
  stars      INT           NOT NULL DEFAULT 5,
  sort_order INT           NOT NULL DEFAULT 0,
  is_active  INTEGER    NOT NULL DEFAULT 1,
  created_at DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id)
);
CREATE TABLE IF NOT EXISTS ticket_replies (
  id         VARCHAR(64) NOT NULL,
  ticket_id  VARCHAR(64) NOT NULL,
  by_name    VARCHAR(160) NOT NULL DEFAULT '',
  body       TEXT        NOT NULL,
  created_at DATETIME    NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id)
);
CREATE TABLE IF NOT EXISTS tickets (
  id         VARCHAR(64) NOT NULL,
  user_id    VARCHAR(64) NOT NULL DEFAULT '',
  name       VARCHAR(160) NOT NULL DEFAULT '',
  email      VARCHAR(190) NOT NULL DEFAULT '',
  subject    VARCHAR(255) NOT NULL DEFAULT '',
  message    TEXT        NOT NULL,
  status     VARCHAR(30) NOT NULL DEFAULT 'Open',
  created_at DATETIME    NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id)
);
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
  UNIQUE (email),
  PRIMARY KEY (id)
);
CREATE TABLE IF NOT EXISTS visitor_logs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  visit_date DATE            NOT NULL,
  ip_hash    VARCHAR(128)    NOT NULL DEFAULT '',
  user_agent VARCHAR(500)    NOT NULL DEFAULT '',
  page_path  VARCHAR(500)    NOT NULL DEFAULT '',
  page_url   VARCHAR(500)    NOT NULL DEFAULT '',
  referer    VARCHAR(500)    NOT NULL DEFAULT '',
  user_id    INTEGER UNSIGNED NULL,
  session_id VARCHAR(120)    NOT NULL DEFAULT '',
  visited_at DATETIME        NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE IF NOT EXISTS webauthn_credentials (
  id            VARCHAR(64)   NOT NULL,
  user_id       VARCHAR(64)   NOT NULL,
  credential_id VARCHAR(512)  NOT NULL,
  public_key    TEXT          NOT NULL,
  sign_count    INTEGER        NOT NULL DEFAULT 0,
  transports    VARCHAR(120)  NOT NULL DEFAULT '',
  label         VARCHAR(120)  NOT NULL DEFAULT '',
  aaguid        VARCHAR(64)   NOT NULL DEFAULT '',
  created_at    DATETIME      NOT NULL DEFAULT CURRENT_TIMESTAMP,
  last_used_at  DATETIME      NULL,
  UNIQUE (credential_id),
  PRIMARY KEY (id)
);
CREATE INDEX IF NOT EXISTS idx_vl_visit_date ON visitor_logs (visit_date);
"#;
