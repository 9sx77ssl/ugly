use std::path::Path;

use crate::config::DecryptConfig;
use crate::crypto::base58;
use crate::error::{Result, UglyError};
use crate::storage::file;

/// Run the decrypt command.
pub fn run_decrypt(config: &DecryptConfig) -> Result<()> {
    let path = Path::new(&config.file);

    if !path.exists() {
        eprintln!("✗ File not found: {}", config.file.display());
        return Err(UglyError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("File not found: {}", config.file.display()),
        )));
    }

    // Prompt for password
    eprint!("Enter decryption password: ");
    let password = rpassword::read_password().map_err(|_| UglyError::PasswordReadFailed)?;

    // Read and decrypt
    let (public_keys, private_keys, timestamps) =
        file::read_encrypted(path, &password)?;

    eprintln!();
    eprintln!("🔓 Decrypted Wallet Information");
    eprintln!("{}", "━".repeat(50));

    if public_keys.is_empty() {
        eprintln!("  No wallets found in file.");
    } else {
        for i in 0..public_keys.len() {
            let pub_key = &public_keys[i];
            let priv_key = &private_keys[i];
            let ts = timestamps[i];

            let pub_addr = base58::encode(pub_key);
            let priv_addr = base58::encode(priv_key);

            // Format timestamp
            let dt = format_timestamp(ts);

            eprintln!("  Wallet #{}:", i + 1);
            eprintln!("    Public Key:  {}", pub_addr);
            eprintln!("    Private Key: {}", priv_addr);
            eprintln!("    Created:     {}", dt);
            if i < public_keys.len() - 1 {
                eprintln!("  {}", "─".repeat(46));
            }
        }
    }

    eprintln!("{}", "━".repeat(50));
    eprintln!("  Total wallets: {}", public_keys.len());
    eprintln!();

    Ok(())
}

fn format_timestamp(unix_secs: u64) -> String {
    // Simple UTC timestamp formatting
    let total_secs = unix_secs;
    let days_since_epoch = total_secs / 86400;
    let time_of_day = total_secs % 86400;

    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Calculate date from days since epoch (1970-01-01)
    let mut year = 1970u64;
    let mut remaining_days = days_since_epoch;

    loop {
        let is_leap = year.is_multiple_of(4)
            && !year.is_multiple_of(100)
            || year.is_multiple_of(400);
        let days_in_year = if is_leap { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    let is_leap = year.is_multiple_of(4)
        && !year.is_multiple_of(100)
        || year.is_multiple_of(400);
    let month_days = [
        31,
        if is_leap { 29 } else { 28 },
        31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
    ];

    let mut month = 1;
    let mut day = remaining_days + 1;
    for &days in &month_days {
        if day <= days as u64 {
            break;
        }
        day -= days as u64;
        month += 1;
    }

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC",
        year, month, day, hours, minutes, seconds
    )
}
