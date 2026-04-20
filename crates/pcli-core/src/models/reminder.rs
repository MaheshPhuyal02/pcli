//! Reminder model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Reminder status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReminderStatus {
    Pending,
    Fired,
    Dismissed,
    Snoozed,
}

impl fmt::Display for ReminderStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReminderStatus::Pending => write!(f, "pending"),
            ReminderStatus::Fired => write!(f, "fired"),
            ReminderStatus::Dismissed => write!(f, "dismissed"),
            ReminderStatus::Snoozed => write!(f, "snoozed"),
        }
    }
}

/// Reference to a task in a specific project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRef {
    /// Project ID
    pub project: String,
    
    /// Task ID
    pub task_id: i32,
}

impl TaskRef {
    /// Parse task reference from string like "projectname task 2"
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        
        if parts.len() >= 3 && parts[1].eq_ignore_ascii_case("task") {
            if let Ok(task_id) = parts[2].parse::<i32>() {
                return Some(TaskRef {
                    project: parts[0].to_lowercase(),
                    task_id,
                });
            }
        }
        
        None
    }
}

impl fmt::Display for TaskRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} task {}", self.project, self.task_id)
    }
}

/// A reminder for future notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reminder {
    /// Unique ID
    pub id: i32,
    
    /// Reminder message
    pub message: String,
    
    /// Optional project reference (for task-linked reminders)
    pub project_id: Option<String>,
    
    /// Optional task reference
    pub task_id: Option<i32>,
    
    /// When to trigger the reminder
    pub remind_at: DateTime<Utc>,
    
    /// Current status
    pub status: ReminderStatus,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

impl Reminder {
    /// Create a new reminder
    pub fn new(message: impl Into<String>, remind_at: DateTime<Utc>) -> Self {
        Self {
            id: 0, // Will be set by storage
            message: message.into(),
            project_id: None,
            task_id: None,
            remind_at,
            status: ReminderStatus::Pending,
            created_at: Utc::now(),
        }
    }
    
    /// Create a reminder linked to a task
    pub fn for_task(task_ref: TaskRef, remind_at: DateTime<Utc>) -> Self {
        let message = format!("{} task {}", task_ref.project, task_ref.task_id);
        Self {
            id: 0,
            message,
            project_id: Some(task_ref.project),
            task_id: Some(task_ref.task_id),
            remind_at,
            status: ReminderStatus::Pending,
            created_at: Utc::now(),
        }
    }
    
    /// Check if reminder should fire now
    pub fn should_fire(&self) -> bool {
        self.status == ReminderStatus::Pending && self.remind_at <= Utc::now()
    }
    
    /// Time remaining until reminder fires
    pub fn time_remaining(&self) -> Option<chrono::Duration> {
        if self.status == ReminderStatus::Pending {
            let remaining = self.remind_at - Utc::now();
            if remaining > chrono::Duration::zero() {
                return Some(remaining);
            }
        }
        None
    }
    
    /// Format time remaining as human-readable string
    pub fn time_remaining_str(&self) -> String {
        match self.time_remaining() {
            Some(dur) => {
                let total_secs = dur.num_seconds();
                if total_secs < 60 {
                    format!("{}s", total_secs)
                } else if total_secs < 3600 {
                    format!("{}m", total_secs / 60)
                } else if total_secs < 86400 {
                    format!("{}h {}m", total_secs / 3600, (total_secs % 3600) / 60)
                } else {
                    format!("{}d {}h", total_secs / 86400, (total_secs % 86400) / 3600)
                }
            }
            None => "now".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_task_ref_parse() {
        let task_ref = TaskRef::parse("calorii task 2").unwrap();
        assert_eq!(task_ref.project, "calorii");
        assert_eq!(task_ref.task_id, 2);
    }
    
    #[test]
    fn test_task_ref_parse_invalid() {
        assert!(TaskRef::parse("invalid").is_none());
        assert!(TaskRef::parse("project task").is_none());
    }
}
