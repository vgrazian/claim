//! Data extraction utilities for converting Monday.com items to application data structures

use crate::monday::Item;
use chrono::NaiveDate;

/// Get the start of the week (Monday) for a given date
pub fn get_week_start(date: NaiveDate) -> NaiveDate {
    use chrono::Datelike;
    date - chrono::Duration::days(date.weekday().num_days_from_monday() as i64)
}

/// Extract date from a Monday.com item
pub fn extract_date_from_item(item: &Item) -> Option<NaiveDate> {
    item.column_values.iter().find_map(|cv| {
        if cv.id.as_deref() == Some("date4") {
            cv.text
                .as_ref()
                .and_then(|text| NaiveDate::parse_from_str(text, "%Y-%m-%d").ok())
        } else {
            None
        }
    })
}

/// Extract activity value from a Monday.com item
/// Parses the JSON value field to get the status index, which maps to activity types
pub fn extract_activity_value_from_item(item: &Item) -> i32 {
    item.column_values
        .iter()
        .find(|cv| cv.id.as_deref() == Some("status"))
        .and_then(|cv| {
            // First try to parse from JSON value field (contains {"index": N})
            if let Some(value) = &cv.value {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(value) {
                    if let Some(index) = parsed.get("index") {
                        if let Some(index_num) = index.as_i64() {
                            return Some(index_num as i32);
                        }
                    }
                }
            }
            // Fallback: try to parse from text field
            cv.text.as_ref().and_then(|text| text.parse::<i32>().ok())
        })
        .unwrap_or(1) // Default to billable (1) instead of vacation (0)
}

/// Extract customer name from a Monday.com item
pub fn extract_customer_from_item(item: &Item) -> String {
    item.column_values
        .iter()
        .find(|cv| cv.id.as_deref() == Some("text__1"))
        .and_then(|cv| cv.text.clone())
        .unwrap_or_default()
}

/// Extract work item from a Monday.com item
pub fn extract_work_item_from_item(item: &Item) -> String {
    item.column_values
        .iter()
        .find(|cv| cv.id.as_deref() == Some("text8__1"))
        .and_then(|cv| cv.text.clone())
        .unwrap_or_default()
}

/// Extract hours from a Monday.com item
pub fn extract_hours_from_item(item: &Item) -> f64 {
    item.column_values
        .iter()
        .find(|cv| cv.id.as_deref() == Some("numbers"))
        .and_then(|cv| cv.text.as_ref())
        .and_then(|text| text.parse::<f64>().ok())
        .unwrap_or(0.0)
}

/// Extract comment from a Monday.com item
pub fn extract_comment_from_item(item: &Item) -> Option<String> {
    item.column_values
        .iter()
        .find(|cv| cv.id.as_deref() == Some("long_text"))
        .and_then(|cv| cv.text.clone())
        .filter(|text| !text.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_get_week_start() {
        // Test with a Wednesday (2024-01-03)
        let date = NaiveDate::from_ymd_opt(2024, 1, 3).unwrap();
        let week_start = get_week_start(date);
        assert_eq!(week_start.weekday(), chrono::Weekday::Mon);
        assert_eq!(week_start, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());

        // Test with a Monday (already week start)
        let monday = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let week_start = get_week_start(monday);
        assert_eq!(week_start, monday);

        // Test with a Sunday
        let sunday = NaiveDate::from_ymd_opt(2024, 1, 7).unwrap();
        let week_start = get_week_start(sunday);
        assert_eq!(week_start, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
    }

    #[test]
    fn test_extract_activity_value_default() {
        use crate::monday::ColumnValue;

        let item = Item {
            id: Some("123".to_string()),
            name: Some("Test".to_string()),
            column_values: vec![],
        };

        // Default to billable (1) when no status column is found
        assert_eq!(extract_activity_value_from_item(&item), 1);
    }

    #[test]
    fn test_extract_customer_default() {
        use crate::monday::ColumnValue;

        let item = Item {
            id: Some("123".to_string()),
            name: Some("Test".to_string()),
            column_values: vec![],
        };

        assert_eq!(extract_customer_from_item(&item), "");
    }

    #[test]
    fn test_extract_hours_default() {
        use crate::monday::ColumnValue;

        let item = Item {
            id: Some("123".to_string()),
            name: Some("Test".to_string()),
            column_values: vec![],
        };

        assert_eq!(extract_hours_from_item(&item), 0.0);
    }

    #[test]
    fn test_extract_comment_empty() {
        use crate::monday::ColumnValue;

        let item = Item {
            id: Some("123".to_string()),
            name: Some("Test".to_string()),
            column_values: vec![],
        };

        assert_eq!(extract_comment_from_item(&item), None);
    }
}

// Made with Bob
