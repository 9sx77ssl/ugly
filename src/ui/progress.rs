use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub fn create_progress_bar() -> ProgressBar {
    let pb = ProgressBar::hidden();
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] {pos} keys checked\n\
             {msg}\n\
             ⚡ {per_sec} keys/s",
        )
        .unwrap()
        .progress_chars("█▓▒░"),
    );
    pb.enable_steady_tick(Duration::from_millis(1000));
    pb
}

#[allow(dead_code)]
pub fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}
