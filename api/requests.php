<?php
declare(strict_types=1);
require_once __DIR__ . '/../config/app.php';

header('Content-Type: application/json; charset=utf-8');

const MTS_STATUSES = ['Pending', 'Under Review', 'Quoted', 'Approved', 'In Progress', 'Completed', 'Rejected'];

function request_owned_by(array $req, array $user): bool {
    if ($user['role'] === 'admin') return true;
    return $req['user_id'] === $user['id']
        || strtolower($req['email']) === strtolower($user['email']);
}

/* KANUNI YA 50%: mradi HAUWEZI kuanza (In Progress) mpaka deposit ya 50% ilipwe. */
function deposit_block_for_request(array $req): string {
    $c = db_one('SELECT * FROM contracts WHERE request_id = ? ORDER BY created_at DESC LIMIT 1', [$req['id']]);
    if (!$c) return '';
    $inv = db_one(
        'SELECT * FROM invoices WHERE tracking_id = ? AND title LIKE ? ORDER BY created_at ASC LIMIT 1',
        [$c['tracking_id'], '%50%Deposit%']
    );
    if (!$inv) {
        return 'HAUWEZI kuanza project: deposit invoice ya 50% bado haipo. Tengeneza kwanza (contracts → Create 50% Deposit Invoice au Invoices → New Invoice yenye "50% Deposit").';
    }
    if ($inv['status'] !== 'Paid') {
        return 'HAUWEZI kuanza project: mteja hajalipa 50% deposit (' . number_format((float)$inv['amount']) . ' TZS — ' . $inv['id'] . '). Lipa 50% kwanza kabla project kuanza.';
    }
    return '';
}

function unique_tracking_id(): string {
    for ($i = 0; $i < 40; $i++) {
        $id = 'MTS-2026-' . random_int(1000, 9999);
        if (!db_one('SELECT id FROM requests WHERE tracking_id = ?', [$id])) return $id;
    }
    return 'MTS-2026-' . random_int(100000, 999999);
}

try {
    switch (action_name()) {

        /* ---------- public: submit the quote wizard ---------- */
        case 'create': {
            require_csrf();
            $name = str_param('name');
            $phone = str_param('phone');
            $email = strtolower(str_param('email'));
            $company = str_param('company');
            $service = str_param('service');
            $title = str_param('title');
            $description = str_param('description');

            if ($name === '' || $phone === '' || $email === '') j_err('Fill name, phone & email.');
            if (!filter_var($email, FILTER_VALIDATE_EMAIL)) j_err('Invalid email address.');
            if ($service === '') j_err('Select a service. / Chagua huduma.');
            if ($title === '' || strlen($description) < 15) {
                j_err('Add project title + detailed requirements (15+ chars).');
            }

            $user = current_user();
            if (!$user) {
                $existing = db_one('SELECT * FROM users WHERE lower(email) = ?', [$email]);
                if ($existing) {
                    if ($existing['status'] !== 'Active') j_err('This account is suspended. Contact 0796 752 645.');
                    login_user($existing['id']);
                    $user = map_user($existing);
                } else {
                    $uid = new_id('U-');
                    $autoPass = 'client' . random_int(1000, 9999);
                    db_run(
                        'INSERT INTO users (id,name,email,phone,company,password_hash,role,status,created_at)
                         VALUES (?,?,?,?,?,?,?,?,now())',
                        [$uid, $name, $email, $phone, $company, password_hash($autoPass, PASSWORD_DEFAULT), 'client', 'Active']
                    );
                    login_user($uid);
                    $user = map_user(db_one('SELECT * FROM users WHERE id = ?', [$uid]));
                }
            }

            $tech = param('tech', []);
            if (!is_array($tech)) $tech = [];

            $rid = new_id('R-');
            $tracking = unique_tracking_id();
            db_run(
                'INSERT INTO requests
                 (id,tracking_id,user_id,name,email,phone,company,service,title,description,tech,platform,budget,deadline,priority,status,admin_note,created_at)
                 VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,now())',
                [
                    $rid, $tracking, $user['id'], $name, $email, $phone, $company, $service, $title, $description,
                    json_encode(array_values($tech), JSON_UNESCAPED_UNICODE),
                    str_param('platform', 'Web Application'),
                    str_param('budget', ''),
                    str_param('deadline', ''),
                    str_param('priority', 'Normal'),
                    'Pending', '',
                ]
            );

            $request = map_request(db_one('SELECT * FROM requests WHERE id = ?', [$rid]));
            notify_new_request($request);
            j_ok(['trackingId' => $tracking, 'request' => $request, 'user' => $user]);
        }

        /* ---------- client ---------- */
        case 'mine': {
            $u = require_user();
            $rows = db_all(
                'SELECT * FROM requests WHERE user_id = ? OR lower(email) = ? ORDER BY created_at DESC',
                [$u['id'], strtolower($u['email'])]
            );
            j_ok(['requests' => array_map('map_request', $rows)]);
        }

        case 'get': {
            $u = require_user();
            $row = db_one('SELECT * FROM requests WHERE id = ? OR tracking_id = ?', [str_param('id'), str_param('trackingId')]);
            if (!$row) j_err('Request not found.', 404);
            if (!request_owned_by($row, $u)) j_err('Not your request.', 403);
            j_ok(['request' => map_request($row)]);
        }

        /* ---------- admin ---------- */
        case 'admin_list': {
            require_admin();
            $q = '%' . strtolower(str_param('q')) . '%';
            $status = str_param('status', 'All');
            $sql = 'SELECT * FROM requests WHERE (lower(title) LIKE ? OR lower(name) LIKE ? OR lower(tracking_id) LIKE ? OR lower(email) LIKE ?)';
            $params = [$q, $q, $q, $q];
            if ($status !== 'All') { $sql .= ' AND status = ?'; $params[] = $status; }
            $sql .= ' ORDER BY created_at DESC';
            j_ok(['requests' => array_map('map_request', db_all($sql, $params))]);
        }

        case 'set_status': {
            require_csrf();
            require_admin();
            $id = str_param('id');
            $status = str_param('status');
            if (!in_array($status, MTS_STATUSES, true)) j_err('Unknown status.');
            $row = db_one('SELECT * FROM requests WHERE id = ?', [$id]);
            if (!$row) j_err('Request not found.', 404);
            if ($status === 'In Progress' && $row['status'] !== 'In Progress') {
                $block = deposit_block_for_request($row);
                if ($block !== '') j_err($block, 402);
            }
            db_run('UPDATE requests SET status = ? WHERE id = ?', [$status, $id]);
            j_ok(['request' => map_request(db_one('SELECT * FROM requests WHERE id = ?', [$id]))]);
        }

        case 'note': {
            require_csrf();
            require_admin();
            $id = str_param('id');
            if (!db_one('SELECT id FROM requests WHERE id = ?', [$id])) j_err('Request not found.', 404);
            db_run('UPDATE requests SET admin_note = ? WHERE id = ?', [str_param('note'), $id]);
            j_ok([]);
        }

        /* admin full edit of a request */
        case 'admin_update': {
            require_csrf();
            require_admin();
            $id = str_param('id');
            $row = db_one('SELECT * FROM requests WHERE id = ?', [$id]);
            if (!$row) j_err('Request not found.', 404);

            $status = str_param('status', $row['status']);
            if (!in_array($status, MTS_STATUSES, true)) j_err('Unknown status.');
            $service = str_param('service', $row['service']);
            $title = str_param('title', $row['title']);
            $description = str_param('description', $row['description']);
            $priority = str_param('priority', $row['priority']);
            $budget = str_param('budget', $row['budget']);
            $deadline = str_param('deadline', $row['deadline']);
            $note = str_param('adminNote', $row['admin_note']);

            if ($title === '' || $service === '') j_err('Service & project title are required.');
            if (strlen($description) < 10) j_err('Description must be at least 10 characters.');

            /* KANUNI YA 50%: zuia mabadiliko hadi In Progress bila deposit iliyolipwa. */
            if ($status === 'In Progress' && $row['status'] !== 'In Progress') {
                $block = deposit_block_for_request($row);
                if ($block !== '') j_err($block, 402);
            }

            db_run(
                'UPDATE requests SET service = ?, title = ?, description = ?, priority = ?, budget = ?, deadline = ?, status = ?, admin_note = ? WHERE id = ?',
                [$service, $title, $description, $priority, $budget, $deadline, $status, $note, $id]
            );
            j_ok(['request' => map_request(db_one('SELECT * FROM requests WHERE id = ?', [$id]))]);
        }

        case 'delete': {
            require_csrf();
            require_admin();
            $id = str_param('id');
            db_run('DELETE FROM contracts WHERE request_id = ?', [$id]);
            db_run('DELETE FROM quotes WHERE request_id = ?', [$id]);
            db_run('DELETE FROM requests WHERE id = ?', [$id]);
            j_ok([]);
        }

        default:
            j_err('Unknown requests action.', 404);
    }
} catch (Throwable $e) {
    error_log('[api/requests] ' . $e->getMessage());
    j_err('Server error. Please try again.', 500);
}
