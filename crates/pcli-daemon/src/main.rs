//! pcli-daemon - Background service for reminders and timer
//!
//! This daemon runs in the background and handles:
//! - Reminder notifications
//! - Timer popup management
//! - IPC with the CLI

mod scheduler;
mod notifier;

use anyhow::Result;
use pcli_core::{Storage, db_path, init};
use tokio::time::{interval, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Initialize app
    init()?;

    tracing::info!("pcli-daemon starting...");

    // Write PID file
    let pid_path = pcli_core::daemon_pid_path();
    let pid = std::process::id();
    if let Err(e) = std::fs::write(&pid_path, pid.to_string()) {
        tracing::error!("Failed to write PID file: {}", e);
        return Err(e.into());
    }
    tracing::info!("Written PID {} to {:?}", pid, pid_path);

    // Open database
    let storage = Storage::open(&db_path())?;

    // Main loop - check reminders every 10 seconds
    let mut ticker = interval(Duration::from_secs(10));

    loop {
        ticker.tick().await;

        // Check for due reminders
        if let Err(e) = scheduler::check_reminders(&storage) {
            tracing::error!("Error checking reminders: {}", e);
        }
    }
}
