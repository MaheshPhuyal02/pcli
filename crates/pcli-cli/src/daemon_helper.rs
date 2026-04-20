
use anyhow::Result;
use colored::Colorize;

/// Ensure the background daemon is running
pub fn ensure_daemon_running() -> Result<()> {
    let pid_path = pcli_core::daemon_pid_path();

    if pid_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&pid_path) {
            if let Ok(pid) = content.trim().parse::<i32>() {
                // Check if process is running
                // On unix catch kill -0
                let status = std::process::Command::new("kill")
                    .arg("-0")
                    .arg(pid.to_string())
                    .output();
                
                if let Ok(output) = status {
                    if output.status.success() {
                        return Ok(());
                    }
                }
            }
        }
    }

    // Determine executable path
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path.parent().expect("Failed to get executable directory");
    let daemon_path = exe_dir.join("pcli-daemon");

    if daemon_path.exists() {
        std::process::Command::new(daemon_path)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?;
    } else {
        // Fallback for development
        println!("{}", "Starting background daemon...".dimmed());
        
        std::process::Command::new("cargo")
            .args(["run", "--bin", "pcli-daemon", "--quiet"])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?;
    }
    
    // Give it a moment to start
    std::thread::sleep(std::time::Duration::from_millis(500));

    Ok(())
}
