# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed
- TUI now shows newly added or edited entries immediately without needing to exit and re-enter; entries are inserted/updated in local state optimistically before the API refresh completes
- Comment field is now correctly saved and reloaded: the extraction logic now reads from column `text2__1` (matching the write path) with a fallback to `long_text` for backward compatibility
- Functional tests now clean up created entries reliably: orphan-cleanup query uses the verbose flag (`-v`) so the `(ID: xxx)` pattern appears in output and can be parsed by `extract_entry_id`
- Comment extraction no longer returns the literal string `"null"` (which Monday.com emits for empty text columns) as a comment value; it is now treated as absent
- Editing an entry in the TUI and clearing the comment field now correctly clears it on Monday.com (the comment column value is always included in the update mutation)


### Added
- L104 absence type support with automatic monthly limit enforcement
  - Added L104 as activity type 13 with LightMagenta color coding
  - Automatic 3-day (24 hours) monthly limit validation
  - Pre-submission validation when adding L104 entries
  - Monthly usage summary in query command output
  - Comprehensive test coverage (9 new tests)
- **Monthly Summary section in TUI**
  - Separate "Monthly Summary" section showing vacation, presales, and L104 usage for the entire calendar month
  - Displays month name and year (e.g., "June 2026")
  - Tracks vacation days taken in the current month
  - Tracks presales days worked in the current month
  - Tracks L104 days with 3-day monthly limit indicator
  - Color-coded warnings: Red when at limit (≥3.0 days), Yellow when approaching (≥2.5 days)
  - Weekly Summary now only shows current week's activity distribution
  - Automatically loads full month data when navigating between weeks
- PRESALES quick-add option as option 0 in TUI quick selection list
  - Hardcoded entry: Customer "PRESALES", Work Item "M.34212", Hours "8"
  - Displayed in green with bold styling for easy identification
  - Automatically sets hours to 8 when selected

### Fixed
- **CRITICAL**: TUI showing incorrect activity types (phantom vacation days)
  - Root cause: `extract_activity_value_from_item()` was only parsing text field and defaulting to 0 (vacation) on failure
  - Fixed to parse JSON value field properly (like CLI does) to extract status index
  - Changed default from vacation (0) to billable (1) for unparseable entries
  - TUI now correctly displays billable, presales, illness entries instead of showing them as vacation
- **CRITICAL**: TUI not displaying customer and work item information
  - Root cause: `extract_customer_from_item()` and `extract_work_item_from_item()` used wrong column IDs (`text` and `text8` instead of `text__1` and `text8__1`)
  - Fixed column IDs to match CLI implementation
  - TUI week view now correctly displays customer and work item for all entries
- Week view incorrectly treating empty days as 8-hour vacation days
  - Removed logic that added 8 hours to weekly total for blank days
  - Weekly totals now accurately reflect only actual entries
- Secure API key storage using system keyring (macOS Keychain, Windows Credential Manager, Linux Secret Service)
- Automatic migration from file-based to keyring-based storage

### Changed
- Organization identifier updated from "yourname" to "vgrazian" in config and cache paths
- API key storage now uses OS-native secure credential storage instead of plain text files

### Security
- **CRITICAL**: API keys are now stored securely in the system keyring instead of plain text JSON files
- Automatic migration of existing API keys to secure storage on first run after update

## [0.2.1] - 2026-01-15

### Added
- Version information with build date (`--version` flag)
- User-specific cache storage for client-workitem pairs
- Smart recent pairs selection in interactive add mode (5 most recent)
- Automatic cache persistence across sessions

### Changed
- Command display enhancement: hours (`-H`) and date (`-D`) positioned as last parameters
- Cache system now stores entries per user using Monday.com user ID
- Improved equivalent command display for better readability

### Fixed
- Cache behavior now properly maintains per-user entries

## [0.2.0] - 2026-01-10

### Added
- Interactive terminal UI with week-based calendar view
- Visual summary chart showing hours distribution
- Entry details panel for selected entries
- Report mode for analyzing work by customer/project
- Form editing with cursor support and validation
- Smart caching with autocomplete for customer/work-item pairs
- Quick-select shortcuts (0-9) for activity types
- Comprehensive keyboard controls for navigation

### Changed
- Default mode is now interactive UI instead of CLI-only
- Improved user experience with visual feedback and animations

## [0.1.0] - 2025-12-01

### Added
- Initial release
- Command-line interface for Monday.com claim management
- Query claims with date filtering and multi-day support
- Add claims with validation and weekend skipping
- Delete claims by ID or criteria
- API key management with validation
- Configuration storage in system directories
- Comprehensive error handling
- Multi-platform support (Linux, macOS, Windows)

### Features
- GraphQL API integration with Monday.com
- Flexible date format support (YYYY-MM-DD, YYYY.MM.DD, YYYY/MM/DD)
- Activity type mapping (vacation, billable, presales, etc.)
- Automatic weekend skipping for multi-day operations
- Verbose mode for debugging
- Confirmation prompts for destructive operations

[Unreleased]: https://github.com/vgrazian/claim/compare/v0.2.1...HEAD
[0.2.1]: https://github.com/vgrazian/claim/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/vgrazian/claim/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/vgrazian/claim/releases/tag/v0.1.0