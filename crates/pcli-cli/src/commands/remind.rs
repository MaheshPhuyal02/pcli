//! Reminder commands

use anyhow::{Result, bail};
use chrono::{Duration, Utc, Local};
use colored::Colorize;
use pcli_core::{Reminder, Storage, TaskRef};

/// Create a reminder
pub fn create(storage: &Storage, duration: &str, message: &str) -> Result<()> {
    let duration_secs = parse_duration(duration)?;
    let remind_at = Utc::now() + Duration::seconds(duration_secs as i64);

    // Check if message references a task (e.g., "calorii task 2")
    let reminder = if let Some(task_ref) = TaskRef::parse(message) {
        Reminder::for_task(task_ref, remind_at)
    } else {
        Reminder::new(message, remind_at)
    };

    let id = storage.create_reminder(&reminder)?;

    // Convert to local time for display
    let local_remind_at = remind_at.with_timezone(&Local);

    println!(
        "{} Reminder #{} set for {}",
        "⏰".green(),
        id,
        local_remind_at.format("%H:%M")
    );
    println!("   \"{}\"", message.cyan());

    Ok(())
}

/// List pending reminders
pub fn list(storage: &Storage) -> Result<()> {
    let reminders = storage.list_pending_reminders()?;

    if reminders.is_empty() {
        println!("{}", "No pending reminders.".dimmed());
        return Ok(());
    }

    println!("{}", "⏰ Reminders".bold());
    println!("{}", "─".repeat(40).dimmed());

    for reminder in reminders {
        let time_str = reminder.time_remaining_str();
        println!(
            "  #{} {} in {}",
            reminder.id.to_string().dimmed(),
            reminder.message.cyan(),
            time_str.yellow()
        );
    }

    Ok(())
}

/// Parse duration string like "10m", "2h", "1d" to seconds
fn parse_duration(s: &str) -> Result<u64> {
    let s = s.trim().to_lowercase();
    
    if s.is_empty() {
        bail!("Duration cannot be empty");
    }

    let (num_str, unit) = if s.ends_with("min") || s.ends_with("mins") {
        let num = s.trim_end_matches("mins").trim_end_matches("min");
        (num, "m")
    } else if s.ends_with("hour") || s.ends_with("hours") {
        let num = s.trim_end_matches("hours").trim_end_matches("hour");
        (num, "h")
    } else if s.ends_with("day") || s.ends_with("days") {
        let num = s.trim_end_matches("days").trim_end_matches("day");
        (num, "d")
    } else {
        let last_char = s.chars().last().unwrap();
        let num = &s[..s.len() - 1];
        (num, match last_char {
            's' => "s",
            'm' => "m",
            'h' => "h",
            'd' => "d",
            _ => bail!("Invalid duration unit. Use: s, m, h, or d"),
        })
    };

    let num: u64 = num_str.parse().map_err(|_| anyhow::anyhow!("Invalid number: {}", num_str))?;

    let seconds = match unit {
        "s" => num,
        "m" => num * 60,
        "h" => num * 3600,
        "d" => num * 86400,
        _ => bail!("Invalid duration unit"),
    };

    Ok(seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("10m").unwrap(), 600);
        assert_eq!(parse_duration("2h").unwrap(), 7200);
        assert_eq!(parse_duration("1d").unwrap(), 86400);
        assert_eq!(parse_duration("30s").unwrap(), 30);
    }
}
