//! Error types for pcli-core

use thiserror::Error;

/// Result type alias for pcli operations
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for pcli
#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Project not found: {0}")]
    ProjectNotFound(String),

    #[error("Task not found: {0}")]
    TaskNotFound(i32),

    #[error("Reminder not found: {0}")]
    ReminderNotFound(i32),

    #[error("No active project. Use 'pcli <project-name>' to switch or 'pcli new <name>' to create one.")]
    NoActiveProject,

    #[error("Project already exists: {0}")]
    ProjectAlreadyExists(String),

    #[error("Invalid status: {0}")]
    InvalidStatus(String),

    #[error("Invalid priority: {0}")]
    InvalidPriority(String),

    #[error("Invalid duration format: {0}")]
    InvalidDuration(String),

    #[error("Invalid date format: {0}")]
    InvalidDate(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("{0}")]
    Other(String),
}
