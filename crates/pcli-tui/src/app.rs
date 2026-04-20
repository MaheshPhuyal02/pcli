//! TUI Application state

use pcli_core::Storage;

/// TUI Application
pub struct App {
    /// Whether the app should exit
    pub should_quit: bool,
}

impl App {
    /// Create new app
    pub fn new(_storage: &Storage) -> Self {
        Self { should_quit: false }
    }

    /// Handle key event
    pub fn on_key(&mut self, key: char) {
        match key {
            'q' => self.should_quit = true,
            _ => {}
        }
    }
}
