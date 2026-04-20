//! Task model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::Error;

/// Task status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
    Cancelled,
}

impl TaskStatus {
    /// Get all variants
    pub fn all() -> &'static [TaskStatus] {
        &[
            TaskStatus::Todo,
            TaskStatus::InProgress,
            TaskStatus::Done,
            TaskStatus::Cancelled,
        ]
    }
    
    /// Check if task is completed
    pub fn is_completed(&self) -> bool {
        matches!(self, TaskStatus::Done | TaskStatus::Cancelled)
    }
    
    /// Get emoji for status
    pub fn emoji(&self) -> &'static str {
        match self {
            TaskStatus::Todo => "○",
            TaskStatus::InProgress => "▶",
            TaskStatus::Done => "✓",
            TaskStatus::Cancelled => "✗",
        }
    }
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskStatus::Todo => write!(f, "todo"),
            TaskStatus::InProgress => write!(f, "in_progress"),
            TaskStatus::Done => write!(f, "done"),
            TaskStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl FromStr for TaskStatus {
    type Err = Error;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "todo" | "t" => Ok(TaskStatus::Todo),
            "in_progress" | "inprogress" | "progress" | "p" | "active" | "a" => {
                Ok(TaskStatus::InProgress)
            }
            "done" | "d" | "complete" | "completed" | "c" => Ok(TaskStatus::Done),
            "cancelled" | "cancel" | "x" => Ok(TaskStatus::Cancelled),
            _ => Err(Error::InvalidStatus(s.to_string())),
        }
    }
}

/// Task priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Low,
    Normal,
    High,
    Urgent,
}

impl Priority {
    /// Get all variants
    pub fn all() -> &'static [Priority] {
        &[Priority::Low, Priority::Normal, Priority::High, Priority::Urgent]
    }
    
    /// Get emoji for priority
    pub fn emoji(&self) -> &'static str {
        match self {
            Priority::Low => "🟢",
            Priority::Normal => "⚪",
            Priority::High => "🟡",
            Priority::Urgent => "🔴",
        }
    }
    
    /// Get sort order (higher = more urgent)
    pub fn order(&self) -> i32 {
        match self {
            Priority::Low => 1,
            Priority::Normal => 2,
            Priority::High => 3,
            Priority::Urgent => 4,
        }
    }
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Normal
    }
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Priority::Low => write!(f, "low"),
            Priority::Normal => write!(f, "normal"),
            Priority::High => write!(f, "high"),
            Priority::Urgent => write!(f, "urgent"),
        }
    }
}

impl FromStr for Priority {
    type Err = Error;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" | "l" | "1" => Ok(Priority::Low),
            "normal" | "n" | "2" => Ok(Priority::Normal),
            "high" | "h" | "3" => Ok(Priority::High),
            "urgent" | "u" | "4" | "critical" => Ok(Priority::Urgent),
            _ => Err(Error::InvalidPriority(s.to_string())),
        }
    }
}

/// A task within a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique ID within the project
    pub id: i32,
    
    /// Project this task belongs to
    pub project_id: String,
    
    /// Task title
    pub title: String,
    
    /// Optional description
    pub description: Option<String>,
    
    /// Current status
    pub status: TaskStatus,
    
    /// Priority level
    pub priority: Priority,
    
    /// Due date
    pub due_date: Option<DateTime<Utc>>,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    
    /// Completion timestamp
    pub completed_at: Option<DateTime<Utc>>,
}

impl Task {
    /// Create a new task
    pub fn new(project_id: impl Into<String>, title: impl Into<String>) -> Self {
        let now = Utc::now();
        
        Self {
            id: 0, // Will be set by storage
            project_id: project_id.into(),
            title: title.into(),
            description: None,
            status: TaskStatus::Todo,
            priority: Priority::default(),
            due_date: None,
            created_at: now,
            updated_at: now,
            completed_at: None,
        }
    }
    
    /// Set priority
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }
    
    /// Set due date
    pub fn with_due_date(mut self, due_date: DateTime<Utc>) -> Self {
        self.due_date = Some(due_date);
        self
    }
    
    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
    
    /// Mark as completed
    pub fn complete(&mut self) {
        self.status = TaskStatus::Done;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }
    
    /// Start working on task
    pub fn start(&mut self) {
        self.status = TaskStatus::InProgress;
        self.updated_at = Utc::now();
    }
    
    /// Check if task is overdue
    pub fn is_overdue(&self) -> bool {
        if let Some(due) = self.due_date {
            !self.status.is_completed() && due < Utc::now()
        } else {
            false
        }
    }
    
    /// Check if task is due today
    pub fn is_due_today(&self) -> bool {
        if let Some(due) = self.due_date {
            let today = Utc::now().date_naive();
            due.date_naive() == today
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_task_creation() {
        let task = Task::new("my-project", "Test task");
        assert_eq!(task.title, "Test task");
        assert_eq!(task.status, TaskStatus::Todo);
        assert_eq!(task.priority, Priority::Normal);
    }
    
    #[test]
    fn test_status_from_str() {
        assert_eq!("todo".parse::<TaskStatus>().unwrap(), TaskStatus::Todo);
        assert_eq!("done".parse::<TaskStatus>().unwrap(), TaskStatus::Done);
        assert_eq!("complete".parse::<TaskStatus>().unwrap(), TaskStatus::Done);
    }
    
    #[test]
    fn test_priority_from_str() {
        assert_eq!("high".parse::<Priority>().unwrap(), Priority::High);
        assert_eq!("h".parse::<Priority>().unwrap(), Priority::High);
        assert_eq!("3".parse::<Priority>().unwrap(), Priority::High);
    }
}
