<?php
declare(strict_types=1);
require_once __DIR__ . '/../config/app.php';
require_once __DIR__ . '/../lib/webauthn.php';

header('Content-Type: application/json; charset=utf-8');

/* ---------- account-verification helpers ----------
 * A signing-in user proves "this account is really mine" with EITHER the
 * 6-digit code emailed to them, OR a short PIN they chose at registration,
 * OR — if they never set a PIN — the last 4 digits of the phone on file
 * ("pattern ya simu"). All three are compared in constant time and the PIN
 * path is rate-limited so it cannot be brute-forced. */

function mts_pin_digits(string $raw): string {
    return preg_replace('/\D+/', '', $raw) ?? '';
}

/** Tanzanian mobile numbers: 0712345678, 255712345678 or +255712345678. */
function mts_phone_valid(string $phone): bool {
    $d = mts_pin_digits($phone);
    if (str_starts_with($d, '255')) $d = '0' . substr($d, 3);
    return (bool)preg_match('/^0[67]\d{8}$/', $d);
}

function mts_phone_tail(string $phone): string {
    $d = mts_pin_digits($phone);
    return strlen($d) >= 4 ? substr($d, -4) : '';
}

/** Verify a submitted PIN against the stored hash or the phone tail. Returns [ok, reason]. */
function mts_check_pin(array $row, string $pin): array {
    $pin = mts_pin_digits($pin);
    if ($pin === '' || strlen($pin) < 4 || strlen($pin) > 6) return [false, 'Enter a 4–6 digit PIN.'];
    if (!empty($row['pin_hash'])) {
        return password_verify($pin, (string)$row['pin_hash']) ? [true, ''] : [false, 'Wrong PIN. Try again.'];
    }
    $tail = mts_phone_tail((string)($row['phone'] ?? ''));
    if ($tail !== '' && hash_equals($tail, $pin)) return [true, ''];
    return [false, 'Wrong PIN. Try again.'];
}

try {
    switch (action_name()) {

        case 'login': {
            require_csrf();
            $email = strtolower(str_param('email'));
            $pass = (string)param('password', '');
            if ($email === '' || $pass === '') j_err('Fill email and password.');
            $row = db_one('SELECT * FROM users WHERE lower(email) = ?', [$email]);
            if (!$row || !password_verify($pass, $row['password_hash'])) {
                j_err('Invalid email or password.');
            }
            if ($row['status'] !== 'Active') {
                j_err('Account is suspended. Contact 0796 752 645.');
            }
            $code = str_pad((string)random_int(0, 999999), 6, '0', STR_PAD_LEFT);
            $pepper = $_ENV['MTS_OTP_PEPPER'] ?? 'MTS_OTP_2026_PEPPER_DAR';
            $codeHash = hash('sha256', $code . '|' . $email . '|' . $pepper);
            $otpId = new_id('OTP-');
            db_run('DELETE FROM otp_codes WHERE email = ? AND purpose = ?', [$email, 'login']);
            db_run(
                'INSERT INTO otp_codes (id, user_id, email, purpose, code_hash, expires_at, created_at)
                 VALUES (?,?,?,?,?, DATE_ADD(NOW(), INTERVAL 10 MINUTE), NOW())',
                [$otpId, $row['id'], $email, 'login', $codeHash]
            );
            try { notify_login_otp($email, $code); } catch (\Throwable $e) { error_log('[otp email] '.$e->getMessage()); }
            $hasPin = !empty($row['pin_hash']);
            $hasTail = mts_phone_tail((string)($row['phone'] ?? '')) !== '';
            j_ok([
                'otp_required' => true,
                'email' => $email,
                'pin_available' => ($hasPin || $hasTail),
                'pin_kind' => $hasPin ? 'pin' : ($hasTail ? 'phone' : ''),
                'otp_debug_only' => (function_exists('getenv') && getenv('OTP_SHOW_IN_RESPONSE')) ? $code : null,
            ]);
        }

        /* Verify identity with the user's PIN (or the last 4 digits of their phone
         * when no PIN was set). Rate-limited: 5 wrong tries locks the account for
         * 15 minutes so a short PIN cannot be brute-forced. */
        case 'verify-login-pin': {
            require_csrf();
            $email = strtolower(str_param('email'));
            $pin = (string)param('pin', '');
            if ($email === '' || $pin === '') j_err('Email and PIN are required.', 400);
            $row = db_one('SELECT * FROM users WHERE lower(email) = ?', [$email]);
            if (!$row) j_err('No account with this email.', 404);
            if ($row['status'] !== 'Active') j_err('Account is suspended. Contact 0796 752 645.', 403);

            if (!empty($row['pin_locked_until']) && strtotime((string)$row['pin_locked_until']) > time()) {
                $mins = max(1, (int)ceil((strtotime((string)$row['pin_locked_until']) - time()) / 60));
                j_err("Too many wrong PIN attempts. Try again in {$mins} minute(s), or use the emailed code.", 429);
            }

            [$ok, $reason] = mts_check_pin($row, $pin);
            if (!$ok) {
                $fails = (int)$row['pin_fails'] + 1;
                if ($fails >= 5) {
                    db_run('UPDATE users SET pin_fails = 0, pin_locked_until = DATE_ADD(NOW(), INTERVAL 15 MINUTE) WHERE id = ?', [$row['id']]);
                    j_err('Too many wrong PIN attempts. Your account is locked for 15 minutes — use the emailed code or try later.', 429);
                }
                db_run('UPDATE users SET pin_fails = ? WHERE id = ?', [$fails, $row['id']]);
                j_err($reason . ' ' . (5 - $fails) . ' attempt(s) left before a 15-minute lock.', 400);
            }

            db_run('UPDATE users SET pin_fails = 0, pin_locked_until = NULL WHERE id = ?', [$row['id']]);
            login_user($row['id']);
            j_ok(['user' => map_user($row)]);
        }

        /* Set or change the account PIN. Requires an active session (or a valid
         * login OTP) so a PIN cannot be attached to an account by a stranger. */
        case 'set-pin': {
            require_csrf();
            $pin = (string)param('pin', '');
            $pin2 = (string)param('pin2', '');
            $digits = mts_pin_digits($pin);
            if (strlen($digits) < 4 || strlen($digits) > 6) j_err('PIN must be 4–6 digits.', 400);
            if ($pin2 !== '' && mts_pin_digits($pin2) !== $digits) j_err('PINs do not match.', 400);

            $u = current_user();
            if ($u) {
                db_run('UPDATE users SET pin_hash = ?, pin_fails = 0, pin_locked_until = NULL WHERE id = ?',
                    [password_hash($digits, PASSWORD_DEFAULT), $u['id']]);
                j_ok(['updated' => true, 'user' => current_user()]);
            }

            /* no session yet — allow setting a PIN right after proving ownership via a login OTP */
            $email = strtolower(str_param('email'));
            $code = trim((string)param('code', ''));
            $otp = $email !== '' && $code !== ''
                ? db_one('SELECT * FROM otp_codes WHERE email = ? AND purpose = ? AND used_at IS NULL ORDER BY created_at DESC LIMIT 1', [$email, 'login'])
                : null;
            if ($otp) {
                $pepper = $_ENV['MTS_OTP_PEPPER'] ?? 'MTS_OTP_2026_PEPPER_DAR';
                $expected = hash('sha256', $code . '|' . $email . '|' . $pepper);
                if (strtotime((string)$otp['expires_at']) >= time() && hash_equals($expected, (string)$otp['code_hash'])) {
                    $row = db_one('SELECT * FROM users WHERE lower(email) = ?', [$email]);
                    if ($row) {
                        db_run('UPDATE users SET pin_hash = ?, pin_fails = 0, pin_locked_until = NULL WHERE id = ?',
                            [password_hash($digits, PASSWORD_DEFAULT), $row['id']]);
                        db_run('UPDATE otp_codes SET used_at = NOW() WHERE id = ?', [$otp['id']]);
                        login_user($row['id']);
                        j_ok(['updated' => true, 'user' => map_user($row)]);
                    }
                }
            }
            j_err('You need to sign in before setting a PIN.', 401);
        }

        case 'verify-login-otp': {
            require_csrf();
            $email = strtolower(str_param('email'));
            $code = trim((string)param('code', ''));
            if ($email === '' || $code === '') j_err('Email and code are required.', 400);
            $otp = db_one('SELECT * FROM otp_codes WHERE email = ? AND purpose = ? AND used_at IS NULL ORDER BY created_at DESC LIMIT 1', [$email, 'login']);
            if (!$otp) j_err('No pending OTP for this email. Sign in again.', 404);
            $pepper = $_ENV['MTS_OTP_PEPPER'] ?? 'MTS_OTP_2026_PEPPER_DAR';
            $expected = hash('sha256', $code . '|' . $email . '|' . $pepper);
            $expired = strtotime((string)$otp['expires_at']) < time();
            if ($expired || !hash_equals($expected, (string)$otp['code_hash'])) {
                j_err('Wrong or expired verification code. Check your email, or Resend.', 400);
            }
            db_run('UPDATE otp_codes SET used_at = NOW() WHERE id = ?', [$otp['id']]);
            $row = db_one('SELECT * FROM users WHERE lower(email) = ?', [$email]);
            if (!$row || $row['status'] !== 'Active') j_err('Account not available.', 403);
            login_user($row['id']);
            j_ok(['user' => map_user($row)]);
        }

        case 'resend-login-otp': {
            require_csrf();
            $email = strtolower(str_param('email'));
            if ($email === '') j_err('Email is required.', 400);
            $row = db_one('SELECT id, status FROM users WHERE lower(email) = ?', [$email]);
            if (!$row) j_err('No account with this email.', 404);
            if ($row['status'] !== 'Active') j_err('Account is suspended.', 403);
            $code = str_pad((string)random_int(0, 999999), 6, '0', STR_PAD_LEFT);
            $pepper = $_ENV['MTS_OTP_PEPPER'] ?? 'MTS_OTP_2026_PEPPER_DAR';
            $codeHash = hash('sha256', $code . '|' . $email . '|' . $pepper);
            $otpId = new_id('OTP-');
            db_run('DELETE FROM otp_codes WHERE email = ? AND purpose = ?', [$email, 'login']);
            db_run(
                'INSERT INTO otp_codes (id, user_id, email, purpose, code_hash, expires_at, created_at)
                 VALUES (?,?,?,?,?, DATE_ADD(NOW(), INTERVAL 10 MINUTE), NOW())',
                [$otpId, $row['id'], $email, 'login', $codeHash]
            );
            try { notify_login_otp($email, $code); } catch (\Throwable $e) { error_log('[otp resend] '.$e->getMessage()); }
            j_ok([
                'otp_required' => true,
                'email' => $email,
                'otp_debug_only' => (function_exists('getenv') && getenv('OTP_SHOW_IN_RESPONSE')) ? $code : null,
            ]);
        }

        case 'forgot-password-init': {
            require_csrf();
            $email = strtolower(str_param('email'));
            if (!filter_var($email, FILTER_VALIDATE_EMAIL)) j_err('Valid email is required.', 400);
            $row = db_one('SELECT id FROM users WHERE lower(email) = ?', [$email]);
            if ($row) {
                $token = bin2hex(random_bytes(32));
                $tokenHash = hash('sha256', $token);
                $prId = new_id('PR-');
                db_run('DELETE FROM password_resets WHERE email = ?', [$email]);
                db_run(
                    'INSERT INTO password_resets (id, email, token_hash, expires_at, created_at)
                     VALUES (?,?,?, DATE_ADD(NOW(), INTERVAL 1 HOUR), NOW())',
                    [$prId, $email, $tokenHash]
                );
                $host = $_SERVER['HTTP_HOST'] ?? 'localhost';
                $scheme = (!empty($_SERVER['HTTPS']) && $_SERVER['HTTPS'] !== 'off') ? 'https' : 'http';
                $link = "$scheme://$host/auth.php?reset_token=" . rawurlencode($token) . "&email=" . rawurlencode($email);
                try { notify_password_reset($email, $link); } catch (\Throwable $e) { error_log('[pw reset email] '.$e->getMessage()); }
            }
            j_ok(['sent' => true]);
        }

        case 'forgot-password-complete': {
            require_csrf();
            $email = strtolower(str_param('email'));
            $token = trim((string)param('token', ''));
            $newPass = (string)param('new_password', '');
            if (!filter_var($email, FILTER_VALIDATE_EMAIL)) j_err('Valid email is required.', 400);
            if (strlen($newPass) < 6) j_err('New password must be 6+ characters.', 400);
            if ($token === '') j_err('Reset token is required.', 400);
            $reset = db_one('SELECT * FROM password_resets WHERE email = ? AND used_at IS NULL ORDER BY created_at DESC LIMIT 1', [$email]);
            if (!$reset) j_err('No pending password reset for this email.', 404);
            $expected = hash('sha256', $token);
            $expired = strtotime((string)$reset['expires_at']) < time();
            if ($expired || !hash_equals($expected, (string)$reset['token_hash'])) {
                j_err('This link is invalid or expired. Request a new password reset.', 400);
            }
            db_run('UPDATE password_resets SET used_at = NOW() WHERE id = ?', [$reset['id']]);
            db_run('UPDATE users SET password_hash = ? WHERE lower(email) = ?', [password_hash($newPass, PASSWORD_DEFAULT), $email]);
            j_ok(['reset' => true]);
        }

        case 'register': {
            require_csrf();
            $name = str_param('name');
            $phone = str_param('phone');
            $email = strtolower(str_param('email'));
            $company = str_param('company');
            $pass = (string)param('password', '');
            $pass2 = (string)param('password2', '');
            $pin = mts_pin_digits((string)param('pin', ''));
            $pin2 = mts_pin_digits((string)param('pin2', ''));

            if ($name === '' || $phone === '' || $email === '') j_err('Fill all required fields.');
            if (!filter_var($email, FILTER_VALIDATE_EMAIL)) j_err('Invalid email address.');
            if (!mts_phone_valid($phone)) j_err('Enter a valid Tanzanian phone number, e.g. 0712 345 678.', 400);
            if (strlen($pass) < 6) j_err('Password must be 6+ characters.');
            if ($pass !== $pass2) j_err('Passwords do not match.');
            if ($pin !== '' && (strlen($pin) < 4 || strlen($pin) > 6)) j_err('PIN must be 4–6 digits.', 400);
            if ($pin !== '' && $pin2 !== '' && $pin !== $pin2) j_err('PINs do not match.', 400);
            if (db_one('SELECT id FROM users WHERE lower(email) = ?', [$email])) {
                j_err('Email already registered. Please sign in.');
            }
            $id = new_id('U-');
            db_run(
                'INSERT INTO users (id,name,email,phone,company,password_hash,pin_hash,role,status,created_at)
                 VALUES (?,?,?,?,?,?,?,?,?,now())',
                [$id, $name, $email, $phone, $company, password_hash($pass, PASSWORD_DEFAULT),
                    $pin !== '' ? password_hash($pin, PASSWORD_DEFAULT) : null, 'client', 'Active']
            );
            login_user($id);
            j_ok(['user' => map_user(db_one('SELECT * FROM users WHERE id = ?', [$id]))]);
        }

        /* ---------- passkeys (WebAuthn) ----------
         * A passkey replaces the password AND the PIN: the device holds a
         * private key that never leaves it, and the phone or laptop asks for
         * its own unlock (fingerprint, face, PIN or pattern) before signing.
         * This site never sees that unlock — only the signature, which it
         * verifies against the public key stored at registration. */

        case 'passkey-register-begin': {
            require_csrf();
            $user = require_user();
            $challenge = wa_b64u_enc(random_bytes(32));
            $_SESSION['wa_reg'] = ['c' => $challenge, 't' => time()];

            $existing = db_all('SELECT credential_id, transports FROM webauthn_credentials WHERE user_id = ?', [$user['id']]);
            j_ok([
                'challenge' => $challenge,
                'rp' => ['id' => wa_rp_id(), 'name' => setting('webauthn_rp_name', 'Mbilinyi Tech Solutions')],
                'user' => [
                    'id' => wa_b64u_enc((string)$user['id']),
                    'name' => (string)$user['email'],
                    'displayName' => (string)$user['name'],
                ],
                'pubKeyCredParams' => [
                    ['type' => 'public-key', 'alg' => -7],     // ES256
                    ['type' => 'public-key', 'alg' => -257],   // RS256
                ],
                'timeout' => 60000,
                'attestation' => 'none',
                'authenticatorSelection' => [
                    'residentKey' => 'preferred',
                    'userVerification' => 'preferred',
                ],
                'excludeCredentials' => array_map(fn($c) => [
                    'type' => 'public-key',
                    'id' => $c['credential_id'],
                    'transports' => array_values(array_filter(explode(',', (string)$c['transports']))),
                ], $existing),
            ]);
        }

        case 'passkey-register-finish': {
            require_csrf();
            $user = require_user();
            $ch = $_SESSION['wa_reg'] ?? null;
            unset($_SESSION['wa_reg']);
            if (!$ch || time() - (int)$ch['t'] > 300) j_err('This registration expired. Please try again.', 400);

            $res = param('response', []);
            if (!is_array($res)) j_err('Malformed registration response.', 400);
            $transports = param('transports', []);
            if (!is_array($transports)) $transports = [];

            $info = wa_verify_registration(
                (string)($res['clientDataJSON'] ?? ''),
                (string)($res['attestationObject'] ?? ''),
                (string)$ch['c'],
                wa_origin(),
                wa_rp_id(),
                $transports
            );

            if (db_one('SELECT id FROM webauthn_credentials WHERE credential_id = ?', [$info['credentialId']])) {
                j_err('This device is already registered.', 409);
            }

            $label = mb_substr(trim(str_param('label', 'Passkey')), 0, 100);
            db_run(
                'INSERT INTO webauthn_credentials
                    (id, user_id, credential_id, public_key, sign_count, transports, label, aaguid, created_at)
                 VALUES (?,?,?,?,?,?,?,?, NOW())',
                [
                    new_id('PK-'), $user['id'], $info['credentialId'], $info['publicKey'],
                    $info['signCount'], $info['transports'], $label, $info['aaguid'],
                ]
            );
            j_ok(['registered' => true, 'label' => $label]);
        }

        /* No email is needed when the device keeps a discoverable passkey: the
         * browser hands back the credential id and we look the owner up. */
        case 'passkey-login-begin': {
            require_csrf();
            $email = strtolower(str_param('email'));
            $allow = [];
            if ($email !== '') {
                $u = db_one('SELECT id FROM users WHERE lower(email) = ?', [$email]);
                if ($u) {
                    $allow = db_all('SELECT credential_id, transports FROM webauthn_credentials WHERE user_id = ?', [$u['id']]);
                }
            }
            $challenge = wa_b64u_enc(random_bytes(32));
            $_SESSION['wa_auth'] = ['c' => $challenge, 't' => time()];
            j_ok([
                'challenge' => $challenge,
                'rpId' => wa_rp_id(),
                'timeout' => 60000,
                'userVerification' => 'preferred',
                'allowCredentials' => array_map(fn($c) => [
                    'type' => 'public-key',
                    'id' => $c['credential_id'],
                    'transports' => array_values(array_filter(explode(',', (string)$c['transports']))),
                ], $allow),
            ]);
        }

        case 'passkey-login-finish': {
            require_csrf();
            $ch = $_SESSION['wa_auth'] ?? null;
            unset($_SESSION['wa_auth']);
            if (!$ch || time() - (int)$ch['t'] > 300) j_err('This sign-in expired. Please try again.', 400);

            $res = param('response', []);
            if (!is_array($res)) j_err('Malformed sign-in response.', 400);

            $row = db_one('SELECT * FROM webauthn_credentials WHERE credential_id = ?', [str_param('id')]);
            if (!$row) j_err('This passkey is not registered on this site.', 404);

            $owner = db_one('SELECT * FROM users WHERE id = ?', [$row['user_id']]);
            if (!$owner) j_err('This passkey is no longer attached to an account.', 404);
            if ($owner['status'] !== 'Active') j_err('Account is suspended. Contact 0796 752 645.', 403);

            try {
                $newCount = wa_verify_assertion(
                    (string)($res['clientDataJSON'] ?? ''),
                    (string)($res['authenticatorData'] ?? ''),
                    (string)($res['signature'] ?? ''),
                    (string)$row['public_key'],
                    (int)$row['sign_count'],
                    (string)$ch['c'],
                    wa_origin(),
                    wa_rp_id()
                );
            } catch (\Throwable $e) {
                error_log('[passkey] ' . $e->getMessage());
                j_err($e->getMessage(), 400);
            }

            db_run('UPDATE webauthn_credentials SET sign_count = ?, last_used_at = NOW() WHERE id = ?', [$newCount, $row['id']]);
            login_user($owner['id']);
            j_ok(['user' => map_user($owner)]);
        }

        case 'passkey-list': {
            $user = require_user();
            $rows = db_all(
                'SELECT id, label, transports, created_at, last_used_at FROM webauthn_credentials WHERE user_id = ? ORDER BY created_at',
                [$user['id']]
            );
            j_ok(['passkeys' => array_map(fn($r) => [
                'id' => $r['id'],
                'label' => (string)$r['label'],
                'transports' => (string)$r['transports'],
                'created_at' => iso((string)$r['created_at']),
                'last_used_at' => $r['last_used_at'] ? iso((string)$r['last_used_at']) : null,
            ], $rows)]);
        }

        case 'passkey-delete': {
            require_csrf();
            $user = require_user();
            $row = db_one('SELECT id FROM webauthn_credentials WHERE id = ? AND user_id = ?', [str_param('id'), $user['id']]);
            if (!$row) j_err('Passkey not found.', 404);
            db_run('DELETE FROM webauthn_credentials WHERE id = ?', [$row['id']]);
            j_ok(['deleted' => true]);
        }

        case 'logout': {
            require_csrf();
            $_SESSION = [];
            if (ini_get('session.use_cookies')) {
                $p = session_get_cookie_params();
                setcookie(session_name(), '', time() - 42000, $p['path'], $p['domain'], (bool)$p['secure'], (bool)$p['httponly']);
            }
            session_destroy();
            j_ok(['user' => null]);
        }

        case 'me': {
            j_ok(['user' => current_user()]);
        }

        default:
            j_err('Unknown auth action.', 404);
    }
} catch (Throwable $e) {
    error_log('[api/auth] ' . $e->getMessage());
    j_err('Server error. Please try again.', 500);
}
