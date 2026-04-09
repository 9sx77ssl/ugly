mod benchmark;
mod cli;
mod config;
mod crypto;
mod decrypt;
mod engine;
mod error;
mod resource;
mod storage;
mod ui;

use clap::Parser;
use cli::Cli;
use config::AppConfig;
use error::Result;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let app_config = AppConfig::from_cli(cli.command)?;

    match app_config {
        AppConfig::Generate(config) => {
            engine::run_generate(&config)?;
        }
        AppConfig::Benchmark(config) => {
            benchmark::run_benchmark(&config)?;
        }
        AppConfig::Decrypt(config) => {
            decrypt::run_decrypt(&config)?;
        }
    }

    Ok(())
}
