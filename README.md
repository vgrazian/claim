# Claim - Rust CLI Application

A command-line application for processing claims with API key authentication.

## Features

- Secure API key storage in system configuration directory
- Interactive setup for first-time users
- Automatic API key loading for subsequent uses
- Masked API key display for security
- Query a specific date
- ad a single entry or multiple dates

# Monday.com Integration

This application connects to Monday.com using your API key to retrieve user information and verify authentication.

## Getting Your Monday.com API Key

1. Log in to your Monday.com account
2. Go to https://your-account.monday.com/admin/integrations/api
3. Generate a new API key or use an existing one
4. Copy the API key when prompted by the application

## API Key Validation

The application validates your API key by:
1. Testing the connection to Monday.com's API
2. Retrieving your user information (ID, name, email)
3. Only saving the API key if validation succeeds

## API Permissions

Your Monday.com API key needs the following permissions:
- Read access to user information
- Access to the GraphQL API

## Error Handling

If you encounter connection errors:
1. Verify your API key is correct
2. Check your internet connection
3. Ensure your Monday.com account is active
4. Verify API key permissions

## Installation

### Prerequisites
- Rust and Cargo installed on your system

### Building from Source
```bash
git clone https://github.com/vgrazian/claim.git
cd claim
cargo build --release
```

The binary will be available at target/release/claim

# Usage
## First Run
On the first execution, the application will prompt you to enter an API key:

```bash
cargo run
# or if built:
./target/release/claim
```

## Output
```text
No API key found. Let's set one up!
Please enter your API key:
[your input here]
API key saved successfully!
Using API key for claims processing...
Processing claims with API key: your******
Claims processed successfully!
```

## Subsequent Runs
After the initial setup, the application will automatically use the stored API key. If the API key needs to be changed you will need to manually delete the config file.

```bash
cargo run
# or if built:
./target/release/claim
```
## Output:
```text
Running for user id *****, user name ****** ******, email ******** for year ####
No command specified. Use --help for available commands.
```

## Query entries for you on a specific date
```bash
cargo run -- query -D 2025-09-15
# or if built:
./target/release/claim query -D 2025-09-15
```

## Output:
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

This shows you have an entry on the date specified and provides customer name, work item and number of hours.


## Add one or more entries on a date - interactive mode
```bash
cargo run -- add
# or if built:
./target/release/claim add
```

## Output:
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

This asks interactively a set of questions and defaults to today's date, billable and 1 day to add.
The last line provided is the commandline you would use to add the same entry directly.


## Command line parameters
-D Day (any ISO format value is accepted, you can use '-', '/' or '.'.).
-c Customer Name
-w Work Item
-H number of hours worked
-d number of repeated days (note: this will skip weekends, so if you select 7 it will add 7 entries skipping saturdays and sundays
-y skip confirmation



# Configuration File Location
The API key is stored in a JSON configuration file. The location varies by operating system.

## Linux
```Linux
~/.config/claim/config.json
```

## macOS
``` Linux
~/Library/Application Support/com.yourname.claim/config.json
```

## Windows
``` Linux
C:\Users\Username\AppData\Roaming\yourname\claim\config\config.json
Linux
```

# Security Notes
The API key is stored in plain text (though in a protected system directory)
When displayed, only the first 4 characters are shown, followed by asterisks
The config file is created with standard file permissions for your user account


# Development
## Building
```bash
cargo build
```
## Running Tests
```bash
cargo test
```
## Running in Debug Mode
```bash
cargo run
```
## Building for Release
```bash
cargo build --release
```

# Project Structure
```text
claim/
├── src/
│   ├── main.rs      # Main application entry point
│   └── config.rs    # Configuration management
├── Cargo.toml       # Project dependencies and metadata
└── README.md        # This file
```

# Dependencies
serde - Serialization/deserialization framework
serde_json - JSON support for Serde
directories - Cross-platform directory location handling

