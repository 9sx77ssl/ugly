use std::time::Duration;

/// Print a progress status line. Uses \r to overwrite the current line.
pub fn print_progress(elapsed: Duration, attempts: u64, kps: f64) {
    let secs = elapsed.as_secs();
    let mins = secs / 60;
    let hrs = mins / 60;
    let rem_mins = mins % 60;
    let rem_secs = secs % 60;

    eprint!(
        "\r  [{:02}:{:02}:{:02}] {} keys checked | {:.0}/s",
        hrs, rem_mins, rem_secs, attempts, kps
    );
}

/// Clear the progress line and move to next line.
pub fn clear_progress() {
    eprintln!();
}

#[allow(dead_code)]
pub fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}
