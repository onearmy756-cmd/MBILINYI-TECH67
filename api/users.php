<?php
declare(strict_types=1);
require_once __DIR__ . '/../config/app.php';

header('Content-Type: application/json; charset=utf-8');

try {
    switch (action_name()) {

        case 'list': {
            require_admin();
            $q = '%' . strtolower(str_param('q')) . '%';
            $rows = db_all(
                'SELECT * FROM users
                 WHERE lower(name) LIKE ? OR lower(email) LIKE ? OR lower(phone) LIKE ? OR lower(coalesce(company,\'\')) LIKE ?
                 ORDER BY created_at ASC',
                [$q, $q, $q, $q]
            );
            $out = [];
            foreach ($rows as $r) {
                $m = map_user($r);
                $m['requestCount'] = (int)db_one(
                    'SELECT count(*) AS c FROM requests WHERE user_id = ? OR lower(email) = ?',
                    [$r['id'], strtolower($r['email'])]
                )['c'];
                $out[] = $m;
            }
            j_ok(['users' => $out]);
        }

        case 'save': {
            require_csrf();
            require_admin();
            $id = str_param('id');
            $name = str_param('name');
            $email = strtolower(str_param('email'));
            $phone = str_param('phone');
            $company = str_param('company');
            $role = in_array(str_param('role'), ['client', 'admin'], true) ? str_param('role') : 'client';
            $status = in_array(str_param('status'), ['Active', 'Suspended'], true) ? str_param('status') : 'Active';
            $pass = (string)param('password', '');

            if ($name === '' || $email === '') j_err('Name & email required.');
            if (!filter_var($email, FILTER_VALIDATE_EMAIL)) j_err('Invalid email address.');
            $dup = db_one('SELECT id FROM users WHERE lower(email) = ? AND id <> ?', [$email, $id]);
            if ($dup) j_err('Email already exists.');

            if ($id === 'new' || $id === '') {
                $newId = new_id('U-');
                db_run(
                    'INSERT INTO users (id,name,email,phone,company,password_hash,role,status,created_at)
                     VALUES (?,?,?,?,?,?,?,?,now())',
                    [$newId, $name, $email, $phone, $company,
                        password_hash($pass !== '' ? $pass : 'client123', PASSWORD_DEFAULT), $role, $status]
                );
            } else {
                $row = db_one('SELECT * FROM users WHERE id = ?', [$id]);
                if (!$row) j_err('Account not found.', 404);
                db_run('UPDATE users SET name = ?, email = ?, phone = ?, company = ?, role = ?, status = ? WHERE id = ?',
                    [$name, $email, $phone, $company, $role, $status, $id]);
                if ($pass !== '') {
                    db_run('UPDATE users SET password_hash = ? WHERE id = ?', [password_hash($pass, PASSWORD_DEFAULT), $id]);
                }
            }
            j_ok([]);
        }

        case 'delete': {
            require_csrf();
            $u = require_admin();
            $id = str_param('id');
            if ($id === 'U-ADMIN' || $id === $u['id']) j_err('Cannot delete the CEO account.');
            db_run('DELETE FROM users WHERE id = ?', [$id]);
            j_ok([]);
        }

        case 'toggle_status': {
            require_csrf();
            $u = require_admin();
            $id = str_param('id');
            if ($id === 'U-ADMIN') j_err('CEO account cannot be suspended.');
            $row = db_one('SELECT * FROM users WHERE id = ?', [$id]);
            if (!$row) j_err('Account not found.', 404);
            $next = $row['status'] === 'Active' ? 'Suspended' : 'Active';
            db_run('UPDATE users SET status = ? WHERE id = ?', [$next, $id]);
            j_ok(['status' => $next]);
        }

        case 'toggle_role': {
            require_csrf();
            $u = require_admin();
            $id = str_param('id');
            if ($id === 'U-ADMIN') j_err('CEO must remain admin.');
            $row = db_one('SELECT * FROM users WHERE id = ?', [$id]);
            if (!$row) j_err('Account not found.', 404);
            $next = $row['role'] === 'admin' ? 'client' : 'admin';
            db_run('UPDATE users SET role = ? WHERE id = ?', [$next, $id]);
            j_ok(['role' => $next]);
        }

        case 'impersonate': {
            require_csrf();
            require_admin();
            $id = str_param('id');
            $row = db_one('SELECT * FROM users WHERE id = ?', [$id]);
            if (!$row) j_err('Account not found.', 404);
            login_user($row['id']);
            j_ok(['user' => map_user($row)]);
        }

        default:
            j_err('Unknown users action.', 404);
    }
} catch (Throwable $e) {
    error_log('[api/users] ' . $e->getMessage());
    j_err('Server error. Please try again.', 500);
}
