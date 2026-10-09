<?php
declare(strict_types=1);

function iso(?string $ts): string {
    if (!$ts) return '';
    $t = strtotime($ts);
    return $t ? date('c', $t) : '';
}

function map_user(array $r): array {
    unset($r['password_hash']);
    return [
        'id' => $r['id'],
        'name' => $r['name'],
        'email' => $r['email'],
        'phone' => $r['phone'] ?? '',
        'company' => $r['company'] ?? '',
        'role' => $r['role'],
        'status' => $r['status'],
        'createdAt' => iso($r['created_at'] ?? null),
    ];
}

function map_request(array $r): array {
    $tech = json_decode($r['tech'] ?? '[]', true);
    return [
        'id' => $r['id'],
        'trackingId' => $r['tracking_id'],
        'userId' => $r['user_id'],
        'name' => $r['name'],
        'email' => $r['email'],
        'phone' => $r['phone'],
        'company' => $r['company'],
        'service' => $r['service'],
        'title' => $r['title'],
        'description' => $r['description'],
        'tech' => is_array($tech) ? $tech : [],
        'platform' => $r['platform'],
        'budget' => $r['budget'],
        'deadline' => $r['deadline'],
        'priority' => $r['priority'],
        'status' => $r['status'],
        'adminNote' => $r['admin_note'],
        'createdAt' => iso($r['created_at'] ?? null),
    ];
}

function map_quote(array $q): array {
    $items = json_decode($q['items'] ?? '[]', true);
    return [
        'id' => $q['id'],
        'requestId' => $q['request_id'],
        'trackingId' => $q['tracking_id'],
        'userId' => $q['user_id'],
        'items' => is_array($items) ? $items : [],
        'subtotal' => (float)$q['subtotal'],
        'vat' => (float)$q['vat'],
        'total' => (float)$q['total'],
        'currency' => $q['currency'],
        'validUntil' => iso($q['valid_until'] ?? null),
        'status' => $q['status'],
        'adminMessage' => $q['admin_message'],
        'createdAt' => iso($q['created_at'] ?? null),
    ];
}

function map_contract(array $c): array {
    return [
        'id' => $c['id'],
        'requestId' => $c['request_id'],
        'quoteId' => $c['quote_id'],
        'trackingId' => $c['tracking_id'],
        'clientName' => $c['client_name'],
        'projectTitle' => $c['project_title'],
        'scope' => $c['scope'],
        'duration' => $c['duration'],
        'total' => (float)$c['total'],
        'status' => $c['status'],
        'clientSig' => ($c['client_sig'] ?? '') !== '' ? $c['client_sig'] : null,
        'clientSignedAt' => iso($c['client_signed_at'] ?? null),
        'adminSig' => ($c['admin_sig'] ?? '') !== '' ? $c['admin_sig'] : null,
        'adminSignedAt' => iso($c['admin_signed_at'] ?? null),
        'createdAt' => iso($c['created_at'] ?? null),
    ];
}

function map_invoice(array $v): array {
    return [
        'id' => $v['id'],
        'trackingId' => $v['tracking_id'],
        'userId' => $v['user_id'],
        'title' => $v['title'],
        'amount' => (float)$v['amount'],
        'status' => $v['status'],
        'method' => $v['method'],
        'dueDate' => iso($v['due_date'] ?? null),
        'createdAt' => iso($v['created_at'] ?? null),
    ];
}

function map_ticket(array $t, array $replies = []): array {
    $out = [
        'id' => $t['id'],
        'userId' => $t['user_id'],
        'name' => $t['name'],
        'email' => $t['email'],
        'subject' => $t['subject'],
        'message' => $t['message'],
        'status' => $t['status'],
        'createdAt' => iso($t['created_at'] ?? null),
        'replies' => [],
    ];
    foreach ($replies as $r) {
        $out['replies'][] = ['by' => $r['by_name'], 'text' => $r['body'], 'at' => iso($r['created_at'])];
    }
    return $out;
}

function map_portfolio_project(array $p): array {
    return [
        'id' => $p['id'],
        'title' => $p['title'],
        'category' => $p['category'] ?? '',
        'description' => $p['description'] ?? '',
        'imageUrl' => $p['image_url'] ?? '',
        'img' => $p['image_url'] ?? '',
        'statLabel' => $p['stat_label'] ?? '',
        's' => $p['stat_label'] ?? '',
        'linkUrl' => $p['link_url'] ?? '',
        'sortOrder' => (int)($p['sort_order'] ?? 0),
        'isActive' => (bool)($p['is_active'] ?? 1),
        'createdAt' => iso($p['created_at'] ?? null),
        'updatedAt' => iso($p['updated_at'] ?? null),
    ];
}

function map_testimonial(array $t): array {
    return [
        'id' => $t['id'],
        'name' => $t['name'],
        'role' => $t['role'] ?? '',
        'r' => $t['role'] ?? '',
        'text' => $t['text'] ?? '',
        't' => $t['text'] ?? '',
        'stars' => (int)($t['stars'] ?? 5),
        's' => (int)($t['stars'] ?? 5),
        'sortOrder' => (int)($t['sort_order'] ?? 0),
        'isActive' => (bool)($t['is_active'] ?? 1),
        'createdAt' => iso($t['created_at'] ?? null),
        'updatedAt' => iso($t['updated_at'] ?? null),
    ];
}
