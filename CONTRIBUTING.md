# Contributing to Claim

Thank you for your interest in contributing to Claim! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Development Workflow](#development-workflow)
- [Code Style Guidelines](#code-style-guidelines)
- [Testing Requirements](#testing-requirements)
- [Pull Request Process](#pull-request-process)
- [Reporting Issues](#reporting-issues)

## Code of Conduct

This project adheres to a code of conduct that all contributors are expected to follow:

- Be respectful and inclusive
- Welcome newcomers and help them get started
- Focus on constructive feedback
- Assume good intentions
- Respect differing viewpoints and experiences

## Getting Started

### Prerequisites

- Rust 1.70 or later
- Cargo (comes with Rust)
- A linker (see platform-specific instructions below)
- Git

### Platform-Specific Setup

**macOS:**
```bash
xcode-select --install
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt-get install build-essential
```

**Windows:**
- Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/)

## Development Setup

1. **Fork the repository** on GitHub

2. **Clone your fork:**
   ```bash
   git clone https://github.com/YOUR-USERNAME/claim.git
   cd claim
   ```

3. **Add upstream remote:**
   ```bash
   git remote add upstream https://github.com/vgrazian/claim.git
   ```

4. **Install dependencies:**
   ```bash
   cargo build
   ```

5. **Run tests to verify setup:**
   ```bash
   cargo test
   ```

## Development Workflow

### Creating a Feature Branch

```bash
git checkout -b feature/your-feature-name
```

Use descriptive branch names:
- `feature/add-export-functionality`
- `fix/memory-leak-in-cache`
- `docs/improve-readme`
- `refactor/split-large-files`

### Making Changes

1. **Write tests first** (Test-Driven Development)
2. **Implement your changes**
3. **Ensure all tests pass**
4. **Update documentation** if needed
5. **Add entry to CHANGELOG.md** under `[Unreleased]`

### Keeping Your Fork Updated

```bash
git fetch upstream
git checkout main
git merge upstream/main
git push origin main
```

## Code Style Guidelines

### Rust Code Style

We follow the official Rust style guidelines:

1. **Format your code:**
   ```bash
   cargo fmt
   ```

2. **Run Clippy for linting:**
   ```bash
   cargo clippy -- -D warnings
   ```

3. **Fix Clippy warnings** before submitting

### Code Quality Standards

- **Use descriptive variable names**: `user_id` not `uid`
- **Add doc comments** to public APIs:
  ```rust
  /// Loads the configuration from the system keyring
  ///
  /// # Errors
  ///
  /// Returns an error if the keyring is inaccessible or the API key is not found
  pub fn load() -> Result<Self> {
      // ...
  }
  ```

- **Keep functions focused**: One function, one responsibility
- **Avoid deep nesting**: Extract complex logic into helper functions
- **Use `Result` and `Option`** appropriately
- **Prefer custom error types** over generic errors

### Documentation Standards

- Add doc comments (`///`) to all public items
- Include examples in doc comments when helpful
- Update README.md for user-facing changes
- Update CHANGELOG.md for all changes

## Testing Requirements

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Skip functional tests (they interact with real API)
SKIP_FUNCTIONAL_TESTS=1 cargo test
```

### Test Coverage Requirements

- **All new features** must include tests
- **Bug fixes** should include regression tests
- **Aim for 70%+ code coverage**
- **Test edge cases** and error conditions

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_descriptive_name() {
        // Arrange
        let input = "test data";
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected_value);
    }

    #[tokio::test]
    async fn test_async_function() {
        // Test async code
    }
}
```

## Pull Request Process

### Before Submitting

1. **Ensure all tests pass:**
   ```bash
   cargo test
   ```

2. **Format your code:**
   ```bash
   cargo fmt
   ```

3. **Run Clippy:**
   ```bash
   cargo clippy -- -D warnings
   ```

4. **Update CHANGELOG.md** under `[Unreleased]`

5. **Update documentation** if needed

### Submitting a Pull Request

1. **Push your branch** to your fork:
   ```bash
   git push origin feature/your-feature-name
   ```

2. **Create a Pull Request** on GitHub

3. **Fill out the PR template** with:
   - Description of changes
   - Related issue numbers
   - Testing performed
   - Screenshots (if UI changes)

4. **Respond to review feedback** promptly

### PR Title Format

Use conventional commit format:
- `feat: Add export to CSV functionality`
- `fix: Resolve memory leak in cache`
- `docs: Update installation instructions`
- `refactor: Split monday.rs into modules`
- `test: Add tests for delete functionality`
- `chore: Update dependencies`

### PR Review Process

- At least one maintainer approval required
- All CI checks must pass
- No merge conflicts
- Code review feedback addressed

## Reporting Issues

### Bug Reports

Include:
- **Description** of the bug
- **Steps to reproduce**
- **Expected behavior**
- **Actual behavior**
- **Environment** (OS, Rust version, etc.)
- **Error messages** or logs
- **Screenshots** if applicable

### Feature Requests

Include:
- **Description** of the feature
- **Use case** and motivation
- **Proposed solution** (if any)
- **Alternatives considered**

### Security Vulnerabilities

**Do not** report security vulnerabilities through public issues.

Instead, follow the process in [SECURITY.md](SECURITY.md):
1. Use GitHub Security Advisories
2. Or email the maintainers directly

## Development Tips

### Useful Commands

```bash
# Build in release mode
cargo build --release

# Run the application
cargo run

# Run with arguments
cargo run -- query -D 2026-01-15

# Check for unused dependencies
cargo +nightly udeps

# Generate documentation
cargo doc --no-deps --open

# Run benchmarks (if available)
cargo bench
```

### Debugging

```bash
# Enable debug logging
RUST_LOG=debug cargo run

# Run with backtrace
RUST_BACKTRACE=1 cargo run
```

### IDE Setup

**VS Code:**
- Install `rust-analyzer` extension
- Install `CodeLLDB` for debugging
- Use workspace settings in `.vscode/settings.json`

**IntelliJ IDEA:**
- Install Rust plugin
- Configure Rust toolchain in settings

## Questions?

- Open a [GitHub Discussion](https://github.com/vgrazian/claim/discussions)
- Check existing [Issues](https://github.com/vgrazian/claim/issues)
- Review the [README](README.md) and [documentation](docs/)

## License

By contributing to Claim, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to Claim! 🎉