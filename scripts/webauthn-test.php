<?php
/**
 * Unit tests for lib/webauthn.php — no browser, no database needed.
 *
 *     php scripts/webauthn-test.php
 *
 * The library is hand-written (no Composer on this host), so these tests build
 * the CBOR / COSE structures by hand and check every branch that decides whether
 * a passkey is accepted: the encoding round-trips, the challenge and origin must
 * match, the relying party id must match, and a frozen signature counter is
 * refused because that can mean the credential was cloned.
 */
declare(strict_types=1);
require __DIR__ . '/../lib/webauthn.php';

$pass = 0;
$fail = 0;

function ok(bool $cond, string $msg): void {
    global $pass, $fail;
    if ($cond) { $pass++; echo "  OK   $msg\n"; }
    else { $fail++; echo "  FAIL $msg\n"; }
}

function throws(callable $fn, string $msg): void {
    try { $fn(); ok(false, $msg . ' (hakuna kosa lilitolewa)'); }
    catch (\Throwable $e) { ok(true, $msg . ' -> ' . $e->getMessage()); }
}

/**
 * A minimal CBOR encoder, used only to build the fixtures below. The library
 * itself only ever needs to decode, so it ships no encoder.
 */
function cbor_enc(mixed $v): string {
    if (is_int($v)) return $v >= 0 ? cbor_head(0, $v) : cbor_head(1, -1 - $v);
    if (is_string($v)) return cbor_head(3, strlen($v)) . $v;
    if (is_array($v)) {
        $isMap = array_keys($v) !== range(0, count($v) - 1) || (count($v) > 0 && !array_is_list($v));
        $out = cbor_head($isMap ? 5 : 4, count($v));
        foreach ($v as $k => $item) $out .= $isMap ? cbor_enc($k) . cbor_enc($item) : cbor_enc($item);
        return $out;
    }
    throw new RuntimeException('cbor_enc: unsupported type');
}

/** Decode a complete CBOR buffer (the library's decoder is offset-based). */
function cbor_dec(string $buf): mixed {
    $off = 0;
    return wa_cbor($buf, $off);
}

function cbor_head(int $major, int $n): string {
    $m = $major << 5;
    if ($n < 24) return chr($m | $n);
    if ($n < 0x100) return chr($m | 24) . chr($n);
    if ($n < 0x10000) return chr($m | 25) . pack('n', $n);
    return chr($m | 26) . pack('N', $n);
}

echo "=== 1. CBOR: namba, string, array, map ===\n";
ok(cbor_dec(hex2bin('01')) === 1, 'int 1');
ok(cbor_dec(hex2bin('26')) === -7, 'int -7');
ok(cbor_dec(hex2bin('1864')) === 100, 'int 100 (1-byte)');
ok(cbor_dec(hex2bin('190100')) === 256, 'int 256 (2-byte)');
ok(cbor_dec(hex2bin('63616263')) === 'abc', "text 'abc'");
ok(cbor_dec(hex2bin('43010203')) === "\x01\x02\x03", 'bytes');
ok(cbor_dec(hex2bin('820102')) === [1, 2], 'array [1,2]');
ok(cbor_dec(hex2bin('a10102')) === [1 => 2], 'map {1:2}');
ok(cbor_dec(hex2bin('c001')) === 1, 'tag unwrapped');

echo "\n=== 2. COSE -> PEM (ES256) ===\n";
$res = openssl_pkey_new(['curve_name' => 'prime256v1', 'private_key_type' => OPENSSL_KEYTYPE_EC]);
$det = openssl_pkey_get_details($res);
$cose = [1 => 2, 3 => -7, -1 => 1, -2 => $det['ec']['x'], -3 => $det['ec']['y']];
ok($cose[1] === 2, 'kty=EC2');
ok($cose[3] === -7, 'crv=P-256');
$pem = wa_cose_to_pem($cose);
ok(str_starts_with($pem, '-----BEGIN PUBLIC KEY-----'), 'PEM imezalishwa');
ok(trim($pem) === trim($det['key']), 'PEM inafanana na ufunguo halisi');

echo "\n=== 3. Uthibitishaji wa sahihi (ES256) ===\n";
$data = 'hello-web';
openssl_sign($data, $sig, $res, OPENSSL_ALGO_SHA256);
ok(openssl_verify($data, $sig, $pem, OPENSSL_ALGO_SHA256) === 1, 'sahihi sahihi inakubaliwa');
$badSig = $sig;
$badSig[0] = chr(ord($badSig[0]) ^ 0xFF);
ok(openssl_verify($data, $badSig, $pem, OPENSSL_ALGO_SHA256) !== 1, 'sahihi mbovu inakataliwa');

echo "\n=== 4. wa_verify_assertion (mtiririko kamili) ===\n";
$rpId = 'localhost';
$origin = 'http://localhost:12000';
$challenge = wa_b64u_enc(random_bytes(32));
$ad = hash('sha256', $rpId, true) . chr(0x01) . pack('N', 5);   // rpIdHash | flags(UP) | counter=5
$cdArr = ['type' => 'webauthn.get', 'challenge' => $challenge, 'origin' => $origin];
$cd = json_encode($cdArr);
$sigData = $ad . hash('sha256', $cd, true);
openssl_sign($sigData, $goodSig, $res, OPENSSL_ALGO_SHA256);

$newCount = wa_verify_assertion(
    wa_b64u_enc($cd), wa_b64u_enc($ad), wa_b64u_enc($goodSig), $pem, 4, $challenge, $origin, $rpId
);
ok($newCount === 5, "assertion imefanikiwa, counter mpya = $newCount");

echo "\n=== 5. Kesi za kukataliwa ===\n";
throws(fn() => wa_verify_assertion(wa_b64u_enc(json_encode(['type' => 'webauthn.get', 'challenge' => $challenge, 'origin' => 'https://evil.test'])), wa_b64u_enc($ad), wa_b64u_enc($goodSig), $pem, 4, $challenge, $origin, $rpId),
    'origin mbaya inakataliwa');
throws(fn() => wa_verify_assertion(wa_b64u_enc(json_encode(['type' => 'webauthn.get', 'challenge' => wa_b64u_enc(random_bytes(32)), 'origin' => $origin])), wa_b64u_enc($ad), wa_b64u_enc($goodSig), $pem, 4, $challenge, $origin, $rpId),
    'challenge mbaya inakataliwa');
throws(fn() => wa_verify_assertion(wa_b64u_enc($cd), wa_b64u_enc($ad), wa_b64u_enc($goodSig), $pem, 4, $challenge, $origin, 'other.test'),
    'rpId mbaya inakataliwa');
throws(fn() => wa_verify_assertion(wa_b64u_enc($cd), wa_b64u_enc($ad), wa_b64u_enc($goodSig), $pem, 9, $challenge, $origin, $rpId),
    'counter imerudi nyuma inakataliwa');
throws(fn() => wa_verify_assertion(wa_b64u_enc($cd), wa_b64u_enc($ad), wa_b64u_enc(str_repeat("\x00", 64)), $pem, 4, $challenge, $origin, $rpId),
    'sahihi imeharibika inakataliwa');
throws(fn() => wa_verify_assertion('not-base64-json!!', wa_b64u_enc($ad), wa_b64u_enc($goodSig), $pem, 4, $challenge, $origin, $rpId),
    'clientDataJSON isiyo sahihi inakataliwa');

echo "\n=== 6. wa_verify_registration (usajili halisi) ===\n";
$regRes = openssl_pkey_new(['curve_name' => 'prime256v1', 'private_key_type' => OPENSSL_KEYTYPE_EC]);
$regDet = openssl_pkey_get_details($regRes);
$regCose = cbor_enc([1 => 2, 3 => -7, -1 => 1, -2 => $regDet['ec']['x'], -3 => $regDet['ec']['y']]);
$credId = random_bytes(32);
$authData = hash('sha256', $rpId, true) . chr(0x41) . pack('N', 0)
          . str_repeat("\x00", 16)                       // AAGUID
          . pack('n', strlen($credId)) . $credId
          . $regCose;
$attObj = cbor_enc(['fmt' => 'none', 'attStmt' => [], 'authData' => $authData]);
$regChallenge = wa_b64u_enc(random_bytes(32));
$regCd = json_encode(['type' => 'webauthn.create', 'challenge' => $regChallenge, 'origin' => $origin]);

$info = wa_verify_registration(wa_b64u_enc($regCd), wa_b64u_enc($attObj), $regChallenge, $origin, $rpId, ['internal']);
ok($info['credentialId'] === wa_b64u_enc($credId), 'credential id imesomwa');
ok(trim($info['publicKey']) === trim($regDet['key']), 'public key imehifadhiwa kama PEM');
ok($info['signCount'] === 0, 'signCount ya awali = 0');
ok($info['transports'] === 'internal', 'transports zimehifadhiwa');
throws(fn() => wa_verify_registration(wa_b64u_enc($regCd), wa_b64u_enc($attObj), wa_b64u_enc(random_bytes(32)), $origin, $rpId, []),
    'challenge ya usajili mbaya inakataliwa');

echo "\n=== 7. b64url round-trip ===\n";
foreach ([$credId, random_bytes(1), random_bytes(200), ''] as $i => $raw) {
    ok(wa_b64u_dec(wa_b64u_enc($raw)) === $raw, "bytes #$i zinazunguka sawa");
}

echo "\nJUMLA: $pass zimefaulu, $fail zimeshindwa\n";
exit($fail === 0 ? 0 : 1);
