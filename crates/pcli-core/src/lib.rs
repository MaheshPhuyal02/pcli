//! pcli-core - Core library for Project CLI
//!
//! This crate provides the shared business logic, models, and storage layer
//! for the pcli ecosystem.

pub mod config;
pub mod error;
pub mod models;
pub mod storage;

pub use config::Config;
pub use error::{Error, Result};
pub use models::{Project, Task, TaskStatus, Priority, Reminder, ReminderStatus, TaskRef, TimeLog};
pub use storage::Storage;

/// Application version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application name
pub const APP_NAME: &str = "pcli";

/// Get the data directory path (~/.pcli)
pub fn data_dir() -> std::path::PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".pcli")
}

/// Get the database path
pub fn db_path() -> std::path::PathBuf {
    data_dir().join("pcli.db")
}

/// Get the config path
pub fn config_path() -> std::path::PathBuf {
    data_dir().join("config.toml")
}

/// Initialize the application (create directories if needed)
pub fn init() -> Result<()> {
    let data_dir = data_dir();
    if !data_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(&data_dir) {
            tracing::error!("Failed to create data directory: {}", e);
            return Err(e.into());
        }
        tracing::info!("Created data directory: {:?}", data_dir);
    }
    Ok(())
}

/// Get the daemon PID file path
pub fn daemon_pid_path() -> std::path::PathBuf {
    data_dir().join("daemon.pid")
}
