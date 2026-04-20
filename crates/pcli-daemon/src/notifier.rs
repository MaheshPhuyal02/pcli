//! System notifications

use anyhow::Result;
#[cfg(not(target_os = "macos"))]
use notify_rust::Notification;
#[cfg(target_os = "macos")]
use std::process::Command;

/// Send a system notification
pub fn send(message: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        // Use AppleScript to display notification with sound
        // "Ping" is a standard system sound
        let script = format!(
            "display notification \"{}\" with title \"⏰ pcli Reminder\" sound name \"Ping\"",
            message.replace("\"", "\\\"")
        );

        Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()?;
    }

    #[cfg(not(target_os = "macos"))]
    {
        Notification::new()
            .summary("⏰ pcli Reminder")
            .body(message)
            .appname("pcli")
            .timeout(10000) // 10 seconds
            .show()?;
    }

    Ok(())
}
