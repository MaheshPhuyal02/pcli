//! Project model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A project containing tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// Unique identifier (slug)
    pub id: String,
    
    /// Display name
    pub name: String,
    
    /// Optional description
    pub description: Option<String>,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl Project {
    /// Create a new project
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        let id = Self::slugify(&name);
        let now = Utc::now();
        
        Self {
            id,
            name,
            description: None,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Create a new project with description
    pub fn with_description(name: impl Into<String>, description: impl Into<String>) -> Self {
        let mut project = Self::new(name);
        project.description = Some(description.into());
        project
    }
    
    /// Convert name to slug (lowercase, replace spaces with hyphens)
    fn slugify(name: &str) -> String {
        name.to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_project_creation() {
        let project = Project::new("My Project");
        assert_eq!(project.id, "my-project");
        assert_eq!(project.name, "My Project");
    }
    
    #[test]
    fn test_slugify() {
        assert_eq!(Project::slugify("Hello World"), "hello-world");
        assert_eq!(Project::slugify("My  Project  Name"), "my-project-name");
        assert_eq!(Project::slugify("Project_123"), "project-123");
    }
}
