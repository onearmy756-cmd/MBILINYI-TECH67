<?php
declare(strict_types=1);
require_once __DIR__ . '/../config/app.php';

header('Content-Type: application/json; charset=utf-8');

/* Status flow: Sent (client anaandikia saini) → Client Signed (admin anaandikia saini) → Signed (mkataba umepimwa) */

function contract_request_row(array $c): ?array {
    return db_one('SELECT * FROM requests WHERE id = ?', [$c['request_id']]);
}

function contract_owned_by_user(array $c, array $user): bool {
    if ($user['role'] === 'admin') return true;
    $req = contract_request_row($c);
    if (!$req) return false;
    return $req['user_id'] === $user['id'] || strtolower($req['email']) === strtolower($user['email']);
}

function valid_signature(string $sig): bool {
    return strlen($sig) > 100 && strpos($sig, 'data:image/') === 0;
}

/* ---------- KANUNI YA 50%: deposit invoice lazima ilipwe kabla project kuanza ---------- */
function deposit_invoice_row(array $c): ?array {
    return db_one(
        'SELECT * FROM invoices WHERE tracking_id = ? AND title LIKE ? ORDER BY created_at ASC LIMIT 1',
        [$c['tracking_id'], '%50%Deposit%']
    );
}

function ensure_deposit_invoice(array $c): array {
    $existing = deposit_invoice_row($c);
    if ($existing) return $existing;
    $req = contract_request_row($c);
    $vid = new_id('INV-');
    db_run(
        'INSERT INTO invoices (id,tracking_id,user_id,title,amount,status,method,due_date,created_at)
         VALUES (?,?,?,?,?,?,?,?,now())',
        [$vid, $c['tracking_id'], $req['user_id'] ?? '', 'Project — 50% Deposit', round((float)$c['total'] / 2, 2),
            'Unpaid', 'M-Pesa / Bank', dt(time() + 14 * 86400)]
    );
    return db_one('SELECT * FROM invoices WHERE id = ?', [$vid]);
}

function deposit_block_message(array $c): string {
    $inv = deposit_invoice_row($c);
    if (!$inv) {
        return 'Deposit invoice bado haipo. Tengeneza 50% deposit invoice, mteja alipe, ndipo mradi unaruhusiwa kuanza.';
    }
    if ($inv['status'] !== 'Paid') {
        return 'Mteja hajalipa 50% deposit (' . number_format((float)$inv['amount']) . ' TZS — ' . $inv['id'] . '). Lipa 50% kwanza kabla project kuanza.';
    }
    return '';
}

try {
    switch (action_name()) {

        case 'mine': {
            $u = require_user();
            $rows = db_all(
                'SELECT c.* FROM contracts c
                 JOIN requests r ON r.id = c.request_id
                 WHERE r.user_id = ? OR lower(r.email) = ?
                 ORDER BY c.created_at DESC',
                [$u['id'], strtolower($u['email'])]
            );
            j_ok(['contracts' => array_map('map_contract', $rows)]);
        }

        case 'admin_list': {
            require_admin();
            $rows = db_all('SELECT * FROM contracts ORDER BY created_at DESC');
            j_ok(['contracts' => array_map('map_contract', $rows)]);
        }

        case 'get': {
            $u = require_user();
            $row = db_one('SELECT * FROM contracts WHERE id = ?', [str_param('id')]);
            if (!$row) j_err('Contract not found.', 404);
            if (!contract_owned_by_user($row, $u)) j_err('Not your contract.', 403);
            $c = map_contract($row);
            $req = contract_request_row($row);
            $c['request'] = $req ? map_request($req) : null;
            $c['viewerRole'] = $u['role'];
            j_ok(['contract' => $c]);
        }

        /* client e-signature (comes first) */
        case 'sign_client': {
            require_csrf();
            $u = require_user();
            $id = str_param('id');
            $sig = (string)param('sig', '');
            if (!valid_signature($sig)) j_err('Please draw your signature first.');

            $row = db_one('SELECT * FROM contracts WHERE id = ?', [$id]);
            if (!$row) j_err('Contract not found.', 404);
            if (!contract_owned_by_user($row, $u)) j_err('Not your contract.', 403);
            if ($row['status'] !== 'Sent') j_err('This contract has already been signed by you.');

            db_run('UPDATE contracts SET client_sig = ?, status = ?, client_signed_at = now() WHERE id = ?',
                [$sig, 'Client Signed', $id]);

            $c = map_contract(db_one('SELECT * FROM contracts WHERE id = ?', [$id]));
            notify_contract_ready($c, 'client');
            j_ok(['contract' => $c]);
        }

        /* admin countersignature (comes second) — only after the client signed AND the 50% deposit is paid */
        case 'sign_admin': {
            require_csrf();
            require_admin();
            $id = str_param('id');
            $sig = (string)param('sig', '');
            if (!valid_signature($sig)) j_err('Please draw your signature first.');

            $row = db_one('SELECT * FROM contracts WHERE id = ?', [$id]);
            if (!$row) j_err('Contract not found.', 404);
            if ($row['status'] !== 'Client Signed') {
                j_err('The client has not signed yet — the client signs first, then you countersign.');
            }

            /* KANUNI YA 50%: hakuna kuanza project bila deposit — inazuia hapa na kwenye set_status/admin_update. */
            $block = deposit_block_message($row);
            if ($block !== '') j_err($block, 402);

            db_run('UPDATE contracts SET admin_sig = ?, status = ?, admin_signed_at = now() WHERE id = ?',
                [$sig, 'Signed', $id]);
            db_run('UPDATE requests SET status = ? WHERE id = ? AND status NOT IN (?, ?)',
                ['In Progress', $row['request_id'], 'Completed', 'Rejected']);

            $c = map_contract(db_one('SELECT * FROM contracts WHERE id = ?', [$id]));
            notify_contract_ready($c, 'admin');
            j_ok(['contract' => $c]);
        }

        /* admin edits scope / duration / price while the client has not signed yet */
        case 'update': {
            require_csrf();
            require_admin();
            $id = str_param('id');
            $row = db_one('SELECT * FROM contracts WHERE id = ?', [$id]);
            if (!$row) j_err('Contract not found.', 404);
            if ($row['status'] !== 'Sent') {
                j_err('Contract is locked: the client already signed. The agreed price can no longer change.');
            }
            $scope = str_param('scope', $row['scope']);
            $duration = str_param('duration', $row['duration']);
            $total = (float)param('total', (float)$row['total']);
            $title = str_param('projectTitle', $row['project_title']);
            if ($scope === '' || $title === '') j_err('Project title & scope are required.');
            if ($total < 0) j_err('Invalid contract value.');

            db_run('UPDATE contracts SET scope = ?, duration = ?, total = ?, project_title = ? WHERE id = ?',
                [$scope, $duration, $total, $title, $id]);
            j_ok(['contract' => map_contract(db_one('SELECT * FROM contracts WHERE id = ?', [$id]))]);
        }

        /* deposit status for the signing panels */
        case 'deposit_info': {
            $u = require_user();
            $row = db_one('SELECT * FROM contracts WHERE id = ?', [str_param('id')]);
            if (!$row) j_err('Contract not found.', 404);
            if (!contract_owned_by_user($row, $u)) j_err('Not your contract.', 403);
            $inv = deposit_invoice_row($row);
            $block = deposit_block_message($row);
            j_ok(['deposit' => [
                'paid' => $block === '',
                'invoiceId' => $inv['id'] ?? null,
                'amount' => $inv ? (float)$inv['amount'] : round((float)$row['total'] / 2, 2),
                'status' => $inv['status'] ?? 'Missing',
                'message' => $block,
            ]]);
        }

        /* admin can generate the 50% deposit invoice from the contract if it does not exist yet */
        case 'create_deposit_invoice': {
            require_csrf();
            require_admin();
            $row = db_one('SELECT * FROM contracts WHERE id = ?', [str_param('id')]);
            if (!$row) j_err('Contract not found.', 404);
            $inv = ensure_deposit_invoice($row);
            j_ok(['invoice' => map_invoice($inv)]);
        }

        case 'create': {
            require_csrf();
            require_admin();
            $requestId = str_param('requestId');
            $req = db_one('SELECT * FROM requests WHERE id = ?', [$requestId]);
            if (!$req) j_err('Request not found.', 404);
            $q = db_one('SELECT * FROM quotes WHERE request_id = ?', [$requestId]);
            if (!$q) j_err('Set a price / quotation first.');

            $existing = db_one('SELECT * FROM contracts WHERE quote_id = ?', [$q['id']]);
            if ($existing) {
                j_ok(['contract' => map_contract($existing), 'created' => false]);
            }
            $cid = new_id('C-');
            db_run(
                'INSERT INTO contracts (id,request_id,quote_id,tracking_id,client_name,project_title,scope,duration,total,status,client_sig,client_signed_at,admin_sig,admin_signed_at,created_at)
                 VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,now())',
                [$cid, $requestId, $q['id'], $req['tracking_id'], $req['name'], $req['title'],
                    mb_substr($req['description'], 0, 400), '45 days', $q['total'], 'Sent', null, null, null, null]
            );
            $c = map_contract(db_one('SELECT * FROM contracts WHERE id = ?', [$cid]));
            ensure_deposit_invoice(db_one('SELECT * FROM contracts WHERE id = ?', [$cid]));
            notify_contract_ready($c, 'create');
            j_ok(['contract' => $c, 'created' => true]);
        }

        /**
         * Contracts are permanent legal records — they are stored for both the
         * client and the admin and are NEVER deleted. The action is kept only
         * so an old bookmarked button gets a clear message instead of a 404.
         */
        case 'delete': {
            require_csrf();
            require_admin();
            j_err('Contracts cannot be deleted — they are permanent legal records kept for both the client and the company. Archive it or correct it instead.', 409);
        }

        default:
            j_err('Unknown contracts action.', 404);
    }
} catch (Throwable $e) {
    error_log('[api/contracts] ' . $e->getMessage());
    j_err('Server error. Please try again.', 500);
}
