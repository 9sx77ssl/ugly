use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use rayon::prelude::*;

use crate::config::GenerateConfig;
use crate::crypto::{
    base58,
    keygen,
    matcher,
};
use crate::error::{Result, UglyError};
use crate::resource::throttle::Throttler;
use crate::storage::file;
use crate::ui::progress;

/// Batch size for key generation.
pub const BATCH_SIZE: usize = 4096;

/// Shared state across worker threads.
pub struct GenerationState {
    pub total_attempts: AtomicU64,
    pub match_found: AtomicBool,
    pub shutdown_requested: AtomicBool,
}

impl GenerationState {
    pub fn new() -> Self {
        Self {
            total_attempts: AtomicU64::new(0),
            match_found: AtomicBool::new(false),
            shutdown_requested: AtomicBool::new(false),
        }
    }
}

/// Separator line for output formatting.
fn sep() -> String {
    "─".repeat(50)
}

/// Main entry point for the generate command.
pub fn run_generate(config: &GenerateConfig) -> Result<()> {
    let state = Arc::new(GenerationState::new());
    let state_clone = Arc::clone(&state);

    // Ctrl+C handler
    ctrlc::set_handler(move || {
        progress::clear_progress();
        eprintln!("\n[!] Shutdown requested, finishing current batch...");
        state_clone.shutdown_requested.store(true, Ordering::Release);
        state_clone.match_found.store(true, Ordering::Release);
    })
    .expect("Failed to set Ctrl+C handler");

    // Banner
    eprintln!();
    eprintln!("ugly v{} -- Solana Vanity Address Generator", env!("CARGO_PKG_VERSION"));
    eprintln!("{}", sep());
    eprintln!("  Pattern:  {}", config.pattern);
    eprintln!("  Threads:  {}", config.threads);
    eprintln!("  CPU:      {}%", config.cpu_limit_percent);
    eprintln!("  Output:   {}", config.output_file.display());
    eprintln!("{}", sep());
    eprintln!();

    let start_time = Instant::now();

    // Background progress updater
    let state_bg = Arc::clone(&state);
    let time_bg = start_time;
    let progress_handle = std::thread::spawn(move || {
        let mut last_attempts: u64 = 0;
        let mut last_time = time_bg;

        while !state_bg.match_found.load(Ordering::Relaxed) {
            let attempts = state_bg.total_attempts.load(Ordering::Relaxed);
            let now = Instant::now();
            let elapsed = now.duration_since(time_bg);

            let delta = attempts.saturating_sub(last_attempts);
            let delta_t = now.duration_since(last_time).as_secs_f64();
            let kps = if delta_t > 0.0 {
                delta as f64 / delta_t
            } else {
                0.0
            };

            progress::print_progress(elapsed, attempts, kps);

            last_attempts = attempts;
            last_time = now;

            std::thread::sleep(Duration::from_millis(200));
        }
        // Final update
        let attempts = state_bg.total_attempts.load(Ordering::Relaxed);
        let elapsed = Instant::now().duration_since(time_bg);
        progress::print_progress(elapsed, attempts, 0.0);
        progress::clear_progress();
    });

    // Rayon thread pool
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(config.threads)
        .build()
        .map_err(|e| UglyError::EncryptionFailed(e.to_string()))?;

    let config_clone = config.clone();
    let state_for_pool = Arc::clone(&state);

    pool.install(|| {
        let state_ref = &state_for_pool;
        let config_ref = &config_clone;
        (0..config.threads).into_par_iter().for_each(|thread_id| {
            worker_thread(thread_id, config_ref, state_ref);
        });
    });

    let _ = progress_handle.join();

    // Check results
    let final_attempts = state.total_attempts.load(Ordering::Relaxed);

    if state.shutdown_requested.load(Ordering::Acquire) {
        eprintln!("\n[!] Generation interrupted.");
        eprintln!("  Total attempts: {}", final_attempts);
        return Err(UglyError::Interrupted);
    }

    if config.max_attempts > 0 && final_attempts >= config.max_attempts {
        eprintln!("\nx No match found within {} attempts.", config.max_attempts);
        return Err(UglyError::NoMatchFound(config.max_attempts));
    }

    Ok(())
}

/// Worker thread: generates keypairs in batches and checks for matches.
fn worker_thread(
    _thread_id: usize,
    config: &GenerateConfig,
    state: &GenerationState,
) {
    let mut batch = keygen::new_batch_buffer(BATCH_SIZE);
    let mut address_buf = String::with_capacity(44);
    let mut throttler = Throttler::new(config.cpu_limit_percent);

    let throttle_check_every: u64 = 16;
    let mut batch_counter: u64 = 0;

    loop {
        if state.match_found.load(Ordering::Acquire) {
            return;
        }

        if config.max_attempts > 0 {
            let current_total = state.total_attempts.load(Ordering::Relaxed);
            if current_total >= config.max_attempts {
                return;
            }
        }

        keygen::generate_keypair_batch(&mut batch);

        for keypair in &batch {
            address_buf.clear();
            base58::encode_into(&keypair.public_key, &mut address_buf);

            if matcher::matches_prefix(&address_buf, &config.pattern) {
                // Only one thread handles the match via CAS
                if state
                    .match_found
                    .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                    .is_err()
                {
                    return;
                }

                // Winner thread handles saving
                handle_match(&address_buf, keypair, config, state);
                return;
            }
        }

        state
            .total_attempts
            .fetch_add(BATCH_SIZE as u64, Ordering::Relaxed);

        batch_counter += 1;
        if batch_counter.is_multiple_of(throttle_check_every) {
            throttler.maybe_sleep();
        }
    }
}

/// Handle a found match: check for duplicates, prompt password, save.
fn handle_match(
    address: &str,
    keypair: &keygen::WalletKeyPair,
    config: &GenerateConfig,
    state: &GenerationState,
) {
    progress::clear_progress();

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let elapsed = state.total_attempts.load(Ordering::Relaxed);

    // Prompt for password
    eprint!("Enter encryption password: ");
    let password = rpassword::read_password().unwrap_or_default();

    // Check for duplicate before saving
    if config.output_file.exists() {
        match file::contains_public_key(&config.output_file, &password, &keypair.public_key) {
            Ok(true) => {
                eprintln!();
                eprintln!("  [!] Duplicate: address {} already in file, skipping.", address);
                return;
            }
            Ok(false) => {}
            Err(_) => {
                // Wrong password or corrupt file — proceed to append anyway
            }
        }
    }

    // Save to file
    let save_result = file::append_wallet_entry(
        &config.output_file,
        &keypair.public_key,
        &keypair.private_key,
        timestamp,
        &password,
    );

    eprintln!();
    eprintln!("{} MATCH FOUND {}", sep(), sep());
    eprintln!("  Address:       {}", address);
    if save_result.is_ok() {
        eprintln!("  Saved to:      {}", config.output_file.display());
    } else {
        eprintln!("  [!] Failed to save to file");
    }
    eprintln!("  Total attempts:  {}", elapsed);
    eprintln!("{}", sep());
}
