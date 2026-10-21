<?php
declare(strict_types=1);

/** Demo data — every section has live demo records so all demo buttons work. */
function mts_seed(PDO $pdo): void {
    $ins = function (string $sql, array $rows) use ($pdo): void {
        $st = $pdo->prepare($sql);
        foreach ($rows as $r) $st->execute($r);
    };
    $noop = ' ON DUPLICATE KEY UPDATE id = id';

    /* users ------------------------------------------------ */
    $userSql = 'INSERT INTO users (id,name,email,phone,company,password_hash,role,status,created_at)
                VALUES (?,?,?,?,?,?,?,?,?)' . $noop;
    $ins($userSql, [
        ['U-ADMIN', 'Jackson Mbilinyi', 'admin@mbilinyitech.co.tz', '0796752645', 'Mbilinyi Tech Solutions',
            password_hash('Admin@2026', PASSWORD_DEFAULT), 'admin', 'Active', '2024-01-15 08:00:00'],
        ['U-DEMO', 'Amina Juma', 'demo@client.com', '0712345678', 'Amina Pharmacy Ltd',
            password_hash('demo123', PASSWORD_DEFAULT), 'client', 'Active', '2025-08-02 09:00:00'],
        ['U-1002', 'Baraka Mwansa', 'baraka@kilimo.co.tz', '0755123456', 'Kilimo Fresh',
            password_hash('demo123', PASSWORD_DEFAULT), 'client', 'Active', '2025-11-10 09:00:00'],
        ['U-1003', 'Sarah Nkya', 'sarah@edu.go.tz', '0767890123', 'Moshi Academy',
            password_hash('demo123', PASSWORD_DEFAULT), 'client', 'Suspended', '2026-01-05 09:00:00'],
    ]);

    /* requests --------------------------------------------- */
    $reqSql = 'INSERT INTO requests
        (id,tracking_id,user_id,name,email,phone,company,service,title,description,tech,platform,budget,deadline,priority,status,admin_note,created_at)
        VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)' . $noop;
    $ins($reqSql, [
        ['R-1', 'MTS-2026-4810', 'U-DEMO', 'Amina Juma', 'demo@client.com', '0712345678', 'Amina Pharmacy Ltd',
            'Custom Software Development', 'Pharmacy Stock, Sales & M-Pesa System',
            'Need full pharmacy system: stock alerts, expiry tracking, sales with receipt printing, M-Pesa integration, daily reports, 3 users with roles.',
            json_encode(['React', 'Python', 'PostgreSQL']), 'Web + Mobile', 'TZS 1.5M – 5M ($600 – $2,000)',
            '2026-11-30', 'Urgent', 'Quoted',
            'Reviewed — quotation sent. Awaiting client approval.', '2026-09-20 10:00:00'],
        ['R-2', 'MTS-2026-5523', 'U-1002', 'Baraka Mwansa', 'baraka@kilimo.co.tz', '0755123456', 'Kilimo Fresh',
            'AI Solutions & Fine-Tuning', 'Swahili WhatsApp Chatbot for Orders',
            'WhatsApp chatbot in Swahili to take vegetable orders, send price lists, confirm delivery zones in Dar es Salaam.',
            json_encode(['Python', 'React']), 'AI / Chatbot Solution', 'TZS 500K – 1.5M ($200 – $600)',
            '2026-10-30', 'Normal', 'Under Review',
            'Analyst assigned. Scoping WhatsApp API costs.', '2026-09-28 11:30:00'],
    ]);

    /* quote ------------------------------------------------- */
    $items = json_encode([
        ['desc' => 'System Analysis & UI/UX Design', 'qty' => 1, 'price' => 450000],
        ['desc' => 'Pharmacy Core (Stock, Sales, Receipts, Roles)', 'qty' => 1, 'price' => 1800000],
        ['desc' => 'M-Pesa Daraja API Integration', 'qty' => 1, 'price' => 550000],
        ['desc' => 'Deployment, Training & 6-Mo Support', 'qty' => 1, 'price' => 400000],
    ]);
    $ins(
        'INSERT INTO quotes (id,request_id,tracking_id,user_id,items,subtotal,vat,total,currency,valid_until,status,admin_message,created_at)
         VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?)' . $noop,
        [['Q-9001', 'R-1', 'MTS-2026-4810', 'U-DEMO', $items, 3200000, 576000, 3776000, 'TZS',
            '2026-10-20 23:59:59', 'Sent',
            'Official quotation. Includes BRELA contract, source code handover & training for 3 staff.',
            '2026-09-22 09:00:00']]
    );

    /* contract (awaits the client signature first) --------- */
    $ins(
        'INSERT INTO contracts (id,request_id,quote_id,tracking_id,client_name,project_title,scope,duration,total,status,client_sig,client_signed_at,admin_sig,admin_signed_at,created_at)
         VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)' . $noop,
        [['C-7001', 'R-1', 'Q-9001', 'MTS-2026-4810', 'Amina Juma', 'Pharmacy Stock, Sales & M-Pesa System',
            'Full pharmacy management system as per approved quotation Q-9001, including deployment, training and 6 months maintenance.',
            '45 days', 3776000, 'Sent', null, null, null, null, '2026-09-23 09:00:00']]
    );

    /* invoice ----------------------------------------------- */
    $ins(
        'INSERT INTO invoices (id,tracking_id,user_id,title,amount,status,method,due_date,created_at)
         VALUES (?,?,?,?,?,?,?,?,?)' . $noop,
        [['INV-3001', 'MTS-2026-4810', 'U-DEMO', 'Pharmacy System — 50% Deposit', 1888000, 'Unpaid',
            'M-Pesa / Bank', '2026-10-25 23:59:59', '2026-09-23 09:30:00']]
    );

    /* ticket + reply ---------------------------------------- */
    $ins(
        'INSERT INTO tickets (id,user_id,name,email,subject,message,status,created_at)
         VALUES (?,?,?,?,?,?,?,?)' . $noop,
        [['T-1', 'U-DEMO', 'Amina Juma', 'demo@client.com', 'Can I pay in two installments?',
            'Hello, is it possible to split the payment 50/50?', 'Open', '2026-09-25 08:00:00']]
    );
    $ins(
        'INSERT INTO ticket_replies (id,ticket_id,by_name,body,created_at)
         VALUES (?,?,?,?,?)' . $noop,
        [['TR-1', 'T-1', 'Jackson Mbilinyi (CEO)',
            'Yes — 50% to start, 50% on delivery. I will reflect this on your invoice.',
            '2026-09-26 10:00:00']]
    );

    /* portfolio_projects + testimonials -------------------------
     * Deliberately NOT seeded: the company is new and has no delivered projects
     * or client reviews yet. The admin adds real ones from the Portfolio and
     * Testimonials tabs, and they appear on the public site automatically. */
}
