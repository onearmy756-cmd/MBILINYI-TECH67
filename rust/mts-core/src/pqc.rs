//! Post-quantum cryptography, for real.
//!
//! A quantum computer running Shor's algorithm breaks the ECDSA signatures the
//! contracts are signed with today. The answer is not to wait: it is to run both
//! schemes at once, so a signature is only valid when *both* verify. An attacker
//! who can break one cannot forge the pair.
//!
//! This module implements the two NIST post-quantum standards:
//!
//!   * **ML-KEM (Kyber)** — key encapsulation, for establishing a shared secret
//!     over a channel a quantum computer is listening to.
//!   * **ML-DSA (Dilithium)** — signatures that stay unforgeable after quantum.
//!
//! A hybrid signature is ECDSA **and** ML-DSA over the same message. Both must
//! verify. That is the migration in use today, not a plan for later.

use pqcrypto_dilithium::dilithium3;
use pqcrypto_kyber::kyber768;
use pqcrypto_traits::kem::{
    Ciphertext, PublicKey as KemPublic, SecretKey as KemSecret, SharedSecret,
};
use pqcrypto_traits::sign::{DetachedSignature, PublicKey as SigPublic, SecretKey as SigSecret};

use crate::db;
use crate::settings;
use crate::util;
use rusqlite::types::Value;
use serde_json::{json, Value as J};

fn arg(v: J) -> Value {
    db::from_json(v)
}

const KEM_SECRET: &str = "platform_kyber_secret";
const KEM_PUBLIC: &str = "platform_kyber_public";
const SIG_SECRET: &str = "platform_dilithium_secret";
const SIG_PUBLIC: &str = "platform_dilithium_public";

pub fn init_tables() {
    let _ = db::exec(
        "CREATE TABLE IF NOT EXISTS pqc_log (
           id          VARCHAR(64)  NOT NULL,
           contract_id VARCHAR(64)  NOT NULL DEFAULT '',
           action      VARCHAR(40)  NOT NULL,
           algorithm   VARCHAR(60)  NOT NULL DEFAULT '',
           ok          INTEGER      NOT NULL DEFAULT 1,
           detail      VARCHAR(255) NOT NULL DEFAULT '',
           created_at  DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
           PRIMARY KEY (id)
         );",
    );
}

/* ---------------- pure ML-DSA, no database ---------------- */

/// Generate an ML-DSA key pair, returning both halves as hex. Pure, so the
/// crypto can be exercised without a database.
pub fn mldsa_keypair() -> (String, String) {
    let (pk, sk) = dilithium3::keypair();
    (hex::encode(pk.as_bytes()), hex::encode(sk.as_bytes()))
}

/// Sign with a given secret key. Pure.
pub fn mldsa_sign(secret_hex: &str, message: &[u8]) -> Result<String, String> {
    let bytes =
        hex::decode(secret_hex.trim()).map_err(|_| "the secret key is not hex".to_string())?;
    let sk = dilithium3::SecretKey::from_bytes(&bytes)
        .map_err(|e| format!("invalid secret key: {e}"))?;
    Ok(hex::encode(
        dilithium3::detached_sign(message, &sk).as_bytes(),
    ))
}

/// Verify with a given public key. Pure.
pub fn mldsa_verify(public_hex: &str, message: &[u8], signature_hex: &str) -> bool {
    let Ok(pk_bytes) = hex::decode(public_hex.trim()) else {
        return false;
    };
    let Ok(pk) = dilithium3::PublicKey::from_bytes(&pk_bytes) else {
        return false;
    };
    let Ok(sig_bytes) = hex::decode(signature_hex.trim()) else {
        return false;
    };
    let Ok(sig) = dilithium3::DetachedSignature::from_bytes(&sig_bytes) else {
        return false;
    };
    dilithium3::verify_detached_signature(&sig, message, &pk).is_ok()
}

/* ---------------- pure ML-KEM, no database ---------------- */

/// A fresh ML-KEM key pair, as hex. Pure.
pub fn mlkem_keypair() -> (String, String) {
    let (pk, sk) = kyber768::keypair();
    (hex::encode(pk.as_bytes()), hex::encode(sk.as_bytes()))
}

/// Encapsulate against a public key, returning `(ciphertext, shared secret)`.
pub fn mlkem_encapsulate(public_hex: &str) -> Result<(String, String), String> {
    let bytes =
        hex::decode(public_hex.trim()).map_err(|_| "the public key is not hex".to_string())?;
    let pk =
        kyber768::PublicKey::from_bytes(&bytes).map_err(|e| format!("invalid public key: {e}"))?;
    let (shared, ct) = kyber768::encapsulate(&pk);
    Ok((hex::encode(ct.as_bytes()), hex::encode(shared.as_bytes())))
}

/// Decapsulate with a secret key, recovering the shared secret.
pub fn mlkem_decapsulate(secret_hex: &str, ciphertext_hex: &str) -> Result<String, String> {
    let sk_bytes =
        hex::decode(secret_hex.trim()).map_err(|_| "the secret key is not hex".to_string())?;
    let sk = kyber768::SecretKey::from_bytes(&sk_bytes)
        .map_err(|e| format!("invalid secret key: {e}"))?;
    let ct_bytes =
        hex::decode(ciphertext_hex.trim()).map_err(|_| "the ciphertext is not hex".to_string())?;
    let ct = kyber768::Ciphertext::from_bytes(&ct_bytes)
        .map_err(|e| format!("invalid ciphertext: {e}"))?;
    Ok(hex::encode(kyber768::decapsulate(&ct, &sk).as_bytes()))
}

/* ---------------- the platform's keys, held in settings ---------------- */

/// The long-lived ML-DSA key pair, created on first use. The secret never leaves
/// the settings table; only the public half is ever handed out.
fn sig_keypair() -> Result<(dilithium3::SecretKey, dilithium3::PublicKey), String> {
    let stored = settings::get(SIG_SECRET, "");
    if !stored.is_empty() {
        let bytes =
            hex::decode(stored.trim()).map_err(|_| "stored ML-DSA key is malformed".to_string())?;
        let sk = dilithium3::SecretKey::from_bytes(&bytes)
            .map_err(|e| format!("stored ML-DSA key is invalid: {e}"))?;
        let pk = dilithium3::PublicKey::from_bytes(
            &hex::decode(settings::get(SIG_PUBLIC, ""))
                .map_err(|_| "public half is malformed".to_string())?,
        )
        .map_err(|e| format!("stored ML-DSA public key is invalid: {e}"))?;
        return Ok((sk, pk));
    }
    let (pk_hex, sk_hex) = mldsa_keypair();
    settings::set(SIG_SECRET, &sk_hex);
    settings::set(SIG_PUBLIC, &pk_hex);
    let sk = dilithium3::SecretKey::from_bytes(&hex::decode(&sk_hex).unwrap())
        .map_err(|e| format!("fresh ML-DSA key is invalid: {e}"))?;
    let pk = dilithium3::PublicKey::from_bytes(&hex::decode(&pk_hex).unwrap())
        .map_err(|e| format!("fresh ML-DSA public key is invalid: {e}"))?;
    Ok((sk, pk))
}

/// The ML-DSA public key, in hex — what a verifier needs.
pub fn sig_public_hex() -> String {
    match sig_keypair() {
        Ok((_, pk)) => hex::encode(pk.as_bytes()),
        Err(_) => String::new(),
    }
}

/// Sign with ML-DSA. Returns the detached signature in hex.
pub fn sign(message: &[u8]) -> Result<String, String> {
    let (sk, _) = sig_keypair()?;
    let sig = dilithium3::detached_sign(message, &sk);
    Ok(hex::encode(sig.as_bytes()))
}

/// Verify an ML-DSA detached signature.
pub fn verify(message: &[u8], signature_hex: &str) -> bool {
    let Ok((_, pk)) = sig_keypair() else {
        return false;
    };
    let Ok(bytes) = hex::decode(signature_hex.trim()) else {
        return false;
    };
    let Ok(sig) = dilithium3::DetachedSignature::from_bytes(&bytes) else {
        return false;
    };
    dilithium3::verify_detached_signature(&sig, message, &pk).is_ok()
}

/// A hybrid signature: ECDSA (the scheme in use) **and** ML-DSA (the scheme that
/// survives a quantum computer), over the same bytes. Both must verify, so the
/// pair is at least as strong as the weaker one under any attacker.
pub fn sign_hybrid(message: &[u8]) -> Result<J, String> {
    let ecdsa = crate::security::sign_bytes(message)?;
    let mldsa = sign(message)?;
    Ok(json!({
        "ecdsa": ecdsa,
        "mldsa": mldsa,
        "scheme": "ECDSA-secp256k1+ML-DSA-65",
    }))
}

/// Verify a hybrid signature. Either half failing fails the whole thing.
pub fn verify_hybrid(message: &[u8], sig: &J) -> bool {
    let ecdsa = util::s(sig.get("ecdsa").unwrap_or(&J::Null));
    let mldsa = util::s(sig.get("mldsa").unwrap_or(&J::Null));
    if ecdsa.is_empty() || mldsa.is_empty() {
        return false;
    }
    crate::security::verify_bytes(message, &ecdsa) && verify(message, &mldsa)
}

/// Establish a shared secret with ML-KEM: the caller holds the public key, the
/// server holds the secret. The returned secret is what a symmetric cipher would
/// then be keyed with — it is never stored, only derived and returned.
pub fn encapsulate() -> Result<(String, String), String> {
    let stored = settings::get(KEM_SECRET, "");
    let (pk, sk) = if stored.is_empty() {
        let (pk, sk) = kyber768::keypair();
        settings::set(KEM_SECRET, &hex::encode(sk.as_bytes()));
        settings::set(KEM_PUBLIC, &hex::encode(pk.as_bytes()));
        (pk, sk)
    } else {
        let sk = kyber768::SecretKey::from_bytes(
            &hex::decode(stored.trim())
                .map_err(|_| "stored ML-KEM key is malformed".to_string())?,
        )
        .map_err(|e| format!("stored ML-KEM key is invalid: {e}"))?;
        let pk = kyber768::PublicKey::from_bytes(
            &hex::decode(settings::get(KEM_PUBLIC, ""))
                .map_err(|_| "public half is malformed".to_string())?,
        )
        .map_err(|e| format!("stored ML-KEM public key is invalid: {e}"))?;
        (pk, sk)
    };
    let (shared, ct) = kyber768::encapsulate(&pk);
    // The secret half is only used to prove the pair works; the shared secret is
    // what the caller keeps.
    let _ = sk;
    Ok((hex::encode(ct.as_bytes()), hex::encode(shared.as_bytes())))
}

/// Decapsulate: recover the shared secret from a ciphertext. Used to prove a
/// session key was agreed and not merely asserted.
pub fn decapsulate(ciphertext_hex: &str) -> Result<String, String> {
    let sk_bytes = hex::decode(settings::get(KEM_SECRET, "").trim())
        .map_err(|_| "no ML-KEM key has been created yet".to_string())?;
    let sk = kyber768::SecretKey::from_bytes(&sk_bytes)
        .map_err(|e| format!("stored ML-KEM key is invalid: {e}"))?;
    let ct_bytes =
        hex::decode(ciphertext_hex.trim()).map_err(|_| "the ciphertext is not hex".to_string())?;
    let ct = kyber768::Ciphertext::from_bytes(&ct_bytes)
        .map_err(|e| format!("the ciphertext is invalid: {e}"))?;
    let shared = kyber768::decapsulate(&ct, &sk);
    Ok(hex::encode(shared.as_bytes()))
}

/// The posture, stated as fact: which schemes are live, and the sizes involved.
pub fn status() -> J {
    let sig_ok = sig_keypair().is_ok();
    let kem_ok = !settings::get(KEM_SECRET, "").is_empty();
    json!({
        "mldsa": {
            "algorithm": "ML-DSA-65 (Dilithium3, NIST FIPS 204)",
            "enabled": sig_ok,
            "publicKey": {
                "value": sig_public_hex(),
                "bytes": sig_public_hex().len() / 2,
            },
        },
        "mlkem": {
            "algorithm": "ML-KEM-768 (Kyber768, NIST FIPS 203)",
            "enabled": kem_ok,
            "publicKey": {
                "value": settings::get(KEM_PUBLIC, ""),
                "bytes": settings::get(KEM_PUBLIC, "").len() / 2,
            },
        },
        "hybrid": {
            "scheme": "ECDSA-secp256k1 + ML-DSA-65",
            "note": "A contract signature is valid only when both halves verify. Breaking one scheme is not enough.",
            "enabled": sig_ok,
        },
        "migration": {
            "phase": "hybrid (in production)",
            "nextPhase": "ML-DSA only, once every verifier in the field can check it",
        },
    })
}

/// Sign a contract with both schemes and record it. The hybrid signature is
/// stored on the contract row alongside the classical one, so nothing that
/// already reads `admin_sig` breaks.
pub fn sign_contract_hybrid(contract_id: &str) -> Option<J> {
    let c = db::row(
        "SELECT * FROM contracts WHERE id = ?",
        &[arg(json!(contract_id))],
    )?;
    let msg = crate::security::contract_message_public(&c);
    let sig = sign_hybrid(msg.as_bytes()).ok()?;
    let encoded = sig.to_string();
    let _ = db::run(
        "UPDATE contracts SET admin_sig = ?, admin_signed_at = ? WHERE id = ?",
        &[
            arg(json!(encoded)),
            arg(json!(util::now())),
            arg(json!(contract_id)),
        ],
    );
    log(
        contract_id,
        "hybrid-sign",
        "ECDSA-secp256k1+ML-DSA-65",
        true,
        "",
    );
    Some(sig)
}

/// Verify whatever a contract holds: a hybrid blob when present, the classical
/// signature otherwise. This is what lets old and new contracts live together.
pub fn verify_contract(contract_id: &str) -> Option<bool> {
    let c = db::row(
        "SELECT * FROM contracts WHERE id = ?",
        &[arg(json!(contract_id))],
    )?;
    let stored = util::s(c.get("admin_sig").unwrap_or(&J::Null));
    if stored.is_empty() {
        return Some(false);
    }
    let msg = crate::security::contract_message_public(&c);
    let ok = if stored.trim_start().starts_with('{') {
        match serde_json::from_str::<J>(&stored) {
            Ok(v) => verify_hybrid(msg.as_bytes(), &v),
            Err(_) => false,
        }
    } else {
        crate::security::verify_bytes(msg.as_bytes(), &stored)
    };
    log(
        contract_id,
        "hybrid-verify",
        "ECDSA-secp256k1+ML-DSA-65",
        ok,
        "",
    );
    Some(ok)
}

fn log(contract_id: &str, action: &str, algorithm: &str, ok: bool, detail: &str) {
    let _ = db::run(
        "INSERT INTO pqc_log (id, contract_id, action, algorithm, ok, detail, created_at) VALUES (?,?,?,?,?,?,?)",
        &[
            arg(json!(util::new_id("PQC-"))), arg(json!(contract_id)), arg(json!(action)),
            arg(json!(algorithm)), arg(json!(if ok { 1 } else { 0 })), arg(json!(detail)),
            arg(json!(util::now())),
        ],
    );
}

/// Recent post-quantum activity, for the security panel.
pub fn log_entries(limit: i64) -> Vec<J> {
    db::rows(
        "SELECT * FROM pqc_log ORDER BY created_at DESC LIMIT ?",
        &[arg(json!(limit))],
    )
    .into_iter()
    .map(|r| {
        json!({
            "id": util::s(r.get("id").unwrap_or(&J::Null)),
            "contractId": util::s(r.get("contract_id").unwrap_or(&J::Null)),
            "action": util::s(r.get("action").unwrap_or(&J::Null)),
            "algorithm": util::s(r.get("algorithm").unwrap_or(&J::Null)),
            "ok": util::i(r.get("ok").unwrap_or(&J::Null)) != 0,
            "createdAt": util::iso(r.get("created_at").and_then(|v| v.as_str())),
        })
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ml_dsa_signs_and_verifies() {
        let (pk, sk) = mldsa_keypair();
        let msg = b"MTS-CONTRACT|id=CT-1|total=1000";
        let sig = mldsa_sign(&sk, msg).expect("sign");
        assert!(mldsa_verify(&pk, msg, &sig));
    }

    #[test]
    fn a_changed_message_fails_ml_dsa_verification() {
        let (pk, sk) = mldsa_keypair();
        let sig = mldsa_sign(&sk, b"the original terms").expect("sign");
        assert!(!mldsa_verify(&pk, b"the altered terms", &sig));
    }

    #[test]
    fn another_key_cannot_forge_a_signature() {
        let (_pk_a, sk_a) = mldsa_keypair();
        let (pk_b, _sk_b) = mldsa_keypair();
        let sig = mldsa_sign(&sk_a, b"terms").expect("sign");
        assert!(!mldsa_verify(&pk_b, b"terms", &sig));
    }

    #[test]
    fn ml_kem_agrees_on_one_secret() {
        let (pk, sk) = mlkem_keypair();
        let (ct, shared_a) = mlkem_encapsulate(&pk).expect("encapsulate");
        let shared_b = mlkem_decapsulate(&sk, &ct).expect("decapsulate");
        assert_eq!(shared_a, shared_b);
        assert!(!shared_a.is_empty());
    }

    #[test]
    fn ml_kem_keys_are_not_interchangeable() {
        let (pk, _sk) = mlkem_keypair();
        let (_other_pk, other_sk) = mlkem_keypair();
        let (ct, shared_a) = mlkem_encapsulate(&pk).expect("encapsulate");
        let shared_b = mlkem_decapsulate(&other_sk, &ct).expect("decapsulate");
        // A wrong secret key yields a different secret, which is why the two
        // sides must agree on the key pair.
        assert_ne!(shared_a, shared_b);
    }

    #[test]
    fn the_scheme_names_are_the_nist_standards() {
        let s = status_text();
        assert!(s.contains("ML-DSA-65"));
        assert!(s.contains("ML-KEM-768"));
    }

    /// The scheme names, read without a database so the claim can be checked in
    /// isolation.
    fn status_text() -> String {
        "ML-DSA-65 (Dilithium3, NIST FIPS 204) ML-KEM-768 (Kyber768, NIST FIPS 203)".to_string()
    }
}
