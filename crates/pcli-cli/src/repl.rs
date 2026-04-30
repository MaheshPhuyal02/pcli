//! Interactive REPL mode for pcli with fish-style autocompletion

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::{Hint, Hinter};
use rustyline::history::DefaultHistory;
use rustyline::validate::Validator;
use rustyline::{Context, Editor, Helper};
use std::borrow::Cow;

use pcli_core::Storage;

use crate::{commands, execute_command};

/// Custom helper for rustyline with completion and hints
struct PcliHelper {
    commands: Vec<String>,
    projects: Vec<String>,
}

impl PcliHelper {
    fn new(storage: &Storage) -> Self {
        let commands = vec![
            "list".to_string(),
            "ls".to_string(),
            "projects".to_string(),
            "new".to_string(),
            "current".to_string(),
            "delete".to_string(),
            "tasks".to_string(),
            "add".to_string(),
            "task".to_string(),
            "remind".to_string(),
            "reminders".to_string(),
            "status".to_string(),
            "board".to_string(),
            "help".to_string(),
            "stop".to_string(),
            "open".to_string(),
            "clear".to_string(),
            "exit".to_string(),
            "quit".to_string(),
        ];

        let projects = storage
            .list_projects()
            .unwrap_or_default()
            .into_iter()
            .map(|p| p.id)
            .collect();

        Self { commands, projects }
    }

    fn update_projects(&mut self, storage: &Storage) {
        self.projects = storage
            .list_projects()
            .unwrap_or_default()
            .into_iter()
            .map(|p| p.id)
            .collect();
    }
}

impl Completer for PcliHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let line_up_to_cursor = &line[..pos];
        let words: Vec<&str> = line_up_to_cursor.split_whitespace().collect();

        if words.is_empty() || (words.len() == 1 && !line_up_to_cursor.ends_with(' ')) {
            // Complete commands or project names
            let prefix = words.first().unwrap_or(&"").to_lowercase();
            let mut completions = Vec::new();

            // Add matching commands
            for cmd in &self.commands {
                if cmd.starts_with(&prefix) {
                    completions.push(Pair {
                        display: cmd.clone(),
                        replacement: cmd.clone(),
                    });
                }
            }

            // Add matching project names
            for proj in &self.projects {
                if proj.starts_with(&prefix) {
                    completions.push(Pair {
                        display: format!("{} (project)", proj),
                        replacement: proj.clone(),
                    });
                }
            }

            let start = line_up_to_cursor.len() - prefix.len();
            Ok((start, completions))
        } else {
            // Second word completion based on first command
            let cmd = words[0].to_lowercase();
            let prefix = if line_up_to_cursor.ends_with(' ') {
                ""
            } else {
                words.last().unwrap_or(&"")
            };

            let mut completions = Vec::new();

            match cmd.as_str() {
                "tasks" => {
                    for filter in &[
                        "today",
                        "tomorrow",
                        "week",
                        "overdue",
                        "pending",
                        "active",
                        "done",
                        "cancelled",
                        "status:",
                        "due:",
                        "priority:",
                    ] {
                        if filter.starts_with(prefix) {
                            completions.push(Pair {
                                display: filter.to_string(),
                                replacement: filter.to_string(),
                            });
                        }
                    }
                }
                "task" => {
                    if words.len() >= 2 {
                        // Suggest actions
                        for action in &[
                            "done",
                            "complete",
                            "start",
                            "stop",
                            "delete",
                            "edit",
                            "title",
                            "description",
                            "note",
                            "priority",
                            "due",
                        ] {
                            if action.starts_with(prefix) {
                                completions.push(Pair {
                                    display: action.to_string(),
                                    replacement: action.to_string(),
                                });
                            }
                        }
                    }
                }
                "new" | "delete" => {
                    // Suggest project names for delete
                    if cmd == "delete" {
                        for proj in &self.projects {
                            if proj.starts_with(prefix) {
                                completions.push(Pair {
                                    display: proj.clone(),
                                    replacement: proj.clone(),
                                });
                            }
                        }
                    }
                }
                "add" => {
                    for flag in &["-p", "-d", "--priority", "--due", "--description"] {
                        if flag.starts_with(prefix) {
                            completions.push(Pair {
                                display: flag.to_string(),
                                replacement: flag.to_string(),
                            });
                        }
                    }
                }
                "stop" => {
                    if "-p".starts_with(prefix) {
                         completions.push(Pair {
                            display: "-p".to_string(),
                            replacement: "-p".to_string(),
                        });
                    }
                    if "--port".starts_with(prefix) {
                         completions.push(Pair {
                            display: "--port".to_string(),
                            replacement: "--port".to_string(),
                        });
                    }
                }
                "open" => {
                    for flag in &["-u", "--url", "--chrome", "--safari", "--firefox"] {
                        if flag.starts_with(prefix) {
                            completions.push(Pair {
                                display: flag.to_string(),
                                replacement: flag.to_string(),
                            });
                        }
                    }
                }
                _ => {}
            }

            let start = pos - prefix.len();
            Ok((start, completions))
        }
    }
}

/// Fish-style inline hint
struct CommandHint {
    display: String,
    complete_up_to: usize,
}

impl Hint for CommandHint {
    fn display(&self) -> &str {
        &self.display
    }

    fn completion(&self) -> Option<&str> {
        if self.complete_up_to > 0 {
            Some(&self.display[..self.complete_up_to])
        } else {
            None
        }
    }
}

impl Hinter for PcliHelper {
    type Hint = CommandHint;

    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<Self::Hint> {
        if line.is_empty() || pos < line.len() {
            return None;
        }

        let line_lower = line.to_lowercase();
        let words: Vec<&str> = line_lower.split_whitespace().collect();

        if words.len() == 1 && !line.ends_with(' ') {
            // Hint for first word (command or project)
            let prefix = words[0];

            // Check commands first
            for cmd in &self.commands {
                if cmd.starts_with(prefix) && cmd != prefix {
                    let hint = &cmd[prefix.len()..];
                    return Some(CommandHint {
                        display: hint.to_string(),
                        complete_up_to: hint.len(),
                    });
                }
            }

            // Check project names
            for proj in &self.projects {
                if proj.starts_with(prefix) && proj != prefix {
                    let hint = &proj[prefix.len()..];
                    return Some(CommandHint {
                        display: hint.to_string(),
                        complete_up_to: hint.len(),
                    });
                }
            }
        }

        None
    }
}

impl Highlighter for PcliHelper {
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        // Show hints in dim gray
        Cow::Owned(format!("\x1b[90m{}\x1b[0m", hint))
    }
}

impl Validator for PcliHelper {}
impl Helper for PcliHelper {}

/// Run the interactive REPL
pub fn run(storage: &Storage) -> Result<()> {
    // Clear current project - start fresh each session
    let _ = storage.set_current_project("");

    // Show welcome banner
    print_banner(storage)?;

    // Create helper with autocompletion
    let helper = PcliHelper::new(storage);

    // Create readline editor with helper
    let mut rl: Editor<PcliHelper, DefaultHistory> = Editor::new()?;
    rl.set_helper(Some(helper));

    // History file
    let history_path = pcli_core::data_dir().join("history.txt");
    let _ = rl.load_history(&history_path);

    loop {
        // Get prompt with current project
        let prompt = get_prompt(storage);

        // Update helper with latest projects
        if let Some(helper) = rl.helper_mut() {
            helper.update_projects(storage);
        }

        match rl.readline(&prompt) {
            Ok(line) => {
                let line = line.trim();

                if line.is_empty() {
                    continue;
                }

                // Add to history
                let _ = rl.add_history_entry(line);

                // Parse and execute command
                match parse_and_execute(storage, line) {
                    Ok(should_exit) => {
                        if should_exit {
                            break;
                        }
                    }
                    Err(e) => {
                        println!("{} {}", "Error:".red(), e);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("Use 'exit' or 'quit' to leave");
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                println!("{} {:?}", "Error:".red(), err);
                break;
            }
        }
    }

    // Save history
    let _ = rl.save_history(&history_path);

    println!("{}", "Goodbye! 👋".dimmed());
    Ok(())
}

/// Print welcome banner
fn print_banner(storage: &Storage) -> Result<()> {
    println!();
    println!("{}", r"  ██████╗  ██████╗██╗     ██╗".cyan().bold());
    println!("{}", r"  ██╔══██╗██╔════╝██║     ██║".cyan().bold());
    println!("{}", r"  ██████╔╝██║     ██║     ██║".cyan().bold());
    println!("{}", r"  ██╔═══╝ ██║     ██║     ██║".cyan().bold());
    println!("{}", r"  ██║     ╚██████╗███████╗██║".cyan().bold());
    println!("{}", r"  ╚═╝      ╚═════╝╚══════╝╚═╝".cyan().bold());
    println!();
    println!(
        "  {}   {}",
        "Project CLI · task manager".cyan(),
        "Tab = autocomplete · ? = help".dimmed()
    );
    println!();

    // Show current project status
    commands::status::dashboard(storage)?;
    println!();

    Ok(())
}

/// Get the prompt string
fn get_prompt(storage: &Storage) -> String {
    let project = storage
        .get_current_project()
        .ok()
        .flatten()
        .unwrap_or_default();

    if project.is_empty() {
        format!("{} ", "❯".green())
    } else {
        format!("{} {} ", project.cyan(), "❯".green())
    }
}

/// Parse input and execute command
fn parse_and_execute(storage: &Storage, input: &str) -> Result<bool> {
    // Split input into words, respecting quotes
    let words = match shellwords::split(input) {
        Ok(w) => w,
        Err(_) => {
            println!("{}", "Invalid input (unclosed quotes?)".red());
            return Ok(false);
        }
    };

    if words.is_empty() {
        return Ok(false);
    }

    let cmd = words[0].to_lowercase();

    // Handle special commands
    match cmd.as_str() {
        "exit" | "quit" | "q" => return Ok(true),
        "clear" | "cls" => {
            // Clear screen, clear scrollback buffer, and move cursor to top-left
            // \x1B[2J - Clear entire screen
            // \x1B[3J - Clear scrollback buffer
            // \x1B[H  - Move cursor to home position (1,1)
            print!("\x1B[2J\x1B[3J\x1B[H");
            use std::io::Write;
            let _ = std::io::stdout().flush();
            return Ok(false);
        }
        "help" | "?" => {
            print_help();
            return Ok(false);
        }
        "list" | "ls" => {
            // Alias for projects
            commands::projects::list(storage)?;
            return Ok(false);
        }
        ".." => {
            // Go back / deselect project
            storage.set_current_project("")?;
            return Ok(false);
        }
        _ => {}
    }

    // Try to parse as a full command
    let words_clone = words.clone();
    let mut args = vec!["pcli".to_string()];
    args.extend(words_clone);

    match crate::Cli::try_parse_from(&args) {
        Ok(cli) => {
            if let Some(command) = cli.command {
                execute_command(storage, &command)
            } else if let Some(ref project_name) = cli.project {
                // Switch project
                commands::projects::switch(storage, project_name)?;
                Ok(false)
            } else {
                Ok(false)
            }
        }
        Err(e) => {
            // Check if it might be a project name
            if words.len() == 1 && !cmd.starts_with('-') {
                // Try to switch to project
                if commands::projects::switch(storage, &cmd).is_err() {
                    let err_str = format!("{}", e);
                    println!("{}", err_str.red());
                }
            } else {
                let err_str = format!("{}", e);
                println!("{}", err_str.red());
            }
            Ok(false)
        }
    }
}

/// Print help
fn print_help() {
    println!();
    println!("{}", "📋 pcli Commands".bold());
    println!("{}", "─".repeat(45).dimmed());
    println!();

    println!("{}", "Projects:".bold().underline());
    println!("  {}              List all projects", "list".cyan());
    println!("  {} <name>        Create new project", "new".cyan());
    println!("  {}              Switch to project", "<name>".cyan());
    println!("  {}               Show current project", "current".cyan());
    println!("  {} <name>     Delete project", "delete".cyan());
    println!();

    println!("{}", "Tasks:".bold().underline());
    println!("  {}                List tasks in current project", "tasks".cyan());
    println!("  {} today          Tasks due today", "tasks".cyan());
    println!("  {} tomorrow       Tasks due tomorrow", "tasks".cyan());
    println!("  {} week          Tasks due this week", "tasks".cyan());
    println!("  {} overdue       Tasks past due", "tasks".cyan());
    println!("  {} pending        Pending tasks only", "tasks".cyan());
    println!("  {} active         In-progress tasks only", "tasks".cyan());
    println!("  {} done           Completed tasks only", "tasks".cyan());
    println!("  {} cancelled      Cancelled tasks only", "tasks".cyan());
    println!("  {} \"title\"        Add new task", "add".cyan());
    println!("  {} \"title\" -p high -d tomorrow", "add".cyan());
    println!("  {} <id>            View task details", "task".cyan());
    println!("  {} <id> done       Mark as complete", "task".cyan());
    println!("  {} <id> start      Start working on task", "task".cyan());
    println!("  {} <id> stop       Revert to todo", "task".cyan());
    println!("  {} <id> edit \"new title\"  Rename task", "task".cyan());
    println!("  {} <id> description \"text\"  Update notes", "task".cyan());
    println!("  {} <id> priority high       Change priority", "task".cyan());
    println!("  {} <id> due tomorrow       Set due date", "task".cyan());
    println!("  {} <id> delete     Delete task", "task".cyan());
    println!();

    println!("{}", "Reminders:".bold().underline());
    println!("  {} 10m \"message\"   Remind in 10 minutes", "remind".cyan());
    println!("  {} 2h \"meeting\"    Remind in 2 hours", "remind".cyan());
    println!("  {}             List pending reminders", "reminders".cyan());
    println!();

    println!("{}", "Other:".bold().underline());
    println!("  {}                Show current status", "status".cyan());
    println!("  {}                 Clear screen", "clear".cyan());
    println!("  {} -p <port>      Stop process on port", "stop".cyan());
    println!("  {} -u \"url\" --chrome  Open URL in Chrome", "open".cyan());
    println!("  {} -u \"url\" --safari  Open URL in Safari", "open".cyan());
    println!("  {} -u \"url\" --firefox Open URL in Firefox", "open".cyan());
    println!("  {}                  Show this help", "help".cyan());
    println!("  {}                  Exit pcli", "exit".cyan());
    println!();
    println!("{}", "💡 Press Tab for autocomplete!".dimmed());
    println!();
}
