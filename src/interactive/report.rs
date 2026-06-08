//! Report generation logic for weekly time tracking summaries

use anyhow::Result;
use chrono::Datelike;
use std::collections::HashMap;

use super::app::ClaimEntry;

/// Generate textual lines for report rows in the same order as UI rendering
/// Returns the text for each data row (excluding header)
pub fn get_report_rows_text(claims: &[ClaimEntry]) -> Result<Vec<String>> {
    let report_data = build_report_data(claims);
    let (billable_data, non_billable_data) = split_and_sort_report_data(&report_data);

    Ok(format_report_rows(&billable_data, &non_billable_data))
}

/// Get the work item for a specific report row by index
pub fn get_report_row_work_item(claims: &[ClaimEntry], idx: usize) -> Result<String> {
    let report_data = build_report_data(claims);
    let (billable_data, non_billable_data) = split_and_sort_report_data(&report_data);

    let total_billable = billable_data.len();

    if idx < total_billable {
        Ok(billable_data[idx].0 .1.clone())
    } else {
        let non_billable_idx = idx - total_billable;
        if non_billable_idx < non_billable_data.len() {
            Ok(non_billable_data[non_billable_idx].0 .2.clone())
        } else {
            anyhow::bail!("Row index out of bounds")
        }
    }
}

/// Get the text for a specific report row by index
pub fn get_report_row_text(claims: &[ClaimEntry], idx: usize) -> Result<String> {
    let rows = get_report_rows_text(claims)?;
    rows.get(idx)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Row index out of bounds"))
}

/// Build report data structure from claims
fn build_report_data(claims: &[ClaimEntry]) -> HashMap<(i32, String, String), [f64; 5]> {
    let mut report_data: HashMap<(i32, String, String), [f64; 5]> = HashMap::new();

    for entry in claims {
        let key = (
            entry.activity_value,
            entry.customer.clone(),
            entry.work_item.clone(),
        );
        let day_index = entry.date.weekday().num_days_from_monday() as usize;
        if day_index < 5 {
            report_data.entry(key).or_insert([0.0; 5])[day_index] += entry.hours;
        }
    }

    report_data
}

/// Split report data into billable and non-billable, then sort
fn split_and_sort_report_data(
    report_data: &HashMap<(i32, String, String), [f64; 5]>,
) -> (
    Vec<((String, String), [f64; 5])>,
    Vec<((i32, String, String), [f64; 5])>,
) {
    let mut billable_data: Vec<_> = report_data
        .iter()
        .filter(|((activity_value, _, _), _)| *activity_value == 1)
        .map(|((_, customer, work_item), hours)| ((customer.clone(), work_item.clone()), *hours))
        .collect();

    let mut non_billable_data: Vec<_> = report_data
        .iter()
        .filter(|((activity_value, _, _), _)| *activity_value != 1)
        .map(|((activity_value, customer, work_item), hours)| {
            (
                (*activity_value, customer.clone(), work_item.clone()),
                *hours,
            )
        })
        .collect();

    // Sort billable data by customer, then work item
    billable_data.sort_by(|a, b| match a.0 .0.cmp(&b.0 .0) {
        std::cmp::Ordering::Equal => a.0 .1.cmp(&b.0 .1),
        other => other,
    });

    // Sort non-billable data by activity value, customer, then work item
    non_billable_data.sort_by(|a, b| match a.0 .0.cmp(&b.0 .0) {
        std::cmp::Ordering::Equal => match a.0 .1.cmp(&b.0 .1) {
            std::cmp::Ordering::Equal => a.0 .2.cmp(&b.0 .2),
            other => other,
        },
        other => other,
    });

    (billable_data, non_billable_data)
}

/// Format report rows into text lines
fn format_report_rows(
    billable_data: &[((String, String), [f64; 5])],
    non_billable_data: &[((i32, String, String), [f64; 5])],
) -> Vec<String> {
    let mut lines = Vec::new();

    // Helper to format a row into text
    let format_row = |label: String, hours: [f64; 5]| -> String {
        let mut parts = Vec::new();
        parts.push(label);
        for hour in &hours {
            if *hour == 0.0 {
                parts.push(String::new());
            } else if *hour % 1.0 == 0.0 {
                parts.push(format!("{:.0}", hour));
            } else {
                parts.push(format!("{:.2}", hour));
            }
        }
        let total: f64 = hours.iter().sum();
        if total % 1.0 == 0.0 {
            parts.push(format!("{:.0}", total));
        } else {
            parts.push(format!("{:.2}", total));
        }
        parts.join("\t")
    };

    // Add billable rows
    for ((customer, work_item), hours) in billable_data {
        let label = format!("{} - {}", customer, work_item);
        lines.push(format_row(label, *hours));
    }

    // Add non-billable rows
    for ((activity_value, customer, work_item), hours) in non_billable_data {
        use crate::utils;
        let activity_name = utils::map_activity_value_to_name(*activity_value as u8);
        let label = if customer.is_empty() && work_item.is_empty() {
            activity_name
        } else if customer.is_empty() {
            format!("{} - {}", activity_name, work_item)
        } else if work_item.is_empty() {
            format!("{} - {}", activity_name, customer)
        } else {
            format!("{} - {} - {}", activity_name, customer, work_item)
        };
        lines.push(format_row(label, *hours));
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn create_test_entry(
        date: NaiveDate,
        activity_value: i32,
        customer: &str,
        work_item: &str,
        hours: f64,
    ) -> ClaimEntry {
        ClaimEntry {
            id: "test".to_string(),
            date,
            activity_type: "Test".to_string(),
            activity_value,
            customer: customer.to_string(),
            work_item: work_item.to_string(),
            hours,
            comment: None,
        }
    }

    #[test]
    fn test_build_report_data_empty() {
        let claims = vec![];
        let report_data = build_report_data(&claims);
        assert!(report_data.is_empty());
    }

    #[test]
    fn test_build_report_data_single_entry() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(); // Monday
        let claims = vec![create_test_entry(date, 1, "Customer A", "Item 1", 8.0)];

        let report_data = build_report_data(&claims);
        assert_eq!(report_data.len(), 1);

        let key = (1, "Customer A".to_string(), "Item 1".to_string());
        assert_eq!(report_data.get(&key).unwrap()[0], 8.0);
    }

    #[test]
    fn test_build_report_data_multiple_days() {
        let monday = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let tuesday = NaiveDate::from_ymd_opt(2024, 1, 2).unwrap();

        let claims = vec![
            create_test_entry(monday, 1, "Customer A", "Item 1", 4.0),
            create_test_entry(tuesday, 1, "Customer A", "Item 1", 4.0),
        ];

        let report_data = build_report_data(&claims);
        let key = (1, "Customer A".to_string(), "Item 1".to_string());
        let hours = report_data.get(&key).unwrap();

        assert_eq!(hours[0], 4.0); // Monday
        assert_eq!(hours[1], 4.0); // Tuesday
    }

    #[test]
    fn test_split_and_sort_report_data() {
        let mut report_data = HashMap::new();
        report_data.insert(
            (1, "Customer B".to_string(), "Item 2".to_string()),
            [1.0, 0.0, 0.0, 0.0, 0.0],
        );
        report_data.insert(
            (1, "Customer A".to_string(), "Item 1".to_string()),
            [2.0, 0.0, 0.0, 0.0, 0.0],
        );
        report_data.insert(
            (2, "Customer C".to_string(), "Item 3".to_string()),
            [3.0, 0.0, 0.0, 0.0, 0.0],
        );

        let (billable, non_billable) = split_and_sort_report_data(&report_data);

        // Check billable sorted by customer
        assert_eq!(billable.len(), 2);
        assert_eq!(billable[0].0 .0, "Customer A");
        assert_eq!(billable[1].0 .0, "Customer B");

        // Check non-billable
        assert_eq!(non_billable.len(), 1);
        assert_eq!(non_billable[0].0 .0, 2);
    }

    #[test]
    fn test_get_report_rows_text_empty() {
        let claims = vec![];
        let rows = get_report_rows_text(&claims).unwrap();
        assert!(rows.is_empty());
    }

    #[test]
    fn test_get_report_row_work_item() {
        let monday = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let claims = vec![
            create_test_entry(monday, 1, "Customer A", "Item 1", 8.0),
            create_test_entry(monday, 2, "Customer B", "Item 2", 4.0),
        ];

        let work_item = get_report_row_work_item(&claims, 0).unwrap();
        assert_eq!(work_item, "Item 1");

        let work_item = get_report_row_work_item(&claims, 1).unwrap();
        assert_eq!(work_item, "Item 2");
    }

    #[test]
    fn test_get_report_row_work_item_out_of_bounds() {
        let claims = vec![];
        let result = get_report_row_work_item(&claims, 0);
        assert!(result.is_err());
    }
}

// Made with Bob
