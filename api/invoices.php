<?php
declare(strict_types=1);
require_once __DIR__ . '/../config/app.php';

header('Content-Type: application/json; charset=utf-8');

function invoice_owned_by_user(array $v, array $user): bool {
    if ($user['role'] === 'admin') return true;
    if ($v['user_id'] === $user['id']) return true;
    return (bool)db_one(
        'SELECT id FROM requests WHERE tracking_id = ? AND (user_id = ? OR lower(email) = ?)',
        [$v['tracking_id'], $user['id'], strtolower($user['email'])]
    );
}

try {
    switch (action_name()) {

        case 'mine': {
            $u = require_user();
            $rows = db_all(
                'SELECT * FROM invoices WHERE user_id = ? OR tracking_id IN
                   (SELECT tracking_id FROM requests WHERE user_id = ? OR lower(email) = ?)
                 ORDER BY created_at DESC',
                [$u['id'], $u['id'], strtolower($u['email'])]
            );
            j_ok(['invoices' => array_map('map_invoice', $rows)]);
        }

        case 'pay': {
            require_csrf();
            $u = require_user();
            $id = str_param('id');
            $ref = str_param('ref');
            if (strlen($ref) < 4) j_err('Enter transaction reference code.');

            $row = db_one('SELECT * FROM invoices WHERE id = ?', [$id]);
            if (!$row) j_err('Invoice not found.', 404);
            if (!invoice_owned_by_user($row, $u)) j_err('Not your invoice.', 403);
            if ($row['status'] === 'Paid') j_err('Invoice already paid.');

            db_run('UPDATE invoices SET status = ?, method = ? WHERE id = ?',
                ['Paid', $row['method'] . ' • Ref ' . $ref, $id]);
            j_ok(['invoice' => map_invoice(db_one('SELECT * FROM invoices WHERE id = ?', [$id]))]);
        }

        case 'admin_list': {
            require_admin();
            $rows = db_all('SELECT * FROM invoices ORDER BY created_at DESC');
            j_ok(['invoices' => array_map('map_invoice', $rows)]);
        }

        /* create / update */
        case 'admin_save': {
            require_csrf();
            require_admin();
            $id = str_param('id');
            $title = str_param('title');
            $amount = (float)param('amount', 0);
            $tracking = str_param('trackingId');
            $method = str_param('method', 'M-Pesa / Bank');
            $status = in_array(str_param('status', 'Unpaid'), ['Unpaid', 'Paid'], true) ? str_param('status') : 'Unpaid';
            $due = str_param('dueDate', date('Y-m-d', time() + 14 * 86400));

            if ($title === '') j_err('Invoice title is required.');
            if ($amount <= 0) j_err('Amount must be greater than zero.');

            $req = db_one('SELECT * FROM requests WHERE tracking_id = ?', [$tracking]);
            $userId = $req['user_id'] ?? '';

            if ($id === 'new' || $id === '') {
                $vid = new_id('INV-');
                db_run(
                    'INSERT INTO invoices (id,tracking_id,user_id,title,amount,status,method,due_date,created_at)
                     VALUES (?,?,?,?,?,?,?,?,now())',
                    [$vid, $tracking, $userId, $title, $amount, $status, $method, dt($due)]
                );
            } else {
                $row = db_one('SELECT * FROM invoices WHERE id = ?', [$id]);
                if (!$row) j_err('Invoice not found.', 404);
                db_run('UPDATE invoices SET tracking_id = ?, user_id = ?, title = ?, amount = ?, status = ?, method = ?, due_date = ? WHERE id = ?',
                    [$tracking, $userId ?: $row['user_id'], $title, $amount, $status, $method, dt($due), $id]);
            }
            j_ok(['invoices' => array_map('map_invoice', db_all('SELECT * FROM invoices ORDER BY created_at DESC'))]);
        }

        case 'admin_delete': {
            require_csrf();
            require_admin();
            db_run('DELETE FROM invoices WHERE id = ?', [str_param('id')]);
            j_ok([]);
        }

        case 'mark_paid': {
            require_csrf();
            require_admin();
            $id = str_param('id');
            db_run('UPDATE invoices SET status = ? WHERE id = ?', ['Paid', $id]);
            j_ok(['invoice' => map_invoice(db_one('SELECT * FROM invoices WHERE id = ?', [$id]))]);
        }

        default:
            j_err('Unknown invoices action.', 404);
    }
} catch (Throwable $e) {
    error_log('[api/invoices] ' . $e->getMessage());
    j_err('Server error. Please try again.', 500);
}
