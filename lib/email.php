<?php
declare(strict_types=1);

require_once __DIR__ . '/smtp.php';

/**
 * Fail-soft transactional email (Resend REST API + SMTP relay, PHP only, no SDK).
 * Any failure is logged and swallowed — an email problem must never block a request.
 */
function resend_send(array $to, string $subject, string $html): bool {
    try {
        $key = getenv('RESEND_API_KEY');
        if (!$key) {
            error_log('[mts-email] RESEND_API_KEY is not set — skipping: ' . $subject);
            return false;
        }
        $to = array_values(array_filter($to, fn($a) => is_string($a) && $a !== ''));
        if (!$to) return false;
        if (!function_exists('curl_init')) {
            error_log('[mts-email] curl extension missing');
            return false;
        }

        $payload = json_encode([
            'from' => 'Mbilinyi Tech Solutions <onboarding@resend.dev>',
            'to' => $to,
            'subject' => $subject,
            'html' => $html,
        ], JSON_UNESCAPED_UNICODE);

        $ch = curl_init('https://api.resend.com/emails');
        curl_setopt_array($ch, [
            CURLOPT_POST => true,
            CURLOPT_RETURNTRANSFER => true,
            CURLOPT_TIMEOUT => 15,
            CURLOPT_HTTPHEADER => [
                'Authorization: Bearer ' . $key,
                'Content-Type: application/json',
            ],
            CURLOPT_POSTFIELDS => $payload,
        ]);
        $res = curl_exec($ch);
        $code = (int)curl_getinfo($ch, CURLINFO_HTTP_CODE);
        $err = curl_error($ch);
        curl_close($ch);

        if ($res === false || $code >= 300) {
            error_log('[mts-email] failed (' . $code . ') ' . $err . ' — ' . $subject);
            return false;
        }
        return true;
    } catch (\Throwable $e) {
        error_log('[mts-email] ' . $e->getMessage());
        return false;
    }
}

function mts_email_shell(string $heading, string $bodyHtml, string $cta = '', string $ctaUrl = ''): string {
    $ctaHtml = '';
    if ($cta !== '' && $ctaUrl !== '') {
        $ctaHtml = '<p style="margin:22px 0"><a href="' . htmlspecialchars($ctaUrl, ENT_QUOTES) . '" style="background:linear-gradient(135deg,#06b6d4,#3b82f6,#8b5cf6);color:#fff;text-decoration:none;padding:12px 22px;border-radius:12px;font-weight:700;display:inline-block">' . htmlspecialchars($cta, ENT_QUOTES) . '</a></p>';
    }
    return '<div style="font-family:Arial,Helvetica,sans-serif;background:#050914;color:#E6ECF5;padding:26px">'
        . '<div style="max-width:560px;margin:0 auto;background:#0E1933;border:1px solid rgba(255,255,255,.1);border-radius:18px;padding:26px">'
        . '<div style="font-weight:800;font-size:17px;letter-spacing:.06em">MBILINYI <span style="color:#22d3ee">TECH</span> SOLUTIONS</div>'
        . '<div style="font-size:11px;color:#8EA0BF;margin-top:4px">BRELA Registered • TIN 192-147-522 • 0796 752 645</div>'
        . '<hr style="border:none;border-top:1px solid rgba(255,255,255,.1);margin:18px 0">'
        . '<h2 style="margin:0 0 10px;font-size:19px">' . $heading . '</h2>'
        . '<div style="font-size:14px;line-height:1.65;color:#C7D2E8">' . $bodyHtml . '</div>'
        . $ctaHtml
        . '<hr style="border:none;border-top:1px solid rgba(255,255,255,.1);margin:18px 0">'
        . '<div style="font-size:11px;color:#8EA0BF">CEO Jackson Mbilinyi • mbilinyitech@gmail.com • @mbilinyitech<br>Sent by Mbilinyi Tech Solutions — Innovate • Secure • Scale | YOUR PROBLEM, OUR SOLUTION</div>'
        . '</div></div>';
}

/**
 * Send through whichever transport is configured.
 *
 * Order: SMTP (Supabase-stored relay) → Resend REST API. Both are fail-soft —
 * `false` is returned on failure and logged, never thrown, so a mail problem
 * can never block a request.
 */
function mts_send(array $to, string $subject, string $html): bool {
    if (function_exists('smtp_configured') && smtp_configured()) {
        if (smtp_send($to, $subject, $html)) return true;
        error_log('[mts-email] SMTP transport failed — falling back to Resend if configured');
    }
    return resend_send($to, $subject, $html);
}

function mts_notify(array $to, string $subject, string $heading, string $body, string $cta = '', string $ctaUrl = ''): void {
    mts_send($to, $subject, mts_email_shell($heading, $body, $cta, $ctaUrl));
}

const MTS_CEO_EMAIL = 'mbilinyitech@gmail.com';

function notify_new_request(array $r): void {
    $body = '<p>Habari <strong>' . htmlspecialchars($r['name']) . '</strong>,</p>'
        . '<p>Asante! Tunapokea ombi lako la huduma.</p>'
        . '<p><strong>Tracking ID:</strong> <span style="color:#22d3ee;font-family:monospace">' . htmlspecialchars($r['trackingId']) . '</span><br>'
        . '<strong>Project:</strong> ' . htmlspecialchars($r['title']) . '<br>'
        . '<strong>Service:</strong> ' . htmlspecialchars($r['service']) . '</p>'
        . '<p>CEO Jackson Mbilinyi atakutumia <strong>nukuu rasmi ya bei</strong> ndani ya saa 24.</p>';
    mts_notify([$r['email'], MTS_CEO_EMAIL], 'Request received — ' . $r['trackingId'],
        'Ombi lako limepokelewa ✓', $body, 'Fuatilia ombi lako',
        'https://' . ($_SERVER['HTTP_HOST'] ?? 'localhost') . '/index.php#track');
}

function notify_quotation_sent(array $q, array $r): void {
    $body = '<p>Nukuu rasmi imepokelewa kwa ombi <strong>' . htmlspecialchars($q['trackingId']) . '</strong>.</p>'
        . '<p><strong>Project:</strong> ' . htmlspecialchars($r['title'] ?? '') . '<br>'
        . '<strong>Total (incl. 18% VAT):</strong> TZS ' . number_format((float)$q['total']) . '<br>'
        . '<strong>Valid until:</strong> ' . htmlspecialchars(substr((string)$q['validUntil'], 0, 10)) . '</p>'
        . '<p>Kagua na idhinisha kwenye portal yako ili kusaini mkataba na kuanza kazi.</p>';
    mts_notify([$r['email'] ?? '', MTS_CEO_EMAIL], 'Official quotation ' . number_format((float)$q['total']) . ' TZS — ' . $q['trackingId'],
        'Nukuu yako ya bei iko tayari 💰', $body, 'Kagua & Idhinisha',
        'https://' . ($_SERVER['HTTP_HOST'] ?? 'localhost') . '/client.php#quotes');
}

/**
 * FR-9 / Task 10 — email ya CEO mara tu mteja akiwa amesaini mkataba.
 * Inaitwa na notify_contract_ready($c, 'client').
 */
function notify_client_signed_contract(array $c): void {
    $adminLink = 'https://' . ($_SERVER['HTTP_HOST'] ?? 'localhost') . '/admin.php#contracts';
    $body = '<p>Klienti <strong>' . htmlspecialchars($c['clientName'] ?? '') . '</strong> amesaini mkataba <strong>'
        . htmlspecialchars($c['id']) . '</strong> (Tracking <strong>' . htmlspecialchars($c['trackingId'] ?? '') . '</strong>).</p>'
        . '<p><strong>' . htmlspecialchars($c['projectTitle']) . '</strong> — TZS ' . number_format((float)$c['total']) . '.</p>'
        . '<p>Sasa ni zamu yako (Admin/CEO) kupigia saini ndipo mradi uanze rasmi. Kumbuka kanuni ya <strong>50/50</strong>: '
        . 'deposit ya 50% (TZS ' . number_format((float)$c['total'] * 0.5) . ') ilipwe kabla ya kuanza kwa mradi.</p>';
    mts_notify([MTS_CEO_EMAIL], 'Contract ' . $c['id'] . ' — client signed, awaiting your countersignature',
        'Client amesaini ➜ saini yako inasubiri', $body, 'Saini Mkataba', $adminLink);
}

/**
 * FR-9 / Task 10 — email ya pande zote mbili (Client + CEO) mara mkataba ukishasainiwa.
 * Inajumuisha masharti ya malipo 50/50 na link ya deposit invoice.
 */
function notify_contract_fully_signed(array $c): void {
    $host = 'https://' . ($_SERVER['HTTP_HOST'] ?? 'localhost');
    $clientLink = $host . '/client.php#contracts';
    $deposit = (float)$c['total'] * 0.5;

    $terms = '<div style="margin-top:14px;padding:16px;background:linear-gradient(135deg,#06203a,#1a0b3a);'
        . 'border:1px solid rgba(34,211,238,.3);border-radius:14px">'
        . '<div style="font-size:11px;letter-spacing:.18em;text-transform:uppercase;font-weight:700;color:#22d3ee;margin-bottom:10px">'
        . 'PAYMENT TERMS — MASHARTI YA MALIPO (50/50)</div>'
        . '<p style="margin:0 0 8px"><strong style="color:#F5B301">1. DEPOSIT (50%):</strong> TZS ' . number_format($deposit)
        . ' — inalipwa <strong>kabla ya kazi kuanza</strong>. Mradi hauanzi hadi deposit ilipwe na CEO aweke saini yake.</p>'
        . '<p style="margin:0 0 8px"><strong style="color:#F5B301">2. MID-PROJECT (50%):</strong> TZS ' . number_format($deposit)
        . ' — inalipwa wakati mradi umekamilikia 50% (kati ya timeline), <strong>kabla ya delivery ya mwisho</strong>.</p>'
        . '<p style="margin:0;color:#8EA0BF;font-size:12px">Si "Deposit + Delivery". Final source code, training na handover '
        . 'hazinapelekwi kabla installment ya 2 haijawekwa. Late payment: penalty 2% kwa wiki.</p></div>';

    $body = '<p>Mkataba <strong>' . htmlspecialchars($c['id']) . '</strong> umesainiwa na pande zote mbili (Client + CEO) ✅</p>'
        . '<p><strong>' . htmlspecialchars($c['projectTitle']) . '</strong> — TZS ' . number_format((float)$c['total'])
        . ' • Tracking <strong>' . htmlspecialchars($c['trackingId'] ?? '') . '</strong></p>'
        . '<p>Client: <strong>' . htmlspecialchars($c['clientName'] ?? '') . '</strong>. Mradi umeanza (In Progress).</p>'
        . $terms
        . '<p style="margin-top:14px">Deposit invoice (50%) inapatikana kwenye <strong>Invoices &amp; Payments</strong> ya portal yako. '
        . 'Lipa kwa M-Pesa <strong>0796 752 645</strong> (Jackson Mbilinyi) tuma risiti kwa WhatsApp.</p>';

    $to = array_values(array_unique(array_filter([MTS_CEO_EMAIL, contract_client_email($c)])));
    mts_notify($to, 'Contract ' . $c['id'] . ' fully signed — project in progress',
        'Mkataba umekamilika ✅', $body, 'Fungua Contract', $clientLink);
}

/**
 * Helper — email ya mteja anayehusiana na mkataba (kutoka jedwali la requests).
 * Hutumika kwa FR-9 ili client apokee email ya mkataba uliosainiwa.
 */
function contract_client_email(array $c): string {
    try {
        if (!empty($c['clientEmail']) && filter_var($c['clientEmail'], FILTER_VALIDATE_EMAIL)) {
            return (string)$c['clientEmail'];
        }
        $rid = $c['requestId'] ?? '';
        if ($rid !== '' && function_exists('db_one')) {
            $r = db_one('SELECT email FROM requests WHERE id = ?', [$rid]);
            if ($r && filter_var($r['email'] ?? '', FILTER_VALIDATE_EMAIL)) return (string)$r['email'];
        }
    } catch (\Throwable $e) {
        error_log('[contract email] ' . $e->getMessage());
    }
    return '';
}

function notify_contract_ready(array $c, string $stage = 'create'): void {
    $link = 'https://' . ($_SERVER['HTTP_HOST'] ?? 'localhost') . '/client.php#contracts';
    if ($stage === 'admin') {
        notify_contract_fully_signed($c);
        return;
    }
    if ($stage === 'client') {
        notify_client_signed_contract($c);
        return;
    }
    $body = '<p>Mkataba <strong>' . htmlspecialchars($c['id']) . '</strong> umetayarishwa kwa project ya <strong>'
        . htmlspecialchars($c['projectTitle']) . '</strong> (TZS ' . number_format((float)$c['total']) . ').</p>'
        . '<p>Fungua portal yako kusaini kwa kidole (e-signature) <strong>kwanza</strong>, kisha CEO atasaini.</p>';
    mts_notify([MTS_CEO_EMAIL], 'Contract ' . $c['id'] . ' — awaiting client e-signature',
        'Mkataba umewekwa ➜ saini inasubiri', $body, 'Saini Mkataba', $link);
}

function notify_ticket_reply(array $t, string $byName, string $text): void {
    $body = '<p><strong>' . htmlspecialchars($t['subject']) . '</strong></p>'
        . '<p>' . nl2br(htmlspecialchars($t['message'])) . '</p>'
        . '<p style="margin-top:14px;padding:12px;background:#0A1226;border-radius:10px"><strong>' . htmlspecialchars($byName) . ':</strong><br>'
        . nl2br(htmlspecialchars($text)) . '</p>';
    $to = ($byName === 'Jackson Mbilinyi (CEO)') ? [$t['email'], MTS_CEO_EMAIL] : [MTS_CEO_EMAIL];
    mts_notify($to, 'Support ticket reply — ' . $t['subject'], 'Reply kwenye ticket yako 🎧', $body, 'Fungua Support Center',
        'https://' . ($_SERVER['HTTP_HOST'] ?? 'localhost') . '/client.php#support');
}

/* ---------- Auth emails (Task 3: OTP + Password Reset) ---------- */

function notify_login_otp(string $email, string $code): void {
    if (!filter_var($email, FILTER_VALIDATE_EMAIL)) return;
    $heading = 'Verification Code — Namba ya Uhakiki';
    $body = '<p>Habari,</p>'
        . '<p>Umekoja login kwenye Mbilinyi Tech Solutions portal. Tuma namba hii ya 6 tarakimu kwenye uhalirishaji wa login:</p>'
        . '<div style="margin:18px 0;padding:18px 22px;background:linear-gradient(135deg,#06203a,#1a0b3a);border:1px solid rgba(34,211,238,.3);border-radius:14px;text-align:center">'
        . '<div style="font-size:11px;color:#8EA0BF;letter-spacing:.2em;text-transform:uppercase;font-weight:700;margin-bottom:6px">Verification Code — Valid 10 min</div>'
        . '<div style="font-size:34px;font-weight:800;letter-spacing:.3em;color:#22d3ee;font-family:Space Grotesk,Arial,sans-serif">' . htmlspecialchars($code) . '</div>'
        . '</div>'
        . '<p>Usifanye matumizi ya namba hii na mtu yeyote. Mfumo utakataa namba baada ya dakika 10 au baada ya majaribio 3 yatokayo.</p>'
        . '<p>Kama hujajaribu ku-login, toa taarifa kwa CEO 0796 752 645 mara moja.</p>';
    mts_notify([$email], 'Your MTS Login OTP — ' . $code, $heading, $body);
}

function notify_password_reset(string $email, string $tokenLink): void {
    if (!filter_var($email, FILTER_VALIDATE_EMAIL)) return;
    $heading = 'Badilisha Neno la Siri — Reset Your Password';
    $body = '<p>Habari,</p>'
        . '<p>Umetuma ombi la kubadilisha neno la Siri kwenye Mbilinyi Tech Solutions portal. Bonyeza kitufe hivi chini (itafanya kazi kwa masaa 1):</p>'
        . '<div style="margin-top:16px;margin-bottom:16px">'
        . '<a href="' . htmlspecialchars($tokenLink) . '" style="display:inline-block;padding:14px 28px;background:linear-gradient(135deg,#06b6d4,#3b82f6,#8b5cf6);color:#fff;text-decoration:none;border-radius:14px;font-weight:700;letter-spacing:.02em;font-size:14px">🔐 Badilisha Neno la Siri / Reset Password</a>'
        . '</div>'
        . '<p>Kama hunataka kubadilisha neno la Siri, puuza barua hii — hakuna mabadiliko yaliyotokea.</p>'
        . '<p>Link haiweswi kutumika kwa mtu mwingine, na inakataa baada ya matumizi ya kwanza.</p>';
    mts_notify([$email], 'MTS — Reset your password (1 hour valid)', $heading, $body,
        'Badilisha Neno la Siri', $tokenLink);
}
