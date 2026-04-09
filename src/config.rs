use crate::cli::Commands;
use crate::error::{Result, UglyError};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct GenerateConfig {
    pub pattern: String,
    pub threads: usize,
    pub cpu_limit_percent: u8,
    pub max_attempts: u64,
    pub output_file: PathBuf,
}

#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    pub duration_secs: u64,
}

#[derive(Debug, Clone)]
pub struct DecryptConfig {
    pub file: PathBuf,
}

pub enum AppConfig {
    Generate(GenerateConfig),
    Benchmark(BenchmarkConfig),
    Decrypt(DecryptConfig),
}

impl AppConfig {
    pub fn from_cli(command: Commands) -> Result<Self> {
        match command {
            Commands::Generate {
                pattern,
                threads,
                cpu_limit,
                max_attempts,
                output,
            } => {
                // Validate pattern
                if pattern.is_empty() || pattern.len() > 32 {
                    return Err(UglyError::InvalidPattern(pattern));
                }
                if !pattern.chars().all(|c| c.is_ascii_alphanumeric()) {
                    return Err(UglyError::InvalidPattern(pattern));
                }

                // Validate and resolve thread count
                let num_cpus = num_cpus::get();
                let resolved_threads = match threads {
                    Some(n) => {
                        if n < 1 || n > num_cpus {
                            return Err(UglyError::InvalidThreadCount { max: num_cpus });
                        }
                        n
                    }
                    None => num_cpus,
                };

                // Validate CPU limit
                if !(10..=100).contains(&cpu_limit) {
                    return Err(UglyError::InvalidCpuLimit);
                }

                Ok(AppConfig::Generate(GenerateConfig {
                    pattern,
                    threads: resolved_threads,
                    cpu_limit_percent: cpu_limit,
                    max_attempts,
                    output_file: output.unwrap_or_else(|| PathBuf::from("./wallets.enc")),
                }))
            }
            Commands::Benchmark { duration } => Ok(AppConfig::Benchmark(BenchmarkConfig {
                duration_secs: duration,
            })),
            Commands::Decrypt { file } => Ok(AppConfig::Decrypt(DecryptConfig { file })),
        }
    }
}
