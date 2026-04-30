//! SQLite storage implementation

use chrono::{DateTime, TimeZone, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

use crate::error::{Error, Result};
use crate::models::*;

/// SQLite storage backend
pub struct Storage {
    conn: Connection,
}

impl Storage {
    /// Open or create database
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        let storage = Self { conn };
        storage.init()?;
        Ok(storage)
    }

    /// Initialize database schema
    fn init(&self) -> Result<()> {
        self.conn.execute_batch(include_str!("schema.sql"))?;
        Ok(())
    }

    // === App State ===

    /// Get current active project
    pub fn get_current_project(&self) -> Result<Option<String>> {
        let result: Option<String> = self
            .conn
            .query_row(
                "SELECT value FROM app_state WHERE key = 'current_project'",
                [],
                |row| row.get(0),
            )
            .optional()?;
        Ok(result)
    }

    /// Set current active project
    pub fn set_current_project(&self, project_id: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO app_state (key, value) VALUES ('current_project', ?1)",
            [project_id],
        )?;
        Ok(())
    }

    // === Projects ===

    /// Create a new project
    pub fn create_project(&self, project: &Project) -> Result<()> {
        self.conn.execute(
            "INSERT INTO projects (id, name, description, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                project.id,
                project.name,
                project.description,
                project.created_at.to_rfc3339(),
                project.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// Get a project by ID
    pub fn get_project(&self, id: &str) -> Result<Option<Project>> {
        let result = self
            .conn
            .query_row(
                "SELECT id, name, description, created_at, updated_at FROM projects WHERE id = ?1",
                [id],
                |row| {
                    Ok(Project {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        description: row.get(2)?,
                        created_at: parse_datetime(row.get::<_, String>(3)?),
                        updated_at: parse_datetime(row.get::<_, String>(4)?),
                    })
                },
            )
            .optional()?;
        Ok(result)
    }

    /// List all projects
    pub fn list_projects(&self) -> Result<Vec<Project>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, description, created_at, updated_at FROM projects ORDER BY name")?;
        let rows = stmt.query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                created_at: parse_datetime(row.get::<_, String>(3)?),
                updated_at: parse_datetime(row.get::<_, String>(4)?),
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// Delete a project
    pub fn delete_project(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM projects WHERE id = ?1", [id])?;
        Ok(())
    }

    /// Count tasks for a project
    pub fn count_tasks(&self, project_id: &str) -> Result<(i32, i32)> {
        let total: i32 = self.conn.query_row(
            "SELECT COUNT(*) FROM tasks WHERE project_id = ?1",
            [project_id],
            |row| row.get(0),
        )?;
        let done: i32 = self.conn.query_row(
            "SELECT COUNT(*) FROM tasks WHERE project_id = ?1 AND status = 'done'",
            [project_id],
            |row| row.get(0),
        )?;
        Ok((done, total))
    }

    // === Tasks ===

    /// Create a new task
    pub fn create_task(&self, task: &Task) -> Result<i32> {
        self.conn.execute(
            "INSERT INTO tasks (project_id, title, description, status, priority, due_date, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                task.project_id,
                task.title,
                task.description,
                task.status.to_string(),
                task.priority.to_string(),
                task.due_date.map(|d| d.to_rfc3339()),
                task.created_at.to_rfc3339(),
                task.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(self.conn.last_insert_rowid() as i32)
    }

    /// Get a task by ID
    pub fn get_task(&self, id: i32) -> Result<Option<Task>> {
        let result = self
            .conn
            .query_row(
                "SELECT id, project_id, title, description, status, priority, due_date, created_at, updated_at, completed_at
                 FROM tasks WHERE id = ?1",
                [id],
                |row| Ok(row_to_task(row)),
            )
            .optional()?;
        Ok(result)
    }

    /// List tasks for a project
    pub fn list_tasks(&self, project_id: &str, status: Option<TaskStatus>) -> Result<Vec<Task>> {
        let sql = match status {
            Some(s) => format!(
                "SELECT id, project_id, title, description, status, priority, due_date, created_at, updated_at, completed_at
                 FROM tasks WHERE project_id = ?1 AND status = '{}' ORDER BY priority DESC, id",
                s
            ),
            None => "SELECT id, project_id, title, description, status, priority, due_date, created_at, updated_at, completed_at
                     FROM tasks WHERE project_id = ?1 ORDER BY status, priority DESC, id".to_string(),
        };
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map([project_id], |row| Ok(row_to_task(row)))?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// Update task status
    pub fn update_task_status(&self, id: i32, status: TaskStatus) -> Result<()> {
        let completed_at = if status == TaskStatus::Done {
            Some(Utc::now().to_rfc3339())
        } else {
            None
        };
        self.conn.execute(
            "UPDATE tasks SET status = ?1, updated_at = ?2, completed_at = ?3 WHERE id = ?4",
            params![status.to_string(), Utc::now().to_rfc3339(), completed_at, id],
        )?;
        Ok(())
    }

    /// Update a task's editable fields
    pub fn update_task(&self, task: &Task) -> Result<()> {
        self.conn.execute(
            "UPDATE tasks SET title = ?1, description = ?2, status = ?3, priority = ?4, due_date = ?5, updated_at = ?6, completed_at = ?7 WHERE id = ?8",
            params![
                task.title,
                task.description,
                task.status.to_string(),
                task.priority.to_string(),
                task.due_date.map(|d| d.to_rfc3339()),
                task.updated_at.to_rfc3339(),
                task.completed_at.map(|d| d.to_rfc3339()),
                task.id,
            ],
        )?;
        Ok(())
    }

    /// Delete a task
    pub fn delete_task(&self, id: i32) -> Result<()> {
        self.conn.execute("DELETE FROM tasks WHERE id = ?1", [id])?;
        Ok(())
    }

    // === Reminders ===

    /// Create a reminder
    pub fn create_reminder(&self, reminder: &Reminder) -> Result<i32> {
        self.conn.execute(
            "INSERT INTO reminders (message, project_id, task_id, remind_at, status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                reminder.message,
                reminder.project_id,
                reminder.task_id,
                reminder.remind_at.to_rfc3339(),
                reminder.status.to_string(),
                reminder.created_at.to_rfc3339(),
            ],
        )?;
        Ok(self.conn.last_insert_rowid() as i32)
    }

    /// List pending reminders
    pub fn list_pending_reminders(&self) -> Result<Vec<Reminder>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, message, project_id, task_id, remind_at, status, created_at
             FROM reminders WHERE status = 'pending' ORDER BY remind_at",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Reminder {
                id: row.get(0)?,
                message: row.get(1)?,
                project_id: row.get(2)?,
                task_id: row.get(3)?,
                remind_at: parse_datetime(row.get::<_, String>(4)?),
                status: ReminderStatus::Pending,
                created_at: parse_datetime(row.get::<_, String>(6)?),
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// Update reminder status
    pub fn update_reminder_status(&self, id: i32, status: ReminderStatus) -> Result<()> {
        self.conn.execute(
            "UPDATE reminders SET status = ?1 WHERE id = ?2",
            params![status.to_string(), id],
        )?;
        Ok(())
    }
}

fn parse_datetime(s: String) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(&s)
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

fn row_to_task(row: &rusqlite::Row) -> Task {
    Task {
        id: row.get(0).unwrap_or(0),
        project_id: row.get(1).unwrap_or_default(),
        title: row.get(2).unwrap_or_default(),
        description: row.get(3).ok(),
        status: row.get::<_, String>(4).ok().and_then(|s| s.parse().ok()).unwrap_or(TaskStatus::Todo),
        priority: row.get::<_, String>(5).ok().and_then(|s| s.parse().ok()).unwrap_or(Priority::Normal),
        due_date: row.get::<_, Option<String>>(6).ok().flatten().map(parse_datetime),
        created_at: row.get::<_, String>(7).ok().map(parse_datetime).unwrap_or_else(Utc::now),
        updated_at: row.get::<_, String>(8).ok().map(parse_datetime).unwrap_or_else(Utc::now),
        completed_at: row.get::<_, Option<String>>(9).ok().flatten().map(parse_datetime),
    }
}
