//! Custom error types for the claim application
//!
//! This module provides structured error handling using thiserror,
//! replacing generic anyhow errors with specific, actionable error types.

use thiserror::Error;

/// Main error type for the claim application
#[derive(Error, Debug)]
pub enum ClaimError {
    /// Configuration-related errors
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// API-related errors
    #[error("API error: {0}")]
    Api(#[from] ApiError),

    /// Validation errors
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Date/time parsing errors
    #[error("Date/time error: {0}")]
    DateTime(String),

    /// Terminal/UI errors
    #[error("Terminal error: {0}")]
    Terminal(String),

    /// Generic error for backward compatibility
    #[error("{0}")]
    Other(String),
}

/// Configuration-related errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("API key not found. Please run the application to set up your API key")]
    ApiKeyNotFound,

    #[error("API key is empty or invalid")]
    InvalidApiKey,

    #[error("Failed to load configuration file: {0}")]
    LoadFailed(String),

    #[error("Failed to save configuration file: {0}")]
    SaveFailed(String),

    #[error("Failed to create config directory: {0}")]
    DirectoryCreationFailed(String),
}

/// API-related errors
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Failed to connect to Monday.com API: {0}")]
    ConnectionFailed(String),

    #[error("API request failed: {0}")]
    RequestFailed(String),

    #[error("Invalid API response: {0}")]
    InvalidResponse(String),

    #[error("Authentication failed. Please check your API key")]
    AuthenticationFailed,

    #[error("User not found or unauthorized")]
    UserNotFound,

    #[error("Board not found: {0}")]
    BoardNotFound(String),

    #[error("Group not found: {0}")]
    GroupNotFound(String),

    #[error("Item not found: {0}")]
    ItemNotFound(String),

    #[error("Failed to create item: {0}")]
    ItemCreationFailed(String),

    #[error("Failed to delete item: {0}")]
    ItemDeletionFailed(String),

    #[error("Failed to update item: {0}")]
    ItemUpdateFailed(String),

    #[error("Rate limit exceeded. Please try again later")]
    RateLimitExceeded,

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
}

/// Validation errors
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Invalid date format: {0}. Expected YYYY-MM-DD, YYYY.MM.DD, or YYYY/MM/DD")]
    InvalidDateFormat(String),

    #[error("Invalid activity type: {0}")]
    InvalidActivityType(String),

    #[error("Invalid hours value: {0}. Must be between 0 and 24")]
    InvalidHours(f64),

    #[error("Invalid days value: {0}. Must be positive")]
    InvalidDays(f64),

    #[error("Customer name is required for this activity type")]
    CustomerRequired,

    #[error("Work item is required for this activity type")]
    WorkItemRequired,

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid field value for {field}: {value}")]
    InvalidFieldValue { field: String, value: String },
}

/// Result type alias for the claim application
pub type Result<T> = std::result::Result<T, ClaimError>;

// Conversion from anyhow::Error for backward compatibility during migration
impl From<anyhow::Error> for ClaimError {
    fn from(err: anyhow::Error) -> Self {
        ClaimError::Other(err.to_string())
    }
}

// Conversion from chrono parse errors
impl From<chrono::ParseError> for ClaimError {
    fn from(err: chrono::ParseError) -> Self {
        ClaimError::DateTime(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ClaimError::Config(ConfigError::ApiKeyNotFound);
        assert!(err.to_string().contains("API key not found"));

        let err = ClaimError::Validation(ValidationError::InvalidHours(25.0));
        assert!(err.to_string().contains("Invalid hours value"));
    }

    #[test]
    fn test_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let claim_err: ClaimError = io_err.into();
        assert!(matches!(claim_err, ClaimError::Io(_)));
    }

    #[test]
    fn test_validation_errors() {
        let err = ValidationError::InvalidDateFormat("2025-13-01".to_string());
        assert!(err.to_string().contains("Invalid date format"));

        let err = ValidationError::InvalidHours(25.0);
        assert!(err.to_string().contains("Must be between 0 and 24"));
    }
}

// Made with Bob
