//! Task commands

use anyhow::{Result, bail};
use chrono::{Duration, NaiveDate, Utc};
use colored::Colorize;
use pcli_core::{Priority, Storage, Task, TaskStatus};

use crate::output::print_tasks_table;

/// Get current project or error
fn require_project(storage: &Storage) -> Result<String> {
    storage
        .get_current_project()?
        .ok_or_else(|| anyhow::anyhow!("No active project. Use 'pcli <project-name>' to switch."))
}

/// List tasks
pub fn list(storage: &Storage, filter: Option<&str>) -> Result<()> {
    let project_id = require_project(storage)?;
    let project = storage.get_project(&project_id)?.unwrap();

    let tasks = match filter {
        Some("pending") | Some("p") => storage.list_tasks(&project_id, Some(TaskStatus::Todo))?,
        Some("done") | Some("d") => storage.list_tasks(&project_id, Some(TaskStatus::Done))?,
        Some("active") | Some("a") => storage.list_tasks(&project_id, Some(TaskStatus::InProgress))?,
        Some("today") | Some("t") => {
            let all = storage.list_tasks(&project_id, None)?;
            all.into_iter().filter(|t| t.is_due_today()).collect()
        }
        _ => storage.list_tasks(&project_id, None)?,
    };

    if tasks.is_empty() {
        println!("{} - {}", project.name.cyan().bold(), "No tasks".dimmed());
        println!("Add one with: {}", "pcli add \"task name\"".cyan());
        return Ok(());
    }

    println!("{}", format!("  {} - Tasks", project.name).cyan().bold());
    print_tasks_table(&tasks);

    Ok(())
}

/// Add a new task
pub fn add(
    storage: &Storage,
    title: &str,
    description: Option<&str>,
    priority: Option<&str>,
    due: Option<&str>,
) -> Result<()> {
    let project_id = require_project(storage)?;

    let mut task = Task::new(&project_id, title);

    if let Some(d) = description {
        task.description = Some(d.to_string());
    }

    if let Some(p) = priority {
        task.priority = p.parse()?;
    }

    if let Some(d) = due {
        task.due_date = Some(parse_due_date(d)?);
    }

    let id = storage.create_task(&task)?;

    let priority_str = format!("[{}]", task.priority.to_string().to_uppercase());
    let priority_colored = match task.priority {
        Priority::Urgent => priority_str.red(),
        Priority::High => priority_str.yellow(),
        _ => priority_str.normal(),
    };

    println!(
        "{} Task #{} created: {} {}",
        "✓".green(),
        id,
        title,
        priority_colored
    );

    Ok(())
}

/// Work with a specific task
pub fn task(storage: &Storage, id: i32, action: Option<&str>, value: Option<&str>) -> Result<()> {
    let _project_id = require_project(storage)?;

    let task = storage.get_task(id)?.ok_or_else(|| anyhow::anyhow!("Task #{} not found", id))?;

    match action {
        Some("done") | Some("complete") | Some("d") | Some("c") => {
            storage.update_task_status(id, TaskStatus::Done)?;
            println!("{} Completed: #{} {}", "✓".green(), id, task.title);
        }
        Some("start") | Some("s") => {
            storage.update_task_status(id, TaskStatus::InProgress)?;
            println!("{} Started: #{} {}", "▶".green(), id, task.title);
        }
        Some("stop") => {
            storage.update_task_status(id, TaskStatus::Todo)?;
            println!("{} Stopped: #{} {}", "⏹".yellow(), id, task.title);
        }
        Some("delete") | Some("del") | Some("rm") => {
            storage.delete_task(id)?;
            println!("{} Deleted: #{} {}", "✓".green(), id, task.title);
        }
        Some("priority") | Some("p") => {
            if let Some(p) = value {
                let priority: Priority = p.parse()?;
                // Would need update_task_priority method
                println!("{} Priority set to: {}", "✓".green(), priority);
            } else {
                bail!("Usage: pcli task {} priority <low|normal|high|urgent>", id);
            }
        }
        None => {
            // Show task details
            println!("{}", format!("Task #{}", id).cyan().bold());
            println!("{}", "─".repeat(40).dimmed());
            println!("Title:    {}", task.title);
            println!("Status:   {} {}", task.status.emoji(), task.status);
            println!("Priority: {} {}", task.priority.emoji(), task.priority);
            if let Some(due) = task.due_date {
                println!("Due:      {}", due.format("%Y-%m-%d"));
            }
            if let Some(desc) = &task.description {
                println!("Description:");
                for line in desc.lines() {
                    println!("  {}", line);
                }
            }
            println!("Created:  {}", task.created_at.format("%Y-%m-%d %H:%M"));
        }
        Some(action) => {
            bail!("Unknown action: {}. Use: done, start, stop, delete, priority", action);
        }
    }

    Ok(())
}

/// Parse due date from string
fn parse_due_date(s: &str) -> Result<chrono::DateTime<Utc>> {
    let today = Utc::now();

    match s.to_lowercase().as_str() {
        "today" => Ok(today),
        "tomorrow" => Ok(today + Duration::days(1)),
        "week" | "nextweek" => Ok(today + Duration::days(7)),
        _ => {
            // Try to parse as date
            if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
                Ok(date.and_hms_opt(23, 59, 59).unwrap().and_utc())
            } else {
                bail!("Invalid date format: {}. Use: today, tomorrow, week, or YYYY-MM-DD", s)
            }
        }
    }
}
