use std::time::{Duration, Instant};
use std::thread;

/// Adaptive CPU throttler that adjusts sleep time based on the configured CPU limit percentage.
pub struct Throttler {
    cpu_limit_percent: f64,
    check_interval: Duration,
    last_check: Instant,
    sleep_duration: Duration,
}

impl Throttler {
    pub fn new(cpu_limit_percent: u8) -> Self {
        Self {
            cpu_limit_percent: cpu_limit_percent as f64,
            check_interval: Duration::from_millis(100),
            last_check: Instant::now(),
            sleep_duration: Duration::ZERO,
        }
    }

    /// Call this periodically in the generation loop. Will sleep if needed.
    #[inline]
    pub fn maybe_sleep(&mut self) {
        // If limit is 100%, no throttling
        if self.cpu_limit_percent >= 99.0 {
            return;
        }

        let elapsed = self.last_check.elapsed();
        if elapsed >= self.check_interval {
            // Adaptive adjustment: ratio of work vs sleep
            let work_ratio = self.cpu_limit_percent / 100.0;
            let work_time = elapsed.as_secs_f64();
            // Desired sleep time to maintain the work ratio
            let desired_sleep = if work_ratio > 0.0 {
                work_time * (1.0 - work_ratio) / work_ratio
            } else {
                work_time
            };

            // Clamp sleep to 0-10ms range for responsiveness
            self.sleep_duration =
                Duration::from_secs_f64(desired_sleep.clamp(0.0, 0.01));
            self.last_check = Instant::now();
        }

        if self.sleep_duration > Duration::ZERO {
            thread::sleep(self.sleep_duration);
        }
    }
}
