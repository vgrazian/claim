# Claim - Rust CLI Application

A command-line application for processing claims with API key authentication.

## Features

- Secure API key storage in system configuration directory
- Interactive setup for first-time users
- Automatic API key loading for subsequent uses
- Masked API key display for security

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
After the initial setup, the application will automatically use the stored API key:

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
The API key is stored in a JSON configuration file. The location varies by operating system:

Linux
```
~/.config/claim/config.json
Linux
```
macOS
Linux
```
~/Library/Application Support/com.yourname.claim/config.json
Linux
```

Windows
Linux
```
C:\Users\Username\AppData\Roaming\yourname\claim\config\config.json
Linux
```
