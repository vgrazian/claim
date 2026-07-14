use anyhow::{anyhow, Result};
use chrono::{DateTime, Datelike, Local, NaiveDate};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Represents a cached client and work item pair
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CachedEntry {
    pub customer: String,
    pub work_item: String,
    pub last_used: String, // ISO 8601 date string
}

/// Cached yearly presales usage for a specific opportunity code
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresalesOpportunityUsage {
    pub opportunity_code: String,
    pub year: i32,
    pub total_hours: f64,
    pub last_updated: String,
}

/// Cache structure for storing recent entries and yearly presales usage per user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryCache {
    pub entries: HashMap<i64, Vec<CachedEntry>>, // user_id -> entries
    #[serde(default)]
    pub presales_usage: HashMap<i64, Vec<PresalesOpportunityUsage>>, // user_id -> yearly usage
    pub last_updated: String,                    // ISO 8601 timestamp
}

impl EntryCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        EntryCache {
            entries: HashMap::new(),
            presales_usage: HashMap::new(),
            last_updated: Local::now().to_rfc3339(),
        }
    }

    /// Get the cache file path
    pub fn get_cache_path() -> Option<PathBuf> {
        ProjectDirs::from("com", "vgrazian", "claim")
            .map(|proj_dirs| proj_dirs.cache_dir().join("entries_cache.json"))
    }

    /// Load cache from disk
    pub fn load() -> Result<Self> {
        let cache_path =
            Self::get_cache_path().ok_or_else(|| anyhow!("Could not determine cache directory"))?;

        if !cache_path.exists() {
            return Ok(Self::new());
        }

        let cache_data = fs::read_to_string(&cache_path)
            .map_err(|e| anyhow!("Failed to read cache file: {}", e))?;

        let cache: EntryCache = serde_json::from_str(&cache_data)
            .map_err(|e| anyhow!("Failed to parse cache: {}", e))?;

        Ok(cache)
    }

    /// Save cache to disk
    pub fn save(&self) -> Result<()> {
        let cache_path =
            Self::get_cache_path().ok_or_else(|| anyhow!("Could not determine cache directory"))?;

        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| anyhow!("Failed to create cache directory: {}", e))?;
        }

        let cache_data = serde_json::to_string_pretty(self)
            .map_err(|e| anyhow!("Failed to serialize cache: {}", e))?;

        fs::write(&cache_path, cache_data)
            .map_err(|e| anyhow!("Failed to write cache file: {}", e))?;

        Ok(())
    }

    /// Add or update entries from query results for a specific user
    /// Deduplicates entries and keeps the most recent date
    pub fn update_from_items(&mut self, user_id: i64, items: &[(String, String, NaiveDate)]) {
        // Use a HashMap to deduplicate and keep the most recent date
        let mut entry_map: HashMap<(String, String), NaiveDate> = HashMap::new();

        // Add existing entries for this user to the map
        if let Some(user_entries) = self.entries.get(&user_id) {
            for entry in user_entries {
                if let Ok(date) = NaiveDate::parse_from_str(&entry.last_used, "%Y-%m-%d") {
                    let key = (entry.customer.clone(), entry.work_item.clone());
                    entry_map
                        .entry(key)
                        .and_modify(|existing_date| {
                            if date > *existing_date {
                                *existing_date = date;
                            }
                        })
                        .or_insert(date);
                }
            }
        }

        // Add new items to the map
        for (customer, work_item, date) in items {
            if !customer.is_empty() && !work_item.is_empty() {
                let key = (customer.clone(), work_item.clone());
                entry_map
                    .entry(key)
                    .and_modify(|existing_date| {
                        if *date > *existing_date {
                            *existing_date = *date;
                        }
                    })
                    .or_insert(*date);
            }
        }

        // Convert map back to vector and sort by date (most recent first)
        let mut entries: Vec<CachedEntry> = entry_map
            .into_iter()
            .map(|((customer, work_item), date)| CachedEntry {
                customer,
                work_item,
                last_used: date.format("%Y-%m-%d").to_string(),
            })
            .collect();

        entries.sort_by(|a, b| b.last_used.cmp(&a.last_used));

        self.entries.insert(user_id, entries);
        self.last_updated = Local::now().to_rfc3339();
    }

    /// Get entries sorted by most recent first for a specific user
    pub fn get_sorted_entries(&self, user_id: i64) -> Vec<CachedEntry> {
        if let Some(user_entries) = self.entries.get(&user_id) {
            let mut entries = user_entries.clone();
            entries.sort_by(|a, b| b.last_used.cmp(&a.last_used));
            entries
        } else {
            Vec::new()
        }
    }

    /// Get unique entries (deduplicated by customer + work_item) for a specific user
    /// Filters out test entries (TEST.DELETE.ME.*)
    /// Prepends a hardcoded PRESALES entry as option 0
    pub fn get_unique_entries(&self, user_id: i64) -> Vec<CachedEntry> {
        let mut seen = std::collections::HashSet::new();
        let mut unique = Vec::new();

        // Add hardcoded PRESALES entry as option 0
        unique.push(CachedEntry {
            customer: PRESALES_CUSTOMER.to_string(),
            work_item: PRESALES_WORK_ITEM.to_string(),
            last_used: Local::now().format("%Y-%m-%d").to_string(),
        });

        for entry in self.get_sorted_entries(user_id) {
            // Filter out test entries
            if entry.customer.starts_with("TEST") || entry.work_item.starts_with("TEST.DELETE.ME.")
            {
                continue;
            }

            let key = (entry.customer.clone(), entry.work_item.clone());
            if seen.insert(key) {
                unique.push(entry);
            }
        }

        unique
    }

    /// Add a single entry for a user (used after successful add operation)
    pub fn add_entry(
        &mut self,
        user_id: i64,
        customer: String,
        work_item: String,
        date: NaiveDate,
    ) {
        if customer.is_empty() || work_item.is_empty() {
            return;
        }

        let user_entries = self.entries.entry(user_id).or_default();

        // Check if this entry already exists
        let date_str = date.format("%Y-%m-%d").to_string();
        if let Some(existing) = user_entries
            .iter_mut()
            .find(|e| e.customer == customer && e.work_item == work_item)
        {
            // Update the date if newer
            if date_str > existing.last_used {
                existing.last_used = date_str;
            }
        } else {
            // Add new entry
            user_entries.push(CachedEntry {
                customer,
                work_item,
                last_used: date_str,
            });
        }

        // Sort by most recent first
        user_entries.sort_by(|a, b| b.last_used.cmp(&a.last_used));

        self.last_updated = Local::now().to_rfc3339();
    }

    /// Store yearly presales opportunity totals for a user.
    pub fn set_presales_usage(
        &mut self,
        user_id: i64,
        usage: Vec<PresalesOpportunityUsage>,
    ) {
        self.presales_usage.insert(user_id, usage);
        self.last_updated = Local::now().to_rfc3339();
    }

    /// Get yearly presales opportunity totals for a user.
    pub fn get_presales_usage(&self, user_id: i64) -> Vec<PresalesOpportunityUsage> {
        self.presales_usage
            .get(&user_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Check if cache is stale (older than specified hours)
    pub fn is_stale(&self, hours: i64) -> bool {
        if let Ok(last_updated) = DateTime::parse_from_rfc3339(&self.last_updated) {
            let now = Local::now();
            let duration = now.signed_duration_since(last_updated);
            duration.num_hours() > hours
        } else {
            true // If we can't parse the date, consider it stale
        }
    }

    /// Check if yearly presales usage is stale for a user-year.
    pub fn is_presales_usage_stale(&self, user_id: i64, year: i32, weeks: i64) -> bool {
        let Some(entries) = self.presales_usage.get(&user_id) else {
            return true;
        };

        let Some(latest) = entries
            .iter()
            .filter(|entry| entry.year == year)
            .filter_map(|entry| DateTime::parse_from_rfc3339(&entry.last_updated).ok())
            .max()
        else {
            return true;
        };

        let duration = Local::now().signed_duration_since(latest);
        duration.num_weeks() >= weeks
    }

    /// Clear all entries for all users
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.entries.clear();
        self.presales_usage.clear();
        self.last_updated = Local::now().to_rfc3339();
    }

    /// Clear entries for a specific user
    #[allow(dead_code)]
    pub fn clear_user(&mut self, user_id: i64) {
        self.entries.remove(&user_id);
        self.presales_usage.remove(&user_id);
        self.last_updated = Local::now().to_rfc3339();
    }
}

pub const PRESALES_CUSTOMER: &str = "PRESALES";
pub const PRESALES_WORK_ITEM: &str = "M.34212";
pub const PRESALES_OPPORTUNITY_HOURS_LIMIT: f64 = 24.0;

pub fn is_presales_opportunity_entry(customer: &str, work_item: &str) -> bool {
    customer == PRESALES_CUSTOMER && work_item == PRESALES_WORK_ITEM
}

pub fn is_valid_opportunity_code(comment: &str) -> bool {
    comment.len() == 18 && comment.chars().all(|c| c.is_ascii_alphanumeric())
}

pub fn calculate_presales_yearly_usage(
    items: &[crate::monday::Item],
    year: i32,
) -> Vec<PresalesOpportunityUsage> {
    let mut usage: HashMap<String, f64> = HashMap::new();

    for item in items {
        let date = item.column_values.iter().find_map(|cv| {
            if cv.id.as_deref() == Some("date4") {
                cv.text
                    .as_ref()
                    .and_then(|text| NaiveDate::parse_from_str(text, "%Y-%m-%d").ok())
            } else {
                None
            }
        });

        let Some(date) = date else {
            continue;
        };

        if date.year() != year {
            continue;
        }

        let customer = item
            .column_values
            .iter()
            .find(|cv| cv.id.as_deref() == Some("text__1"))
            .and_then(|cv| cv.text.clone())
            .unwrap_or_default();
        let work_item = item
            .column_values
            .iter()
            .find(|cv| cv.id.as_deref() == Some("text8__1"))
            .and_then(|cv| cv.text.clone())
            .unwrap_or_default();
        let comment = item
            .column_values
            .iter()
            .find(|cv| cv.id.as_deref() == Some("text2__1") || cv.id.as_deref() == Some("long_text"))
            .and_then(|cv| cv.text.clone())
            .filter(|text| !text.is_empty() && text != "null")
            .unwrap_or_default();
        let hours = item
            .column_values
            .iter()
            .find(|cv| cv.id.as_deref() == Some("numbers__1"))
            .and_then(|cv| cv.text.as_ref())
            .and_then(|text| text.parse::<f64>().ok())
            .unwrap_or(0.0);

        if !is_presales_opportunity_entry(&customer, &work_item) || !is_valid_opportunity_code(&comment) {
            continue;
        }

        *usage.entry(comment).or_insert(0.0) += hours;
    }

    let now = Local::now().to_rfc3339();
    let mut results: Vec<_> = usage
        .into_iter()
        .map(|(opportunity_code, total_hours)| PresalesOpportunityUsage {
            opportunity_code,
            year,
            total_hours,
            last_updated: now.clone(),
        })
        .collect();
    results.sort_by(|a, b| a.opportunity_code.cmp(&b.opportunity_code));
    results
}

impl Default for EntryCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monday::{ColumnValue, Item};

    const TEST_USER_ID: i64 = 12345;

    #[test]
    fn test_cache_new() {
        let cache = EntryCache::new();
        assert!(cache.entries.is_empty());
        assert!(!cache.last_updated.is_empty());
    }

    #[test]
    fn test_update_from_items() {
        let mut cache = EntryCache::new();
        let date1 = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2025, 1, 20).unwrap();

        let items = vec![
            ("Customer A".to_string(), "WI-001".to_string(), date1),
            ("Customer B".to_string(), "WI-002".to_string(), date2),
            ("Customer A".to_string(), "WI-001".to_string(), date2), // Duplicate with newer date
        ];

        cache.update_from_items(TEST_USER_ID, &items);

        let user_entries = cache.entries.get(&TEST_USER_ID).unwrap();
        assert_eq!(user_entries.len(), 2);
        // Should keep the most recent date for Customer A + WI-001
        let entry_a = user_entries
            .iter()
            .find(|e| e.customer == "Customer A")
            .unwrap();
        assert_eq!(entry_a.last_used, "2025-01-20");
    }

    #[test]
    fn test_get_sorted_entries() {
        let mut cache = EntryCache::new();
        let date1 = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2025, 1, 20).unwrap();

        let items = vec![
            ("Customer A".to_string(), "WI-001".to_string(), date1),
            ("Customer B".to_string(), "WI-002".to_string(), date2),
        ];

        cache.update_from_items(TEST_USER_ID, &items);
        let sorted = cache.get_sorted_entries(TEST_USER_ID);

        // Most recent should be first
        assert_eq!(sorted[0].customer, "Customer B");
        assert_eq!(sorted[1].customer, "Customer A");
    }

    #[test]
    fn test_get_unique_entries() {
        let mut cache = EntryCache::new();
        let user_entries = vec![
            CachedEntry {
                customer: "Customer A".to_string(),
                work_item: "WI-001".to_string(),
                last_used: "2025-01-20".to_string(),
            },
            CachedEntry {
                customer: "Customer A".to_string(),
                work_item: "WI-001".to_string(),
                last_used: "2025-01-15".to_string(),
            },
            CachedEntry {
                customer: "Customer B".to_string(),
                work_item: "WI-002".to_string(),
                last_used: "2025-01-18".to_string(),
            },
        ];
        cache.entries.insert(TEST_USER_ID, user_entries);

        let unique = cache.get_unique_entries(TEST_USER_ID);
        // Should have 3 entries: PRESALES (hardcoded) + 2 unique user entries
        assert_eq!(unique.len(), 3);
        // First entry should be PRESALES
        assert_eq!(unique[0].customer, PRESALES_CUSTOMER);
        assert_eq!(unique[0].work_item, PRESALES_WORK_ITEM);
    }

    #[test]
    fn test_is_stale() {
        let mut cache = EntryCache::new();

        // Fresh cache should not be stale
        assert!(!cache.is_stale(24));

        // Set last_updated to 2 days ago
        let two_days_ago = Local::now() - chrono::Duration::days(2);
        cache.last_updated = two_days_ago.to_rfc3339();

        // Should be stale if checking for 24 hours
        assert!(cache.is_stale(24));

        // Should not be stale if checking for 72 hours
        assert!(!cache.is_stale(72));
    }

    #[test]
    fn test_clear() {
        let mut cache = EntryCache::new();
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();

        cache.update_from_items(
            TEST_USER_ID,
            &[("Customer A".to_string(), "WI-001".to_string(), date)],
        );

        assert_eq!(cache.entries.len(), 1);

        cache.clear();
        assert_eq!(cache.entries.len(), 0);
    }

    #[test]
    fn test_empty_customer_or_work_item_filtered() {
        let mut cache = EntryCache::new();
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();

        let items = vec![
            ("Customer A".to_string(), "WI-001".to_string(), date),
            ("".to_string(), "WI-002".to_string(), date), // Empty customer
            ("Customer B".to_string(), "".to_string(), date), // Empty work item
        ];

        cache.update_from_items(TEST_USER_ID, &items);

        // Should only have the valid entry
        let user_entries = cache.entries.get(&TEST_USER_ID).unwrap();
        assert_eq!(user_entries.len(), 1);
        assert_eq!(user_entries[0].customer, "Customer A");
    }

    #[test]
    fn test_add_entry() {
        let mut cache = EntryCache::new();
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();

        cache.add_entry(
            TEST_USER_ID,
            "Customer A".to_string(),
            "WI-001".to_string(),
            date,
        );

        let user_entries = cache.entries.get(&TEST_USER_ID).unwrap();
        assert_eq!(user_entries.len(), 1);
        assert_eq!(user_entries[0].customer, "Customer A");
        assert_eq!(user_entries[0].work_item, "WI-001");
    }

    #[test]
    fn test_add_entry_updates_existing() {
        let mut cache = EntryCache::new();
        let date1 = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2025, 1, 20).unwrap();

        cache.add_entry(
            TEST_USER_ID,
            "Customer A".to_string(),
            "WI-001".to_string(),
            date1,
        );
        cache.add_entry(
            TEST_USER_ID,
            "Customer A".to_string(),
            "WI-001".to_string(),
            date2,
        );

        let user_entries = cache.entries.get(&TEST_USER_ID).unwrap();
        assert_eq!(user_entries.len(), 1);
        assert_eq!(user_entries[0].last_used, "2025-01-20");
    }

    #[test]
    fn test_multiple_users() {
        let mut cache = EntryCache::new();
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();

        cache.add_entry(100, "Customer A".to_string(), "WI-001".to_string(), date);
        cache.add_entry(200, "Customer B".to_string(), "WI-002".to_string(), date);

        assert_eq!(cache.entries.len(), 2);
        // Each user gets PRESALES + their own entry = 2 entries
        assert_eq!(cache.get_unique_entries(100).len(), 2);
        assert_eq!(cache.get_unique_entries(200).len(), 2);
        // User 300 has no entries, but still gets PRESALES = 1 entry
        assert_eq!(cache.get_unique_entries(300).len(), 1);
    }

    #[test]
    fn test_is_valid_opportunity_code() {
        assert!(is_valid_opportunity_code("006gR0000063GeJQAU"));
        assert!(!is_valid_opportunity_code("006gR0000063GeJQA"));
        assert!(!is_valid_opportunity_code("006gR0000063GeJQA-"));
    }

    #[test]
    fn test_calculate_presales_yearly_usage() {
        let items = vec![
            Item {
                id: Some("1".to_string()),
                name: Some("one".to_string()),
                column_values: vec![
                    ColumnValue { id: Some("date4".to_string()), value: None, text: Some("2026-02-10".to_string()) },
                    ColumnValue { id: Some("text__1".to_string()), value: None, text: Some(PRESALES_CUSTOMER.to_string()) },
                    ColumnValue { id: Some("text8__1".to_string()), value: None, text: Some(PRESALES_WORK_ITEM.to_string()) },
                    ColumnValue { id: Some("text2__1".to_string()), value: None, text: Some("006gR0000063GeJQAU".to_string()) },
                    ColumnValue { id: Some("numbers__1".to_string()), value: None, text: Some("8".to_string()) },
                ],
            },
            Item {
                id: Some("2".to_string()),
                name: Some("two".to_string()),
                column_values: vec![
                    ColumnValue { id: Some("date4".to_string()), value: None, text: Some("2026-03-10".to_string()) },
                    ColumnValue { id: Some("text__1".to_string()), value: None, text: Some(PRESALES_CUSTOMER.to_string()) },
                    ColumnValue { id: Some("text8__1".to_string()), value: None, text: Some(PRESALES_WORK_ITEM.to_string()) },
                    ColumnValue { id: Some("text2__1".to_string()), value: None, text: Some("006gR0000063GeJQAU".to_string()) },
                    ColumnValue { id: Some("numbers__1".to_string()), value: None, text: Some("4".to_string()) },
                ],
            },
            Item {
                id: Some("3".to_string()),
                name: Some("three".to_string()),
                column_values: vec![
                    ColumnValue { id: Some("date4".to_string()), value: None, text: Some("2025-03-10".to_string()) },
                    ColumnValue { id: Some("text__1".to_string()), value: None, text: Some(PRESALES_CUSTOMER.to_string()) },
                    ColumnValue { id: Some("text8__1".to_string()), value: None, text: Some(PRESALES_WORK_ITEM.to_string()) },
                    ColumnValue { id: Some("text2__1".to_string()), value: None, text: Some("006gR000006239eQAA".to_string()) },
                    ColumnValue { id: Some("numbers__1".to_string()), value: None, text: Some("7.5".to_string()) },
                ],
            },
            Item {
                id: Some("4".to_string()),
                name: Some("four".to_string()),
                column_values: vec![
                    ColumnValue { id: Some("date4".to_string()), value: None, text: Some("2026-03-10".to_string()) },
                    ColumnValue { id: Some("text__1".to_string()), value: None, text: Some(PRESALES_CUSTOMER.to_string()) },
                    ColumnValue { id: Some("text8__1".to_string()), value: None, text: Some(PRESALES_WORK_ITEM.to_string()) },
                    ColumnValue { id: Some("text2__1".to_string()), value: None, text: Some("invalid".to_string()) },
                    ColumnValue { id: Some("numbers__1".to_string()), value: None, text: Some("3".to_string()) },
                ],
            },
        ];

        let usage = calculate_presales_yearly_usage(&items, 2026);
        assert_eq!(usage.len(), 1);
        assert_eq!(usage[0].opportunity_code, "006gR0000063GeJQAU");
        assert_eq!(usage[0].total_hours, 12.0);
    }
}

// Made with Bob
