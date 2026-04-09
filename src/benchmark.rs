use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::config::BenchmarkConfig;
use crate::crypto::keygen;
use crate::error::Result;

const BATCH_SIZE: usize = 4096;

/// Run the benchmark command.
pub fn run_benchmark(config: &BenchmarkConfig) -> Result<()> {
    let num_cpus = num_cpus::get();

    eprintln!();
    eprintln!("⚡ Benchmark: measuring key generation performance");
    eprintln!("{}", "━".repeat(50));
    eprintln!("  Duration: {} seconds", config.duration_secs);
    eprintln!("  Testing 1..{} threads", num_cpus);
    eprintln!("{}", "━".repeat(50));
    eprintln!();

    let mut results: Vec<(usize, f64)> = Vec::with_capacity(num_cpus);

    for threads in 1..=num_cpus {
        let attempts = Arc::new(AtomicU64::new(0));
        let running = Arc::new(AtomicBool::new(true));
        let start = Instant::now();
        let duration = Duration::from_secs(config.duration_secs);

        let attempts_clone = Arc::clone(&attempts);
        let running_clone = Arc::clone(&running);

        // Spawn worker threads using std::thread
        let handles: Vec<_> = (0..threads)
            .map(|_| {
                let attempts = Arc::clone(&attempts);
                let running = Arc::clone(&running);
                std::thread::spawn(move || {
                    let mut local_count: u64 = 0;

                    // Pre-allocate batch
                    let mut batch = keygen::new_batch_buffer(BATCH_SIZE);

                    while running.load(Ordering::Acquire) && start.elapsed() < duration {
                        keygen::generate_keypair_batch(&mut batch);
                        local_count += BATCH_SIZE as u64;
                        attempts.fetch_add(BATCH_SIZE as u64, Ordering::Relaxed);
                    }

                    local_count
                })
            })
            .collect();

        // Ctrl+C handler for current iteration
        let running_inner = Arc::clone(&running_clone);
        let _ = ctrlc::set_handler(move || {
            eprintln!("\n⚠  Benchmark interrupted.");
            running_inner.store(false, Ordering::Release);
        });

        for h in handles {
            let _ = h.join();
        }

        if !running_clone.load(Ordering::Acquire) {
            eprintln!("\nBenchmark cancelled.");
            return Ok(());
        }

        let elapsed = start.elapsed().as_secs_f64();
        let total = attempts_clone.load(Ordering::Relaxed);
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
        eprintln!("{}", "━".repeat(50));
        eprintln!(
            "✓ Recommended: {} threads ({:.2} keys/sec)",
            best.0, best.1
        );
        eprintln!("{}", "━".repeat(50));
    }

    Ok(())
}
