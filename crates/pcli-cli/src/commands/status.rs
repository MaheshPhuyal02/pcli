//! Status and dashboard commands

use anyhow::Result;
use colored::Colorize;
use pcli_core::{Storage, TaskStatus};

/// Show current status
pub fn show(storage: &Storage) -> Result<()> {
    match storage.get_current_project()? {
        Some(project_id) => {
            if let Some(project) = storage.get_project(&project_id)? {
                let tasks = storage.list_tasks(&project_id, Some(TaskStatus::InProgress))?;

                if tasks.is_empty() {
                    println!("{}", "No active task.".dimmed());
                } else {
                    let task = &tasks[0];
                    println!("{} Working on #{}: {}", "▶".green(), task.id, task.title);
                }
            }
        }
        None => {
            println!("{}", "No active project.".dimmed());
        }
    }

    Ok(())
}

/// Show dashboard
pub fn dashboard(storage: &Storage) -> Result<()> {
    match storage.get_current_project()? {
        Some(project_id) => {
            if let Some(project) = storage.get_project(&project_id)? {
                let (done, total) = storage.count_tasks(&project_id)?;
                let pending = total - done;

                // Header
                println!("╭───────────────────────────────────────╮");
                println!("│  {} {:<32}│", "📁".to_string(), project.name.cyan().bold());
                println!("├───────────────────────────────────────┤");

                // Stats
                println!(
                    "│  {} {} done  {} {} pending               │",
                    "✓".green(),
                    done,
                    "○".yellow(),
                    pending
                );
                println!("├───────────────────────────────────────┤");

                // Active tasks
                let active = storage.list_tasks(&project_id, Some(TaskStatus::InProgress))?;
                if !active.is_empty() {
                    println!("│  {}                              │", "Active:".bold());
                    for task in active.iter().take(3) {
                        println!(
                            "│   {} #{} {:<27}│",
                            "▶".green(),
                            task.id,
                            truncate(&task.title, 27)
                        );
                    }
                    println!("├───────────────────────────────────────┤");
                }

                // Pending tasks
                let pending_tasks = storage.list_tasks(&project_id, Some(TaskStatus::Todo))?;
                if !pending_tasks.is_empty() {
                    println!("│  {}                            │", "Pending:".bold());
                    for task in pending_tasks.iter().take(5) {
                        let due = task
                            .due_date
                            .map(|d| format!(" [{}]", d.format("%b %d")))
                            .unwrap_or_default();
                        println!(
                            "│   {} #{} {:<23}{}│",
                            "○".dimmed(),
                            task.id,
                            truncate(&task.title, 23),
                            due.yellow()
                        );
                    }
                    println!("├───────────────────────────────────────┤");
                }

                // Reminders
                let reminders = storage.list_pending_reminders()?;
                if !reminders.is_empty() {
                    println!("│  {}                          │", "⏰ Reminders:".bold());
                    for reminder in reminders.iter().take(3) {
                        println!(
                            "│   • \"{}\" in {}      │",
                            truncate(&reminder.message, 20),
                            reminder.time_remaining_str().yellow()
                        );
                    }
                }

                println!("╰───────────────────────────────────────╯");
            }
        }
        None => {
            println!("╭───────────────────────────────────────╮");
            println!("│  {} {}                     │", "📋".to_string(), "pcli".cyan().bold());
            println!("├───────────────────────────────────────┤");
            println!("│  {}              │", "No active project.".dimmed());
            println!("│                                       │");
            println!("│  Get started:                         │");
            println!("│    {} Create project   │", "pcli new <name>".cyan());
            println!("│    {}  Switch project   │", "pcli <name>".cyan());
            println!("│    {}  List projects    │", "pcli projects".cyan());
            println!("╰───────────────────────────────────────╯");
        }
    }

    Ok(())
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len - 3])
    } else {
        s.to_string()
    }
}
