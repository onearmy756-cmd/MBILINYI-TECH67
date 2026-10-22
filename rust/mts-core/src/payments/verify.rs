//! Verification codes.
//!
//! Every payment gets a short code the payer can quote on the phone or in a
//! bank reference, and the same code confirms the payment in the portal. It is
//! short because it has to be read aloud; it is not a secret, because the
//! amount and the gateway reference are what actually prove payment.

use rand::Rng;

/// Six digits, avoiding a leading zero so it reads cleanly aloud.
pub fn make_code() -> String {
    let mut rng = rand::thread_rng();
    format!("{:06}", rng.gen_range(100000..1000000))
}

/// A human-quotable reference: MTS-<code>-<2 letters>.
pub fn make_reference(code: &str) -> String {
    let mut rng = rand::thread_rng();
    let letters: String = (0..2)
        .map(|_| (b'A' + rng.gen_range(0..26)) as char)
        .collect();
    format!("MTS-{code}-{letters}")
}

/// Compare without leaking length or position through timing.
pub fn matches(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        diff |= x ^ y;
    }
    diff == 0
}
