<?php
declare(strict_types=1);
require_once __DIR__ . '/../config/app.php';

header('Content-Type: application/json; charset=utf-8');

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
            j_ok([
                'otp_required' => true,
                'email' => $email,
                'otp_debug_only' => (function_exists('getenv') && getenv('OTP_SHOW_IN_RESPONSE')) ? $code : null,
            ]);
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

            if ($name === '' || $phone === '' || $email === '') j_err('Fill all required fields.');
            if (!filter_var($email, FILTER_VALIDATE_EMAIL)) j_err('Invalid email address.');
            if (strlen($pass) < 6) j_err('Password must be 6+ characters.');
            if ($pass !== $pass2) j_err('Passwords do not match.');
            if (db_one('SELECT id FROM users WHERE lower(email) = ?', [$email])) {
                j_err('Email already registered. Please sign in.');
            }
            $id = new_id('U-');
            db_run(
                'INSERT INTO users (id,name,email,phone,company,password_hash,role,status,created_at)
                 VALUES (?,?,?,?,?,?,?,?,now())',
                [$id, $name, $email, $phone, $company, password_hash($pass, PASSWORD_DEFAULT), 'client', 'Active']
            );
            login_user($id);
            j_ok(['user' => map_user(db_one('SELECT * FROM users WHERE id = ?', [$id]))]);
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
