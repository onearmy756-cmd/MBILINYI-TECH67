<?php
declare(strict_types=1);
require_once __DIR__ . '/../config/app.php';

header('Content-Type: application/json; charset=utf-8');

/* Public tracking lookup — powers the Track Order section and the chatbot. */
try {
    $id = strtoupper(str_param('id'));
    if ($id === '') j_err('Enter a Tracking ID.', 400);

    $req = db_one('SELECT * FROM requests WHERE upper(tracking_id) = ?', [$id]);
    if (!$req) j_err('Tracking ID not found. Check your ID or contact 0796 752 645 on WhatsApp.', 404);

    $quote = db_one('SELECT * FROM quotes WHERE request_id = ? ORDER BY created_at DESC LIMIT 1', [$req['id']]);
    $contract = db_one('SELECT * FROM contracts WHERE request_id = ? ORDER BY created_at DESC LIMIT 1', [$req['id']]);

    j_ok([
        'request' => map_request($req),
        'quote' => $quote ? map_quote($quote) : null,
        'contract' => $contract ? map_contract($contract) : null,
    ]);
} catch (Throwable $e) {
    error_log('[api/track] ' . $e->getMessage());
    j_err('Server error. Please try again.', 500);
}
