# Markdown Style Guide

This document defines the markdown formatting standards for the Claim project documentation.

## General Principles

1. **Consistency**: Use the same formatting throughout all documents
2. **Readability**: Prioritize clear, scannable content
3. **Accessibility**: Ensure content is accessible to all users
4. **Maintainability**: Make documents easy to update

## Formatting Rules

### Headers

- Use ATX-style headers (`#`) instead of Setext-style (`===` or `---`)
- Include a space after the `#` symbols
- Use sentence case for headers (capitalize first word and proper nouns)
- Leave one blank line before and after headers (except at document start)

```markdown
# Main Title

## Section Header

### Subsection Header
```

### Lists

- Use `-` for unordered lists (not `*` or `+`)
- Use `1.` for ordered lists (numbers auto-increment)
- Indent nested lists with 2 spaces
- Leave blank lines before and after lists

```markdown
- First item
- Second item
  - Nested item
  - Another nested item
- Third item

1. First step
2. Second step
3. Third step
```

### Code Blocks

- Use fenced code blocks with language identifiers
- Indent code blocks with 3 backticks
- Always specify the language for syntax highlighting

````markdown
```rust
fn main() {
    println!("Hello, world!");
}
```

```bash
cargo build --release
```
````

### Inline Code

- Use single backticks for inline code
- Use for: commands, file names, variable names, short code snippets

```markdown
Run `cargo test` to execute tests.
Edit the `Cargo.toml` file.
Set the `RUST_LOG` environment variable.
```

### Links

- Use descriptive link text (not "click here")
- Use reference-style links for repeated URLs
- Use relative paths for internal documentation

```markdown
See the [installation guide](INSTALL.md) for details.
Check out [Rust documentation][rust-docs] for more information.

[rust-docs]: https://doc.rust-lang.org/
```

### Emphasis

- Use `**bold**` for strong emphasis
- Use `*italic*` for mild emphasis
- Use `***bold italic***` sparingly
- Don't use underscores for emphasis

```markdown
This is **important**.
This is *emphasized*.
This is ***very important***.
```

### Tables

- Align columns with pipes
- Use header separator with at least 3 dashes
- Left-align text, right-align numbers

```markdown
| Feature      | Status | Priority |
|--------------|--------|----------|
| Error types  | Done   | High     |
| CI/CD        | Done   | High     |
| Logging      | Done   | Medium   |
```

### Blockquotes

- Use `>` for blockquotes
- Add space after `>`
- Use for notes, warnings, tips

```markdown
> **Note**: This is an important note.

> **Warning**: This action cannot be undone.
```

### Horizontal Rules

- Use `---` (three dashes) for horizontal rules
- Leave blank lines before and after

```markdown
Section content here.

---

Next section content.
```

## Document Structure

### README.md Structure

1. **Title and Description**: Project name and brief description
2. **Badges** (optional): Build status, version, license
3. **Table of Contents** (for long documents)
4. **Quick Start**: Minimal steps to get started
5. **Installation**: Detailed installation instructions
6. **Usage**: How to use the project
7. **Examples**: Practical examples
8. **Configuration**: Configuration options
9. **Contributing**: How to contribute
10. **License**: License information

### Other Documentation

- **CHANGELOG.md**: Follow [Keep a Changelog](https://keepachangelog.com/) format
- **CONTRIBUTING.md**: Guidelines for contributors
- **SECURITY.md**: Security policy and vulnerability reporting
- **LICENSE**: License text (plain text, not markdown)

## Best Practices

### File Names

- Use UPPERCASE for important docs: `README.md`, `LICENSE`, `CHANGELOG.md`
- Use descriptive names: `INSTALLATION.md`, `API_REFERENCE.md`
- Use underscores or hyphens: `STYLE_GUIDE.md` or `style-guide.md`

### Line Length

- Aim for 80-100 characters per line for readability
- Break long lines at natural points (after punctuation)
- Don't break URLs or code blocks

### Whitespace

- Use blank lines to separate sections
- Don't use multiple consecutive blank lines
- End files with a single newline
- No trailing whitespace

### Special Characters

- Use proper Unicode characters: `—` (em dash), `'` (apostrophe)
- Use HTML entities when needed: `&nbsp;`, `&copy;`
- Escape special markdown characters: `\*`, `\#`, `\[`

## Examples

### Good Example

```markdown
# Project Name

A brief description of what this project does.

## Installation

Install using cargo:

```bash
cargo install project-name
```

## Usage

Basic usage example:

```rust
use project_name::Client;

let client = Client::new();
client.connect()?;
```

## Features

- **Fast**: Optimized for performance
- **Safe**: Memory-safe Rust implementation
- **Easy**: Simple, intuitive API

## License

MIT License - see [LICENSE](LICENSE) for details.

```

### Bad Example

```markdown
#Project Name
A brief description of what this project does.
##Installation
Install using cargo:
```

cargo install project-name

```
##Usage
Basic usage example:
```

use project_name::Client;
let client = Client::new();
client.connect()?;

```
##Features
* Fast: Optimized for performance
* Safe: Memory-safe Rust implementation
* Easy: Simple, intuitive API
##License
MIT License - see LICENSE for details.
```

## Validation

### Automated Checks

Use these tools to validate markdown:

```bash
# Install markdownlint
npm install -g markdownlint-cli

# Check all markdown files
markdownlint '**/*.md'

# Fix auto-fixable issues
markdownlint '**/*.md' --fix
```

### Manual Review Checklist

- [ ] Headers use ATX style with proper spacing
- [ ] Lists use consistent markers (`-` for unordered)
- [ ] Code blocks have language identifiers
- [ ] Links use descriptive text
- [ ] Tables are properly aligned
- [ ] No trailing whitespace
- [ ] File ends with single newline
- [ ] Line length is reasonable (80-100 chars)

## Project-Specific Guidelines

### Claim Project

1. **Command Examples**: Always show both short and long forms
2. **Output Examples**: Include realistic output with masked sensitive data
3. **Version Numbers**: Use semantic versioning (e.g., `v0.1.0`)
4. **File Paths**: Use platform-agnostic examples when possible
5. **API Keys**: Never include real API keys, use placeholders

## References

- [CommonMark Spec](https://commonmark.org/)
- [GitHub Flavored Markdown](https://github.github.com/gfm/)
- [Markdown Guide](https://www.markdownguide.org/)
- [Keep a Changelog](https://keepachangelog.com/)

---

**Last Updated**: January 2026
