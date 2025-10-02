use anyhow::{anyhow, Result};
use chrono::prelude::*;
use std::time::{Duration, Instant};

/// Utility functions for the claim application

// ===== STRING UTILITIES =====

/// Masks an API key for safe logging (shows first 4 characters, masks the rest)
pub fn mask_api_key(api_key: &str) -> String {
    if api_key.len() <= 4 {
        "*".repeat(api_key.len())
    } else {
        let visible_part = &api_key[..4];
        let masked_part = "*".repeat(api_key.len() - 4);
        format!("{}{}", visible_part, masked_part)
    }
}

/// Truncates a string to a maximum length, adding "..." if truncated
pub fn truncate_string(s: &str, max_length: usize) -> String {
    if s.len() <= max_length {
        s.to_string()
    } else {
        format!("{}...", &s[..max_length.saturating_sub(3)])
    }
}

/// Escapes special characters in a string for JSON/GraphQL
pub fn escape_string(s: &str) -> String {
    s.replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Checks if a string is empty or contains only whitespace
pub fn is_blank(s: &str) -> bool {
    s.trim().is_empty()
}

// ===== DATE/TIME UTILITIES =====

/// Gets the current year as i32
pub fn get_current_year() -> i32 {
    Local::now().year()
}

/// Validates a date string in multiple formats (YYYY-MM-DD, YYYY.MM.DD, YYYY/MM/DD)
pub fn validate_date(date_str: &str) -> Result<()> {
    let formats = ["%Y-%m-%d", "%Y.%m.%d", "%Y/%m/%d"];

    for format in &formats {
        if chrono::NaiveDate::parse_from_str(date_str, format).is_ok() {
            return Ok(());
        }
    }

    Err(anyhow!(
        "Invalid date format: {}. Please use YYYY-MM-DD, YYYY.MM.DD, or YYYY/MM/DD format.",
        date_str
    ))
}

/// Normalizes a date string to YYYY-MM-DD format
pub fn normalize_date(date_str: &str) -> String {
    let formats = ["%Y-%m-%d", "%Y.%m.%d", "%Y/%m/%d"];

    for format in &formats {
        if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, format) {
            return date.format("%Y-%m-%d").to_string();
        }
    }

    // If we can't parse it, return the original (this shouldn't happen if validate_date was called first)
    date_str.to_string()
}

/// Calculates working dates (skips weekends) from a start date for a given number of days
pub fn calculate_working_dates(start_date: NaiveDate, target_days: i64) -> Vec<NaiveDate> {
    let mut dates = Vec::new();
    let mut current_date = start_date;
    let mut days_added = 0;

    while days_added < target_days {
        // Check if it's a weekday (Monday = 1, Friday = 5)
        let weekday = current_date.weekday().number_from_monday();
        if weekday <= 5 {
            dates.push(current_date);
            days_added += 1;
        }

        // Move to next day
        current_date = current_date + chrono::Duration::days(1);
    }

    dates
}

/// Formats a duration for human-readable output
pub fn format_duration(duration: Duration) -> String {
    if duration.as_secs() > 0 {
        format!("{:.2}s", duration.as_secs_f64())
    } else if duration.as_millis() > 0 {
        format!("{}ms", duration.as_millis())
    } else {
        format!("{}μs", duration.as_micros())
    }
}

/// Gets a friendly weekday name from a date
pub fn get_weekday_name(date: &NaiveDate) -> String {
    match date.weekday() {
        Weekday::Mon => "Monday".to_string(),
        Weekday::Tue => "Tuesday".to_string(),
        Weekday::Wed => "Wednesday".to_string(),
        Weekday::Thu => "Thursday".to_string(),
        Weekday::Fri => "Friday".to_string(),
        Weekday::Sat => "Saturday".to_string(),
        Weekday::Sun => "Sunday".to_string(),
    }
}

/// Checks if a date is a weekend
pub fn is_weekend(date: &NaiveDate) -> bool {
    let weekday = date.weekday().number_from_monday();
    weekday > 5
}

// ===== ACTIVITY TYPE UTILITIES =====

/// Maps activity type string to numeric value
pub fn map_activity_type_to_value(activity_type: &str) -> u8 {
    match activity_type.to_lowercase().as_str() {
        "vacation" => 0,
        "billable" => 1,
        "holding" => 2,
        "education" => 3,
        "work_reduction" => 4,
        "tbd" => 5,
        "holiday" => 6,
        "presales" => 7,
        "illness" => 8,
        "paid_not_worked" => 9,
        "intellectual_capital" => 10,
        "business_development" => 11,
        "overhead" => 12,
        _ => {
            println!(
                "Warning: Unknown activity type '{}', defaulting to billable (1)",
                activity_type
            );
            1 // Default to billable for unknown types
        }
    }
}

/// Maps activity numeric value to string name
pub fn map_activity_value_to_name(value: u8) -> String {
    match value {
        0 => "vacation".to_string(),
        1 => "billable".to_string(),
        2 => "holding".to_string(),
        3 => "education".to_string(),
        4 => "work_reduction".to_string(),
        5 => "tbd".to_string(),
        6 => "holiday".to_string(),
        7 => "presales".to_string(),
        8 => "illness".to_string(),
        9 => "paid_not_worked".to_string(),
        10 => "intellectual_capital".to_string(),
        11 => "business_development".to_string(),
        12 => "overhead".to_string(),
        _ => format!("unknown({})", value),
    }
}

// ===== PERFORMANCE UTILITIES =====

/// A simple timer for performance measurement
pub struct Timer {
    start: Instant,
    message: String,
}

impl Timer {
    /// Creates a new timer with a message
    pub fn new(message: &str) -> Self {
        Timer {
            start: Instant::now(),
            message: message.to_string(),
        }
    }

    /// Stops the timer and returns the duration
    pub fn stop(self) -> Duration {
        self.start.elapsed()
    }

    /// Stops the timer and prints the elapsed time
    pub fn stop_and_print(self) -> Duration {
        let duration = self.start.elapsed();
        println!("⏱️  {} took {}", self.message, format_duration(duration));
        duration
    }

    /// Stops the timer and returns a formatted message
    pub fn stop_with_message(self) -> String {
        let duration = self.start.elapsed();
        format!("{} took {}", self.message, format_duration(duration))
    }
}

/// Measures the execution time of a closure
pub fn measure_time<F, R>(name: &str, f: F) -> R
where
    F: FnOnce() -> R,
{
    let timer = Timer::new(name);
    let result = f();
    timer.stop_and_print();
    result
}

// ===== FORMATTING UTILITIES =====

/// Formats a number of hours with proper pluralization
pub fn format_hours(hours: f64) -> String {
    if hours == 1.0 {
        format!("{:.1} hour", hours)
    } else {
        format!("{:.1} hours", hours)
    }
}

/// Formats a number of days with proper pluralization
pub fn format_days(days: f64) -> String {
    if days == 1.0 {
        format!("{:.1} day", days)
    } else {
        format!("{:.1} days", days)
    }
}

/// Creates a progress bar string
pub fn create_progress_bar(current: usize, total: usize, width: usize) -> String {
    if total == 0 {
        return "[]".to_string();
    }

    let progress = (current as f64 / total as f64).min(1.0);
    let filled = (progress * width as f64).round() as usize;
    let empty = width - filled;

    format!("[{}{}]", "=".repeat(filled), " ".repeat(empty))
}

/// Formats a percentage value
pub fn format_percentage(value: f64, total: f64) -> String {
    if total == 0.0 {
        "0.0%".to_string()
    } else {
        format!("{:.1}%", (value / total) * 100.0)
    }
}

// ===== VALIDATION UTILITIES =====

/// Validates that hours are within a reasonable range (0-24)
pub fn validate_hours(hours: f64) -> Result<()> {
    if hours < 0.0 {
        Err(anyhow!("Hours cannot be negative"))
    } else if hours > 24.0 {
        Err(anyhow!("Hours cannot exceed 24"))
    } else {
        Ok(())
    }
}

/// Validates that days are within a reasonable range (0-365)
pub fn validate_days(days: f64) -> Result<()> {
    if days < 0.0 {
        Err(anyhow!("Days cannot be negative"))
    } else if days > 365.0 {
        Err(anyhow!("Days cannot exceed 365"))
    } else {
        Ok(())
    }
}

/// Validates an email address format (basic validation)
pub fn validate_email(email: &str) -> Result<()> {
    if email.contains('@') && email.contains('.') && email.len() > 5 {
        Ok(())
    } else {
        Err(anyhow!("Invalid email format: {}", email))
    }
}

// ===== DEBUG UTILITIES =====

/// Pretty-prints a JSON value for debugging
pub fn pretty_print_json(value: &serde_json::Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| "Invalid JSON".to_string())
}

/// Creates a debug header for section separation in logs
pub fn debug_header(title: &str) -> String {
    let line = "=".repeat(title.len() + 4);
    format!("{}\n  {}  \n{}", line, title, line)
}

/// Creates a debug section for organized logging
pub fn debug_section<F>(title: &str, verbose: bool, content: F)
where
    F: FnOnce() -> String,
{
    if verbose {
        println!("\n{}\n{}\n", debug_header(title), content());
    }
}

// ===== ITEM PROCESSING UTILITIES =====

/// Helper function to extract date from an item column values
pub fn extract_item_date(column_values: &[crate::monday::ColumnValue]) -> Option<String> {
    for col in column_values {
        if let Some(col_id) = &col.id {
            if col_id == "date4" {
                // Try to parse from value field (JSON format)
                if let Some(value) = &col.value {
                    if value != "null" && !value.is_empty() {
                        if let Ok(parsed_value) = serde_json::from_str::<serde_json::Value>(value) {
                            if let Some(date_obj) = parsed_value.get("date") {
                                if let Some(date_str) = date_obj.as_str() {
                                    // Normalize the date format to YYYY-MM-DD
                                    if let Ok(naive_date) =
                                        NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                                    {
                                        return Some(naive_date.format("%Y-%m-%d").to_string());
                                    }
                                    // Try other common date formats
                                    if let Ok(naive_date) =
                                        NaiveDate::parse_from_str(date_str, "%Y/%m/%d")
                                    {
                                        return Some(naive_date.format("%Y-%m-%d").to_string());
                                    }
                                    if let Ok(naive_date) =
                                        NaiveDate::parse_from_str(date_str, "%Y.%m.%d")
                                    {
                                        return Some(naive_date.format("%Y-%m-%d").to_string());
                                    }
                                    // Return the original string if parsing fails
                                    return Some(date_str.to_string());
                                }
                            }
                        }
                    }
                }
                // Fallback: try to parse from text field
                if let Some(text) = &col.text {
                    if !text.is_empty() && text != "null" {
                        // Normalize the date format
                        if let Ok(naive_date) = NaiveDate::parse_from_str(text, "%Y-%m-%d") {
                            return Some(naive_date.format("%Y-%m-%d").to_string());
                        }
                        if let Ok(naive_date) = NaiveDate::parse_from_str(text, "%Y/%m/%d") {
                            return Some(naive_date.format("%Y-%m-%d").to_string());
                        }
                        if let Ok(naive_date) = NaiveDate::parse_from_str(text, "%Y.%m.%d") {
                            return Some(naive_date.format("%Y-%m-%d").to_string());
                        }
                        return Some(text.to_string());
                    }
                }
            }
        }
    }
    None
}

/// Helper function to extract specific column value
pub fn extract_column_value(
    column_values: &[crate::monday::ColumnValue],
    column_id: &str,
) -> String {
    for col in column_values {
        if let Some(col_id) = &col.id {
            if col_id == column_id {
                if let Some(value) = &col.value {
                    if value != "null" && !value.is_empty() {
                        // Try to parse JSON value for complex columns
                        if let Ok(parsed_value) = serde_json::from_str::<serde_json::Value>(value) {
                            if let Some(text) = parsed_value.as_str() {
                                return text.to_string();
                            }
                        }
                        return value.to_string();
                    }
                }
                if let Some(text) = &col.text {
                    if !text.is_empty() && text != "null" {
                        return text.to_string();
                    }
                }
            }
        }
    }
    "".to_string()
}

/// Helper function to check if an item matches the specified date
pub fn is_item_matching_date(
    column_values: &[crate::monday::ColumnValue],
    target_date: &str,
) -> bool {
    for col in column_values {
        if let Some(col_id) = &col.id {
            if col_id == "date4" {
                // Parse the date column value to check if it matches the target date
                if let Some(value) = &col.value {
                    if let Ok(parsed_value) = serde_json::from_str::<serde_json::Value>(value) {
                        if let Some(date_obj) = parsed_value.get("date") {
                            if let Some(date_str) = date_obj.as_str() {
                                // Compare the date part only (ignore time if present)
                                if date_str.starts_with(target_date) {
                                    return true;
                                }
                            }
                        }
                    }
                }
                // Also check the text field as fallback
                if let Some(text) = &col.text {
                    if text.starts_with(target_date) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration as StdDuration;

    #[test]
    fn test_mask_api_key() {
        assert_eq!(mask_api_key("12345678"), "1234****");
        assert_eq!(mask_api_key("1234"), "****");
        assert_eq!(mask_api_key("123"), "***");
        assert_eq!(mask_api_key(""), "");
    }

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("short", 10), "short");
        assert_eq!(truncate_string("very long string", 9), "very l...");
        assert_eq!(truncate_string("short st", 9), "short st");
        assert_eq!(truncate_string("", 9), "");
        assert_eq!(truncate_string("abc", 3), "abc");
        assert_eq!(truncate_string("abcd", 3), "...");
    }

    #[test]
    fn test_validate_date() {
        assert!(validate_date("2025-09-15").is_ok());
        assert!(validate_date("2025.09.15").is_ok());
        assert!(validate_date("2025/09/15").is_ok());
        assert!(validate_date("invalid-date").is_err());
    }

    #[test]
    fn test_normalize_date() {
        assert_eq!(normalize_date("2025-09-15"), "2025-09-15");
        assert_eq!(normalize_date("2025.09.15"), "2025-09-15");
        assert_eq!(normalize_date("2025/09/15"), "2025-09-15");
    }

    #[test]
    fn test_calculate_working_dates() {
        let start_date = NaiveDate::from_ymd_opt(2025, 9, 15).unwrap(); // Monday
        let dates = calculate_working_dates(start_date, 5);

        assert_eq!(dates.len(), 5);
        assert_eq!(dates[0].weekday(), Weekday::Mon);
        assert_eq!(dates[4].weekday(), Weekday::Fri);

        // Test that weekends are skipped
        let weekend_start = NaiveDate::from_ymd_opt(2025, 9, 13).unwrap(); // Saturday
        let weekend_dates = calculate_working_dates(weekend_start, 2);
        assert_eq!(weekend_dates.len(), 2);
        assert_eq!(weekend_dates[0].weekday(), Weekday::Mon); // Should skip to Monday
    }

    #[test]
    fn test_format_duration() {
        let duration = StdDuration::from_secs(2);
        assert_eq!(format_duration(duration), "2.00s");

        let duration = StdDuration::from_millis(150);
        assert_eq!(format_duration(duration), "150ms");

        let duration = StdDuration::from_micros(500);
        assert_eq!(format_duration(duration), "500μs");
    }

    #[test]
    fn test_timer() {
        let timer = Timer::new("test operation");
        thread::sleep(StdDuration::from_millis(10));
        let duration = timer.stop();
        assert!(duration.as_millis() >= 10);
    }

    #[test]
    fn test_activity_type_mapping() {
        assert_eq!(map_activity_type_to_value("billable"), 1);
        assert_eq!(map_activity_type_to_value("vacation"), 0);
        assert_eq!(map_activity_type_to_value("unknown"), 1); // default

        assert_eq!(map_activity_value_to_name(1), "billable");
        assert_eq!(map_activity_value_to_name(0), "vacation");
        assert_eq!(map_activity_value_to_name(99), "unknown(99)");
    }

    #[test]
    fn test_validation_functions() {
        assert!(validate_hours(8.0).is_ok());
        assert!(validate_hours(-1.0).is_err());
        assert!(validate_hours(25.0).is_err());

        assert!(validate_days(5.0).is_ok());
        assert!(validate_days(-1.0).is_err());
        assert!(validate_days(400.0).is_err());

        assert!(validate_email("test@example.com").is_ok());
        assert!(validate_email("invalid").is_err());
    }

    #[test]
    fn test_format_functions() {
        assert_eq!(format_hours(1.0), "1.0 hour");
        assert_eq!(format_hours(2.5), "2.5 hours");

        assert_eq!(format_days(1.0), "1.0 day");
        assert_eq!(format_days(3.5), "3.5 days");

        assert_eq!(format_percentage(25.0, 100.0), "25.0%");
        assert_eq!(format_percentage(0.0, 0.0), "0.0%");
    }

    #[test]
    fn test_progress_bar() {
        assert_eq!(create_progress_bar(5, 10, 10), "[=====     ]");
        assert_eq!(create_progress_bar(0, 10, 10), "[          ]");
        assert_eq!(create_progress_bar(10, 10, 10), "[==========]");
    }

    #[test]
    fn test_weekday_utilities() {
        let date = NaiveDate::from_ymd_opt(2025, 9, 15).unwrap(); // Monday
        assert_eq!(get_weekday_name(&date), "Monday");
        assert!(!is_weekend(&date));

        let weekend_date = NaiveDate::from_ymd_opt(2025, 9, 13).unwrap(); // Saturday
        assert!(is_weekend(&weekend_date));
    }

    #[test]
    fn test_get_current_year() {
        let year = get_current_year();
        let current_year = Local::now().year();
        assert_eq!(year, current_year);
    }
}
