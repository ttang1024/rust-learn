//! High-entropy random secrets and their storage hashes.
//!
//! Used for refresh tokens and controller keys. Such secrets are 256 random
//! bits, so there is nothing to brute-force: a fast hash (SHA-256) is enough,
//! unlike passwords, which need a deliberately slow one (Argon2id).

use sha2::{Digest, Sha256};

use crate::application::{BoxError, SecretGenerator};

/// 32 random bytes from the operating system, hex-encoded (64 characters).
pub fn random_secret() -> Result<String, BoxError> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).map_err(|err| format!("getrandom failed: {err}"))?;
    Ok(hex(&bytes))
}

/// SHA-256 of `secret`, hex-encoded. Storing only this means a database
/// leak does not reveal usable secrets.
pub fn sha256_hex(secret: &str) -> String {
    hex(&Sha256::digest(secret.as_bytes()))
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut out, byte| {
            // Writing to a String cannot fail.
            let _ = write!(out, "{byte:02x}");
            out
        })
}

/// The production `SecretGenerator`.
#[derive(Debug, Clone, Copy, Default)]
pub struct RandomSecrets;

impl SecretGenerator for RandomSecrets {
    fn generate(&self) -> Result<String, BoxError> {
        random_secret()
    }

    fn hash(&self, secret: &str) -> String {
        sha256_hex(secret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secrets_are_random_hex() {
        let (a, b) = (random_secret().unwrap(), random_secret().unwrap());
        assert_eq!(a.len(), 64);
        assert!(a.bytes().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(a, b);
    }

    #[test]
    fn sha256_matches_the_standard_test_vector() {
        assert_eq!(
            sha256_hex("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
