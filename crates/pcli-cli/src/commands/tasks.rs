//! Task commands

use anyhow::{Result, bail};
use chrono::{Duration, NaiveDate, Utc};
use colored::Colorize;
use pcli_core::{Priority, Storage, Task, TaskStatus};
use std::str::FromStr;

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

    let tasks = if let Some(filter) = filter {
        get_filtered_tasks(storage, &project_id, filter)?
    } else {
        storage.list_tasks(&project_id, None)?
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

fn get_filtered_tasks(storage: &Storage, project_id: &str, filter: &str) -> Result<Vec<Task>> {
    let normalized = filter.to_lowercase();

    if let Some((key, value)) = normalized.split_once(':') {
        return match key {
            "status" => get_status_tasks(storage, project_id, value),
            "due" => get_due_tasks(storage, project_id, value),
            "priority" => get_priority_tasks(storage, project_id, value),
            _ => bail!("Unknown task filter: {}", filter),
        };
    }

    match normalized.as_str() {
        "pending" | "todo" | "t" => get_status_tasks(storage, project_id, "todo"),
        "active" | "inprogress" | "in_progress" | "progress" | "a" => {
            get_status_tasks(storage, project_id, "in_progress")
        }
        "done" | "d" | "complete" | "completed" | "c" => get_status_tasks(storage, project_id, "done"),
        "cancelled" | "cancel" | "x" => get_status_tasks(storage, project_id, "cancelled"),
        "today" => get_due_tasks(storage, project_id, "today"),
        "tomorrow" | "tm" => get_due_tasks(storage, project_id, "tomorrow"),
        "week" | "nextweek" | "w" => get_due_tasks(storage, project_id, "week"),
        "overdue" | "o" => get_due_tasks(storage, project_id, "overdue"),
        _ => bail!("Unknown task filter: {}", filter),
    }
}

fn get_status_tasks(storage: &Storage, project_id: &str, value: &str) -> Result<Vec<Task>> {
    match TaskStatus::from_str(value) {
        Ok(status) => Ok(storage.list_tasks(project_id, Some(status))?),
        Err(_) => bail!("Unknown task status: {}", value),
    }
}

fn get_priority_tasks(storage: &Storage, project_id: &str, value: &str) -> Result<Vec<Task>> {
    let priority = match value.parse::<Priority>() {
        Ok(priority) => priority,
        Err(_) => bail!("Unknown task priority: {}", value),
    };

    let tasks = storage.list_tasks(project_id, None)?;
    Ok(tasks.into_iter().filter(|task| task.priority == priority).collect())
}

fn get_due_tasks(storage: &Storage, project_id: &str, value: &str) -> Result<Vec<Task>> {
    let tasks = storage.list_tasks(project_id, None)?;
    let today = Utc::now().date_naive();

    Ok(match value {
        "today" => tasks.into_iter().filter(|t| t.is_due_today()).collect(),
        "tomorrow" | "tm" => tasks
            .into_iter()
            .filter(|t| {
                t.due_date
                    .map(|due| due.date_naive() == today + Duration::days(1))
                    .unwrap_or(false)
            })
            .collect(),
        "week" | "nextweek" | "w" => tasks
            .into_iter()
            .filter(|t| {
                t.due_date.map(|due| {
                    let due_date = due.date_naive();
                    due_date >= today && due_date <= today + Duration::days(7)
                })
                .unwrap_or(false)
            })
            .collect(),
        "overdue" | "o" => tasks.into_iter().filter(|t| t.is_overdue()).collect(),
        _ => bail!("Unknown due filter: {}", value),
    })
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

    let mut task = storage.get_task(id)?.ok_or_else(|| anyhow::anyhow!("Task #{} not found", id))?;

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
                task.priority = priority;
                task.updated_at = Utc::now();
                storage.update_task(&task)?;
                println!("{} Priority updated: #{} {}", "✓".green(), id, priority);
            } else {
                bail!("Usage: pcli task {} priority <low|normal|high|urgent>", id);
            }
        }
        Some("due") => {
            if let Some(date) = value {
                task.due_date = Some(parse_due_date(date)?);
                task.updated_at = Utc::now();
                storage.update_task(&task)?;
                println!("{} Due date updated: #{} {}", "✓".green(), id, task.due_date.unwrap().format("%Y-%m-%d"));
            } else {
                bail!("Usage: pcli task {} due <today|tomorrow|week|YYYY-MM-DD>", id);
            }
        }
        Some("description") | Some("desc") | Some("note") => {
            if let Some(desc) = value {
                task.description = Some(desc.to_string());
                task.updated_at = Utc::now();
                storage.update_task(&task)?;
                println!("{} Description updated for task #{}", "✓".green(), id);
            } else {
                bail!("Usage: pcli task {} description <text>", id);
            }
        }
        Some("edit") | Some("title") => {
            if let Some(title) = value {
                task.title = title.to_string();
                task.updated_at = Utc::now();
                storage.update_task(&task)?;
                println!("{} Title updated: #{} {}", "✓".green(), id, task.title);
            } else {
                bail!("Usage: pcli task {} edit <new title>", id);
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
