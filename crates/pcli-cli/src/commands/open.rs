//! Open URLs in browsers

use anyhow::{Context, Result};
use colored::*;
use std::process::Command;

/// Supported browsers
#[derive(Debug, Clone, Copy)]
pub enum Browser {
    Chrome,
    Safari,
    Firefox,
    Default,
}

impl Browser {
    /// Get the macOS application name for the browser
    fn app_name(&self) -> &'static str {
        match self {
            Browser::Chrome => "Google Chrome",
            Browser::Safari => "Safari",
            Browser::Firefox => "Firefox",
            Browser::Default => "",
        }
    }
}

/// Open a URL in the specified browser
pub fn open_url(url: &str, browser: Browser) -> Result<()> {
    // Normalize URL - add https:// if no protocol specified
    let normalized_url = if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        format!("https://{}", url)
    };

    let result = match browser {
        Browser::Default => {
            // Use the default browser
            Command::new("open")
                .arg(&normalized_url)
                .status()
                .context("Failed to open URL with default browser")?
        }
        _ => {
            // Use a specific browser application
            Command::new("open")
                .arg("-a")
                .arg(browser.app_name())
                .arg(&normalized_url)
                .status()
                .context(format!("Failed to open URL with {}", browser.app_name()))?
        }
    };

    if result.success() {
        let browser_name = match browser {
            Browser::Default => "default browser",
            _ => browser.app_name(),
        };
        println!(
            "{} {} in {}",
            "Opened".green(),
            normalized_url.cyan(),
            browser_name.yellow()
        );
    } else {
        println!(
            "{} Failed to open URL",
            "Error:".red()
        );
    }

    Ok(())
}
