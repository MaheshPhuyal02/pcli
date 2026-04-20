//! pcli - Project CLI
//!
//! A developer-focused task management CLI tool with interactive REPL mode.

mod commands;
mod output;
mod repl;
mod daemon_helper;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

use pcli_core::{Storage, db_path, init};

#[derive(Parser)]
#[command(name = "pcli")]
#[command(author, version, about = "Project CLI - Developer task management", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Switch to or show project by name
    #[arg(value_name = "PROJECT")]
    pub project: Option<String>,
}

#[derive(Subcommand, Clone)]
pub enum Commands {
    /// List all projects
    Projects,

    /// Create a new project
    New {
        /// Project name
        name: String,
    },

    /// Show current project
    Current,

    /// Delete a project
    Delete {
        /// Project name
        name: String,
    },

    /// List tasks
    Tasks {
        /// Filter: today, pending, done
        #[arg(value_name = "FILTER")]
        filter: Option<String>,
    },

    /// Add a new task
    Add {
        /// Task title
        title: String,

        /// Longer description / notes
        #[arg(long)]
        description: Option<String>,

        /// Priority (low, normal, high, urgent)
        #[arg(short, long)]
        priority: Option<String>,

        /// Due date
        #[arg(short, long)]
        due: Option<String>,
    },

    /// Work with a specific task
    Task {
        /// Task ID
        id: i32,

        /// Action: done, complete, start, stop, edit, delete, priority, due, note
        #[arg(value_name = "ACTION")]
        action: Option<String>,

        /// Value for action (e.g., priority level)
        #[arg(value_name = "VALUE")]
        value: Option<String>,
    },

    /// Set a reminder
    Remind {
        /// Duration (e.g., 10m, 2h, 1d)
        duration: String,

        /// Reminder message
        message: String,
    },

    /// List reminders
    Reminders,

    /// Show status (current timer, active task)
    Status,

    /// Interactive Kanban board
    Board,

    /// Exit interactive mode
    Exit,

    /// Quit interactive mode
    Quit,

    /// Clear screen
    Clear,

    /// Stop process on port
    Stop {
        /// Port number to kill process on
        #[arg(short, long)]
        port: u16,
    },

    /// Open URL in browser
    Open {
        /// URL to open
        #[arg(short, long)]
        url: String,

        /// Open in Google Chrome
        #[arg(long = "chrome")]
        chrome: bool,

        /// Open in Safari
        #[arg(long = "safari")]
        safari: bool,

        /// Open in Firefox
        #[arg(long = "firefox")]
        firefox: bool,
    },

    /// Show help
    ReplHelp,
}

fn main() -> Result<()> {
    // Initialize app directories
    init()?;

    // Ensure daemon is running
    if let Err(e) = daemon_helper::ensure_daemon_running() {
        eprintln!("Warning: Failed to start background daemon: {}", e);
    }

    // Open database
    let storage = Storage::open(&db_path())?;

    // Parse CLI
    let cli = Cli::parse();

    // Handle commands
    match cli.command {
        Some(Commands::Projects) => commands::projects::list(&storage)?,
        Some(Commands::New { name }) => commands::projects::create(&storage, &name)?,
        Some(Commands::Current) => commands::projects::current(&storage)?,
        Some(Commands::Delete { name }) => commands::projects::delete(&storage, &name)?,

        Some(Commands::Tasks { filter }) => commands::tasks::list(&storage, filter.as_deref())?,
        Some(Commands::Add { title, description, priority, due }) => commands::tasks::add(
            &storage,
            &title,
            description.as_deref(),
            priority.as_deref(),
            due.as_deref(),
        )?,
        Some(Commands::Task { id, action, value }) => {
            commands::tasks::task(&storage, id, action.as_deref(), value.as_deref())?
        }

        Some(Commands::Remind { duration, message }) => {
            commands::remind::create(&storage, &duration, &message)?
        }
        Some(Commands::Reminders) => commands::remind::list(&storage)?,

        Some(Commands::Status) => commands::status::show(&storage)?,
        Some(Commands::Board) => {
            println!("{}", "TUI board coming soon!".yellow());
        }

        Some(Commands::Exit) | Some(Commands::Quit) => {}
        Some(Commands::Clear) => {}
        Some(Commands::Stop { port }) => {
            commands::system::kill_process_on_port(port)?;
        }
        Some(Commands::Open { url, chrome, safari, firefox }) => {
            let browser = if chrome {
                commands::open::Browser::Chrome
            } else if safari {
                commands::open::Browser::Safari
            } else if firefox {
                commands::open::Browser::Firefox
            } else {
                commands::open::Browser::Default
            };
            commands::open::open_url(&url, browser)?;
        }
        Some(Commands::ReplHelp) => {
            Cli::parse_from(["pcli", "--help"]);
        }

        None => {
            // Enter interactive REPL mode
            // Note: We intentionally ignore the `project` argument here to prevent
            // auto-switching behavior, as per user requirement.
            repl::run(&storage)?;
        }
    }

    Ok(())
}

/// Execute a command (used by both main and REPL)
pub fn execute_command(storage: &Storage, cmd: &Commands) -> Result<bool> {
    match cmd {
        Commands::Projects => commands::projects::list(storage)?,
        Commands::New { name } => commands::projects::create(storage, name)?,
        Commands::Current => commands::projects::current(storage)?,
        Commands::Delete { name } => commands::projects::delete(storage, name)?,

        Commands::Tasks { filter } => commands::tasks::list(storage, filter.as_deref())?,
        Commands::Add { title, description, priority, due } => commands::tasks::add(
            storage,
            title,
            description.as_deref(),
            priority.as_deref(),
            due.as_deref(),
        )?,
        Commands::Task { id, action, value } => {
            commands::tasks::task(storage, *id, action.as_deref(), value.as_deref())?
        }

        Commands::Remind { duration, message } => {
            commands::remind::create(storage, duration, message)?
        }
        Commands::Reminders => commands::remind::list(storage)?,

        Commands::Status => commands::status::show(storage)?,
        Commands::Board => {
            println!("{}", "TUI board coming soon!".yellow());
        }

        Commands::Exit | Commands::Quit => return Ok(true), // Signal to exit
        Commands::Clear => print!("\x1B[2J\x1B[1;1H"), // Clear screen
        Commands::Stop { port } => {
            commands::system::kill_process_on_port(*port)?;
        }
        Commands::Open { url, chrome, safari, firefox } => {
            let browser = if *chrome {
                commands::open::Browser::Chrome
            } else if *safari {
                commands::open::Browser::Safari
            } else if *firefox {
                commands::open::Browser::Firefox
            } else {
                commands::open::Browser::Default
            };
            commands::open::open_url(url, browser)?;
        }
        Commands::ReplHelp => {
            println!("{}", "Available commands:".bold());
            println!("  {}           List all projects", "projects".cyan());
            println!("  {} <name>     Create new project", "new".cyan());
            println!("  {} <name>        Switch to project", "<project>".cyan());
            println!("  {}            Show current project", "current".cyan());
            println!("  {}        List tasks", "tasks".cyan());
            println!("  {} \"title\"   Add new task", "add".cyan());
            println!("  {} <id> done    Complete task", "task".cyan());
            println!("  {} 10m \"msg\"  Set reminder", "remind".cyan());
            println!("  {}          List reminders", "reminders".cyan());
            println!("  {}             Clear screen", "clear".cyan());
            println!("  {} -p <port>  Stop process on port", "stop".cyan());
            println!("  {}        Exit pcli", "exit/quit".cyan());
        }
    }
    Ok(false)
}
