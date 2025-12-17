//! Form handling for add/edit operations

use chrono::NaiveDate;

/// Form field types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormField {
    Date,
    ActivityType,
    Customer,
    WorkItem,
    Hours,
    Comment,
}

impl FormField {
    /// Get all fields in order
    pub fn all() -> Vec<FormField> {
        vec![
            FormField::Date,
            FormField::ActivityType,
            FormField::Customer,
            FormField::WorkItem,
            FormField::Hours,
            FormField::Comment,
        ]
    }

    /// Get the next field
    pub fn next(&self) -> FormField {
        match self {
            FormField::Date => FormField::ActivityType,
            FormField::ActivityType => FormField::Customer,
            FormField::Customer => FormField::WorkItem,
            FormField::WorkItem => FormField::Hours,
            FormField::Hours => FormField::Comment,
            FormField::Comment => FormField::Date,
        }
    }

    /// Get the previous field
    pub fn previous(&self) -> FormField {
        match self {
            FormField::Date => FormField::Comment,
            FormField::ActivityType => FormField::Date,
            FormField::Customer => FormField::ActivityType,
            FormField::WorkItem => FormField::Customer,
            FormField::Hours => FormField::WorkItem,
            FormField::Comment => FormField::Hours,
        }
    }

    /// Get field label
    pub fn label(&self) -> &'static str {
        match self {
            FormField::Date => "Date",
            FormField::ActivityType => "Activity Type",
            FormField::Customer => "Customer",
            FormField::WorkItem => "Work Item",
            FormField::Hours => "Hours",
            FormField::Comment => "Comment",
        }
    }
}

/// Form data for add/edit operations
#[derive(Debug, Clone)]
pub struct FormData {
    pub date: String,
    pub activity_type: String,
    pub customer: String,
    pub work_item: String,
    pub hours: String,
    pub comment: String,
    pub current_field: FormField,
    pub focus_on_cache: bool,
    pub selected_cache_index: usize,
    pub cursor_position: usize,
}

impl FormData {
    /// Create new empty form
    pub fn new() -> Self {
        Self {
            date: String::new(),
            activity_type: "billable".to_string(),
            customer: String::new(),
            work_item: String::new(),
            hours: "8".to_string(),
            comment: String::new(),
            current_field: FormField::Date,
            focus_on_cache: false,
            selected_cache_index: 0,
            cursor_position: 0,
        }
    }

    /// Create form from existing entry
    pub fn from_entry(
        date: NaiveDate,
        activity_type: String,
        customer: String,
        work_item: String,
        hours: f64,
        comment: Option<String>,
    ) -> Self {
        let date_str = date.format("%Y-%m-%d").to_string();
        Self {
            date: date_str.clone(),
            activity_type,
            customer,
            work_item,
            hours: hours.to_string(),
            comment: comment.unwrap_or_default(),
            current_field: FormField::Date,
            focus_on_cache: false,
            selected_cache_index: 0,
            cursor_position: date_str.len(), // Start at end of date field
        }
    }

    /// Get the current field value
    pub fn get_field_value(&self, field: FormField) -> &str {
        match field {
            FormField::Date => &self.date,
            FormField::ActivityType => &self.activity_type,
            FormField::Customer => &self.customer,
            FormField::WorkItem => &self.work_item,
            FormField::Hours => &self.hours,
            FormField::Comment => &self.comment,
        }
    }

    /// Get mutable reference to current field value
    pub fn get_current_field_mut(&mut self) -> &mut String {
        match self.current_field {
            FormField::Date => &mut self.date,
            FormField::ActivityType => &mut self.activity_type,
            FormField::Customer => &mut self.customer,
            FormField::WorkItem => &mut self.work_item,
            FormField::Hours => &mut self.hours,
            FormField::Comment => &mut self.comment,
        }
    }

    /// Move to next field
    pub fn next_field(&mut self) {
        if !self.focus_on_cache {
            self.current_field = self.current_field.next();
        }
    }

    /// Move to previous field
    pub fn previous_field(&mut self) {
        if !self.focus_on_cache {
            self.current_field = self.current_field.previous();
        }
    }

    /// Toggle focus between form and cache panel
    pub fn toggle_focus(&mut self) {
        self.focus_on_cache = !self.focus_on_cache;
    }

    /// Select cache entry (fill customer and work item)
    pub fn apply_cache_entry(&mut self, customer: String, work_item: String) {
        self.customer = customer;
        self.work_item = work_item;
        self.focus_on_cache = false;
        self.current_field = FormField::Hours;
    }

    /// Validate form data
    pub fn validate(&self) -> Result<(), String> {
        if self.date.is_empty() {
            return Err("Date is required".to_string());
        }

        // Customer and work item are only required for billable activities
        let requires_customer_workitem = matches!(
            self.activity_type.as_str(),
            "billable" | "presales" | "overhead" | "business_development" | "intellectual_capital"
        );

        if requires_customer_workitem {
            if self.customer.is_empty() {
                return Err("Customer is required for this activity type".to_string());
            }

            if self.work_item.is_empty() {
                return Err("Work item is required for this activity type".to_string());
            }
        }

        if self.hours.is_empty() {
            return Err("Hours is required".to_string());
        }

        // Validate hours is a number
        if self.hours.parse::<f64>().is_err() {
            return Err("Hours must be a valid number".to_string());
        }

        Ok(())
    }

    /// Insert character at cursor position
    pub fn insert_char(&mut self, c: char) {
        let pos = self.cursor_position;
        let field = self.get_current_field_mut();
        let pos = pos.min(field.len());
        field.insert(pos, c);
        self.cursor_position = pos + 1;
    }

    /// Delete character before cursor (backspace)
    pub fn delete_char_before(&mut self) {
        let cursor_pos = self.cursor_position;
        if cursor_pos > 0 {
            let field = self.get_current_field_mut();
            let pos = cursor_pos.min(field.len());
            if pos > 0 && pos <= field.len() {
                field.remove(pos - 1);
                self.cursor_position = pos - 1;
            }
        }
    }

    /// Delete character at cursor (delete key)
    pub fn delete_char_at(&mut self) {
        let cursor_pos = self.cursor_position;
        let field = self.get_current_field_mut();
        let pos = cursor_pos.min(field.len());
        if pos < field.len() {
            field.remove(pos);
        }
    }

    /// Move cursor left
    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    /// Move cursor right
    pub fn move_cursor_right(&mut self) {
        let field_len = self.get_field_value(self.current_field).len();
        if self.cursor_position < field_len {
            self.cursor_position += 1;
        }
    }

    /// Move cursor to start of field
    pub fn move_cursor_to_start(&mut self) {
        self.cursor_position = 0;
    }

    /// Move cursor to end of field
    pub fn move_cursor_to_end(&mut self) {
        self.cursor_position = self.get_field_value(self.current_field).len();
    }

    /// Update cursor position when changing fields
    pub fn update_cursor_for_field(&mut self) {
        let field_len = self.get_field_value(self.current_field).len();
        self.cursor_position = field_len; // Start at end of new field
    }
}

impl Default for FormData {
    fn default() -> Self {
        Self::new()
    }
}

// Made with Bob
