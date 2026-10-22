<?php
declare(strict_types=1);

/**
 * Passkeys (WebAuthn) — a dependency-free implementation.
 *
 * The host has no Composer, so the pieces WebAuthn needs live here: a CBOR
 * reader for the attestation object, a COSE→PEM converter so OpenSSL can
 * verify signatures, and the two verification routines (registration and
 * assertion) described by the W3C spec.
 *
 * Attestation statements are deliberately NOT validated. Validating them
 * requires a curated list of authenticator trust anchors and buys nothing for
 * a site like this — we only care that the credential was created by the
 * device that now holds the private key. Every registration is therefore
 * treated as "none" attestation.
 */

/* ---------- base64url (RFC 4648 §5) ---------- */

function wa_b64u_dec(string $s): string {
    $s = strtr($s, '-_', '+/');
    $pad = strlen($s) % 4;
    if ($pad !== 0) $s .= str_repeat('=', 4 - $pad);
    $raw = base64_decode($s, true);
    if ($raw === false) throw new RuntimeException('Malformed base64url value.');
    return $raw;
}

function wa_b64u_enc(string $raw): string {
    return rtrim(strtr(base64_encode($raw), '+/', '-_'), '=');
}

/* ---------- CBOR (RFC 8949 subset used by WebAuthn) ---------- */

/**
 * Decode one CBOR item, advancing $off past it.
 *
 * Handles the subset that authenticators actually emit: unsigned/negative
 * integers, byte and text strings, arrays, maps, tags (unwrapped) and the
 * simple values. Indefinite-length items are rejected rather than guessed at —
 * no authenticator produces them for a COSE key.
 */
function wa_cbor(string $b, int &$off): mixed {
    if ($off >= strlen($b)) throw new RuntimeException('CBOR: unexpected end of data.');
    $ib = ord($b[$off++]);
    $major = $ib >> 5;
    $ai = $ib & 0x1F;

    if ($ai === 31) throw new RuntimeException('CBOR: indefinite lengths are not supported.');
    if ($ai < 24)       $val = $ai;
    elseif ($ai === 24) { $val = ord($b[$off]); $off += 1; }
    elseif ($ai === 25) { $val = unpack('n', substr($b, $off, 2))[1]; $off += 2; }
    elseif ($ai === 26) { $val = unpack('N', substr($b, $off, 4))[1]; $off += 4; }
    elseif ($ai === 27) { $val = unpack('J', substr($b, $off, 8))[1]; $off += 8; }
    else throw new RuntimeException('CBOR: reserved additional information ' . $ai . '.');

    switch ($major) {
        case 0: return $val;                       // unsigned
        case 1: return -1 - $val;                  // negative
        case 2:                                    // byte string
        case 3:                                    // text string
            $s = substr($b, $off, $val);
            if (strlen($s) !== $val) throw new RuntimeException('CBOR: truncated string.');
            $off += $val;
            return $s;
        case 4:                                    // array
            $arr = [];
            for ($i = 0; $i < $val; $i++) $arr[] = wa_cbor($b, $off);
            return $arr;
        case 5:                                    // map
            $map = [];
            for ($i = 0; $i < $val; $i++) {
                $k = wa_cbor($b, $off);
                $map[is_int($k) ? $k : (string)$k] = wa_cbor($b, $off);
            }
            return $map;
        case 6:                                    // tag — unwrap the value
            return wa_cbor($b, $off);
        case 7:                                    // simple values / floats
            return match ($ai) {
                20 => false,
                21 => true,
                22, 23 => null,
                default => $val,
            };
    }
    throw new RuntimeException('CBOR: unsupported major type ' . $major . '.');
}

/* ---------- DER / PEM ---------- */

function wa_der_len(int $n): string {
    if ($n < 0x80) return chr($n);
    $bytes = '';
    while ($n > 0) { $bytes = chr($n & 0xFF) . $bytes; $n >>= 8; }
    return chr(0x80 | strlen($bytes)) . $bytes;
}

function wa_der(int $tag, string $content): string {
    return chr($tag) . wa_der_len(strlen($content)) . $content;
}

function wa_der_int(string $bytes): string {
    $bytes = ltrim($bytes, "\x00");
    if ($bytes === '') $bytes = "\x00";
    if (ord($bytes[0]) > 0x7F) $bytes = "\x00" . $bytes;   // keep it positive
    return wa_der(0x02, $bytes);
}

function wa_pem(string $label, string $der): string {
    return "-----BEGIN {$label}-----\n"
        . chunk_split(base64_encode($der), 64, "\n")
        . "-----END {$label}-----\n";
}

/**
 * Convert a COSE_Key (CBOR map) into a PEM public key OpenSSL understands.
 *
 * ES256 is by far the most common — it is what Android, iOS and Windows Hello
 * hand back for a passkey — but RS256 is still emitted by some security keys,
 * so both are supported.
 */
function wa_cose_to_pem(array $cose): string {
    $kty = $cose[1] ?? null;

    if ($kty === 2) {                              // EC2
        if (($cose[-1] ?? null) !== 1) throw new RuntimeException('Unsupported elliptic curve (only P-256).');
        $x = (string)($cose[-2] ?? '');
        $y = (string)($cose[-3] ?? '');
        if (strlen($x) !== 32 || strlen($y) !== 32) throw new RuntimeException('Malformed EC public key.');
        // id-ecPublicKey + prime256v1, then the uncompressed point.
        $alg = hex2bin('301306072A8648CE3D020106082A8648CE3D030107');
        $der = wa_der(0x30, $alg . wa_der(0x03, "\x00\x04" . $x . $y));
        return wa_pem('PUBLIC KEY', $der);
    }

    if ($kty === 3) {                              // RSA
        $n = (string)($cose[-1] ?? '');
        $e = (string)($cose[-2] ?? '');
        if ($n === '' || $e === '') throw new RuntimeException('Malformed RSA public key.');
        $rsa = wa_der(0x30, wa_der_int($n) . wa_der_int($e));
        $alg = hex2bin('300D06092A864886F70D0101010500');   // rsaEncryption + NULL
        return wa_pem('PUBLIC KEY', wa_der(0x30, $alg . wa_der(0x03, "\x00" . $rsa)));
    }

    throw new RuntimeException('Unsupported COSE key type.');
}

/* ---------- authenticator data ---------- */

/**
 * Split authenticatorData into its parts.
 *
 * Layout: rpIdHash(32) | flags(1) | signCount(4) | [aaguid(16) | credIdLen(2)
 * | credId | COSE key]. The trailing block is present only when the AT flag is
 * set, which is the case for registration but not for a login assertion.
 */
function wa_parse_auth_data(string $ad): array {
    if (strlen($ad) < 37) throw new RuntimeException('authenticatorData is too short.');
    $flags = ord($ad[32]);
    $out = [
        'rpIdHash'  => substr($ad, 0, 32),
        'flags'     => $flags,
        'signCount' => unpack('N', substr($ad, 33, 4))[1],
        'up'        => (bool)($flags & 0x01),
        'uv'        => (bool)($flags & 0x04),
        'at'        => (bool)($flags & 0x40),
    ];

    if ($out['at']) {
        if (strlen($ad) < 55) throw new RuntimeException('authenticatorData: attested credential data is missing.');
        $out['aaguid'] = substr($ad, 37, 16);
        $len = unpack('n', substr($ad, 53, 2))[1];
        if (strlen($ad) < 55 + $len) throw new RuntimeException('authenticatorData: truncated credential id.');
        $out['credId'] = substr($ad, 55, $len);
        $off = 55 + $len;
        $coseStart = $off;
        $out['cose'] = wa_cbor($ad, $off);
        $out['coseRaw'] = substr($ad, $coseStart, $off - $coseStart);
    }
    return $out;
}

/* ---------- relying party identity ---------- */

/** The rpId is a bare hostname; an admin override wins over the request host. */
function wa_rp_id(): string {
    $set = trim(setting('webauthn_rp_id', ''));
    if ($set !== '') return $set;
    return (string)preg_replace('/:\d+$/', '', (string)($_SERVER['HTTP_HOST'] ?? ''));
}

/** The origin the browser will report, honouring the reverse proxy's header. */
function wa_origin(): string {
    $set = trim(setting('webauthn_origin', ''));
    if ($set !== '') return $set;

    $proto = (string)($_SERVER['HTTP_X_FORWARDED_PROTO'] ?? '');
    if ($proto !== '') {
        $scheme = strtolower(trim(explode(',', $proto)[0]));
    } else {
        $https = (string)($_SERVER['HTTPS'] ?? '');
        $scheme = ($https !== '' && strtolower($https) !== 'off') ? 'https' : 'http';
    }
    return $scheme . '://' . (string)($_SERVER['HTTP_HOST'] ?? '');
}

/* ---------- ceremony verification ---------- */

/** Validate clientDataJSON and return the decoded fields. */
function wa_check_client_data(string $json, string $type, string $challengeB64u, string $origin): array {
    $cd = json_decode(wa_b64u_dec($json), true);
    if (!is_array($cd)) throw new RuntimeException('clientDataJSON is not valid JSON.');
    if (($cd['type'] ?? '') !== $type) throw new RuntimeException('Unexpected ceremony type.');
    if (!hash_equals($challengeB64u, (string)($cd['challenge'] ?? ''))) {
        throw new RuntimeException('The challenge does not match. Please try again.');
    }
    if (!hash_equals($origin, (string)($cd['origin'] ?? ''))) {
        throw new RuntimeException('This page is not allowed to use your passkey.');
    }
    return $cd;
}

/**
 * Verify a registration response and return the credential to persist.
 *
 * @param array $transports transports the browser reported, if any
 */
function wa_verify_registration(
    string $clientDataJson,
    string $attestationObject,
    string $challengeB64u,
    string $origin,
    string $rpId,
    array $transports = []
): array {
    wa_check_client_data($clientDataJson, 'webauthn.create', $challengeB64u, $origin);

    $off = 0;
    $att = wa_cbor(wa_b64u_dec($attestationObject), $off);
    if (!is_array($att) || !isset($att['authData'])) throw new RuntimeException('Malformed attestation object.');

    $ad = wa_parse_auth_data((string)$att['authData']);
    if (!$ad['up']) throw new RuntimeException('The device did not confirm a user was present.');
    if (!$ad['at']) throw new RuntimeException('The device returned no credential.');
    if (!hash_equals(hash('sha256', $rpId, true), $ad['rpIdHash'])) {
        throw new RuntimeException('This passkey belongs to a different site.');
    }

    return [
        'credentialId' => wa_b64u_enc($ad['credId']),
        'publicKey'    => wa_cose_to_pem($ad['cose']),
        'signCount'    => (int)$ad['signCount'],
        'aaguid'       => bin2hex($ad['aaguid']),
        'transports'   => implode(',', array_slice(array_map('strval', $transports), 0, 8)),
        'fmt'          => (string)($att['fmt'] ?? 'none'),
    ];
}

/**
 * Verify a login assertion against a stored credential.
 *
 * @param string $pem            the credential's public key
 * @param int    $storedSignCount the counter from the last successful login
 * @return int the new signature counter to store
 */
function wa_verify_assertion(
    string $clientDataJson,
    string $authenticatorData,
    string $signature,
    string $pem,
    int $storedSignCount,
    string $challengeB64u,
    string $origin,
    string $rpId
): int {
    wa_check_client_data($clientDataJson, 'webauthn.get', $challengeB64u, $origin);

    $adRaw = wa_b64u_dec($authenticatorData);
    $ad = wa_parse_auth_data($adRaw);
    if (!$ad['up']) throw new RuntimeException('The device did not confirm a user was present.');
    if (!hash_equals(hash('sha256', $rpId, true), $ad['rpIdHash'])) {
        throw new RuntimeException('This passkey belongs to a different site.');
    }

    $signed = $adRaw . hash('sha256', wa_b64u_dec($clientDataJson), true);
    $ok = openssl_verify($signed, wa_b64u_dec($signature), $pem, OPENSSL_ALGO_SHA256);
    if ($ok !== 1) throw new RuntimeException('The passkey signature did not verify.');

    /* A counter that fails to advance can mean the credential was cloned, so
     * reject it. Some authenticators always report 0 (they keep no counter),
     * and those are exempt — otherwise every login after the first would fail. */
    $new = (int)$ad['signCount'];
    if ($new !== 0 || $storedSignCount !== 0) {
        if ($new <= $storedSignCount) {
            throw new RuntimeException('This passkey looks cloned. Sign in another way and re-register it.');
        }
    }
    return $new;
}
