//! Reminder scheduler

use anyhow::Result;
use pcli_core::{ReminderStatus, Storage};

use crate::notifier;

/// Check for due reminders and fire them
pub fn check_reminders(storage: &Storage) -> Result<()> {
    let reminders = storage.list_pending_reminders()?;

    for reminder in reminders {
        if reminder.should_fire() {
            tracing::info!("Firing reminder: {}", reminder.message);

            // Send notification
            notifier::send(&reminder.message)?;

            // Mark as fired
            storage.update_reminder_status(reminder.id, ReminderStatus::Fired)?;
        }
    }

    Ok(())
}
