//! Utility functions for the claim application

use anyhow::{anyhow, Result};
use chrono::prelude::*;

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
        current_date += chrono::Duration::days(1);
    }

    dates
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
        "l104" => 13,
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
        13 => "l104".to_string(),
        _ => format!("unknown({})", value),
    }
}

// ===== L104 VALIDATION =====

/// Maximum allowed L104 days per calendar month
pub const L104_MAX_DAYS_PER_MONTH: f64 = 3.0;

/// Maximum allowed L104 hours per calendar month (3 days * 8 hours)
pub const L104_MAX_HOURS_PER_MONTH: f64 = 24.0;

/// Calculate total L104 hours for a given month from a list of entries
/// Returns (total_hours, total_days, entries_count)
pub fn calculate_l104_monthly_total(
    entries: &[(String, String, NaiveDate, String, f64)], // (customer, work_item, date, activity_type, hours)
    year: i32,
    month: u32,
) -> (f64, f64, usize) {
    let mut total_hours = 0.0;
    let mut entry_count = 0;

    for (_, _, date, activity_type, hours) in entries {
        if activity_type.to_lowercase() == "l104" && date.year() == year && date.month() == month {
            // If hours is 0, assume it's a full day (8 hours)
            let entry_hours = if *hours > 0.0 { *hours } else { 8.0 };
            total_hours += entry_hours;
            entry_count += 1;
        }
    }

    let total_days = total_hours / 8.0;
    (total_hours, total_days, entry_count)
}

/// Check if adding L104 hours would exceed monthly limit
/// Returns Ok(()) if within limit, Err with message if would exceed
pub fn validate_l104_monthly_limit(
    existing_entries: &[(String, String, NaiveDate, String, f64)],
    new_date: NaiveDate,
    new_hours: f64,
) -> Result<()> {
    let year = new_date.year();
    let month = new_date.month();

    let (current_hours, current_days, _) =
        calculate_l104_monthly_total(existing_entries, year, month);

    // If new_hours is 0, assume it's a full day (8 hours)
    let hours_to_add = if new_hours > 0.0 { new_hours } else { 8.0 };
    let new_total_hours = current_hours + hours_to_add;
    let new_total_days = new_total_hours / 8.0;

    if new_total_hours > L104_MAX_HOURS_PER_MONTH {
        return Err(anyhow!(
            "L104 monthly limit exceeded: {} hours ({:.1} days) already used in {}-{:02}. \
             Adding {:.1} hours would total {:.1} hours ({:.1} days), exceeding the limit of {} hours ({} days).",
            current_hours,
            current_days,
            year,
            month,
            hours_to_add,
            new_total_hours,
            new_total_days,
            L104_MAX_HOURS_PER_MONTH,
            L104_MAX_DAYS_PER_MONTH
        ));
    }

    Ok(())
}

// ===== MONDAY.COM UTILITIES =====

/// Default group ID used when year-specific group is not found
const DEFAULT_GROUP_ID: &str = "new_group_mkkbbd2q";

/// Gets the year group ID from a board by matching the year title
pub fn get_year_group_id(board: &crate::monday::Board, year: &str) -> String {
    if let Some(groups) = &board.groups {
        for group in groups {
            if group.title == year {
                return group.id.clone();
            }
        }
    }
    // Fallback to a default group ID if not found
    DEFAULT_GROUP_ID.to_string()
}

// ===== ITEM PROCESSING UTILITIES =====
// (Removed unused utility functions)

// Note: extract_item_date, extract_column_value, and is_item_matching_date
// have been removed as they are unused. If needed in the future, they can be
// found in git history.

#[allow(dead_code)]
fn extract_item_date_placeholder(column_values: &[crate::monday::ColumnValue]) -> Option<String> {
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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_activity_type_mapping() {
        assert_eq!(map_activity_type_to_value("billable"), 1);
        assert_eq!(map_activity_type_to_value("vacation"), 0);
        assert_eq!(map_activity_type_to_value("holding"), 2);
        assert_eq!(map_activity_type_to_value("l104"), 13);
        assert_eq!(map_activity_type_to_value("unknown"), 1); // default

        assert_eq!(map_activity_value_to_name(1), "billable");
        assert_eq!(map_activity_value_to_name(0), "vacation");
        assert_eq!(map_activity_value_to_name(9), "paid_not_worked");
        assert_eq!(map_activity_value_to_name(10), "intellectual_capital");
        assert_eq!(map_activity_value_to_name(11), "business_development");
        assert_eq!(map_activity_value_to_name(12), "overhead");
        assert_eq!(map_activity_value_to_name(13), "l104");
        assert_eq!(map_activity_value_to_name(99), "unknown(99)");
    }

    #[test]
    fn test_get_current_year() {
        let year = get_current_year();
        let current_year = Local::now().year();
        assert_eq!(year, current_year);
    }

    #[test]
    fn test_calculate_l104_monthly_total_empty() {
        let entries: Vec<(String, String, NaiveDate, String, f64)> = vec![];
        let (hours, days, count) = calculate_l104_monthly_total(&entries, 2026, 6);
        assert_eq!(hours, 0.0);
        assert_eq!(days, 0.0);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_calculate_l104_monthly_total_with_hours() {
        let date1 = NaiveDate::from_ymd_opt(2026, 6, 10).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
        let entries = vec![
            (
                "Customer".to_string(),
                "Work".to_string(),
                date1,
                "l104".to_string(),
                8.0,
            ),
            (
                "Customer".to_string(),
                "Work".to_string(),
                date2,
                "l104".to_string(),
                8.0,
            ),
        ];
        let (hours, days, count) = calculate_l104_monthly_total(&entries, 2026, 6);
        assert_eq!(hours, 16.0);
        assert_eq!(days, 2.0);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_calculate_l104_monthly_total_zero_hours() {
        let date1 = NaiveDate::from_ymd_opt(2026, 6, 10).unwrap();
        let entries = vec![(
            "Customer".to_string(),
            "Work".to_string(),
            date1,
            "l104".to_string(),
            0.0,
        )];
        let (hours, days, count) = calculate_l104_monthly_total(&entries, 2026, 6);
        assert_eq!(hours, 8.0); // 0 hours treated as full day (8 hours)
        assert_eq!(days, 1.0);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_calculate_l104_monthly_total_different_months() {
        let date1 = NaiveDate::from_ymd_opt(2026, 6, 10).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2026, 7, 15).unwrap();
        let entries = vec![
            (
                "Customer".to_string(),
                "Work".to_string(),
                date1,
                "l104".to_string(),
                8.0,
            ),
            (
                "Customer".to_string(),
                "Work".to_string(),
                date2,
                "l104".to_string(),
                8.0,
            ),
        ];
        let (hours, days, count) = calculate_l104_monthly_total(&entries, 2026, 6);
        assert_eq!(hours, 8.0); // Only June entry
        assert_eq!(days, 1.0);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_calculate_l104_monthly_total_mixed_activity_types() {
        let date1 = NaiveDate::from_ymd_opt(2026, 6, 10).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
        let entries = vec![
            (
                "Customer".to_string(),
                "Work".to_string(),
                date1,
                "l104".to_string(),
                8.0,
            ),
            (
                "Customer".to_string(),
                "Work".to_string(),
                date2,
                "billable".to_string(),
                8.0,
            ),
        ];
        let (hours, days, count) = calculate_l104_monthly_total(&entries, 2026, 6);
        assert_eq!(hours, 8.0); // Only L104 entry
        assert_eq!(days, 1.0);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_validate_l104_monthly_limit_within_limit() {
        let date1 = NaiveDate::from_ymd_opt(2026, 6, 10).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
        let entries = vec![(
            "Customer".to_string(),
            "Work".to_string(),
            date1,
            "l104".to_string(),
            8.0,
        )];
        let result = validate_l104_monthly_limit(&entries, date2, 8.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_l104_monthly_limit_at_limit() {
        let date1 = NaiveDate::from_ymd_opt(2026, 6, 10).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
        let date3 = NaiveDate::from_ymd_opt(2026, 6, 20).unwrap();
        let entries = vec![
            (
                "Customer".to_string(),
                "Work".to_string(),
                date1,
                "l104".to_string(),
                8.0,
            ),
            (
                "Customer".to_string(),
                "Work".to_string(),
                date2,
                "l104".to_string(),
                8.0,
            ),
        ];
        let result = validate_l104_monthly_limit(&entries, date3, 8.0);
        assert!(result.is_ok()); // 16 + 8 = 24, exactly at limit
    }

    #[test]
    fn test_validate_l104_monthly_limit_exceeds() {
        let date1 = NaiveDate::from_ymd_opt(2026, 6, 10).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
        let date3 = NaiveDate::from_ymd_opt(2026, 6, 20).unwrap();
        let entries = vec![
            (
                "Customer".to_string(),
                "Work".to_string(),
                date1,
                "l104".to_string(),
                8.0,
            ),
            (
                "Customer".to_string(),
                "Work".to_string(),
                date2,
                "l104".to_string(),
                8.0,
            ),
            (
                "Customer".to_string(),
                "Work".to_string(),
                date3,
                "l104".to_string(),
                8.0,
            ),
        ];
        let date4 = NaiveDate::from_ymd_opt(2026, 6, 25).unwrap();
        let result = validate_l104_monthly_limit(&entries, date4, 8.0);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("L104 monthly limit exceeded"));
    }

    #[test]
    fn test_validate_l104_monthly_limit_different_month() {
        let date1 = NaiveDate::from_ymd_opt(2026, 6, 10).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
        let date3 = NaiveDate::from_ymd_opt(2026, 6, 20).unwrap();
        let entries = vec![
            (
                "Customer".to_string(),
                "Work".to_string(),
                date1,
                "l104".to_string(),
                8.0,
            ),
            (
                "Customer".to_string(),
                "Work".to_string(),
                date2,
                "l104".to_string(),
                8.0,
            ),
            (
                "Customer".to_string(),
                "Work".to_string(),
                date3,
                "l104".to_string(),
                8.0,
            ),
        ];
        // Adding to July should be OK even though June is at limit
        let date4 = NaiveDate::from_ymd_opt(2026, 7, 5).unwrap();
        let result = validate_l104_monthly_limit(&entries, date4, 8.0);
        assert!(result.is_ok());
    }
}
