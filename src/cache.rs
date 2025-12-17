use anyhow::{anyhow, Result};
use chrono::{DateTime, Local, NaiveDate};
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

/// Cache structure for storing recent entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryCache {
    pub entries: Vec<CachedEntry>,
    pub last_updated: String, // ISO 8601 timestamp
}

impl EntryCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        EntryCache {
            entries: Vec::new(),
            last_updated: Local::now().to_rfc3339(),
        }
    }

    /// Get the cache file path
    pub fn get_cache_path() -> Option<PathBuf> {
        ProjectDirs::from("com", "yourname", "claim")
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

    /// Add or update entries from query results
    /// Deduplicates entries and keeps the most recent date
    pub fn update_from_items(&mut self, items: &[(String, String, NaiveDate)]) {
        // Use a HashMap to deduplicate and keep the most recent date
        let mut entry_map: HashMap<(String, String), NaiveDate> = HashMap::new();

        // Add existing entries to the map
        for entry in &self.entries {
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

        self.entries = entries;
        self.last_updated = Local::now().to_rfc3339();
    }

    /// Get entries sorted by most recent first
    pub fn get_sorted_entries(&self) -> Vec<CachedEntry> {
        let mut entries = self.entries.clone();
        entries.sort_by(|a, b| b.last_used.cmp(&a.last_used));
        entries
    }

    /// Get unique entries (deduplicated by customer + work_item)
    pub fn get_unique_entries(&self) -> Vec<CachedEntry> {
        let mut seen = std::collections::HashSet::new();
        let mut unique = Vec::new();

        for entry in self.get_sorted_entries() {
            let key = (entry.customer.clone(), entry.work_item.clone());
            if seen.insert(key) {
                unique.push(entry);
            }
        }

        unique
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

    /// Clear all entries
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.entries.clear();
        self.last_updated = Local::now().to_rfc3339();
    }
}

impl Default for EntryCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        cache.update_from_items(&items);

        assert_eq!(cache.entries.len(), 2);
        // Should keep the most recent date for Customer A + WI-001
        let entry_a = cache
            .entries
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

        cache.update_from_items(&items);
        let sorted = cache.get_sorted_entries();

        // Most recent should be first
        assert_eq!(sorted[0].customer, "Customer B");
        assert_eq!(sorted[1].customer, "Customer A");
    }

    #[test]
    fn test_get_unique_entries() {
        let mut cache = EntryCache::new();
        cache.entries = vec![
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

        let unique = cache.get_unique_entries();
        assert_eq!(unique.len(), 2);
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

        cache.update_from_items(&vec![(
            "Customer A".to_string(),
            "WI-001".to_string(),
            date,
        )]);

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

        cache.update_from_items(&items);

        // Should only have the valid entry
        assert_eq!(cache.entries.len(), 1);
        assert_eq!(cache.entries[0].customer, "Customer A");
    }
}

// Made with Bob
