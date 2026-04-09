use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "ugly",
    version,
    about = "High-performance Solana vanity address generator with encrypted storage"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate a vanity address matching a pattern
    Generate {
        /// Pattern to match at the beginning of the address (prefix match)
        #[arg(short, long)]
        pattern: String,

        /// Number of threads to use (default: auto-detect CPU cores)
        #[arg(short, long)]
        threads: Option<usize>,

        /// CPU usage limit percentage (10-100, default: 85)
        #[arg(long, default_value = "85")]
        cpu_limit: u8,

        /// Maximum number of attempts before giving up (0 = infinite)
        #[arg(long, default_value = "0")]
        max_attempts: u64,

        /// Output file for encrypted wallet storage
        #[arg(short, long, default_value = "./wallets.enc")]
        output: Option<PathBuf>,
    },

    /// Benchmark key generation performance
    Benchmark {
        /// Benchmark duration in seconds (default: 10)
        #[arg(short, long, default_value = "10")]
        duration: u64,
    },

    /// Decrypt and display stored wallet information
    Decrypt {
        /// Encrypted wallet file to decrypt
        #[arg(short, long)]
        file: PathBuf,
    },
}
