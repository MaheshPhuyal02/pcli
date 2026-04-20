//! Time log model for tracking time spent on tasks

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A time log entry for a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeLog {
    /// Unique ID
    pub id: i32,
    
    /// Task this log belongs to
    pub task_id: i32,
    
    /// Project this log belongs to
    pub project_id: String,
    
    /// When the timer started
    pub started_at: DateTime<Utc>,
    
    /// When the timer ended (None if still running)
    pub ended_at: Option<DateTime<Utc>>,
    
    /// Duration in seconds (calculated when ended)
    pub duration_seconds: Option<i64>,
    
    /// Whether this was a pomodoro session
    pub is_pomodoro: bool,
}

impl TimeLog {
    /// Create a new time log entry (starts now)
    pub fn start(project_id: impl Into<String>, task_id: i32) -> Self {
        Self {
            id: 0,
            task_id,
            project_id: project_id.into(),
            started_at: Utc::now(),
            ended_at: None,
            duration_seconds: None,
            is_pomodoro: false,
        }
    }
    
    /// Create a pomodoro session
    pub fn pomodoro(project_id: impl Into<String>, task_id: i32) -> Self {
        let mut log = Self::start(project_id, task_id);
        log.is_pomodoro = true;
        log
    }
    
    /// Stop the timer
    pub fn stop(&mut self) {
        let now = Utc::now();
        self.ended_at = Some(now);
        self.duration_seconds = Some((now - self.started_at).num_seconds());
    }
    
    /// Check if timer is running
    pub fn is_running(&self) -> bool {
        self.ended_at.is_none()
    }
    
    /// Get elapsed time
    pub fn elapsed(&self) -> chrono::Duration {
        match self.ended_at {
            Some(end) => end - self.started_at,
            None => Utc::now() - self.started_at,
        }
    }
    
    /// Get elapsed time as seconds
    pub fn elapsed_seconds(&self) -> i64 {
        self.elapsed().num_seconds()
    }
    
    /// Format elapsed time as HH:MM:SS
    pub fn elapsed_str(&self) -> String {
        let total_secs = self.elapsed_seconds();
        let hours = total_secs / 3600;
        let mins = (total_secs % 3600) / 60;
        let secs = total_secs % 60;
        format!("{:02}:{:02}:{:02}", hours, mins, secs)
    }
    
    /// Format duration as human-readable string
    pub fn duration_human(&self) -> String {
        let total_secs = self.elapsed_seconds();
        if total_secs < 60 {
            format!("{}s", total_secs)
        } else if total_secs < 3600 {
            format!("{}m {}s", total_secs / 60, total_secs % 60)
        } else {
            let hours = total_secs / 3600;
            let mins = (total_secs % 3600) / 60;
            format!("{}h {}m", hours, mins)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_time_log_creation() {
        let log = TimeLog::start("my-project", 1);
        assert!(log.is_running());
        assert_eq!(log.task_id, 1);
    }
    
    #[test]
    fn test_elapsed_str() {
        let mut log = TimeLog::start("my-project", 1);
        log.started_at = Utc::now() - chrono::Duration::seconds(3661);
        assert_eq!(log.elapsed_str(), "01:01:01");
    }
}
