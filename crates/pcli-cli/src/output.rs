//! Output formatting utilities

use colored::{ColoredString, Colorize};
use pcli_core::Task;

const ID_W: usize = 4;
const STATUS_W: usize = 14;
const PRIORITY_W: usize = 10;
const DUE_W: usize = 10;
const TITLE_MIN: usize = 30;

fn title_width() -> usize {
    let cols = crossterm::terminal::size().map(|(c, _)| c as usize).unwrap_or(80);
    let overhead = ID_W + STATUS_W + PRIORITY_W + DUE_W + 6;
    cols.saturating_sub(overhead).max(TITLE_MIN)
}

fn truncate(s: &str, max_len: usize) -> String {
    let count = s.chars().count();
    if count <= max_len {
        return s.to_string();
    }
    let keep = max_len.saturating_sub(1);
    let mut out: String = s.chars().take(keep).collect();
    out.push('…');
    out
}

fn pad(s: &str, width: usize) -> String {
    let visible = s.chars().count();
    if visible >= width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - visible))
    }
}

/// Print tasks as a formatted table
pub fn print_tasks_table(tasks: &[Task]) {
    let title_w = title_width();
    let has_description = tasks.iter().any(|t| t.description.is_some());

    let bar = |w: usize| "─".repeat(w);
    println!(
        "┌{}┬{}┬{}┬{}┬{}┐",
        bar(ID_W), bar(title_w + 2), bar(STATUS_W), bar(PRIORITY_W), bar(DUE_W)
    );
    println!(
        "│ {} │ {} │ {} │ {} │ {} │",
        pad("#", 2).bold(),
        pad("Title", title_w).bold(),
        pad("Status", STATUS_W - 2).bold(),
        pad("Priority", PRIORITY_W - 2).bold(),
        pad("Due", DUE_W - 2).bold(),
    );
    println!(
        "├{}┼{}┼{}┼{}┼{}┤",
        bar(ID_W), bar(title_w + 2), bar(STATUS_W), bar(PRIORITY_W), bar(DUE_W)
    );

    for task in tasks {
        let status_plain = format!("{} {}", task.status.emoji(), task.status);
        let status_padded = pad(&status_plain, STATUS_W - 2);
        let status_cell: ColoredString = match task.status {
            pcli_core::TaskStatus::Done => status_padded.green(),
            pcli_core::TaskStatus::InProgress => status_padded.cyan(),
            pcli_core::TaskStatus::Cancelled => status_padded.red(),
            _ => status_padded.normal(),
        };

        let priority_plain = format!("{} {}", task.priority.emoji(), task.priority);

        let due_plain = task
            .due_date
            .map(|d| d.format("%b %d").to_string())
            .unwrap_or_else(|| "-".to_string());
        let due_padded = pad(&due_plain, DUE_W - 2);
        let due_cell: ColoredString = if task.is_overdue() {
            due_padded.red()
        } else if task.is_due_today() {
            due_padded.yellow()
        } else {
            due_padded.normal()
        };

        let mark = if task.description.is_some() { "✎" } else { " " };
        let title_cell = format!("{} {}", mark, truncate(&task.title, title_w.saturating_sub(2)));

        println!(
            "│ {} │ {} │ {} │ {} │ {} │",
            pad(&task.id.to_string(), 2),
            pad(&title_cell, title_w),
            status_cell,
            pad(&priority_plain, PRIORITY_W - 2),
            due_cell,
        );
    }

    println!(
        "└{}┴{}┴{}┴{}┴{}┘",
        bar(ID_W), bar(title_w + 2), bar(STATUS_W), bar(PRIORITY_W), bar(DUE_W)
    );

    if has_description {
        println!(
            "{}",
            "  ✎ = has description · run `pcli task <id>` for full details".dimmed()
        );
    }
}
