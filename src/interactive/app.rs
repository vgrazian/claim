//! Application state and logic for the interactive UI

use anyhow::Result;
use chrono::{Datelike, Local, NaiveDate};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::cache::EntryCache;
use crate::monday::{Item, MondayClient, MondayUser};
use crate::utils;

use super::form::FormData;
use super::messages::{Message, MessageType};

/// Application mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    /// Normal viewing/navigation mode
    Normal,
    /// Adding a new entry
    AddEntry,
    /// Editing an existing entry
    EditEntry,
    /// Confirming deletion
    DeleteEntry,
    /// Help screen
    Help,
    /// Report view
    Report,
}

/// Claim entry data structure
#[derive(Debug, Clone)]
pub struct ClaimEntry {
    pub id: String,
    pub date: NaiveDate,
    pub activity_type: String,
    pub activity_value: i32,
    pub customer: String,
    pub work_item: String,
    pub hours: f64,
    pub comment: Option<String>,
}

impl ClaimEntry {
    /// Create a ClaimEntry from a Monday.com Item
    pub fn from_item(item: &Item) -> Option<Self> {
        let date = extract_date_from_item(item)?;
        let activity_value = extract_activity_value_from_item(item);
        let activity_type = utils::map_activity_value_to_name(activity_value as u8);

        Some(ClaimEntry {
            id: item.id.clone().unwrap_or_default(),
            date,
            activity_type,
            activity_value,
            customer: extract_customer_from_item(item),
            work_item: extract_work_item_from_item(item),
            hours: extract_hours_from_item(item),
            comment: extract_comment_from_item(item),
        })
    }
}

/// Main application state
pub struct App {
    /// Current week start date (always Monday)
    pub current_week_start: NaiveDate,
    /// Selected day in the current week
    pub selected_day: Option<NaiveDate>,
    /// Index of selected entry on the selected day
    pub selected_entry_index: Option<usize>,
    /// All loaded claims
    pub claims: Vec<ClaimEntry>,
    /// Entry cache for autocomplete
    pub cache: EntryCache,
    /// Current application mode
    pub mode: AppMode,
    /// Messages to display
    pub messages: Vec<Message>,
    /// Monday.com client
    pub client: MondayClient,
    /// Current user
    pub user: MondayUser,
    /// Current year group ID (internal Monday.com ID)
    pub group_id: String,
    /// Current year (display name, e.g., "2025")
    pub current_year: String,
    /// Whether data is currently loading
    pub loading: bool,
    /// Loading message for spinner
    pub loading_message: String,
    /// Form data for add/edit operations
    pub form_data: Option<FormData>,
    /// ID of entry being edited (None for add mode)
    pub editing_entry_id: Option<String>,
    /// Week start for data loading
    pub week_start: NaiveDate,
    /// Selected row index in report mode (None means no selection)
    pub selected_report_row: Option<usize>,
}

impl App {
    /// Create a new App instance
    pub async fn new(client: MondayClient, user: MondayUser) -> Result<Self> {
        let today = Local::now().naive_local().date();
        let current_week_start = get_week_start(today);

        // Load cache
        let cache = EntryCache::load().unwrap_or_else(|_| EntryCache::new());

        // Get current year and group ID (need to do this before creating app)
        let current_year = today.format("%Y").to_string();
        let board = client.get_board_with_groups("6500270039", false).await?;
        let group_id = crate::utils::get_year_group_id(&board, &current_year);

        let mut app = App {
            current_week_start,
            selected_day: Some(today),
            selected_entry_index: None,
            claims: Vec::new(),
            cache,
            mode: AppMode::Normal,
            messages: vec![Message::new(
                MessageType::Info,
                "Initializing...".to_string(),
            )],
            client,
            user,
            group_id,
            current_year: current_year.clone(),
            loading: true,
            loading_message: "Refreshing cache...".to_string(),
            form_data: None,
            editing_entry_id: None,
            week_start: current_week_start,
            selected_report_row: None,
        };

        // Refresh cache on startup (like -r option)
        app.refresh_cache().await?;

        // Load initial data
        app.load_week_data().await?;

        Ok(app)
    }
    /// Refresh cache from Monday.com (like -r option)
    pub async fn refresh_cache(&mut self) -> Result<()> {
        self.loading = true;
        self.loading_message = "Refreshing cache from last 4 weeks...".to_string();

        let board_id = "6500270039";
        let current_year = utils::get_current_year().to_string();

        // Query last 4 weeks (28 days)
        let today = Local::now().naive_local().date();
        let start_date = today - chrono::Duration::days(28);

        // Get the group ID for the current year
        let board = self.client.get_board_with_groups(board_id, false).await?;
        let group_id = utils::get_year_group_id(&board, &current_year);

        // Query all items for the user in the current year
        let all_items = self
            .client
            .query_items_with_filters(
                board_id,
                &group_id,
                self.user.id,
                &[], // Empty date filter - get all items for the user
                500,
                false,
            )
            .await?;

        // Extract customer and work item pairs from items, filtering by date range and billable only
        let mut entries = Vec::new();
        for item in &all_items {
            let customer = extract_customer_from_item(item);
            let work_item = extract_work_item_from_item(item);
            let date = extract_date_from_item(item);
            let activity_value = extract_activity_value_from_item(item);

            // Only include billable entries (activity_value == 1)
            if activity_value == 1 && !customer.is_empty() && !work_item.is_empty() {
                if let Some(d) = date {
                    // Only include items within the last 4 weeks
                    if d >= start_date && d <= today {
                        entries.push((customer, work_item, d));
                    }
                }
            }
        }

        self.cache.update_from_items(&entries);
        self.cache.save()?;

        self.loading = false;
        self.messages.push(Message::new(
            MessageType::Success,
            format!(
                "Cache refreshed with {} unique entries",
                self.cache.get_unique_entries().len()
            ),
        ));

        Ok(())
    }

    /// Load data for the current week
    pub async fn load_week_data(&mut self) -> Result<()> {
        self.loading = true;
        self.messages.clear();
        self.messages.push(Message::new(
            MessageType::Info,
            "Loading week data...".to_string(),
        ));

        let board_id = "6500270039";
        let current_year = utils::get_current_year().to_string();

        // Get the board and group ID
        let board = self.client.get_board_with_groups(board_id, false).await?;
        let group_id = utils::get_year_group_id(&board, &current_year);

        // Calculate date range for the week (Monday to Friday)
        let dates = utils::calculate_working_dates(self.current_week_start, 5);

        // Convert dates to strings for the API
        let date_strings: Vec<String> = dates
            .iter()
            .map(|d| d.format("%Y-%m-%d").to_string())
            .collect();

        // Query items for the week
        let items = self
            .client
            .query_items_with_filters(board_id, &group_id, self.user.id, &date_strings, 100, false)
            .await?;

        // Convert items to ClaimEntry
        self.claims = items.iter().filter_map(ClaimEntry::from_item).collect();

        self.loading = false;
        self.messages.clear();
        self.messages.push(Message::new(
            MessageType::Success,
            format!(
                "Loaded {} entries for week of {}",
                self.claims.len(),
                self.current_week_start.format("%b %d, %Y")
            ),
        ));

        Ok(())
    }

    /// Handle keyboard events
    pub async fn handle_event(&mut self, event: KeyEvent) -> Result<bool> {
        match self.mode {
            AppMode::Normal => self.handle_normal_mode(event).await,
            AppMode::Help => self.handle_help_mode(event),
            AppMode::AddEntry => self.handle_add_mode(event).await,
            AppMode::EditEntry => self.handle_edit_mode(event).await,
            AppMode::DeleteEntry => self.handle_delete_mode(event).await,
            AppMode::Report => self.handle_report_mode(event).await,
        }
    }

    /// Handle events in normal mode
    async fn handle_normal_mode(&mut self, event: KeyEvent) -> Result<bool> {
        match event.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                return Ok(false); // Exit application
            }
            KeyCode::Char('?') => {
                self.mode = AppMode::Help;
            }
            // Tab: Navigate weeks forward
            KeyCode::Tab => {
                if event
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::SHIFT)
                {
                    self.previous_week().await?;
                } else {
                    self.next_week().await?;
                }
            }
            KeyCode::BackTab => {
                self.previous_week().await?;
            }
            // Arrow keys: Navigate days (left/right) and entries (up/down)
            KeyCode::Left => {
                self.select_previous_day();
            }
            KeyCode::Right => {
                self.select_next_day();
            }
            KeyCode::Up => {
                self.select_previous_entry();
            }
            KeyCode::Down => {
                self.select_next_entry();
            }
            // Number keys: Jump to specific day
            KeyCode::Char('1') => {
                self.select_day(0);
            }
            KeyCode::Char('2') => {
                self.select_day(1);
            }
            KeyCode::Char('3') => {
                self.select_day(2);
            }
            KeyCode::Char('4') => {
                self.select_day(3);
            }
            KeyCode::Char('5') => {
                self.select_day(4);
            }
            // Update data (refresh cache and reload)
            KeyCode::Char('u') | KeyCode::Char('U') => {
                self.refresh_cache().await?;
                self.load_week_data().await?;
            }
            // Show report view
            KeyCode::Char('p') | KeyCode::Char('P') => {
                self.mode = AppMode::Report;
                self.selected_report_row = Some(0); // Start with first row selected
            }
            // Add entry
            KeyCode::Char('a') | KeyCode::Char('A') => {
                self.start_add_mode();
            }
            // Edit entry (only if an entry is selected)
            KeyCode::Char('e') | KeyCode::Char('E') | KeyCode::Enter => {
                if self.selected_entry_index.is_some() {
                    self.start_edit_mode();
                }
            }
            // Delete entry
            KeyCode::Char('d') | KeyCode::Char('D') => {
                if self.selected_entry_index.is_some() {
                    self.mode = AppMode::DeleteEntry;
                    self.messages.clear();
                    self.messages.push(Message::new(
                        MessageType::Warning,
                        "⚠️  DELETE CONFIRMATION - Press 'y' to confirm, any other key to cancel"
                            .to_string(),
                    ));
                }
            }
            // Jump to current week
            KeyCode::Home => {
                let today = Local::now().naive_local().date();
                self.current_week_start = get_week_start(today);
                self.selected_day = Some(today);
                self.load_week_data().await?;
            }
            _ => {}
        }
        Ok(true)
    }

    /// Handle events in help mode
    fn handle_help_mode(&mut self, event: KeyEvent) -> Result<bool> {
        match event.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') | KeyCode::Char('h') => {
                self.mode = AppMode::Normal;
            }
            _ => {}
        }
        Ok(true)
    }

    /// Handle events in report mode
    async fn handle_report_mode(&mut self, event: KeyEvent) -> Result<bool> {
        match event.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('p') | KeyCode::Char('P') => {
                self.mode = AppMode::Normal;
                self.selected_report_row = None; // Clear selection when exiting
            }
            // Tab: Navigate weeks forward
            KeyCode::Tab => {
                if event
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::SHIFT)
                {
                    self.previous_week().await?;
                } else {
                    self.next_week().await?;
                }
                self.selected_report_row = Some(0); // Reset to first row after week change
            }
            KeyCode::BackTab => {
                self.previous_week().await?;
                self.selected_report_row = Some(0); // Reset to first row after week change
            }
            // Up/Down: Navigate rows
            KeyCode::Up => {
                if let Some(current) = self.selected_report_row {
                    if current > 0 {
                        self.selected_report_row = Some(current - 1);
                    }
                }
            }
            KeyCode::Down => {
                if let Some(current) = self.selected_report_row {
                    // Count total data rows (excluding header, separator, and total)
                    let billable_count = self
                        .claims
                        .iter()
                        .filter(|e| e.activity_value == 1)
                        .map(|e| (e.customer.clone(), e.work_item.clone()))
                        .collect::<std::collections::HashSet<_>>()
                        .len();
                    let non_billable_count = self
                        .claims
                        .iter()
                        .filter(|e| e.activity_value != 1)
                        .map(|e| (e.activity_value, e.customer.clone(), e.work_item.clone()))
                        .collect::<std::collections::HashSet<_>>()
                        .len();

                    let has_separator = non_billable_count > 0;
                    let max_row =
                        billable_count + non_billable_count + (if has_separator { 1 } else { 0 })
                            - 1;

                    if current < max_row {
                        self.selected_report_row = Some(current + 1);
                    }
                }
            }
            _ => {}
        }
        Ok(true)
    }

    /// Handle events in add mode
    async fn handle_add_mode(&mut self, event: KeyEvent) -> Result<bool> {
        if let Some(form) = &mut self.form_data {
            match event.code {
                KeyCode::Esc => {
                    self.form_data = None;
                    self.mode = AppMode::Normal;
                    self.messages.clear();
                    self.messages
                        .push(Message::new(MessageType::Info, "Add cancelled".to_string()));
                }
                KeyCode::Tab => {
                    if event
                        .modifiers
                        .contains(crossterm::event::KeyModifiers::SHIFT)
                    {
                        form.previous_field();
                    } else {
                        if form.focus_on_cache {
                            form.focus_on_cache = false;
                        } else {
                            form.toggle_focus();
                        }
                    }
                }
                KeyCode::Left => {
                    if !form.focus_on_cache {
                        form.move_cursor_left();
                    }
                }
                KeyCode::Right => {
                    if !form.focus_on_cache {
                        form.move_cursor_right();
                    }
                }
                KeyCode::Home => {
                    if !form.focus_on_cache {
                        form.move_cursor_to_start();
                    }
                }
                KeyCode::End => {
                    if !form.focus_on_cache {
                        form.move_cursor_to_end();
                    }
                }
                KeyCode::Up => {
                    if form.focus_on_cache {
                        if form.selected_cache_index > 0 {
                            form.selected_cache_index -= 1;
                        }
                    } else {
                        form.previous_field();
                        form.update_cursor_for_field();
                    }
                }
                KeyCode::Down => {
                    if form.focus_on_cache {
                        let cache_size = self.cache.get_unique_entries().len();
                        if form.selected_cache_index < cache_size.saturating_sub(1) {
                            form.selected_cache_index += 1;
                        }
                    } else {
                        form.next_field();
                        form.update_cursor_for_field();
                    }
                }
                KeyCode::Enter => {
                    if form.focus_on_cache {
                        // Apply selected cache entry
                        let entries = self.cache.get_unique_entries();
                        if let Some(entry) = entries.get(form.selected_cache_index) {
                            form.apply_cache_entry(entry.customer.clone(), entry.work_item.clone());
                        }
                    } else {
                        // Save the form
                        match form.validate() {
                            Ok(_) => {
                                // Clone form data before async call
                                let form_clone = form.clone();
                                self.form_data = None;
                                self.mode = AppMode::Normal;
                                self.messages.clear();

                                // Save to Monday.com
                                let result = self.save_new_entry(&form_clone).await;

                                match result {
                                    Ok(_) => {
                                        self.messages.push(Message::new(
                                            MessageType::Success,
                                            "Entry added successfully".to_string(),
                                        ));
                                        // Refresh week data to show new entry
                                        let _ = self.load_week_data().await;
                                    }
                                    Err(e) => {
                                        self.messages.push(Message::new(
                                            MessageType::Error,
                                            format!("Failed to add entry: {}", e),
                                        ));
                                    }
                                }
                            }
                            Err(err) => {
                                self.messages.clear();
                                self.messages.push(Message::new(
                                    MessageType::Error,
                                    format!("Validation error: {}", err),
                                ));
                            }
                        }
                    }
                }
                KeyCode::Char(c) => {
                    // Handle number keys for selection
                    if c.is_ascii_digit() {
                        let digit = c.to_digit(10).unwrap() as usize;

                        // Check current field context
                        match form.current_field {
                            super::form::FormField::ActivityType => {
                                // Select activity type by number
                                if let Some(activity_name) =
                                    super::activity_types::get_activity_type_by_number(digit as u8)
                                {
                                    form.activity_type = activity_name.to_string();
                                }
                            }
                            super::form::FormField::Customer | super::form::FormField::WorkItem => {
                                // Select from cache by number
                                let entries = self.cache.get_unique_entries();
                                if digit < entries.len() && digit < 10 {
                                    let entry = &entries[digit];
                                    form.apply_cache_entry(
                                        entry.customer.clone(),
                                        entry.work_item.clone(),
                                    );
                                }
                            }
                            _ => {
                                // For other fields, just add the character
                                if !form.focus_on_cache {
                                    form.insert_char(c);
                                }
                            }
                        }
                    } else {
                        // Non-digit character - add to current field
                        if !form.focus_on_cache {
                            form.insert_char(c);
                        }
                    }
                }
                KeyCode::Backspace => {
                    if !form.focus_on_cache {
                        form.delete_char_before();
                    }
                }
                KeyCode::Delete => {
                    if !form.focus_on_cache {
                        form.delete_char_at();
                    }
                }
                _ => {}
            }
        }
        Ok(true)
    }

    /// Handle events in edit mode
    async fn handle_edit_mode(&mut self, event: KeyEvent) -> Result<bool> {
        if let Some(form) = &mut self.form_data {
            match event.code {
                KeyCode::Esc => {
                    self.form_data = None;
                    self.editing_entry_id = None;
                    self.mode = AppMode::Normal;
                    self.messages.clear();
                    self.messages.push(Message::new(
                        MessageType::Info,
                        "Edit cancelled".to_string(),
                    ));
                }
                KeyCode::Tab => {
                    if event
                        .modifiers
                        .contains(crossterm::event::KeyModifiers::SHIFT)
                    {
                        form.previous_field();
                    } else {
                        if form.focus_on_cache {
                            form.focus_on_cache = false;
                        } else {
                            form.toggle_focus();
                        }
                    }
                }
                KeyCode::Left => {
                    if !form.focus_on_cache {
                        form.move_cursor_left();
                    }
                }
                KeyCode::Right => {
                    if !form.focus_on_cache {
                        form.move_cursor_right();
                    }
                }
                KeyCode::Home => {
                    if !form.focus_on_cache {
                        form.move_cursor_to_start();
                    }
                }
                KeyCode::End => {
                    if !form.focus_on_cache {
                        form.move_cursor_to_end();
                    }
                }
                KeyCode::Up => {
                    if form.focus_on_cache {
                        if form.selected_cache_index > 0 {
                            form.selected_cache_index -= 1;
                        }
                    } else {
                        form.previous_field();
                        form.update_cursor_for_field();
                    }
                }
                KeyCode::Down => {
                    if form.focus_on_cache {
                        let cache_size = self.cache.get_unique_entries().len();
                        if form.selected_cache_index < cache_size.saturating_sub(1) {
                            form.selected_cache_index += 1;
                        }
                    } else {
                        form.next_field();
                        form.update_cursor_for_field();
                    }
                }
                KeyCode::Enter => {
                    if form.focus_on_cache {
                        // Apply selected cache entry
                        let entries = self.cache.get_unique_entries();
                        if let Some(entry) = entries.get(form.selected_cache_index) {
                            form.apply_cache_entry(entry.customer.clone(), entry.work_item.clone());
                        }
                    } else {
                        // Save the form
                        match form.validate() {
                            Ok(_) => {
                                // Clone form data and entry ID before async call
                                let form_clone = form.clone();
                                let entry_id_clone = self.editing_entry_id.clone();
                                self.form_data = None;
                                self.editing_entry_id = None;
                                self.mode = AppMode::Normal;
                                self.messages.clear();

                                // Update on Monday.com
                                let result = self.update_entry(&form_clone, &entry_id_clone).await;

                                match result {
                                    Ok(_) => {
                                        self.messages.push(Message::new(
                                            MessageType::Success,
                                            "Entry updated successfully".to_string(),
                                        ));
                                        // Refresh week data to show updated entry
                                        let _ = self.load_week_data().await;
                                    }
                                    Err(e) => {
                                        self.messages.push(Message::new(
                                            MessageType::Error,
                                            format!("Failed to update entry: {}", e),
                                        ));
                                    }
                                }
                            }
                            Err(err) => {
                                self.messages.clear();
                                self.messages.push(Message::new(
                                    MessageType::Error,
                                    format!("Validation error: {}", err),
                                ));
                            }
                        }
                    }
                }
                KeyCode::Char(c) => {
                    // Handle number keys for selection
                    if c.is_ascii_digit() {
                        let digit = c.to_digit(10).unwrap() as usize;

                        // Check current field context
                        match form.current_field {
                            super::form::FormField::ActivityType => {
                                // Select activity type by number
                                if let Some(activity_name) =
                                    super::activity_types::get_activity_type_by_number(digit as u8)
                                {
                                    form.activity_type = activity_name.to_string();
                                }
                            }
                            super::form::FormField::Customer | super::form::FormField::WorkItem => {
                                // Select from cache by number
                                let entries = self.cache.get_unique_entries();
                                if digit < entries.len() && digit < 10 {
                                    let entry = &entries[digit];
                                    form.apply_cache_entry(
                                        entry.customer.clone(),
                                        entry.work_item.clone(),
                                    );
                                }
                            }
                            _ => {
                                // For other fields, just add the character
                                if !form.focus_on_cache {
                                    form.insert_char(c);
                                }
                            }
                        }
                    } else {
                        // Non-digit character - add to current field
                        if !form.focus_on_cache {
                            form.insert_char(c);
                        }
                    }
                }
                KeyCode::Backspace => {
                    if !form.focus_on_cache {
                        form.delete_char_before();
                    }
                }
                KeyCode::Delete => {
                    if !form.focus_on_cache {
                        form.delete_char_at();
                    }
                }
                _ => {}
            }
        }
        Ok(true)
    }

    /// Handle events in delete mode
    async fn handle_delete_mode(&mut self, event: KeyEvent) -> Result<bool> {
        match event.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                // Get the selected entry ID
                if let Some(day) = self.selected_day {
                    if let Some(idx) = self.selected_entry_index {
                        let entries_on_day: Vec<_> =
                            self.claims.iter().filter(|e| e.date == day).collect();

                        if let Some(entry) = entries_on_day.get(idx) {
                            let entry_id = entry.id.clone();

                            // Delete from Monday.com
                            self.mode = AppMode::Normal;
                            self.messages.clear();

                            match self.client.delete_item(&entry_id, false).await {
                                Ok(_) => {
                                    self.messages.push(Message::new(
                                        MessageType::Success,
                                        "Entry deleted successfully".to_string(),
                                    ));

                                    // Refresh week data to update the view
                                    let _ = self.load_week_data().await;

                                    // Adjust selection if needed
                                    let remaining_entries: Vec<_> =
                                        self.claims.iter().filter(|e| e.date == day).collect();

                                    if remaining_entries.is_empty() {
                                        self.selected_entry_index = None;
                                    } else if idx >= remaining_entries.len() {
                                        self.selected_entry_index =
                                            Some(remaining_entries.len() - 1);
                                    }
                                }
                                Err(e) => {
                                    self.messages.push(Message::new(
                                        MessageType::Error,
                                        format!("Failed to delete entry: {}", e),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
            _ => {
                // Any other key cancels the delete
                self.mode = AppMode::Normal;
                self.messages.clear();
                self.messages.push(Message::new(
                    MessageType::Info,
                    "Delete cancelled".to_string(),
                ));
            }
        }
        Ok(true)
    }

    /// Navigate to previous week
    async fn previous_week(&mut self) -> Result<()> {
        self.current_week_start = self.current_week_start - chrono::Duration::days(7);
        self.selected_day = Some(self.current_week_start);
        self.selected_entry_index = None;
        self.load_week_data().await
    }

    /// Navigate to next week
    async fn next_week(&mut self) -> Result<()> {
        self.current_week_start = self.current_week_start + chrono::Duration::days(7);
        self.selected_day = Some(self.current_week_start);
        self.selected_entry_index = None;
        self.load_week_data().await
    }

    /// Select a specific day of the week (0 = Monday, 4 = Friday)
    fn select_day(&mut self, day_offset: i64) {
        self.selected_day = Some(self.current_week_start + chrono::Duration::days(day_offset));
        self.selected_entry_index = None;
    }
    /// Select the previous day in the week
    fn select_previous_day(&mut self) {
        if let Some(current_day) = self.selected_day {
            let weekday = current_day.weekday().num_days_from_monday();
            if weekday > 0 {
                self.selected_day = Some(current_day - chrono::Duration::days(1));
                self.selected_entry_index = None;
            }
        }
    }

    /// Select the next day in the week
    fn select_next_day(&mut self) {
        if let Some(current_day) = self.selected_day {
            let weekday = current_day.weekday().num_days_from_monday();
            if weekday < 4 {
                // Friday is day 4
                self.selected_day = Some(current_day + chrono::Duration::days(1));
                self.selected_entry_index = None;
            }
        }
    }

    /// Select the previous entry on the current day
    fn select_previous_entry(&mut self) {
        if let Some(day) = self.selected_day {
            let entries_on_day: Vec<_> = self
                .claims
                .iter()
                .enumerate()
                .filter(|(_, e)| e.date == day)
                .collect();

            if !entries_on_day.is_empty() {
                if let Some(current_idx) = self.selected_entry_index {
                    if current_idx > 0 {
                        self.selected_entry_index = Some(current_idx - 1);
                    }
                } else {
                    self.selected_entry_index = Some(entries_on_day.len() - 1);
                }
            }
        }
    }

    /// Select the next entry on the current day
    fn select_next_entry(&mut self) {
        if let Some(day) = self.selected_day {
            let entries_on_day: Vec<_> = self
                .claims
                .iter()
                .enumerate()
                .filter(|(_, e)| e.date == day)
                .collect();

            if !entries_on_day.is_empty() {
                if let Some(current_idx) = self.selected_entry_index {
                    if current_idx < entries_on_day.len() - 1 {
                        self.selected_entry_index = Some(current_idx + 1);
                    }
                } else {
                    self.selected_entry_index = Some(0);
                }
            }
        }
    }

    /// Get entries for a specific date
    pub fn get_entries_for_date(&self, date: NaiveDate) -> Vec<&ClaimEntry> {
        self.claims.iter().filter(|e| e.date == date).collect()
    }

    /// Get total hours for the current week
    pub fn get_week_total_hours(&self) -> f64 {
        self.claims.iter().map(|e| e.hours).sum()
    }

    /// Start add mode with empty form
    fn start_add_mode(&mut self) {
        let mut form = FormData::new();

        // Set date to selected day or today
        if let Some(day) = self.selected_day {
            form.date = day.format("%Y-%m-%d").to_string();
        } else {
            form.date = Local::now()
                .naive_local()
                .date()
                .format("%Y-%m-%d")
                .to_string();
        }

        self.form_data = Some(form);
        self.editing_entry_id = None;
        self.mode = AppMode::AddEntry;
        self.messages.clear();
        self.messages.push(Message::new(
            MessageType::Info,
            "Add mode - Tab to navigate fields, Enter to save, Esc to cancel".to_string(),
        ));
    }

    /// Start edit mode with selected entry data
    fn start_edit_mode(&mut self) {
        if let Some(day) = self.selected_day {
            if let Some(idx) = self.selected_entry_index {
                let entries_on_day: Vec<_> = self.claims.iter().filter(|e| e.date == day).collect();

                if let Some(entry) = entries_on_day.get(idx) {
                    let form = FormData::from_entry(
                        entry.date,
                        entry.activity_type.clone(),
                        entry.customer.clone(),
                        entry.work_item.clone(),
                        entry.hours,
                        entry.comment.clone(),
                    );

                    self.form_data = Some(form);
                    self.editing_entry_id = Some(entry.id.clone());
                    self.mode = AppMode::EditEntry;
                    self.messages.clear();
                    self.messages.push(Message::new(
                        MessageType::Info,
                        "Edit mode - Tab to navigate fields, Enter to save, Esc to cancel"
                            .to_string(),
                    ));
                }
            }
        }
    }

    /// Get the currently selected entry for editing
    pub fn get_selected_entry(&self) -> Option<&ClaimEntry> {
        if let Some(day) = self.selected_day {
            if let Some(idx) = self.selected_entry_index {
                let entries_on_day: Vec<_> = self.claims.iter().filter(|e| e.date == day).collect();
                return entries_on_day.get(idx).copied();
            }
        }
        None
    }

    /// Save a new entry to Monday.com
    async fn save_new_entry(&self, form: &FormData) -> Result<()> {
        use crate::utils::map_activity_type_to_value;
        use serde_json::json;

        let activity_value = map_activity_type_to_value(&form.activity_type);
        let date_str = &form.date; // date is already a String in YYYY-MM-DD format

        let mut column_values = json!({});

        // Set person column
        column_values["person"] = json!({
            "personsAndTeams": [
                {
                    "id": self.user.id,
                    "kind": "person"
                }
            ]
        });

        // Set date column
        column_values["date4"] = json!({
            "date": date_str.clone()
        });

        // Set activity type column
        column_values["status"] = json!({
            "index": activity_value
        });

        // Set customer name
        if !form.customer.is_empty() {
            column_values["text__1"] = json!(form.customer);
        }

        // Set work item
        if !form.work_item.is_empty() {
            column_values["text8__1"] = json!(form.work_item);
        }

        // Set comment
        if !form.comment.is_empty() {
            column_values["text2__1"] = json!(form.comment);
        }

        // Set hours
        column_values["numbers__1"] = json!(form.hours.to_string());

        // Create the item
        self.client
            .create_item_verbose(
                "6500270039",
                &self.group_id,
                &self.user.name,
                &column_values,
                false,
            )
            .await?;

        Ok(())
    }

    /// Update an existing entry on Monday.com
    async fn update_entry(&self, form: &FormData, entry_id: &Option<String>) -> Result<()> {
        use crate::utils::map_activity_type_to_value;
        use serde_json::json;

        // Get the entry ID
        let entry_id = entry_id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No entry ID for update"))?;

        let activity_value = map_activity_type_to_value(&form.activity_type);
        let date_str = &form.date; // date is already a String in YYYY-MM-DD format

        let mut column_values = json!({});

        // Set date column
        column_values["date4"] = json!({
            "date": date_str.clone()
        });

        // Set activity type column
        column_values["status"] = json!({
            "index": activity_value
        });

        // Set customer name
        if !form.customer.is_empty() {
            column_values["text__1"] = json!(form.customer);
        }

        // Set work item
        if !form.work_item.is_empty() {
            column_values["text8__1"] = json!(form.work_item);
        }

        // Set comment
        if !form.comment.is_empty() {
            column_values["text2__1"] = json!(form.comment);
        }

        // Set hours
        column_values["numbers__1"] = json!(form.hours.to_string());

        // Update the item
        self.client
            .update_item_verbose(entry_id, &column_values, false)
            .await?;

        Ok(())
    }

    /// Reload data from Monday.com
    async fn load_data(&mut self) -> Result<()> {
        self.loading = true;

        // Query items for the current week
        let start_date = self.current_week_start;
        let end_date = start_date + chrono::Duration::days(4); // Monday to Friday

        // Convert dates to strings for the API
        let start_str = start_date.format("%Y-%m-%d").to_string();
        let end_str = end_date.format("%Y-%m-%d").to_string();

        let items = self
            .client
            .query_items_with_filters(
                "6500270039",
                &self.group_id,
                self.user.id,
                &[start_str, end_str],
                500,
                false,
            )
            .await?;

        // Convert items to ClaimEntry
        self.claims.clear();
        for item in items {
            if let Some(date) = extract_date_from_item(&item) {
                let activity_value = extract_activity_value_from_item(&item);
                let activity_type = utils::map_activity_value_to_name(activity_value as u8);
                let customer = extract_customer_from_item(&item);
                let work_item = extract_work_item_from_item(&item);
                let comment = extract_comment_from_item(&item);
                let hours = extract_hours_from_item(&item);
                let id = item.id.unwrap_or_default();

                self.claims.push(ClaimEntry {
                    id,
                    date,
                    activity_type,
                    activity_value,
                    customer,
                    work_item,
                    comment,
                    hours,
                });
            }
        }

        self.loading = false;
        Ok(())
    }
}
/// Get the Monday of the week containing the given date
fn get_week_start(date: NaiveDate) -> NaiveDate {
    let weekday = date.weekday().num_days_from_monday();
    date - chrono::Duration::days(weekday as i64)
}

// Helper functions to extract data from Monday.com items

fn extract_date_from_item(item: &Item) -> Option<NaiveDate> {
    for col in &item.column_values {
        if col.id.as_deref() == Some("date4") {
            if let Some(text) = &col.text {
                if let Ok(date) = NaiveDate::parse_from_str(text, "%Y-%m-%d") {
                    return Some(date);
                }
            }
        }
    }
    None
}

fn extract_activity_value_from_item(item: &Item) -> i32 {
    for col in &item.column_values {
        if col.id.as_deref() == Some("status") {
            // Parse from the value field which contains JSON like {"index": 1}
            if let Some(value) = &col.value {
                if let Ok(parsed_value) = serde_json::from_str::<serde_json::Value>(value) {
                    if let Some(status_index) = parsed_value.get("index") {
                        if let Some(index_num) = status_index.as_i64() {
                            return index_num as i32;
                        }
                    }
                }
            }
        }
    }
    1 // Default to billable
}

fn extract_customer_from_item(item: &Item) -> String {
    for col in &item.column_values {
        if col.id.as_deref() == Some("text__1") {
            if let Some(text) = &col.text {
                return text.clone();
            }
        }
    }
    String::new()
}

fn extract_work_item_from_item(item: &Item) -> String {
    for col in &item.column_values {
        if col.id.as_deref() == Some("text8__1") {
            if let Some(text) = &col.text {
                return text.clone();
            }
        }
    }
    String::new()
}

fn extract_hours_from_item(item: &Item) -> f64 {
    for col in &item.column_values {
        if col.id.as_deref() == Some("numbers__1") {
            if let Some(text) = &col.text {
                return text.parse().unwrap_or(0.0);
            }
        }
    }
    0.0
}

fn extract_comment_from_item(item: &Item) -> Option<String> {
    for col in &item.column_values {
        if col.id.as_deref() == Some("text") {
            if let Some(text) = &col.text {
                if !text.is_empty() {
                    return Some(text.clone());
                }
            }
        }
    }
    None
}

// Made with Bob
