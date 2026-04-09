use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::config::BenchmarkConfig;
use crate::crypto::keygen;
use crate::error::Result;

const BATCH_SIZE: usize = 4096;

fn sep() -> String {
    "─".repeat(50)
}

/// Run the benchmark command.
pub fn run_benchmark(config: &BenchmarkConfig) -> Result<()> {
    let num_cpus = num_cpus::get();

    eprintln!();
    eprintln!("Benchmark: measuring key generation performance");
    eprintln!("{}", sep());
    eprintln!("  Duration: {} seconds", config.duration_secs);
    eprintln!("  Testing 1..{} threads", num_cpus);
    eprintln!("{}", sep());
    eprintln!();

    let mut results: Vec<(usize, f64)> = Vec::with_capacity(num_cpus);

    // Setup Ctrl+C handler ONCE
    let interrupted = Arc::new(AtomicBool::new(false));
    let interrupted_clone = Arc::clone(&interrupted);
    ctrlc::set_handler(move || {
        if interrupted_clone.swap(true, Ordering::Release) {
            // Second Ctrl+C — force exit
            std::process::exit(1);
        }
        eprintln!("\n[!] Benchmark interrupted. Press Ctrl+C again to force quit.");
    })
    .expect("Failed to set Ctrl+C handler");

    for threads in 1..=num_cpus {
        if interrupted.load(Ordering::Acquire) {
            eprintln!("\nBenchmark cancelled.");
            return Ok(());
        }

        let attempts = Arc::new(AtomicU64::new(0));
        let start = Instant::now();
        let duration = Duration::from_secs(config.duration_secs);

        // Spawn worker threads
        let handles: Vec<_> = (0..threads)
            .map(|_| {
                let attempts = Arc::clone(&attempts);
                let interrupted = Arc::clone(&interrupted);
                std::thread::spawn(move || {
                    let mut local_count: u64 = 0;
                    let mut batch = keygen::new_batch_buffer(BATCH_SIZE);

                    while !interrupted.load(Ordering::Acquire) && start.elapsed() < duration {
                        keygen::generate_keypair_batch(&mut batch);
                        local_count += BATCH_SIZE as u64;
                        attempts.fetch_add(BATCH_SIZE as u64, Ordering::Relaxed);
                    }

                    local_count
                })
            })
            .collect();

        for h in handles {
            let _ = h.join();
        }

        if interrupted.load(Ordering::Acquire) {
            eprintln!("\nBenchmark cancelled.");
            return Ok(());
        }

        let elapsed = start.elapsed().as_secs_f64();
        let total = attempts.load(Ordering::Relaxed);
        let kps = if elapsed > 0.0 {
            total as f64 / elapsed
        } else {
            0.0
        };

        results.push((threads, kps));
        eprintln!("  {:>3} threads: {:>14.2} keys/sec", threads, kps);
    }

    // Find best
    if let Some(best) = results.iter().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap()) {
        eprintln!();
        eprintln!("{}", sep());
        eprintln!(
            "Recommended: {} threads ({:.2} keys/sec)",
            best.0, best.1
        );
        eprintln!("{}", sep());
    }

    Ok(())
}
