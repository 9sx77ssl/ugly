use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

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

/// Batch size for key generation (balances throughput vs responsiveness).
pub const BATCH_SIZE: usize = 4096;

/// Global atomic state shared across threads.
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

/// Main entry point for the generate command.
pub fn run_generate(config: &GenerateConfig) -> Result<()> {
    let state = Arc::new(GenerationState::new());
    let state_clone = Arc::clone(&state);

    // Setup Ctrl+C handler
    ctrlc::set_handler(move || {
        eprintln!("\n⚠  Shutdown requested, finishing current batch...");
        state_clone.shutdown_requested.store(true, Ordering::Release);
        state_clone.match_found.store(true, Ordering::Release);
    })
    .expect("Failed to set Ctrl+C handler");

    // Create UI
    let pb = progress::create_progress_bar();
    pb.set_message(format!(
        "Pattern: \"{}\" | Threads: {} | CPU: {}%",
        config.pattern, config.threads, config.cpu_limit_percent
    ));
    pb.set_length(u64::MAX);

    // Banner
    eprintln!();
    eprintln!("🔑 Ugly v{} — Solana Vanity Address Generator", env!("CARGO_PKG_VERSION"));
    eprintln!("{}", "━".repeat(50));
    eprintln!("  Pattern:  {}", config.pattern);
    eprintln!("  Threads:  {}", config.threads);
    eprintln!("  CPU:      {}%", config.cpu_limit_percent);
    eprintln!("  Output:   {}", config.output_file.display());
    eprintln!("{}", "━".repeat(50));
    eprintln!();

    // Spawn a background thread to update the progress bar
    let pb_clone = pb.clone();
    let state_pb = Arc::clone(&state);
    let pb_thread = std::thread::spawn(move || {
        while !state_pb.match_found.load(Ordering::Relaxed) {
            let attempts = state_pb.total_attempts.load(Ordering::Relaxed);
            pb_clone.set_position(attempts);
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
        // Final update
        let attempts = state_pb.total_attempts.load(Ordering::Relaxed);
        pb_clone.set_position(attempts);
    });

    // Run generation with rayon
    let config_clone = config.clone();
    let state_for_threads = Arc::clone(&state);

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(config.threads)
        .build()
        .map_err(|e| UglyError::EncryptionFailed(e.to_string()))?;

    pool.install(|| {
        (0..config.threads).into_par_iter().for_each(|thread_id| {
            worker_thread(
                thread_id,
                &config_clone,
                &state_for_threads,
            );
        })
    });

    // Wait for progress bar thread
    let _ = pb_thread.join();

    // Check results
    let final_attempts = state.total_attempts.load(Ordering::Relaxed);

    if state.shutdown_requested.load(Ordering::Acquire) {
        eprintln!("\n⚠  Generation interrupted.");
        eprintln!("  Total attempts: {}", final_attempts);
        return Err(UglyError::Interrupted);
    }

    // If no match was found and max_attempts reached:
    if config.max_attempts > 0 && final_attempts >= config.max_attempts {
        eprintln!("\n✗ No match found within {} attempts.", config.max_attempts);
        return Err(UglyError::NoMatchFound(config.max_attempts));
    }

    // Match was found and handled in worker thread
    Ok(())
}

/// Worker thread function: generates keypairs in batches and checks for pattern matches.
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
        // Check for shutdown or match
        if state.match_found.load(Ordering::Acquire) {
            return;
        }

        // Check max attempts
        if config.max_attempts > 0 {
            let current_total = state.total_attempts.load(Ordering::Relaxed);
            if current_total >= config.max_attempts {
                return;
            }
        }

        // Generate batch
        keygen::generate_keypair_batch(&mut batch);

        // Check each keypair in the batch
        for keypair in &batch {
            address_buf.clear();
            base58::encode_into(&keypair.public_key, &mut address_buf);

            if matcher::matches_prefix(&address_buf, &config.pattern) {
                // Use compare-and-swap to ensure ONLY ONE thread handles the match
                let already_handled = state.match_found.compare_exchange(
                    false,
                    true,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                );

                if already_handled.is_err() {
                    // Another thread already found a match, just return
                    return;
                }

                // This thread won the race — handle the match
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                let elapsed = state.total_attempts.load(Ordering::Relaxed);

                // Get password once
                let password = prompt_for_password();

                // Save to encrypted file
                let save_result = file::append_wallet_entry(
                    &config.output_file,
                    &keypair.public_key,
                    &keypair.private_key,
                    timestamp,
                    &password,
                );

                eprintln!();
                eprintln!("{} MATCH FOUND!", "✓".repeat(50));
                eprintln!("  Public Key:  {}", address_buf);
                if save_result.is_ok() {
                    eprintln!("  Saved to:    {}", config.output_file.display());
                } else {
                    eprintln!("  ⚠ Failed to save to file");
                }
                eprintln!("  Total attempts: {}", elapsed);
                eprintln!("{}", "━".repeat(50));

                return;
            }
        }

        // Update atomic counter
        state
            .total_attempts
            .fetch_add(BATCH_SIZE as u64, Ordering::Relaxed);

        // Throttle check
        batch_counter += 1;
        if batch_counter.is_multiple_of(throttle_check_every) {
            throttler.maybe_sleep();
        }
    }
}

/// Prompt user for password (hidden input).
fn prompt_for_password() -> String {
    eprint!("Enter encryption password: ");
    rpassword::read_password().unwrap_or_default()
}
