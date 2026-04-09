use thiserror::Error;

#[allow(clippy::enum_variant_names)]
#[derive(Error, Debug)]
pub enum UglyError {
    #[error("Invalid pattern: '{0}' — must be 1-32 alphanumeric characters")]
    InvalidPattern(String),

    #[error("Thread count must be between 1 and {max}")]
    InvalidThreadCount { max: usize },

    #[error("CPU limit must be between 10 and 100")]
    InvalidCpuLimit,

    #[allow(dead_code)]
    #[error("Memory limit exceeded: {0} MB used (limit: 512 MB)")]
    MemoryLimitExceeded(u64),

    #[error("Failed to generate keypair: {0}")]
    KeyGenerationFailed(#[from] ed25519_dalek::SignatureError),

    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: wrong password or corrupted file")]
    DecryptionFailed,

    #[error("Invalid file format: missing magic bytes")]
    InvalidFileFormat,

    #[error("File error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to read password")]
    PasswordReadFailed,

    #[error("No match found within {0} attempts")]
    NoMatchFound(u64),

    #[allow(dead_code)]
    #[error("Benchmark interrupted")]
    BenchmarkInterrupted,

    #[error("Generation interrupted by user")]
    Interrupted,
}

pub type Result<T> = std::result::Result<T, UglyError>;
