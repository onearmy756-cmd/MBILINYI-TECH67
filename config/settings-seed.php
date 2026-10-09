<?php
declare(strict_types=1);

/**
 * Idempotent seed for `settings` + `pricing_plans`.
 *
 * Only inserts when a row is missing, so the CEO can edit values from the admin
 * panel without the next request quietly reverting them. Everything here is the
 * starting point the system boots with; the admin UI owns it afterwards.
 */

function mts_seed_settings(PDO $pdo): void {
    $now = date('Y-m-d H:i:s');

    /* ---------------- payment + company defaults ---------------- */
    $defaults = [
        /* --- Bank transfer --- */
        'pay_bank_name'    => 'NMB Bank',
        'pay_bank_account' => '51710099563',
        'pay_bank_holder'  => 'JACKSON MBILINYI',
        'pay_bank_branch'  => 'Tanzania',
        'pay_bank_swift'   => 'NMBTZTZ',

        /* --- Mobile money --- */
        'pay_mobile_network' => 'Vodacom',
        'pay_mobile_number'  => '0796752645',
        'pay_mobile_holder'  => 'JACKSON MBILINYI',

        /* --- Who is asking for payment / general contact --- */
        'pay_contact_person' => 'Jackson Mbilinyi (CEO)',
        'pay_currency'       => 'TZS',
        'pay_instructions'   => 'Lipa kwa NMB Bank (51710099563) au Vodacom M-Pesa (0796752645) — jina: JACKSON MBILINYI. Tuma risiti kwa WhatsApp 0796 752 645 mara baada ya malipo.',

        /* --- Company --- */
        'company_location' => 'Dodoma, Tanzania',
        'company_work_mode'=> 'Online / Remote',
        'company_domain'   => 'mbilinyitech.co.tz',
        'company_email'    => 'info@mbilinyitech.co.tz',
        'company_phone'    => '0796 752 645',
    ];

    /* --- Service prices (admin-editable; chatbot + pricing page read these) ---
     * Keys are price_<code>; labels live in service_price_list(). */
    $servicePrices = [
        'website_basic'  => 300000,
        'website_business'=> 800000,
        'ecommerce'      => 1500000,
        'mobile_app'     => 2500000,
        'custom_system'  => 3000000,
        'whatsapp_bot'   => 600000,
        'ai_finetune'    => 1200000,
        'security_audit' => 800000,
        'pentest'        => 2500000,
        'maintenance'    => 150000,
        'brela'          => 150000,
        'tra_tax'        => 100000,
    ];
    foreach ($servicePrices as $code => $price) {
        $defaults['price_' . $code] = (string)$price;
    }

    $ins = $pdo->prepare('INSERT IGNORE INTO settings (skey, svalue, updated_at) VALUES (?, ?, ?)');
    foreach ($defaults as $k => $v) {
        $ins->execute([$k, (string)$v, $now]);
    }

    /* ---------------- pricing plans ---------------- */
    $count = (int)$pdo->query('SELECT COUNT(*) FROM pricing_plans')->fetchColumn();
    if ($count > 0) return;

    /* group_name mirrors the two blocks on pricing.php: maint | sec */
    $plans = [
        ['P-STDCARE', 'maint', 'Starter Care',        150000,  '/month',    1, 1,
         '["Monthly health check & updates","Bug fixes (up to 5 hrs)","Weekly backups","Email support (48hr)","Uptime monitoring"]'],
        ['P-BIZSLA',  'maint', 'Business SLA',        450000,  '/month',    1, 2,
         '["Everything in Starter","Priority fixes (20 hrs) + 24hr response","Security patches & SSL","Monthly performance report","WhatsApp priority line","Small feature tweaks"]'],
        ['P-ENT247',  'maint', 'Enterprise 24/7',     1200000, '/month',    1, 3,
         '["Dedicated engineer","Unlimited critical fixes","99.9% uptime guarantee","Daily backups + DR","On-call 24/7","Quarterly security audit"]'],
        ['P-SECAUD',  'sec',   'Security Audit',      800000,  'one-time',  1, 1,
         '["Full OWASP Top-10 scan","Vulnerability report + fixes plan","SSL & headers hardening","1 staff training session"]'],
        ['P-PENTEST', 'sec',   'Penetration Test',    2500000, 'per system',1, 2,
         '["Manual + automated pentest","Exploit verification","Executive + technical reports","Free re-test after fixes"]'],
        ['P-MGTSEC',  'sec',   'Managed Security',    900000,  '/month',    1, 3,
         '["Firewall + WAF management","Malware monitoring & removal","Monthly scans & patching","Incident response 24/7"]'],
    ];

    $insPlan = $pdo->prepare(
        'INSERT IGNORE INTO pricing_plans (id, code, group_name, name, price, currency, period, features, is_active, sort_order, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, 1, ?, ?)'
    );
    foreach ($plans as $p) {
        [$id, $group, $name, $price, $period, $active, $sort, $features] = $p;
        $insPlan->execute([$id, $id, $group, $name, $price, 'TZS', $period, $features, $sort, $now]);
    }
}
