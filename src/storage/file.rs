use std::fs;
use std::path::Path;
use zeroize::Zeroize;

use crate::error::{Result, UglyError};
use crate::storage::encrypt;

/// Magic bytes for the file format: "UGLY"
const MAGIC: [u8; 4] = *b"UGLY";
const VERSION: u8 = 0x01;
const SALT_LEN: usize = 16;
const HEADER_LEN: usize = 4 + 1 + SALT_LEN; // magic + version + salt

/// Decrypted wallet data from file.
pub type DecryptedWallets = (Vec<[u8; 32]>, Vec<[u8; 64]>, Vec<u64>);

/// File format:
/// [0..4]   Magic: b"UGLY"
/// [4..5]   Version: 0x01
/// [5..21]  Salt (16 bytes)
/// [21..]   Nonce(12) + AES-256-GCM ciphertext
///
/// Encrypt and write wallet data to file.
#[allow(dead_code)]
pub fn write_encrypted(
    path: &Path,
    public_key: &[u8; 32],
    private_key: &[u8; 64],
    timestamp: u64,
    password: &str,
) -> Result<()> {
    // Build plaintext payload: [pubkey(32)][privkey(64)][timestamp(8)]
    let mut plaintext = Vec::with_capacity(32 + 64 + 8);
    plaintext.extend_from_slice(public_key);
    plaintext.extend_from_slice(private_key);
    plaintext.extend_from_slice(&timestamp.to_le_bytes());

    // Generate salt
    let salt = encrypt::generate_salt();

    // Derive encryption key
    let key = encrypt::derive_key(password, &salt)?;

    // Encrypt payload
    let encrypted = encrypt::encrypt(&plaintext, &key)?;

    // Write file: [magic][version][salt][encrypted(nonce+ciphertext)]
    let mut file_data = Vec::with_capacity(HEADER_LEN + encrypted.len());
    file_data.extend_from_slice(&MAGIC);
    file_data.push(VERSION);
    file_data.extend_from_slice(&salt);
    file_data.extend_from_slice(&encrypted);

    fs::write(path, &file_data)?;

    // Zeroize sensitive data
    plaintext.zeroize();

    Ok(())
}

/// Read and decrypt wallet data from file.
/// Returns (public_keys, private_keys, timestamps).
pub fn read_encrypted(
    path: &Path,
    password: &str,
) -> Result<DecryptedWallets> {
    let file_data = fs::read(path)?;

    // Validate header
    if file_data.len() < HEADER_LEN + 12 {
        // at least nonce
        return Err(UglyError::InvalidFileFormat);
    }

    if file_data[..4] != MAGIC {
        return Err(UglyError::InvalidFileFormat);
    }

    if file_data[4] != VERSION {
        return Err(UglyError::InvalidFileFormat);
    }

    // Extract salt
    let salt: [u8; 16] = file_data[5..21].try_into().unwrap();

    // Derive key
    let key = encrypt::derive_key(password, &salt)?;

    // Decrypt
    let encrypted_data = &file_data[HEADER_LEN..];
    let plaintext = encrypt::decrypt(encrypted_data, &key)?;

    // Parse entries: each entry is 32 + 64 + 8 = 104 bytes
    const ENTRY_SIZE: usize = 32 + 64 + 8;
    if plaintext.len() % ENTRY_SIZE != 0 {
        return Err(UglyError::DecryptionFailed);
    }

    let num_entries = plaintext.len() / ENTRY_SIZE;
    let mut public_keys = Vec::with_capacity(num_entries);
    let mut private_keys = Vec::with_capacity(num_entries);
    let mut timestamps = Vec::with_capacity(num_entries);

    for i in 0..num_entries {
        let offset = i * ENTRY_SIZE;
        let pub_key: [u8; 32] = plaintext[offset..offset + 32].try_into().unwrap();
        let priv_key: [u8; 64] = plaintext[offset + 32..offset + 96].try_into().unwrap();
        let ts = u64::from_le_bytes(
            plaintext[offset + 96..offset + 104].try_into().unwrap(),
        );

        public_keys.push(pub_key);
        private_keys.push(priv_key);
        timestamps.push(ts);
    }

    Ok((public_keys, private_keys, timestamps))
}

/// Append a wallet entry to an existing encrypted file.
/// Reads, decrypts, appends, re-encrypts, and rewrites the file.
pub fn append_wallet_entry(
    path: &Path,
    public_key: &[u8; 32],
    private_key: &[u8; 64],
    timestamp: u64,
    password: &str,
) -> Result<()> {
    let mut existing_pub = Vec::new();
    let mut existing_priv = Vec::new();
    let mut existing_ts = Vec::new();

    // If file exists, read existing entries
    if path.exists() {
        let (pubs, privs, timestamps) = read_encrypted(path, password)?;
        existing_pub = pubs;
        existing_priv = privs;
        existing_ts = timestamps;
    }

    // Append new entry
    existing_pub.push(*public_key);
    existing_priv.push(*private_key);
    existing_ts.push(timestamp);

    // Build plaintext for all entries
    let mut plaintext =
        Vec::with_capacity(existing_pub.len() * (32 + 64 + 8));

    for i in 0..existing_pub.len() {
        plaintext.extend_from_slice(&existing_pub[i]);
        plaintext.extend_from_slice(&existing_priv[i]);
        plaintext.extend_from_slice(&existing_ts[i].to_le_bytes());
    }

    // Generate new salt and re-encrypt everything
    let salt = encrypt::generate_salt();
    let key = encrypt::derive_key(password, &salt)?;
    let encrypted = encrypt::encrypt(&plaintext, &key)?;

    // Write file
    let mut file_data = Vec::with_capacity(HEADER_LEN + encrypted.len());
    file_data.extend_from_slice(&MAGIC);
    file_data.push(VERSION);
    file_data.extend_from_slice(&salt);
    file_data.extend_from_slice(&encrypted);

    fs::write(path, &file_data)?;

    // Zeroize
    plaintext.zeroize();
    existing_priv.zeroize();

    Ok(())
}
