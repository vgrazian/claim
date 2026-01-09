//! Activity type definitions and utilities

/// Activity type with display information
#[derive(Debug, Clone)]
pub struct ActivityType {
    pub number: u8,
    pub name: &'static str,
    pub display_name: &'static str,
}

impl ActivityType {
    pub const fn new(number: u8, name: &'static str, display_name: &'static str) -> Self {
        Self {
            number,
            name,
            display_name,
        }
    }
}

/// Get all available activity types
pub fn get_all_activity_types() -> Vec<ActivityType> {
    vec![
        ActivityType::new(0, "vacation", "Vacation"),
        ActivityType::new(1, "billable", "Billable (default)"),
        ActivityType::new(2, "holding", "Holding"),
        ActivityType::new(3, "education", "Education"),
        ActivityType::new(4, "work_reduction", "Work Reduction"),
        ActivityType::new(5, "tbd", "TBD"),
        ActivityType::new(6, "holiday", "Holiday"),
        ActivityType::new(7, "presales", "Presales"),
        ActivityType::new(8, "illness", "Illness"),
        ActivityType::new(9, "paid_not_worked", "Paid Not Worked"),
        ActivityType::new(10, "intellectual_capital", "Intellectual Capital"),
        ActivityType::new(11, "business_development", "Business Development"),
        ActivityType::new(12, "overhead", "Overhead"),
    ]
}

/// Get activity type by number
pub fn get_activity_type_by_number(number: u8) -> Option<&'static str> {
    match number {
        0 => Some("vacation"),
        1 => Some("billable"),
        2 => Some("holding"),
        3 => Some("education"),
        4 => Some("work_reduction"),
        5 => Some("tbd"),
        6 => Some("holiday"),
        7 => Some("presales"),
        8 => Some("illness"),
        9 => Some("paid_not_worked"),
        10 => Some("intellectual_capital"),
        11 => Some("business_development"),
        12 => Some("overhead"),
        _ => None,
    }
}

/// Check if a character is a valid activity type number
#[allow(dead_code)]
pub fn is_valid_activity_number(c: char) -> bool {
    c.is_ascii_digit()
}

/// Parse activity type number from char
#[allow(dead_code)]
pub fn parse_activity_number(c: char) -> Option<u8> {
    c.to_digit(10).map(|d| d as u8)
}

// Made with Bob
