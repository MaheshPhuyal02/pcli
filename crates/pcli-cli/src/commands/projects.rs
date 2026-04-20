//! Project commands

use anyhow::{Result, bail};
use colored::Colorize;
use pcli_core::{Project, Storage};

/// List all projects
pub fn list(storage: &Storage) -> Result<()> {
    let projects = storage.list_projects()?;
    let current = storage.get_current_project()?;

    if projects.is_empty() {
        println!("{}", "No projects yet.".dimmed());
        println!("Create one with: {}", "pcli new <name>".cyan());
        return Ok(());
    }

    println!("{}", "📁 Projects".bold());
    println!("{}", "─".repeat(40).dimmed());

    for project in projects {
        let (done, total) = storage.count_tasks(&project.id)?;
        let is_current = current.as_ref().map(|c| c == &project.id).unwrap_or(false);
        
        let marker = if is_current { "▶".green() } else { " ".normal() };
        let name = if is_current {
            project.name.green().bold()
        } else {
            project.name.normal()
        };
        
        println!("{} {}  ({}/{})", marker, name, done, total);
    }

    Ok(())
}

/// Create a new project
pub fn create(storage: &Storage, name: &str) -> Result<()> {
    let project = Project::new(name);
    
    // Check if exists
    if storage.get_project(&project.id)?.is_some() {
        bail!("Project '{}' already exists", name);
    }

    storage.create_project(&project)?;
    storage.set_current_project(&project.id)?;

    println!("{} Created project: {}", "✓".green(), project.name.cyan());
    println!("{} Switched to: {}", "✓".green(), project.name.cyan());

    Ok(())
}

/// Switch to a project
pub fn switch(storage: &Storage, name: &str) -> Result<()> {
    let id = name.to_lowercase();
    
    // First try exact match on ID
    if let Some(project) = storage.get_project(&id)? {
        storage.set_current_project(&project.id)?;
        println!("Switched to: {} {}", project.name.cyan(), "✓".green());
        return Ok(());
    }

    // Try to find by name prefix
    let projects = storage.list_projects()?;
    let matches: Vec<_> = projects
        .iter()
        .filter(|p| p.id.starts_with(&id) || p.name.to_lowercase().starts_with(&id))
        .collect();

    match matches.len() {
        0 => {
            println!("{}", format!("Project '{}' not found.", name).red());
            println!("Create it with: {}", format!("pcli new {}", name).cyan());
        }
        1 => {
            let project = matches[0];
            storage.set_current_project(&project.id)?;
            println!("Switched to: {} {}", project.name.cyan(), "✓".green());
        }
        _ => {
            println!("{}", "Multiple projects match:".yellow());
            for p in matches {
                println!("  - {}", p.name);
            }
        }
    }

    Ok(())
}

/// Show current project
pub fn current(storage: &Storage) -> Result<()> {
    match storage.get_current_project()? {
        Some(id) => {
            if let Some(project) = storage.get_project(&id)? {
                println!("Current project: {}", project.name.cyan());
            } else {
                println!("{}", "No active project.".dimmed());
            }
        }
        None => {
            println!("{}", "No active project.".dimmed());
            println!("Switch with: {}", "pcli <project-name>".cyan());
        }
    }

    Ok(())
}

/// Delete a project
pub fn delete(storage: &Storage, name: &str) -> Result<()> {
    let id = name.to_lowercase();
    
    if let Some(project) = storage.get_project(&id)? {
        storage.delete_project(&project.id)?;
        println!("{} Deleted project: {}", "✓".green(), project.name);
        
        // Clear current if it was this project
        if storage.get_current_project()? == Some(id) {
            storage.set_current_project("")?;
        }
    } else {
        println!("{}", format!("Project '{}' not found.", name).red());
    }

    Ok(())
}
