//! Logging configuration using the tracing framework
//!
//! This module sets up structured logging with different levels and outputs.
//! Logs can be controlled via the RUST_LOG environment variable.

use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize the logging system
///
/// This sets up tracing with:
/// - Environment-based log level filtering (RUST_LOG)
/// - Formatted output with timestamps
/// - Optional file logging
///
/// # Environment Variables
///
/// - `RUST_LOG`: Controls log level (e.g., "debug", "info", "warn", "error")
///   - Default: "claim=info"
///   - Examples:
///     - `RUST_LOG=debug` - Show all debug logs
///     - `RUST_LOG=claim=trace` - Show trace logs for claim crate only
///     - `RUST_LOG=claim::monday=debug` - Debug logs for monday module only
///
/// # Examples
///
/// ```no_run
/// use claim::logging;
///
/// // Initialize logging at the start of main
/// logging::init();
///
/// // Use tracing macros throughout the code
/// tracing::info!("Application started");
/// tracing::debug!("Processing request");
/// tracing::error!("Failed to connect: {}", error);
/// ```
pub fn init() {
    // Create an environment filter with a default level
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("claim=info,warn"));

    // Set up the subscriber with formatting
    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .with_target(true)
                .with_thread_ids(false)
                .with_thread_names(false)
                .with_line_number(true)
                .with_file(false),
        )
        .init();
}

/// Initialize logging with file output
///
/// This creates a log file in the system's log directory and writes
/// all logs to both stdout and the file.
///
/// # Arguments
///
/// * `log_file_name` - Name of the log file (e.g., "claim.log")
///
/// # Returns
///
/// Returns `Ok(())` if successful, or an error if file creation fails.
///
/// # Examples
///
/// ```no_run
/// use claim::logging;
///
/// // Initialize with file logging
/// logging::init_with_file("claim.log").expect("Failed to initialize logging");
/// ```
#[allow(dead_code)]
pub fn init_with_file(log_file_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    use tracing_appender::rolling::{RollingFileAppender, Rotation};

    // Get the log directory
    let log_dir =
        if let Some(proj_dirs) = directories::ProjectDirs::from("com", "yourname", "claim") {
            let log_path = proj_dirs.data_dir().join("logs");
            std::fs::create_dir_all(&log_path)?;
            log_path
        } else {
            std::env::current_dir()?
        };

    // Create a rolling file appender (rotates daily)
    let file_appender = RollingFileAppender::new(Rotation::DAILY, log_dir, log_file_name);

    // Create an environment filter
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("claim=info,warn"));

    // Set up the subscriber with both console and file output
    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().with_writer(std::io::stdout))
        .with(fmt::layer().with_writer(file_appender).with_ansi(false))
        .init();

    Ok(())
}

/// Initialize logging for tests
///
/// This sets up minimal logging suitable for test environments.
/// Only errors and warnings are shown by default.
#[allow(dead_code)]
pub fn init_test() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("error"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().with_test_writer())
        .try_init()
        .ok(); // Ignore errors if already initialized
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_does_not_panic() {
        // This test just ensures init doesn't panic
        // We can't actually test the output without more complex setup
        init_test();
    }

    #[test]
    fn test_logging_macros() {
        init_test();

        // Test that logging macros compile and don't panic
        tracing::trace!("trace message");
        tracing::debug!("debug message");
        tracing::info!("info message");
        tracing::warn!("warn message");
        tracing::error!("error message");
    }

    #[test]
    fn test_structured_logging() {
        init_test();

        // Test structured logging with fields
        tracing::info!(user_id = 12345, action = "test", "Structured log message");
    }
}

// Made with Bob
