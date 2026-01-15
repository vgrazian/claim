# Cross-Platform Distribution Guide

## Overview

To distribute the `claim` executable for macOS, Windows, and Linux, you need to build platform-specific binaries. Rust supports cross-compilation, but the easiest approach is to use GitHub Actions for automated builds or build on each target platform.

## Building for Different Platforms

### Option 1: GitHub Actions (Recommended)

Create automated builds for all platforms using GitHub Actions. This ensures consistent, reproducible builds.

**Create `.github/workflows/release.yml`:**

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'
  workflow_dispatch:

jobs:
  build:
    name: Build ${{ matrix.target }}
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            artifact_name: claim
            asset_name: claim-linux-x86_64
          - os: ubuntu-latest
            target: aarch64-unknown-linux-gnu
            artifact_name: claim
            asset_name: claim-linux-aarch64
          - os: macos-latest
            target: x86_64-apple-darwin
            artifact_name: claim
            asset_name: claim-macos-x86_64
          - os: macos-latest
            target: aarch64-apple-darwin
            artifact_name: claim
            asset_name: claim-macos-aarch64
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            artifact_name: claim.exe
            asset_name: claim-windows-x86_64.exe

    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: ${{ matrix.target }}
          override: true
      
      - name: Build
        run: cargo build --release --target ${{ matrix.target }}
      
      - name: Upload artifact
        uses: actions/upload-artifact@v3
        with:
          name: ${{ matrix.asset_name }}
          path: target/${{ matrix.target }}/release/${{ matrix.artifact_name }}
```

### Option 2: Manual Cross-Compilation

#### For macOS (on macOS)

```bash
# Intel Macs
cargo build --release --target x86_64-apple-darwin

# Apple Silicon (M1/M2/M3)
cargo build --release --target aarch64-apple-darwin

# Universal binary (both architectures)
rustup target add x86_64-apple-darwin aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
lipo -create \
  target/x86_64-apple-darwin/release/claim \
  target/aarch64-apple-darwin/release/claim \
  -output claim-universal
```

#### For Linux (on Linux or macOS with cross)

```bash
# Install cross-compilation tool
cargo install cross

# x86_64 Linux
cross build --release --target x86_64-unknown-linux-gnu

# ARM64 Linux
cross build --release --target aarch64-unknown-linux-gnu
```

#### For Windows (on Windows or with cross)

```bash
# On Windows
cargo build --release --target x86_64-pc-windows-msvc

# On macOS/Linux using cross
cross build --release --target x86_64-pc-windows-msvc
```

### Option 3: Build on Each Platform

The simplest but most time-consuming approach:

1. **On macOS:**

   ```bash
   cargo build --release
   # Output: target/release/claim
   ```

2. **On Linux:**

   ```bash
   cargo build --release
   # Output: target/release/claim
   ```

3. **On Windows:**

   ```bash
   cargo build --release
   # Output: target\release\claim.exe
   ```

## Distribution Methods

### 1. GitHub Releases

Upload pre-built binaries to GitHub Releases:

```bash
# Create a new release
gh release create v0.2.1 \
  target/release/claim-macos-x86_64 \
  target/release/claim-macos-aarch64 \
  target/release/claim-linux-x86_64 \
  target/release/claim-windows-x86_64.exe \
  --title "Release v0.2.1" \
  --notes "Release notes here"
```

### 2. Cargo Install (Source Distribution)

Users can install directly from source:

```bash
cargo install --git https://github.com/vgrazian/claim.git
```

### 3. Package Managers

#### Homebrew (macOS/Linux)

Create a Homebrew formula:

```ruby
class Claim < Formula
  desc "Monday.com time tracking CLI"
  homepage "https://github.com/vgrazian/claim"
  url "https://github.com/vgrazian/claim/archive/v0.2.1.tar.gz"
  sha256 "..."
  
  depends_on "rust" => :build
  
  def install
    system "cargo", "install", *std_cargo_args
  end
end
```

#### Chocolatey (Windows)

Create a Chocolatey package for Windows distribution.

#### Snap (Linux)

Create a snapcraft.yaml for Linux distribution.

## Platform-Specific Considerations

### macOS

- **Code Signing:** For distribution outside the App Store, you may need to sign the binary
- **Notarization:** Required for macOS 10.15+ to avoid Gatekeeper warnings
- **Universal Binary:** Consider creating universal binaries for both Intel and Apple Silicon

### Windows

- **Antivirus:** Some antivirus software may flag unsigned executables
- **Code Signing:** Consider signing with a certificate for better trust
- **Dependencies:** Ensure all runtime dependencies are included or statically linked

### Linux

- **Static Linking:** Consider using `musl` target for better portability:

  ```bash
  rustup target add x86_64-unknown-linux-musl
  cargo build --release --target x86_64-unknown-linux-musl
  ```

- **Distribution:** Different distros may require different approaches

## Testing Builds

Before distributing, test on each platform:

```bash
# Check binary info
file target/release/claim

# Check dependencies (Linux)
ldd target/release/claim

# Check dependencies (macOS)
otool -L target/release/claim

# Test execution
./target/release/claim --version
```

## Automated Release Workflow

1. **Tag a release:**

   ```bash
   git tag -a v0.2.1 -m "Release v0.2.1"
   git push origin v0.2.1
   ```

2. **GitHub Actions automatically:**
   - Builds for all platforms
   - Runs tests
   - Creates GitHub Release
   - Uploads binaries

3. **Users download:**
   - Visit GitHub Releases page
   - Download appropriate binary for their platform
   - Make executable (Unix): `chmod +x claim`
   - Move to PATH location

## Quick Start for Users

### macOS/Linux

```bash
# Download
curl -L https://github.com/vgrazian/claim/releases/latest/download/claim-macos-aarch64 -o claim

# Make executable
chmod +x claim

# Move to PATH
sudo mv claim /usr/local/bin/
```

### Windows

```powershell
# Download from GitHub Releases
# Move to a directory in PATH, e.g., C:\Program Files\claim\
```

## Current Project Status

The project currently builds for the local platform. To enable cross-platform distribution:

1. Add GitHub Actions workflow (recommended)
2. Or set up cross-compilation tools locally
3. Create release process documentation
4. Consider package manager distribution for easier installation
