//! Event handling for the interactive UI

use anyhow::Result;
use crossterm::event::{self, Event, KeyEvent};
use std::time::Duration;

/// Event handler for keyboard input
pub struct EventHandler {
    /// Timeout for polling events
    timeout: Duration,
}

impl EventHandler {
    /// Create a new EventHandler
    pub fn new() -> Self {
        Self {
            timeout: Duration::from_millis(100),
        }
    }

    /// Get the next keyboard event, if available
    pub fn next(&self) -> Result<Option<KeyEvent>> {
        if event::poll(self.timeout)? {
            if let Event::Key(key_event) = event::read()? {
                return Ok(Some(key_event));
            }
        }
        Ok(None)
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

// Made with Bob
