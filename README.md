# Claim - Rust CLI Application

A command-line application for processing claims with API key authentication.

## NAME

claim - Monday.com claim management tool

## SYNOPSIS

**claim** [*OPTIONS*]

**claim** **query** [*QUERY_OPTIONS*]

**claim** **add** [*ADD_OPTIONS*]

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
2. Go to https://your-account.monday.com/admin/integrations/api
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

### Subsequent Runs

After the initial setup, the application will automatically use the stored API key:

```bash
cargo run
# or if built:
./target/release/claim
```

**Output:**
```text
Running for user id *****, user name ***** *****, email ******** for year ####
No command specified. Use --help for available commands.
```

## COMMANDS

### query

Query claims from Monday.com board.

```bash
claim query [--date DATE] [--limit LIMIT] [-v]
```

**Options:**
- `-D, --date DATE`: Date to filter claims (YYYY-MM-DD, YYYY.MM.DD, or YYYY/MM/DD format)
- `--limit LIMIT`: Number of rows to display (default: 5)
- `-v, --verbose`: Verbose output

**Example:**
```bash
cargo run -- query -D 2025-09-15
# or if built:
./target/release/claim query -D 2025-09-15
```

**Output:**
```text
Running for user id ****, user name *** ****, email *** for year ###

=== FILTERED ITEMS for User **** **** ===
Date filter: 2025-09-15
Found 1 items for user **** *****:

1. ***** **** (ID: #########)
   Columns:
     Subitems           : {}
     Person             : {"personsAndTeams":[{"id":*****,"kind":"person"}]}
     Status             : {"index":1,"post_id":null,"changed_at":"2025-09-19T13:58:54.400Z"}
     Date               : {"date":"2025-09-15"}
     Text               : "*****customer name*****"
     Text 8             : "*****work item*****"
     Numbers            : "4"

✅ Found 1 total items matching date filter: 2025-09-15
```

### add

Add a new claim entry.

```bash
claim add [--date DATE] [--activity-type TYPE] [--customer CUSTOMER] [--work-item WORK_ITEM] [--hours HOURS] [--days DAYS] [-y] [-v]
```

**Options:**
- `-D, --date DATE`: Date (YYYY-MM-DD format, defaults to today)
- `-t, --activity-type TYPE`: Activity type: vacation, billable, holding, education, work_reduction, tbd, holiday, presales, illness, boh1, boh2, boh3 (default: billable)
- `-c, --customer CUSTOMER`: Customer name
- `-w, --work-item WORK_ITEM`: Work item
- `-H, --hours HOURS`: Number of hours worked
- `-d, --days DAYS`: Number of working days (default: 1, skips weekends)
- `-y, --yes`: Skip confirmation prompt
- `-v, --verbose`: Verbose output

**Interactive Mode:**
If no options are provided, the command runs in interactive mode:

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
Customer name (optional): CUST-NAME
Work item (optional): WI.12344
Number of hours (optional): 1
Number of working days (optional, default: 1, skips weekends):

=== Adding Claim for User ===
User ID: ****, Name: *** ***, Email: ***
Year: ####

=== Claim Details ===
Date: 2025-09-23
Activity Type: billable (value: 1)
Customer: CUST-NAME
Work Item: WI.12344
Hours: 1
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
   claim add -D 2025-09-23 -c "CUST-NAME" -w "WI.12344" -H 1
```

## EXAMPLES

### Query claims for a specific date:
```bash
claim query -D 2025-09-15
```

### Add a single claim entry:
```bash
claim add -D 2025-09-23 -c "Customer Name" -w "WI.12344" -H 8
```

### Add multiple days of claims:
```bash
claim add -D 2025-09-23 -c "Customer" -w "PROJ-123" -H 8 -d 5
```

### Add claim non-interactively:
```bash
claim add -D 2025-09-23 -c "Customer" -w "ITEM-456" -H 6 -y
```

### Verbose query with increased limit:
```bash
claim query -D 2025-09-15 --limit 20 -v
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
- Rust and Cargo installed on your system

### Building from Source
```bash
git clone https://github.com/vgrazian/claim.git
cd claim
cargo build --release
```

The binary will be available at `target/release/claim`

## DEVELOPMENT

### Building
```bash
cargo build
```

### Running Tests
```bash
cargo test
```

### Running in Debug Mode
```bash
cargo run
```

### Building for Release
```bash
cargo build --release
```

## PROJECT STRUCTURE
```
claim/
├── src/
│   ├── main.rs      # Main application entry point
│   └── config.rs    # Configuration management
├── Cargo.toml       # Project dependencies and metadata
└── README.md        # This file
```

## DEPENDENCIES

- **serde** - Serialization/deserialization framework
- **serde_json** - JSON support for Serde
- **directories** - Cross-platform directory location handling
- **reqwest** - HTTP client for API calls
- **tokio** - Async runtime
- **anyhow** - Error handling
- **chrono** - Date/time handling
- **clap** - Command-line argument parsing

## Monday.com Integration

This application connects to Monday.com using your API key to retrieve user information and verify authentication. It specifically works with boards that have the following column structure:
- Person column (user assignment)
- Date column (claim date)
- Status column (activity type)
- Text columns (customer, work item)
- Numbers column (hours)

The application automatically handles weekend skipping when adding multiple days and provides comprehensive error handling for API interactions.