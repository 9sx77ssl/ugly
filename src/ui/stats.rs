use std::time::{Duration, Instant};

pub struct PerformanceStats {
    pub keys_per_second: f64,
    pub elapsed: Duration,
    pub total_attempts: u64,
}

pub struct StatsCalculator {
    start_time: Instant,
    last_update: Instant,
    last_attempts: u64,
}

impl StatsCalculator {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start_time: now,
            last_update: now,
            last_attempts: 0,
        }
    }

    pub fn calculate(&mut self, current_attempts: u64) -> PerformanceStats {
        let now = Instant::now();
        let elapsed = now.duration_since(self.start_time);
        let delta_attempts = current_attempts.saturating_sub(self.last_attempts);
        let delta_time = now.duration_since(self.last_update).as_secs_f64();

        let keys_per_sec = if delta_time > 0.0 {
            delta_attempts as f64 / delta_time
        } else {
            0.0
        };

        self.last_update = now;
        self.last_attempts = current_attempts;

        PerformanceStats {
            keys_per_second: keys_per_sec,
            elapsed,
            total_attempts: current_attempts,
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}
