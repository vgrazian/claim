# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- PRESALES quick-add option as option 0 in TUI quick selection list
  - Hardcoded entry: Customer "PRESALES", Work Item "M.34212", Hours "8"
  - Displayed in green with bold styling for easy identification
  - Automatically sets hours to 8 when selected

### Fixed
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