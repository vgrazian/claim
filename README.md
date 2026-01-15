# Claim - Rust CLI Application

A command-line application for processing claims with API key authentication.

## Version

Current version: **0.2.1**

To check your installed version:

```bash
claim --version      # Short version
claim -V            # Short version
claim --version     # Long version with build date
```

## NAME

claim - Monday.com claim management tool

## SYNOPSIS

**claim** [*OPTIONS*]

**claim** **query** [*QUERY_OPTIONS*]

**claim** **add** [*ADD_OPTIONS*]

**claim** **delete** [*DELETE_OPTIONS*]

## DESCRIPTION

**claim** is a command-line application for processing claims with Monday.com API integration. It provides secure API key storage, interactive setup, and functionality to query and add claim entries to Monday.com boards.

The application automatically handles API key validation, stores credentials securely in the system configuration directory, and provides both interactive and command-line modes for claim management.

## FIRST-TIME SETUP

On first execution, the application will prompt for a Monday.com API key:

```bash
cargo run
# or if built:
./target/release/claim
```

**Output:**

```text
No API key found. Let's set one up!
Please enter your API key:
[your input here]
API key saved successfully!
```

### Getting Your Monday.com API Key

1. Log in to your Monday.com account
2. Go to <https://your-account.monday.com/admin/integrations/api>
3. Generate a new API key or use an existing one
4. Copy the API key when prompted by the application

### API Key Validation

The application validates your API key by:

1. Testing the connection to Monday.com's API
2. Retrieving your user information (ID, name, email)
3. Only saving the API key if validation succeeds

### API Permissions

Your Monday.com API key needs the following permissions:

- Read access to user information
- Access to the GraphQL API

## USAGE

### Interactive UI Mode (Default)

When run without any command, the application launches an interactive terminal UI:

```bash
cargo run
# or if built:
./target/release/claim
```

The interactive UI provides:

- **Week-based calendar view** with all your claim entries
- **Visual summary chart** showing hours distribution
- **Entry details panel** for selected entries
- **Report mode** for analyzing work by customer/project
- **Intuitive keyboard controls** for navigation and editing

#### Interactive UI Controls

**Normal Mode:**

- `←/→` or `Tab/Shift+Tab`: Navigate between weeks
- `↑/↓`: Navigate between days
- `j/k`: Navigate between entries on selected day
- `a`: Add new entry
- `e`: Edit selected entry
- `d`: Delete selected entry
- `u`: Update/refresh data from Monday.com
- `p`: Switch to Report mode
- `h` or `?`: Show help
- `q`: Quit application

**Add/Edit Mode:**

- `Tab`: Move to next field
- `Shift+Tab`: Move to previous field
- `←/→`: Move cursor within field
- `Home/End`: Jump to start/end of field
- `Backspace/Delete`: Remove characters
- `0-9`: Quick select from activity types or cache
- `Enter`: Save entry
- `Esc`: Cancel

**Report Mode:**

- `Tab/Shift+Tab`: Navigate between weeks
- `↑/↓`: Navigate between report rows
- `Esc`: Return to normal mode
- `q`: Quit application

**Features:**

- **Smart caching**: Recently used customers and work items appear in quick-select panel
- **Visual cursor**: See exactly where you're typing in form fields
- **Activity type shortcuts**: Press 0-9 to quickly select activity types
- **Automatic refresh**: Data updates after add/edit/delete operations
- **Report view**: Analyze hours by customer and work item with daily breakdown

### Command-Line Mode

For automation and scripting, all commands are available via CLI:

```bash
claim query [OPTIONS]
claim add [OPTIONS]
claim delete [OPTIONS]
```

## COMMANDS

### query

Query claims from Monday.com board.

```bash
claim query [--date DATE] [--customer CUSTOMER] [--work-item WORK_ITEM] [--days DAYS] [--limit LIMIT] [-v]
```

**Options:**

- `-D, --date DATE`: Date to filter claims (YYYY-MM-DD, YYYY.MM.DD, or YYYY/MM/DD format)
- `-c, --customer CUSTOMER`: Customer name to filter on (optional to generate report)
- `-w, --work-item WORK_ITEM`: Work item to filter on (optional to generate report)
- `-d, --days DAYS`: Number of working days to query (default: 1, skips weekends)
- `--limit LIMIT`: Number of rows to display (default: 5)
- `-v, --verbose`: Verbose output

**Examples:**

```bash
# Query a single day
claim query -D 2025-09-15

# Query a full work week (5 days starting from specified date)
claim query -D 2025-09-15 -d 5

# Query 10 days with increased limit and verbose output
claim query -D 2025-09-01 -d 10 --limit 20 -v

# Run a weekly report for all entries related to customer CUST1 and work item WI.1001
claim query -D 2025-09-15 -c CUST1 -w WI.1001 -d 5
```

**Output for multi-day query:**

```text
Running for user id *****, user name ***** *****, email ******** for year ####

=== CLAIMS SUMMARY for User ***** ***** ===
Date Range: 2025-04-07 to 2025-04-11

Date         Status       Customer             Work Item       Comment    Hours
------------------------------------------------------------------------------
2025-04-07   presales     CUSTOMER_A           PROJ-123        my comment 8
2025-04-08   presales     CUSTOMER_A           PROJ-123                   8
2025-04-09   presales     CUSTOMER_A           PROJ-123                   8
2025-04-10   billable     CUSTOMER_B           TASK-456                   4
2025-04-10   presales     CUSTOMER_C           WI-789          another    4
2025-04-11   presales     CUSTOMER_C           WI-789          comment    8
------------------------------------------------------------------------------
TOTAL                                                                     40.0

Found 6 items across 5 days

✅ Found 6 total items matching date range: 2025-04-07 to 2025-04-11
```

### add

Add a new claim entry with enhanced features including smart caching and command display.

```bash
claim add [--date DATE] [--activity-type TYPE] [--customer CUSTOMER] [--work-item WORK_ITEM] [--comment COMMENT] [--hours HOURS] [--days DAYS] [--yes] [--verbose]
```

**Options:**

- `-D, --date DATE`: Date (YYYY-MM-DD format, defaults to today)
- `-t, --activity-type TYPE`: Activity type: vacation, billable, holding, education, work_reduction, tbd, holiday, presales, illness, paid_not_worked, intellectual_capital, business_development, overhead (default: billable), the corresponding numerical value can be used (see table at the end, the list is also presented to the user)
- `-c, --customer CUSTOMER`: Customer name
- `-w, --work-item WORK_ITEM`: Work item
- `-k, --comment COMMENT`: Comment
- `-H, --hours HOURS`: Number of hours worked
- `-d, --days DAYS`: Number of working days (default: 1, skips weekends)
- `-y, --yes`: Skip confirmation prompt
- `-v, --verbose`: Verbose output

**Interactive Mode with Smart Caching:**
If no options are provided, the command runs in interactive mode with access to your 5 most recently used client-workitem pairs:

```bash
cargo run -- add
# or if built:
./target/release/claim add
```

**Output:**

```text
Running for user id ***, user name *** ***, email *** for year ####

=== Add New Claim ===
Enter claim details (press Enter to skip optional fields):
Date (YYYY-MM-DD, YYYY.MM.DD, or YYYY/MM/DD, optional - default: today):
Activity type (optional, default: billable):
Customer name (optional): CUSTOMER_NAME
Work item (optional): WI-12344
Number of hours (optional): 8
Number of working days (optional, default: 1, skips weekends):

=== Adding Claim for User ===
User ID: ****, Name: *** ***, Email: ***
Year: ####

=== Claim Details ===
Date: 2025-09-23
Activity Type: billable (value: 1)
Customer: CUSTOMER NAME
Work Item: WI.12344
Hours: 8
Days requested: 1
Actual working days: 1

📅 Dates that will be created (weekends skipped):
  1. 2025-09-23 (Tuesday)

Found group '2025' with ID: new_group_mkkbbd2q

🚀 Ready to create 1 item(s) on Monday.com
Do you want to proceed? (y/N)
y

🔄 Creating items on Monday.com...
Creating item for 2025-09-23 (1 of 1)...
✅ Successfully created item

🎉 Successfully created 1 out of 1 items

💡 Equivalent command line:
   claim add -c "CUSTOMER NAME" -w "WI.12344" -H 8 -D 2025-09-23
```

**Note:** The equivalent command now displays hours and date as the last parameters for better readability.

### delete

Delete a claim item by ID or by matching criteria (date + customer + work item).

```bash
# Delete by ID
claim delete --id ID [OPTIONS]

# Delete by criteria
claim delete --date DATE --customer CUSTOMER --work-item WORK_ITEM [OPTIONS]
```

**Options:**

- `-x, --id ID`: Item ID to delete (find in your query output)
- `-D, --date DATE`: Date to filter claims (YYYY-MM-DD, YYYY.MM.DD, or YYYY/MM/DD format)
- `-c, --customer CUSTOMER`: Customer name to filter by
- `-w, --wi WORK_ITEM`: Work item to filter by
- `-y, --yes`: Skip confirmation prompt
- `-v, --verbose`: Verbose output

**Note:** You must provide either:

1. Item ID (`-x/--id`), OR
2. All three criteria: Date (`-D/--date`) + Customer (`-c/--customer`) + Work Item (`-w/--wi`)

**Examples:**

```bash
# Delete by ID with confirmation
claim delete -x 9971372083

# Delete by ID without confirmation
claim delete -x 9971372083 -y

# Delete by criteria (date + customer + work item)
claim delete -D 2025-12-10 -c "CUSTOMER_A" -w "PROJ-123" -y

# Delete with verbose output
claim delete -x 9971372083 -v
```

**Output:**

```text
Running for user id 51921473, user name John Doe, email john.doe@domain.com for year 2025

=== Delete Claim Item ===
User: John Doe 

Item ID to delete: 9971372083

📋 Item Details:
Name: John Doe
ID: 9971372083
Columns:
Date: 2025-09-29
Status: illness
Hours: 8

🗑️ Are you sure you want to delete this item?
This action cannot be undone! 
y
/
N
y/N
y

🔄 Deleting item...
✅ Item deleted successfully!
```

**Finding Item IDs:**
To find the ID of an item you want to delete:

1. Run `claim query` to list your claims
2. Look for the `ID: **********` value in the output
3. Use that ID with the delete command

## EXAMPLES

### Query Examples

**Query claims for a specific date:**

```bash
claim query -D 2025-09-15
```

**Query multiple days with weekend skipping:**

```bash
claim query -D 2025-09-15 -d 7 # Will show 5 business days skips weekends
```

**Query to generate a customer or work item specifi report:**

```bash
claim query -D 2025-09-15 -c CUST02 -w WI.1002 -d 7 # Will show 5 business days skips weekends
```

**Query with custom limit and verbose output:**

```bash
claim query -D 2025-09-01 -d 10 --limit 15 -v
```

**Query current week:**

```bash
# Assuming today is Monday, query the current work week
claim query -d 5
```

### Add Examples

**Add a single claim entry, include a comment:**

```bash
claim add -D 2025-09-23 -c "CUSTOMER_A" -w "PROJ-123" -k "my comment" -H 8
```

**Add multiple days of claims:**

```bash
claim add -D 2025-09-23 -c "CUSTOMER_B" -w "TASK-456" -H 6 -d 5
```

**Add claim with specific activity type:**

```bash
claim add -D 2025-09-23 -t vacation -d 3
```

**Add claim non-interactively:**

```bash
claim add -D 2025-09-23 -c "CUSTOMER_C" -w "WI-789" -H 4 -y
```

**Add claim for today with verbose output:**

```bash
claim add -c "CUSTOMER_D" -w "PROJ-999" -H 7 -v
```

### Delete Examples

**Delete a claim by ID with confirmation:**

```bash
claim delete -x 9971372083
```

**Delete a claim by ID without confirmation:**

```bash
claim delete -x 9971372083 -y
```

**Delete by criteria (date + customer + work item):**

```bash
claim delete -D 2025-12-10 -c "CUSTOMER_A" -w "PROJ-123" -y
```

**Delete multiple matching entries by criteria:**

```bash
# This will find and delete all entries matching the criteria
claim delete -D 2025-12-10 -c "TEST" -w "DELETE.ME" -y
```

**Delete with verbose output to see details:**

```bash
claim delete -x 9971372083 -v
```

## CONFIGURATION FILE LOCATION

The API key is stored in a JSON configuration file. The location varies by operating system:

### Linux

```
~/.config/claim/config.json
```

### macOS

```
~/Library/Application Support/com.yourname.claim/config.json
```

### Windows

```
C:UsersUsernameAppDataRoamingyournameclaimconfigconfig.json
```

## SECURITY NOTES

- The API key is stored in plain text (though in a protected system directory)
- When displayed, only the first 4 characters are shown, followed by asterisks
- The config file is created with standard file permissions for your user account
- API keys are validated before being saved to ensure they work with Monday.com
- If you need to change your API key, you must manually delete the configuration file

## ERROR HANDLING

If you encounter connection errors:

1. Verify your API key is correct
2. Check your internet connection
3. Ensure your Monday.com account is active
4. Verify API key permissions
5. Check that the Monday.com board structure matches expected format

## INSTALLATION

### Prerequisites

- Rust and Cargo installed on your system, a linker is also required

### Installing linker on macos

 install the Xcode Command Line Tools by running `xcode-select --install` in the Terminal. This provides the necessary C/C++ compiler and linker, which Rust uses to build your project.

### Building from Source

```bash
git clone https://github.com/vgrazian/claim.git
cd claim
cargo build --release
```

The binary will be available at `./target/release/claim`

## DEVELOPMENT

### Running Tests

```bash
cargo test
```

How to run the functional tests:

```bash
cargo test --test functional_tests
```

NOTE: Functional tests now automatically track and cleanup all created entries. No manual cleanup is required.

Or run the specific demonstration script

```bash
chmod +x run_functional_tests.sh
./run_functional_tests.sh
```

### Running in Debug Mode

```bash
cargo run
```

## PROJECT STRUCTURE

The project has been refactored into a modular structure:

```
claim/
├── src/
│ ├── main.rs              # CLI setup and command routing
│ ├── config.rs            # Configuration management and API key handling
│ ├── monday.rs            # Monday.com API client and data structures
│ ├── query.rs             # Query command functionality
│ ├── delete.rs            # Delete command functionality
│ ├── add.rs               # Add command functionality
│ ├── cache.rs             # Entry caching for autocomplete
│ ├── time.rs              # Date/time utilities
│ ├── utils.rs             # Utility functions
│ ├── selenium.rs          # Browser automation (if needed)
│ └── interactive/         # Interactive UI module
│     ├── mod.rs           # Module exports
│     ├── app.rs           # Main application state and logic
│     ├── ui.rs            # UI rendering
│     ├── events.rs        # Event handling
│     ├── form.rs          # Form data structures
│     ├── form_ui.rs       # Form rendering with cursor support
│     ├── week_view.rs     # Week calendar view
│     ├── summary_chart.rs # Hours distribution chart
│     ├── entry_details.rs # Entry details panel
│     ├── activity_types.rs# Activity type definitions
│     ├── messages.rs      # Status messages
│     ├── dialogs.rs       # Dialog components
│     └── utils.rs         # UI utilities
├── Cargo.toml             # Project dependencies and metadata
└── README.md              # This file
```

## DEPENDENCIES

### Core Dependencies

- **serde** - Serialization/deserialization framework
- **serde_json** - JSON support for Serde
- **directories** - Cross-platform directory location handling
- **reqwest** - HTTP client for API calls
- **tokio** - Async runtime
- **anyhow** - Error handling
- **chrono** - Date/time handling
- **clap** - Command-line argument parsing

### Interactive UI Dependencies

- **ratatui** - Terminal UI framework
- **crossterm** - Cross-platform terminal manipulation

## Monday.com Integration

This application connects to Monday.com using your API key to retrieve user information and verify authentication. It specifically works with boards that have the following column structure:

- Person column (user assignment)
- Date column (claim date)
- Status column (activity type)
- Text columns (customer, work item, comment)
- Numbers column (hours)

The application automatically handles weekend skipping when adding multiple days and provides comprehensive error handling for API interactions.

### Multi-Day Query Features

The query command with the `-d` option provides:

- **Automatic weekend skipping** - Only business days are included in the range
- **Simplified table view** - Clean summary format for multiple days
- **Multiple entries per day support** - Handles cases where users have multiple claims on the same date
- **Total hours calculation** - Automatic sum of hours across the period
- **Empty day indication** - Shows dates with no entries for complete timeline
- **Dog walking** - A nice dog walking whle you wait

### Activity Type Mapping

The application maps between human-readable activity types and their corresponding numeric values:

| Activity Type         | Value |
|-----------------------|-------|
| vacation              | 0     |
| billable              | 1     |
| holding               | 2     |
| education             | 3     |
| work_reduction        | 4     |
| tbd                   | 5     |
| holiday               | 6     |
| presales              | 7     |
| illness               | 8     |
| paid_not_worked       | 9     |
| intellectual_capital  | 10    |
| business_development  | 11    |
| overhead              | 12    |

## Recent Updates (2026-01-15)

### Version 0.2.0 - Enhanced Add Feature

**1. Command Display Enhancement**

- Modified equivalent command display to position hours (`-H`) and date (`-D`) as the last parameters
- Improves command readability and consistency

**2. User-Specific Cache Storage**

- Implemented per-user cache storage for client-workitem pairs
- Each user's recent entries are stored separately using their Monday.com user ID
- Cache automatically persists across sessions

**3. Smart Recent Pairs Selection**

- Interactive add mode now displays the 5 most recently used client-workitem pairs
- Users can quickly select by entering the corresponding number (1-5)
- Manual entry still available for new or unlisted pairs
- Pairs are sorted by most recent usage

**4. Automatic Cache Persistence**

- Query operations automatically save client-workitem pairs to user-specific cache
- Add operations save used pairs after successful execution
- Cache updates happen transparently in the background

**5. Version Information**

- Added `--version` and `-V` flags to display version information
- Long version (`--version`) includes build date and time
- Build date automatically captured during compilation

### Cache Behavior

The cache system now:

- Stores entries per user (identified by Monday.com user ID)
- Maintains the 5 most recently used client-workitem pairs per user
- Updates automatically after query and add operations
- Persists to disk in the system cache directory
- Provides quick selection in interactive add mode

### Breaking Changes

None - all changes are backward compatible. Existing cache files will be automatically migrated to the new per-user format.

## Documentation

For detailed documentation, see the [docs/](docs/) directory:

- **[Interactive UI Guide](docs/features/interactive-ui.md)** - Complete guide to the terminal interface
- **[Cache System](docs/features/cache-system.md)** - Understanding the caching feature
- **[Testing Guide](docs/development/testing.md)** - Running and writing tests
- **[Implementation Guide](docs/development/implementation-guide.md)** - Technical details
- **[Project Analysis](docs/development/project-analysis.md)** - Comprehensive project overview
- **[Contributing Guide](docs/contributing/markdown-style-guide.md)** - Documentation standards

See [docs/README.md](docs/README.md) for the complete documentation index.

---

Pls be patient, Readme is rarely up to date

As you got down here you can pet Virgilio the cat:

~~~ text
 _._     _,-'""`-._
(,-.`._,'(       |\`-/|
    `-.-' \ )-`( , o o)
          `-    \`_`"'-
~~~

purr purrr...
