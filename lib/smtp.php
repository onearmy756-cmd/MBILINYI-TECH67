<?php
declare(strict_types=1);

/**
 * Minimal SMTP client (STARTTLS + AUTH LOGIN) — no external dependencies.
 *
 * The project's mail is relayed through the SMTP account stored in Supabase
 * (smtp_host / smtp_port / smtp_user / smtp_pass). PHP has no built-in SMTP
 * client and hosts rarely have sendmail, so this speaks the protocol directly.
 *
 * Fail-soft by contract: every failure is logged and `false` is returned so an
 * email problem never blocks a request.
 */
function smtp_config(): array {
    return [
        'host'        => getenv('SMTP_HOST') ?: '',
        'port'        => (int)(getenv('SMTP_PORT') ?: 587),
        'user'        => getenv('SMTP_USER') ?: '',
        'pass'        => getenv('SMTP_PASS') ?: '',
        'from'        => getenv('SMTP_FROM') ?: '',
        'sender_name' => getenv('SMTP_SENDER_NAME') ?: 'Mbilinyi Tech Solutions',
        'timeout'     => (int)(getenv('SMTP_TIMEOUT') ?: 20),
    ];
}

function smtp_configured(): bool {
    $c = smtp_config();
    return $c['host'] !== '' && $c['user'] !== '' && $c['pass'] !== '';
}

/** Read one server reply line (handles multi-line "250-..." continuations). */
function smtp_read_line($fp): string {
    $line = '';
    while (($chunk = fgets($fp, 1024)) !== false) {
        $line .= $chunk;
        if (strlen($chunk) < 2 || substr($chunk, -2) !== "\r\n") continue;
        // a final line is "NNN " or "NNN\r\n"; continuation lines end with "-"
        if (isset($chunk[3]) && $chunk[3] === '-') continue;
        break;
    }
    return $line;
}

function smtp_expect($fp, string $expected, string $step): void {
    $reply = smtp_read_line($fp);
    if (strncmp($reply, $expected, strlen($expected)) !== 0) {
        throw new RuntimeException("SMTP {$step} failed — got: " . trim($reply));
    }
}

/**
 * Send one HTML email over SMTP.
 *
 * @param string|string[] $to  recipient(s)
 * @return bool true when the server accepted the message for delivery
 */
function smtp_send(array $to, string $subject, string $html): bool {
    $c = smtp_config();
    if (!smtp_configured()) {
        error_log('[smtp] not configured (SMTP_HOST / SMTP_USER / SMTP_PASS missing) — skipping: ' . $subject);
        return false;
    }

    $to = array_values(array_unique(array_filter($to, fn($a) => is_string($a) && filter_var($a, FILTER_VALIDATE_EMAIL))));
    if (!$to) {
        error_log('[smtp] no valid recipients — skipping: ' . $subject);
        return false;
    }

    $fromAddr = $c['from'] !== '' ? $c['from'] : $c['user'];
    if (!filter_var($fromAddr, FILTER_VALIDATE_EMAIL)) $fromAddr = $c['user'];

    $host = $c['host'];
    $port = $c['port'];
    $errno = 0; $errstr = '';
    $target = ($port === 465 ? 'ssl://' : '') . $host;

    $fp = @stream_socket_client("tcp://{$target}:{$port}", $errno, $errstr, $c['timeout']);
    if (!$fp) {
        error_log("[smtp] connect failed to {$host}:{$port} — {$errstr} ({$errno})");
        return false;
    }
    stream_set_timeout($fp, $c['timeout']);

    try {
        smtp_expect($fp, '220', 'connect');
        fwrite($fp, 'EHLO ' . (getenv('APP_DOMAIN') ?: 'localhost') . "\r\n");
        smtp_expect($fp, '250', 'EHLO');

        if ($port === 587) {
            fwrite($fp, "STARTTLS\r\n");
            smtp_expect($fp, '220', 'STARTTLS');
            // @ — a missing openssl build must fall through to the catch below
            // instead of printing a warning into the response body.
            if (!@stream_socket_enable_crypto($fp, true, STREAM_CRYPTO_METHOD_TLS_CLIENT)) {
                throw new RuntimeException('TLS negotiation failed (openssl/crypto unavailable)');
            }
            fwrite($fp, 'EHLO ' . (getenv('APP_DOMAIN') ?: 'localhost') . "\r\n");
            smtp_expect($fp, '250', 'EHLO after STARTTLS');
        }

        fwrite($fp, "AUTH LOGIN\r\n");
        smtp_expect($fp, '334', 'AUTH');
        fwrite($fp, base64_encode($c['user']) . "\r\n");
        smtp_expect($fp, '334', 'AUTH username');
        fwrite($fp, base64_encode($c['pass']) . "\r\n");
        smtp_expect($fp, '235', 'AUTH password');

        $angleFrom = '<' . $fromAddr . '>';
        fwrite($fp, "MAIL FROM:{$angleFrom}\r\n");
        smtp_expect($fp, '250', 'MAIL FROM');

        foreach ($to as $rcpt) {
            fwrite($fp, 'RCPT TO:<' . $rcpt . ">\r\n");
            smtp_expect($fp, '250', 'RCPT TO ' . $rcpt);
        }

        fwrite($fp, "DATA\r\n");
        smtp_expect($fp, '354', 'DATA');

        $headers  = 'From: ' . smtp_header_pair($c['sender_name'], $fromAddr) . "\r\n";
        $headers .= 'To: ' . implode(', ', array_map(fn($t) => smtp_header_pair('', $t), $to)) . "\r\n";
        $headers .= 'Subject: ' . smtp_mime_encode($subject) . "\r\n";
        $headers .= 'Date: ' . date('r') . "\r\n";
        $headers .= "Message-ID: <" . bin2hex(random_bytes(12)) . '@' . (getenv('APP_DOMAIN') ?: 'localhost') . ">\r\n";
        $headers .= "MIME-Version: 1.0\r\n";
        $headers .= "Content-Type: text/html; charset=UTF-8\r\n";
        $headers .= "Content-Transfer-Encoding: 8bit\r\n";
        $headers .= "\r\n";

        // dot-stuffing per RFC 5321, and normalise line endings
        $body = str_replace(["\r\n", "\r", "\n"], "\r\n", $html);
        $body = preg_replace('/^\./m', '..', $body);
        $payload = $headers . $body . "\r\n.\r\n";

        fwrite($fp, $payload);
        smtp_expect($fp, '250', 'message send');

        fwrite($fp, "QUIT\r\n");
        fclose($fp);
        return true;
    } catch (\Throwable $e) {
        error_log('[smtp] ' . $e->getMessage() . ' — subject: ' . $subject);
        @fclose($fp);
        return false;
    }
}

/** "Display Name" <addr> with RFC 2047 encoding when the name is not ASCII. */
function smtp_header_pair(string $name, string $addr): string {
    $name = trim($name);
    if ($name === '') return $addr;
    if (!preg_match('/[^\x20-\x7E]/', $name)) {
        return sprintf('"%s" <%s>', str_replace(['"', '\\'], ['', '\\\\'], $name), $addr);
    }
    return '=?' . 'UTF-8' . '?B?' . base64_encode($name) . '?= <' . $addr . '>';
}

/** RFC 2047 encoded-word for a subject that may contain non-ASCII characters. */
function smtp_mime_encode(string $subject): string {
    $subject = str_replace(["\r", "\n"], ' ', $subject);
    if (!preg_match('/[^\x20-\x7E]/', $subject)) return $subject;
    return '=?UTF-8?B?' . base64_encode($subject) . '?=';
}
