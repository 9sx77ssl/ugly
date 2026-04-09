use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// Signals the progress thread to stop.
pub static PROGRESS_DONE: AtomicBool = AtomicBool::new(false);

/// Width of the terminal progress line.
const LINE_WIDTH: usize = 52;

/// Print a progress status line using \r to overwrite in-place.
pub fn print_progress(elapsed: Duration, attempts: u64, kps: f64) {
    if PROGRESS_DONE.load(Ordering::Relaxed) {
        return;
    }
    let secs = elapsed.as_secs();
    let mins = secs / 60;
    let hrs = mins / 60;
    let rem_mins = mins % 60;
    let rem_secs = secs % 60;

    let line = format!(
        "  [{:02}:{:02}:{:02}] {} keys checked | {:.0}/s",
        hrs, rem_mins, rem_secs, attempts, kps
    );

    // Pad to line width and use \r to stay on same line
    let padded = format!("{:<width$}", line, width = LINE_WIDTH);
    eprint!("\r{}", padded);
}

/// Clear the progress line and move to next line.
pub fn clear_progress() {
    PROGRESS_DONE.store(true, Ordering::Release);
    // Overwrite the line with spaces and move down
    eprint!("\r{: <width$}\r\n", "", width = LINE_WIDTH);
}

#[allow(dead_code)]
pub fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}
