//! WebAuthn verification.
//!
//! A passkey lets someone sign in with the phone or laptop's own unlock —
//! fingerprint, face, screen-lock PIN or pattern. A browser cannot read that
//! unlock: WebAuthn is the standard that delegates it to the operating system
//! and hands the site back only a signature. So the site verifies a signature
//! over a challenge it issued, and never sees the unlock itself.
//!
//! The pieces:
//!   * decode the CBOR the authenticator returns,
//!   * turn its COSE public key into something the crypto library accepts,
//!   * check the challenge, the origin and the relying-party id,
//!   * check the signature counter, which catches a cloned authenticator.

use base64::Engine;
use k256::ecdsa::{signature::Verifier, Signature, VerifyingKey};
use sha2::{Digest, Sha256};

pub fn b64u_enc(raw: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(raw)
}

pub fn b64u_dec(s: &str) -> Result<Vec<u8>, String> {
    let trimmed = s.trim();
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(trimmed)
        .or_else(|_| base64::engine::general_purpose::STANDARD.decode(trimmed))
        .map_err(|_| "Malformed base64url value.".to_string())
}

pub struct Registration {
    pub credential_id: String,
    pub public_key: String,
    pub sign_count: u32,
    pub aaguid: String,
    pub transports: String,
}

/// Minimal CBOR reader. WebAuthn uses a small, well-defined slice of CBOR, so a
/// full library would be more surface than the job needs.
pub fn cbor(input: &[u8], off: &mut usize) -> Result<CborValue, String> {
    if *off >= input.len() {
        return Err("CBOR: unexpected end of input.".into());
    }
    let first = input[*off];
    *off += 1;
    let major = first >> 5;
    let info = first & 0x1F;
    let arg = match info {
        0..=23 => info as u64,
        24 => {
            let v = *input.get(*off).ok_or("CBOR: truncated 1-byte length.")? as u64;
            *off += 1;
            v
        }
        25 => {
            let b = input
                .get(*off..*off + 2)
                .ok_or("CBOR: truncated 2-byte length.")?;
            *off += 2;
            u16::from_be_bytes([b[0], b[1]]) as u64
        }
        26 => {
            let b = input
                .get(*off..*off + 4)
                .ok_or("CBOR: truncated 4-byte length.")?;
            *off += 4;
            u32::from_be_bytes([b[0], b[1], b[2], b[3]]) as u64
        }
        27 => {
            let b = input
                .get(*off..*off + 8)
                .ok_or("CBOR: truncated 8-byte length.")?;
            *off += 8;
            u64::from_be_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]])
        }
        31 => u64::MAX, // indefinite length: not used by authenticators here
        _ => return Err("CBOR: reserved length.".into()),
    };
    match major {
        0 => Ok(CborValue::Uint(arg)),
        1 => Ok(CborValue::Int(-1 - arg as i64)),
        2 => {
            let end = *off + arg as usize;
            let v = input
                .get(*off..end)
                .ok_or("CBOR: truncated byte string.")?
                .to_vec();
            *off = end;
            Ok(CborValue::Bytes(v))
        }
        3 => {
            let end = *off + arg as usize;
            let v = input
                .get(*off..end)
                .ok_or("CBOR: truncated text string")?
                .to_vec();
            *off = end;
            Ok(CborValue::Text(String::from_utf8_lossy(&v).to_string()))
        }
        4 => {
            let mut items = Vec::new();
            for _ in 0..arg {
                items.push(cbor(input, off)?);
            }
            Ok(CborValue::Array(items))
        }
        5 => {
            let mut map = Vec::new();
            for _ in 0..arg {
                let k = cbor(input, off)?;
                let v = cbor(input, off)?;
                map.push((k, v));
            }
            Ok(CborValue::Map(map))
        }
        6 => cbor(input, off), // a tag: the value underneath is what matters
        7 => Ok(CborValue::Simple(arg)),
        _ => Err("CBOR: unsupported major type.".into()),
    }
}

#[derive(Debug, Clone)]
pub enum CborValue {
    Uint(u64),
    Int(i64),
    Bytes(Vec<u8>),
    Text(String),
    Array(Vec<CborValue>),
    Map(Vec<(CborValue, CborValue)>),
    Simple(u64),
}

impl CborValue {
    pub fn get(&self, key: i64) -> Option<&CborValue> {
        match self {
            CborValue::Map(m) => m.iter().find_map(|(k, v)| match k {
                CborValue::Int(i) if *i == key => Some(v),
                CborValue::Uint(u) if *u as i64 == key => Some(v),
                _ => None,
            }),
            _ => None,
        }
    }
    pub fn get_str(&self, key: &str) -> Option<&CborValue> {
        match self {
            CborValue::Map(m) => m.iter().find_map(|(k, v)| match k {
                CborValue::Text(t) if t == key => Some(v),
                _ => None,
            }),
            _ => None,
        }
    }
    pub fn as_bytes(&self) -> Vec<u8> {
        match self {
            CborValue::Bytes(b) => b.clone(),
            CborValue::Text(t) => t.as_bytes().to_vec(),
            _ => vec![],
        }
    }
    pub fn as_i64(&self) -> i64 {
        match self {
            CborValue::Int(i) => *i,
            CborValue::Uint(u) => *u as i64,
            _ => 0,
        }
    }
    pub fn as_str(&self) -> String {
        match self {
            CborValue::Text(t) => t.clone(),
            _ => String::new(),
        }
    }
}

/// The COSE key (EC2 / P-256) turned into the uncompressed point the verifier
/// wants: 0x04 || X || Y.
fn cose_to_point(cose: &CborValue) -> Result<Vec<u8>, String> {
    let kty = cose.get(1).map(|v| v.as_i64()).unwrap_or(0);
    if kty != 2 {
        return Err("This passkey does not use an EC2 key.".into());
    }
    let crv = cose.get(-1).map(|v| v.as_i64()).unwrap_or(0);
    if crv != 1 {
        return Err("This passkey does not use the P-256 curve.".into());
    }
    let x = cose
        .get(-2)
        .map(|v| v.as_bytes())
        .ok_or("COSE: no x coordinate.")?;
    let y = cose
        .get(-3)
        .map(|v| v.as_bytes())
        .ok_or("COSE: no y coordinate.")?;
    let mut out = vec![0x04];
    out.extend_from_slice(&x);
    out.extend_from_slice(&y);
    Ok(out)
}

/// Split the authenticatorData and read the parts that matter.
struct AuthData {
    rp_id_hash: Vec<u8>,
    flags: u8,
    sign_count: u32,
    cred_id: Vec<u8>,
    cose: Option<CborValue>,
}

fn parse_auth_data(ad: &[u8]) -> Result<AuthData, String> {
    if ad.len() < 37 {
        return Err("authenticatorData is too short.".into());
    }
    let flags = ad[32];
    let sign_count = u32::from_be_bytes([ad[33], ad[34], ad[35], ad[36]]);
    let mut out = AuthData {
        rp_id_hash: ad[0..32].to_vec(),
        flags,
        sign_count,
        cred_id: vec![],
        cose: None,
    };
    if flags & 0x40 != 0 {
        // Attested credential data: AAGUID, then a 2-byte length, then the id.
        if ad.len() < 55 {
            return Err("authenticatorData: attested credential data is missing.".into());
        }
        let len = u16::from_be_bytes([ad[53], ad[54]]) as usize;
        if ad.len() < 55 + len {
            return Err("authenticatorData: truncated credential id.".into());
        }
        out.cred_id = ad[55..55 + len].to_vec();
        let mut off = 55 + len;
        out.cose = Some(cbor(ad, &mut off)?);
    }
    Ok(out)
}

fn check_client_data(
    json: &str,
    want_type: &str,
    challenge: &str,
    origin: &str,
) -> Result<(), String> {
    let v: serde_json::Value =
        serde_json::from_str(json).map_err(|_| "Malformed clientDataJSON.".to_string())?;
    let t = v.get("type").and_then(|x| x.as_str()).unwrap_or("");
    if t != want_type {
        return Err(format!("Wrong clientData type: expected {want_type}."));
    }
    let ch = v.get("challenge").and_then(|x| x.as_str()).unwrap_or("");
    if ch != challenge {
        return Err("The challenge does not match. Please try again.".into());
    }
    let orig = v.get("origin").and_then(|x| x.as_str()).unwrap_or("");
    if orig != origin {
        return Err("This page is not allowed to use your passkey.".into());
    }
    Ok(())
}

/// Verify a registration attestation and extract the public key.
pub fn verify_registration(
    client_data_json: &str,
    attestation_object: &str,
    challenge: &str,
    origin: &str,
    rp_id: &str,
    transports: &[String],
) -> Result<Registration, String> {
    let cd = String::from_utf8(b64u_dec(client_data_json)?)
        .map_err(|_| "clientDataJSON is not UTF-8.".to_string())?;
    check_client_data(&cd, "webauthn.create", challenge, origin)?;

    let raw = b64u_dec(attestation_object)?;
    let mut off = 0;
    let att = cbor(&raw, &mut off)?;
    let ad_bytes = att
        .get_str("authData")
        .map(|v| v.as_bytes())
        .ok_or("Malformed attestation object.")?;
    let ad = parse_auth_data(&ad_bytes)?;
    if ad.flags & 0x01 == 0 {
        return Err("The device did not confirm a user was present.".into());
    }
    if ad.flags & 0x40 == 0 {
        return Err("The device returned no credential.".into());
    }
    let expected = Sha256::digest(rp_id.as_bytes());
    if ad.rp_id_hash.as_slice() != expected.as_slice() {
        return Err("This passkey belongs to a different site.".into());
    }
    let cose = ad.cose.ok_or("The device returned no public key.")?;
    let point = cose_to_point(&cose)?;
    let key = VerifyingKey::from_sec1_bytes(&point)
        .map_err(|_| "The passkey public key is not valid.".to_string())?;
    let der = key.to_encoded_point(false).as_bytes().to_vec();
    Ok(Registration {
        credential_id: b64u_enc(&ad.cred_id),
        public_key: hex::encode(der),
        sign_count: ad.sign_count,
        aaguid: hex::encode(&ad_bytes[37..53.min(ad_bytes.len())]),
        transports: transports.join(","),
    })
}

/// Verify a sign-in assertion. Returns the new signature counter.
#[allow(clippy::too_many_arguments)]
pub fn verify_assertion(
    client_data_json: &str,
    authenticator_data: &str,
    signature: &str,
    public_key_hex: &str,
    stored_sign_count: i64,
    challenge: &str,
    origin: &str,
    rp_id: &str,
) -> Result<u32, String> {
    let cd = String::from_utf8(b64u_dec(client_data_json)?)
        .map_err(|_| "clientDataJSON is not UTF-8.".to_string())?;
    check_client_data(&cd, "webauthn.get", challenge, origin)?;

    let ad = b64u_dec(authenticator_data)?;
    let parsed = parse_auth_data(&ad)?;
    if parsed.flags & 0x01 == 0 {
        return Err("The device did not confirm a user was present.".into());
    }
    let expected = Sha256::digest(rp_id.as_bytes());
    if parsed.rp_id_hash.as_slice() != expected.as_slice() {
        return Err("This passkey belongs to a different site.".into());
    }

    // The signature is over authenticatorData || SHA256(clientDataJSON).
    let cd_hash = Sha256::digest(cd.as_bytes());
    let mut signed = ad.clone();
    signed.extend_from_slice(&cd_hash);

    let point =
        hex::decode(public_key_hex).map_err(|_| "Stored passkey key is unreadable.".to_string())?;
    let key = VerifyingKey::from_sec1_bytes(&point)
        .map_err(|_| "Stored passkey key is not valid.".to_string())?;
    let sig_raw = b64u_dec(signature)?;
    let sig = Signature::from_slice(&sig_raw)
        .map_err(|_| "The passkey signature is malformed.".to_string())?;
    key.verify(&signed, &sig)
        .map_err(|_| "The passkey signature did not verify.".to_string())?;

    // A counter that fails to advance can mean the credential was cloned.
    // Authenticators that keep no counter always report 0, and are exempt.
    let new = parsed.sign_count;
    if new != 0 || stored_sign_count != 0 {
        if (new as i64) <= stored_sign_count {
            return Err(
                "This passkey looks cloned. Sign in another way and re-register it.".into(),
            );
        }
    }
    Ok(new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn b64url_round_trip() {
        for raw in [vec![], vec![1u8], vec![255u8; 200]] {
            assert_eq!(b64u_dec(&b64u_enc(&raw)).unwrap(), raw);
        }
    }

    #[test]
    fn cbor_reads_the_shapes_authenticators_send() {
        let mut off = 0;
        assert_eq!(cbor(&[0x01], &mut off).unwrap().as_i64(), 1);
        let mut off = 0;
        assert_eq!(cbor(&[0x26], &mut off).unwrap().as_i64(), -7);
        let mut off = 0;
        assert_eq!(cbor(&[0x18, 0x64], &mut off).unwrap().as_i64(), 100);
        let mut off = 0;
        assert_eq!(cbor(&[0x19, 0x01, 0x00], &mut off).unwrap().as_i64(), 256);
        let mut off = 0;
        assert_eq!(
            cbor(&[0x63, b'a', b'b', b'c'], &mut off).unwrap().as_str(),
            "abc"
        );
        let mut off = 0;
        assert_eq!(
            cbor(&[0x43, 1, 2, 3], &mut off).unwrap().as_bytes(),
            vec![1, 2, 3]
        );
        let mut off = 0;
        let arr = cbor(&[0x82, 0x01, 0x02], &mut off).unwrap();
        match arr {
            CborValue::Array(a) => assert_eq!(a.len(), 2),
            _ => panic!("expected an array"),
        }
        let mut off = 0;
        let map = cbor(&[0xa1, 0x01, 0x02], &mut off).unwrap();
        assert_eq!(map.get(1).unwrap().as_i64(), 2);
    }

    #[test]
    fn client_data_must_match_the_challenge_and_origin() {
        let cd = r#"{"type":"webauthn.get","challenge":"abc","origin":"http://localhost"}"#;
        assert!(check_client_data(cd, "webauthn.get", "abc", "http://localhost").is_ok());
        assert!(check_client_data(cd, "webauthn.get", "xyz", "http://localhost").is_err());
        assert!(check_client_data(cd, "webauthn.get", "abc", "https://evil.test").is_err());
        assert!(check_client_data(cd, "webauthn.create", "abc", "http://localhost").is_err());
    }

    #[test]
    fn a_made_up_signature_is_refused() {
        let key = k256::ecdsa::SigningKey::random(&mut rand::thread_rng());
        let vk = VerifyingKey::from(&key);
        let point = hex::encode(vk.to_encoded_point(false).as_bytes());
        let rp = "localhost";
        let challenge = b64u_enc(&[7u8; 32]);
        let cd = format!(
            r#"{{"type":"webauthn.get","challenge":"{challenge}","origin":"http://localhost"}}"#
        );
        let mut ad = Sha256::digest(rp.as_bytes()).to_vec();
        ad.push(0x01);
        ad.extend_from_slice(&5u32.to_be_bytes());
        // A signature over different bytes must not verify.
        let bogus = Signature::from_slice(&[0u8; 64]);
        if let Ok(sig) = bogus {
            let signed = [ad.clone(), Sha256::digest(cd.as_bytes()).to_vec()].concat();
            assert!(vk.verify(&signed, &sig).is_err());
        }
        // And a real signature must.
        use k256::ecdsa::signature::Signer;
        let signed = [ad.clone(), Sha256::digest(cd.as_bytes()).to_vec()].concat();
        let real: Signature = key.sign(&signed);
        assert!(vk.verify(&signed, &real).is_ok());
        let _ = point;
    }
}
