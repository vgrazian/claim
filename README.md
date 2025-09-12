# Claim - Rust CLI Application

A command-line application for processing claims with API key authentication.

## Features

- Secure API key storage in system configuration directory
- Interactive setup for first-time users
- Automatic API key loading for subsequent uses
- Masked API key display for security

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
```
## Output:
```text
Loaded API key: your******
Using API key for claims processing...
Processing claims with API key: your******
Claims processed successfully!
```

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

