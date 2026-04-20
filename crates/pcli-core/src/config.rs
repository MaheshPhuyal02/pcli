//! Configuration management

use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::error::{Error, Result};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// General settings
    #[serde(default)]
    pub general: GeneralConfig,
    
    /// Timer settings
    #[serde(default)]
    pub timer: TimerConfig,
    
    /// Notification settings
    #[serde(default)]
    pub notifications: NotificationConfig,
    
    /// UI settings
    #[serde(default)]
    pub ui: UiConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            timer: TimerConfig::default(),
            notifications: NotificationConfig::default(),
            ui: UiConfig::default(),
        }
    }
}

/// General settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Default priority for new tasks
    #[serde(default = "default_priority")]
    pub default_priority: String,
    
    /// Date format
    #[serde(default = "default_date_format")]
    pub date_format: String,
    
    /// Time format
    #[serde(default = "default_time_format")]
    pub time_format: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            default_priority: default_priority(),
            date_format: default_date_format(),
            time_format: default_time_format(),
        }
    }
}

fn default_priority() -> String {
    "normal".to_string()
}

fn default_date_format() -> String {
    "%Y-%m-%d".to_string()
}

fn default_time_format() -> String {
    "%H:%M".to_string()
}

/// Timer settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerConfig {
    /// Default pomodoro duration in minutes
    #[serde(default = "default_pomodoro_minutes")]
    pub default_pomodoro_minutes: u32,
    
    /// Break duration in minutes
    #[serde(default = "default_break_minutes")]
    pub break_minutes: u32,
    
    /// Long break duration in minutes
    #[serde(default = "default_long_break_minutes")]
    pub long_break_minutes: u32,
    
    /// Number of pomodoros before long break
    #[serde(default = "default_pomodoros_until_long_break")]
    pub pomodoros_until_long_break: u32,
}

impl Default for TimerConfig {
    fn default() -> Self {
        Self {
            default_pomodoro_minutes: default_pomodoro_minutes(),
            break_minutes: default_break_minutes(),
            long_break_minutes: default_long_break_minutes(),
            pomodoros_until_long_break: default_pomodoros_until_long_break(),
        }
    }
}

fn default_pomodoro_minutes() -> u32 {
    25
}

fn default_break_minutes() -> u32 {
    5
}

fn default_long_break_minutes() -> u32 {
    15
}

fn default_pomodoros_until_long_break() -> u32 {
    4
}

/// Notification settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// Whether notifications are enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// Whether to play sound with notifications
    #[serde(default = "default_true")]
    pub sound: bool,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sound: true,
        }
    }
}

fn default_true() -> bool {
    true
}

/// UI settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Color theme
    #[serde(default = "default_theme")]
    pub theme: String,
    
    /// Whether to show completed tasks
    #[serde(default = "default_true")]
    pub show_completed_tasks: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            show_completed_tasks: true,
        }
    }
}

fn default_theme() -> String {
    "dark".to_string()
}

impl Config {
    /// Load config from file, or create default
    pub fn load(path: &Path) -> Result<Self> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            toml::from_str(&content)
                .map_err(|e| Error::Config(e.to_string()))
        } else {
            Ok(Self::default())
        }
    }
    
    /// Save config to file
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| Error::Serialization(e.to_string()))?;
        std::fs::write(path, content)?;
        Ok(())
    }
    
    /// Load or create default config
    pub fn load_or_create(path: &Path) -> Result<Self> {
        if path.exists() {
            Self::load(path)
        } else {
            let config = Self::default();
            config.save(path)?;
            Ok(config)
        }
    }
}
