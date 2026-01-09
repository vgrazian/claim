//! Message handling for the interactive UI

use std::time::{Duration, Instant};

/// Message type for styling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    Info,
    Success,
    Warning,
    Error,
}

/// A message to display to the user
#[derive(Debug, Clone)]
pub struct Message {
    pub message_type: MessageType,
    pub text: String,
    #[allow(dead_code)]
    pub timestamp: Instant,
}

impl Message {
    /// Create a new message
    pub fn new(message_type: MessageType, text: String) -> Self {
        Self {
            message_type,
            text,
            timestamp: Instant::now(),
        }
    }

    /// Check if the message has expired (older than 10 seconds)
    #[allow(dead_code)]
    pub fn is_expired(&self) -> bool {
        self.timestamp.elapsed() > Duration::from_secs(10)
    }

    /// Get the icon for this message type
    pub fn icon(&self) -> &str {
        match self.message_type {
            MessageType::Info => "ℹ",
            MessageType::Success => "✓",
            MessageType::Warning => "⚠",
            MessageType::Error => "✗",
        }
    }
}

// Made with Bob
