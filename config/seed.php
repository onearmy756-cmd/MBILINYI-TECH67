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

    /* portfolio_projects (from PORTFOLIO array in app.js) ----- */
    $ppSql = 'INSERT INTO portfolio_projects (id,title,category,description,image_url,stat_label,link_url,sort_order,is_active,created_at)
              VALUES (?,?,?,?,?,?,?,?,?,?)' . $noop;
    $ins($ppSql, [
        ['P-01', 'PharmaPlus Management Suite', 'Pharmacy • React + Python + M-Pesa',
            'Stock, expiry alerts, sales & M-Pesa receipts for 12 branches.',
            'https://images.unsplash.com/photo-1576091160399-112ba8d25d1d?w=800&q=80',
            '+38% revenue tracked', '', 0, 1, '2026-04-10 09:00:00'],
        ['P-02', 'ShulePay Fees Portal', 'Education • React + Node + SMS',
            'Fee tracking, receipts & parent SMS for 3,000+ students.',
            'https://images.unsplash.com/photo-1503676260728-1c00da094a0b?w=800&q=80',
            'Zero fee leakage', '', 1, 1, '2026-05-12 09:00:00'],
        ['P-03', 'KilimoFresh WhatsApp Bot', 'AI • Python • Swahili NLP',
            'Swahili ordering bot handling 500+ orders/week.',
            'https://images.unsplash.com/photo-1464226184884-fa280b87c399?w=800&q=80',
            '4.9★ customer rating', '', 2, 1, '2026-06-08 09:00:00'],
        ['P-04', 'RustPay Microservice', 'Fintech • Rust + PostgreSQL',
            '12ms transaction engine processing 1M+ records daily.',
            'https://images.unsplash.com/photo-1551288049-bebda4e38f71?w=800&q=80',
            '12ms avg latency', '', 3, 1, '2026-07-05 09:00:00'],
        ['P-05', 'Hotel Booking + R Analytics', 'Hospitality • R + React',
            'Bookings, occupancy forecasts & revenue dashboards.',
            'https://images.unsplash.com/photo-1566073771259-6a8506099945?w=800&q=80',
            '+27% occupancy', '', 4, 1, '2026-08-10 09:00:00'],
        ['P-06', 'SaccoGuard Security Hardening', 'Security • Audit + WAF',
            'Full pentest & hardening for a 8,000-member SACCO.',
            'https://images.unsplash.com/photo-1550751827-4bd374c3f58b?w=800&q=80',
            '0 breaches since', '', 5, 1, '2026-09-01 09:00:00'],
    ]);

    /* testimonials (from TESTIS array in app.js) --------------- */
    $tmSql = 'INSERT INTO testimonials (id,name,role,text,stars,sort_order,is_active,created_at)
              VALUES (?,?,?,?,?,?,?,?)' . $noop;
    $ins($tmSql, [
        ['TM-01', 'Amina Juma', 'Director, Amina Pharmacy — Dar es Salaam',
            'Jackson built our entire pharmacy system with M-Pesa. Official quotation, BRELA contract, training — everything was professional. Stock losses dropped to almost zero.',
            5, 0, 1, '2026-09-05 09:00:00'],
        ['TM-02', 'Eng. Baraka Mwansa', 'CEO, Kilimo Fresh',
            'The Swahili WhatsApp bot takes orders while we sleep. Clients think we hired 5 new staff! Best investment we made this year.',
            5, 1, 1, '2026-09-06 09:00:00'],
        ['TM-03', 'Sarah Nkya', 'Headmistress, Moshi Academy',
            'Fee collection used to be chaos. Now parents get instant SMS receipts and we see every shilling. The maintenance plan is worth every cent.',
            5, 2, 1, '2026-09-07 09:00:00'],
        ['TM-04', 'David Mwakalinga', 'CTO, Fintech Startup — Nairobi',
            'Their Rust microservice handles our peak load effortlessly. International quality, African pricing. The security audit alone saved us from a major breach.',
            5, 3, 1, '2026-09-08 09:00:00'],
    ]);
}
