use ed25519_dalek::SigningKey;
use rand_core::OsRng;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// A wallet result containing cryptographic key material.
/// Implements ZeroizeOnDrop to securely clear memory when dropped.
#[derive(Zeroize, ZeroizeOnDrop)]
#[allow(dead_code)]
pub struct WalletKeyPair {
    pub public_key: [u8; 32],
    pub private_key: [u8; 64],
}

#[allow(dead_code)]
impl WalletKeyPair {
    pub fn public_key_base58(&self) -> String {
        crate::crypto::base58::encode(&self.public_key)
    }
}

/// Generate a single ed25519 keypair using OS-provided randomness.
#[inline]
#[allow(dead_code)]
pub fn generate_keypair() -> WalletKeyPair {
    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();

    let mut public_key = [0u8; 32];
    let mut private_key = [0u8; 64];

    public_key.copy_from_slice(verifying_key.as_bytes());
    // The full keypair bytes: 32-byte seed + 32-byte verifying key
    let kp_bytes = signing_key.to_keypair_bytes();
    private_key.copy_from_slice(&kp_bytes);

    WalletKeyPair {
        public_key,
        private_key,
    }
}

/// Generate a batch of keypairs. Pre-allocated to avoid reallocations.
#[inline]
pub fn generate_keypair_batch(batch: &mut [WalletKeyPair]) {
    for keypair in batch.iter_mut() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();

        keypair.public_key.copy_from_slice(verifying_key.as_bytes());
        let kp_bytes = signing_key.to_keypair_bytes();
        keypair.private_key.copy_from_slice(&kp_bytes);
    }
}

/// Create a new pre-allocated batch buffer.
pub fn new_batch_buffer(size: usize) -> Vec<WalletKeyPair> {
    let mut batch = Vec::with_capacity(size);
    for _ in 0..size {
        batch.push(WalletKeyPair {
            public_key: [0u8; 32],
            private_key: [0u8; 64],
        });
    }
    batch
}
