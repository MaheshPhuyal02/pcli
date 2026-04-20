use anyhow::{Context, Result};
use colored::*;
use std::process::Command;

/// Kill the process running on the specified port
pub fn kill_process_on_port(port: u16) -> Result<()> {
    // 1. Find the PID using lsof -ti :<port>
    let output = Command::new("lsof")
        .arg("-ti")
        .arg(format!(":{}", port))
        .output()
        .context("Failed to execute lsof command. Make sure lsof is installed.")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let pids: Vec<&str> = stdout.trim().split('\n').filter(|s| !s.is_empty()).collect();

    if pids.is_empty() {
        println!("{}", format!("No process found running on port {}", port).yellow());
        return Ok(());
    }

    // 2. Kill each PID
    for pid in pids {
        println!("Found process {} on port {}. Killing...", pid, port);

        let kill_status = Command::new("kill")
            .arg("-9")
            .arg(pid)
            .status()
            .context(format!("Failed to kill process {}", pid))?;

        if kill_status.success() {
            println!("{}", format!("Successfully killed process {} on port {}", pid, port).green());
        } else {
            println!("{}", format!("Failed to kill process {} on port {}", pid, port).red());
        }
    }

    Ok(())
}
