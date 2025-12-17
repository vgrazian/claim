//! Utility functions for the interactive UI

use super::messages::MessageType;
use ratatui::style::{Color, Style};

/// Get color for activity type
pub fn get_activity_color(activity_type: &str) -> Color {
    match activity_type.to_lowercase().as_str() {
        "billable" => Color::Green,
        "vacation" => Color::Blue,
        "presales" => Color::Cyan,
        "overhead" => Color::Yellow,
        "illness" => Color::Red,
        "holiday" => Color::Magenta,
        "education" => Color::LightBlue,
        "holding" => Color::Gray,
        _ => Color::White,
    }
}

/// Get style for message type
pub fn get_message_style(message_type: MessageType) -> Style {
    match message_type {
        MessageType::Info => Style::default().fg(Color::Cyan),
        MessageType::Success => Style::default().fg(Color::Green),
        MessageType::Warning => Style::default().fg(Color::Yellow),
        MessageType::Error => Style::default().fg(Color::Red),
    }
}

/// Truncate string to fit width
pub fn truncate_str(s: &str, max_width: usize) -> String {
    if s.len() <= max_width {
        s.to_string()
    } else if max_width <= 3 {
        "...".to_string()
    } else {
        format!("{}...", &s[..max_width - 3])
    }
}

/// Format hours with one decimal place
pub fn format_hours(hours: f64) -> String {
    format!("{:.1}h", hours)
}

/// Get weekday name
pub fn get_weekday_name(weekday: chrono::Weekday) -> &'static str {
    match weekday {
        chrono::Weekday::Mon => "Mon",
        chrono::Weekday::Tue => "Tue",
        chrono::Weekday::Wed => "Wed",
        chrono::Weekday::Thu => "Thu",
        chrono::Weekday::Fri => "Fri",
        chrono::Weekday::Sat => "Sat",
        chrono::Weekday::Sun => "Sun",
    }
}

// Made with Bob
