<?php
declare(strict_types=1);
require_once __DIR__ . '/../config/app.php';

header('Content-Type: application/json; charset=utf-8');

function quote_request_row(array $q): ?array {
    return db_one('SELECT * FROM requests WHERE id = ?', [$q['request_id']]);
}

function request_owned_by_user(array $req, array $user): bool {
    if ($user['role'] === 'admin') return true;
    return $req['user_id'] === $user['id']
        || strtolower($req['email']) === strtolower($user['email']);
}

function ensure_contract_for_quote(array $q, array $req): array {
    $existing = db_one('SELECT * FROM contracts WHERE quote_id = ?', [$q['id']]);
    if ($existing) return $existing;
    $cid = new_id('C-');
    db_run(
        'INSERT INTO contracts (id,request_id,quote_id,tracking_id,client_name,project_title,scope,duration,total,status,client_sig,client_signed_at,admin_sig,admin_signed_at,created_at)
         VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,now())',
        [$cid, $req['id'], $q['id'], $q['tracking_id'], $req['name'], $req['title'],
            mb_substr($req['description'], 0, 400), '45 days', $q['total'], 'Sent', null, null, null, null]
    );
    return db_one('SELECT * FROM contracts WHERE id = ?', [$cid]);
}

function ensure_invoice_for_quote(array $q): array {
    $existing = db_one('SELECT * FROM invoices WHERE tracking_id = ? AND title LIKE ?', [$q['tracking_id'], '%50%Deposit%']);
    if ($existing) return $existing;
    $vid = new_id('INV-');
    db_run(
        'INSERT INTO invoices (id,tracking_id,user_id,title,amount,status,method,due_date,created_at)
         VALUES (?,?,?,?,?,?,?,?,now())',
        [$vid, $q['tracking_id'], $q['user_id'], 'Project — 50% Deposit', round((float)$q['total'] / 2, 2),
            'Unpaid', 'M-Pesa / Bank', dt(time() + 14 * 86400)]
    );
    return db_one('SELECT * FROM invoices WHERE id = ?', [$vid]);
}

try {
    switch (action_name()) {

        case 'mine': {
            $u = require_user();
            $rows = db_all(
                'SELECT q.* FROM quotes q
                 JOIN requests r ON r.id = q.request_id
                 WHERE q.user_id = ? OR r.user_id = ? OR lower(r.email) = ?
                 ORDER BY q.created_at DESC',
                [$u['id'], $u['id'], strtolower($u['email'])]
            );
            j_ok(['quotes' => array_map('map_quote', $rows)]);
        }

        case 'get': {
            $u = require_user();
            $row = db_one('SELECT * FROM quotes WHERE id = ?', [str_param('id')]);
            if (!$row) j_err('Quotation not found.', 404);
            $req = quote_request_row($row);
            if (!$req || !request_owned_by_user($req, $u)) j_err('Not your quotation.', 403);
            j_ok([
                'quote' => map_quote($row),
                'requestTitle' => $req['title'],
                'clientName' => $req['name'],
                'clientEmail' => $req['email'],
            ]);
        }

        case 'respond': {
            require_csrf();
            $u = require_user();
            $id = str_param('id');
            $decision = str_param('decision');
            if (!in_array($decision, ['Approved', 'Rejected'], true)) j_err('Unknown decision.');

            $q = db_one('SELECT * FROM quotes WHERE id = ?', [$id]);
            if (!$q) j_err('Quotation not found.', 404);
            $req = quote_request_row($q);
            if (!$req || !request_owned_by_user($req, $u)) j_err('Not your quotation.', 403);

            db_run('UPDATE quotes SET status = ? WHERE id = ?', [$decision, $id]);

            if ($decision === 'Approved') {
                db_run('UPDATE requests SET status = ?, admin_note = ? WHERE id = ?',
                    ['Approved', 'Quotation approved by client — contract generated.', $req['id']]);
                ensure_contract_for_quote($q, $req);
                ensure_invoice_for_quote($q);
                notify_contract_ready(map_contract(db_one('SELECT * FROM contracts WHERE quote_id = ?', [$id])));
            } else {
                db_run('UPDATE requests SET status = ? WHERE id = ?', ['Rejected', $req['id']]);
            }
            j_ok(['status' => $decision]);
        }

        case 'admin_list': {
            require_admin();
            $rows = db_all('SELECT * FROM quotes ORDER BY created_at DESC');
            $out = [];
            foreach ($rows as $r) {
                $m = map_quote($r);
                $req = quote_request_row($r);
                $m['requestTitle'] = $req['title'] ?? '';
                $out[] = $m;
            }
            j_ok(['quotes' => $out]);
        }

        case 'admin_save': {
            require_csrf();
            require_admin();
            $requestId = str_param('requestId');
            $req = db_one('SELECT * FROM requests WHERE id = ?', [$requestId]);
            if (!$req) j_err('Request not found.', 404);

            $items = param('items', []);
            if (!is_array($items) || !$items) j_err('Add at least one line item.');
            $clean = [];
            foreach ($items as $it) {
                $desc = trim((string)($it['desc'] ?? ''));
                if ($desc === '') j_err('Describe every line item.');
                $clean[] = [
                    'desc' => $desc,
                    'qty' => max(1, (int)($it['qty'] ?? 1)),
                    'price' => max(0, (float)($it['price'] ?? 0)),
                ];
            }
            $sub = 0.0;
            foreach ($clean as $it) $sub += $it['qty'] * $it['price'];
            $vat = round($sub * 0.18, 2);
            $total = $sub + $vat;

            $valid = str_param('validUntil', date('Y-m-d', time() + 14 * 86400));
            $ts = dt(strtotime($valid) ?: time() + 14 * 86400);
            $status = in_array(str_param('status', 'Sent'), ['Sent', 'Draft'], true) ? str_param('status', 'Sent') : 'Sent';
            $message = str_param('adminMessage');

            $existing = db_one('SELECT * FROM quotes WHERE request_id = ?', [$requestId]);
            if ($existing) {
                db_run(
                    'UPDATE quotes SET items = ?, subtotal = ?, vat = ?, total = ?, valid_until = ?, status = ?, admin_message = ? WHERE id = ?',
                    [json_encode($clean, JSON_UNESCAPED_UNICODE), $sub, $vat, $total, $ts, $status, $message, $existing['id']]
                );
                $qid = $existing['id'];
            } else {
                $qid = new_id('Q-');
                db_run(
                    'INSERT INTO quotes (id,request_id,tracking_id,user_id,items,subtotal,vat,total,currency,valid_until,status,admin_message,created_at)
                     VALUES (?,?,?,?,?,?,?,?,?,?,?,?,now())',
                    [$qid, $requestId, $req['tracking_id'], $req['user_id'], json_encode($clean, JSON_UNESCAPED_UNICODE),
                        $sub, $vat, $total, 'TZS', $ts, $status, $message]
                );
            }

            db_run('UPDATE requests SET status = ?, admin_note = ? WHERE id = ?', [
                'Quoted',
                'Official quotation TZS ' . number_format($total) . ' sent. Valid till ' . date('d M Y', strtotime($ts)) . '.',
                $requestId,
            ]);

            $qrow = db_one('SELECT * FROM quotes WHERE id = ?', [$qid]);
            $qm = map_quote($qrow);
            if ($status === 'Sent') {
                notify_quotation_sent($qm, map_request($req));
            }
            j_ok(['quote' => $qm]);
        }

        case 'delete': {
            require_csrf();
            require_admin();
            $id = str_param('id');
            db_run('DELETE FROM quotes WHERE id = ?', [$id]);
            j_ok([]);
        }

        default:
            j_err('Unknown quotes action.', 404);
    }
} catch (Throwable $e) {
    error_log('[api/quotes] ' . $e->getMessage());
    j_err('Server error. Please try again.', 500);
}
