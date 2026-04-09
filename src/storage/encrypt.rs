use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use argon2::{Argon2, Params};
use rand_core::RngCore;
use zeroize::Zeroize;

use crate::error::{Result, UglyError};

/// Derive a 32-byte encryption key from a password and salt using Argon2id.
pub fn derive_key(password: &str, salt: &[u8; 16]) -> Result<[u8; 32]> {
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        Params::new(64 * 1024, 3, 1, Some(32))
            .map_err(|e| UglyError::EncryptionFailed(e.to_string()))?,
    );

    let mut key = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| UglyError::EncryptionFailed(e.to_string()))?;

    Ok(key)
}

/// Encrypt data using AES-256-GCM. Returns nonce + ciphertext.
pub fn encrypt(data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(key.into());
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, data)
        .map_err(|e| UglyError::EncryptionFailed(e.to_string()))?;

    let mut result = Vec::with_capacity(12 + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);

    nonce_bytes.zeroize();
    Ok(result)
}

/// Decrypt data using AES-256-GCM. Expects nonce prepended to ciphertext.
pub fn decrypt(nonce_and_ciphertext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>> {
    if nonce_and_ciphertext.len() < 12 {
        return Err(UglyError::DecryptionFailed);
    }

    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from_slice(&nonce_and_ciphertext[..12]);
    let ciphertext = &nonce_and_ciphertext[12..];

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| UglyError::DecryptionFailed)
}

/// Generate a random 16-byte salt.
pub fn generate_salt() -> [u8; 16] {
    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);
    salt
}
