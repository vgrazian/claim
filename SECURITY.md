# Security Policy

## Supported Versions

We release patches for security vulnerabilities for the following versions:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| < 0.1   | :x:                |

## Reporting a Vulnerability

We take the security of the Claim project seriously. If you believe you have found a security vulnerability, please report it to us as described below.

### How to Report a Security Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report them via one of the following methods:

1. **GitHub Security Advisories** (Preferred)
   - Go to the [Security tab](../../security/advisories) of this repository
   - Click "Report a vulnerability"
   - Fill out the form with details about the vulnerability

2. **Email**
   - Send an email to the project maintainers
   - Include the word "SECURITY" in the subject line
   - Provide detailed information about the vulnerability

### What to Include in Your Report

Please include the following information in your report:

- **Type of vulnerability** (e.g., authentication bypass, SQL injection, XSS, etc.)
- **Full paths of source file(s)** related to the vulnerability
- **Location of the affected source code** (tag/branch/commit or direct URL)
- **Step-by-step instructions** to reproduce the issue
- **Proof-of-concept or exploit code** (if possible)
- **Impact of the vulnerability** including how an attacker might exploit it
- **Any potential mitigations** you've identified

### What to Expect

After you submit a report, you can expect:

1. **Acknowledgment**: We will acknowledge receipt of your vulnerability report within 48 hours
2. **Assessment**: We will investigate and assess the vulnerability within 7 days
3. **Updates**: We will keep you informed about our progress
4. **Resolution**: We will work on a fix and coordinate disclosure timing with you
5. **Credit**: We will credit you in the security advisory (unless you prefer to remain anonymous)

### Security Update Process

When we receive a security bug report, we will:

1. Confirm the problem and determine affected versions
2. Audit code to find any similar problems
3. Prepare fixes for all supported versions
4. Release new versions as soon as possible
5. Publish a security advisory on GitHub

## Security Best Practices for Users

### API Key Security

- **Never commit your API key** to version control
- **Store API keys securely** in the configuration file (located in system config directory)
- **Rotate API keys regularly** on Monday.com
- **Use environment variables** for CI/CD pipelines instead of hardcoded keys
- **Limit API key permissions** to only what's necessary

### Configuration File Security

The configuration file is stored at:

- **Linux**: `~/.config/claim/config.json`
- **macOS**: `~/Library/Application Support/com.yourname.claim/config.json`
- **Windows**: `C:\Users\Username\AppData\Roaming\yourname\claim\config\config.json`

Ensure this file has appropriate permissions:

```bash
# Linux/macOS
chmod 600 ~/.config/claim/config.json
```

### Network Security

- **Use HTTPS only** - The application only communicates with Monday.com over HTTPS
- **Verify SSL certificates** - The application validates SSL certificates by default
- **Use trusted networks** - Avoid using the application on untrusted networks

### Dependency Security

We use automated tools to monitor dependencies for known vulnerabilities:

- **GitHub Dependabot** - Automatically creates PRs for dependency updates
- **cargo-audit** - Runs in CI to check for security advisories
- **Regular updates** - We regularly update dependencies to patch vulnerabilities

### Build Security

- **Verify releases** - Download releases only from official GitHub releases
- **Check checksums** - Verify file integrity using provided checksums (when available)
- **Build from source** - For maximum security, build from source after reviewing the code

## Known Security Considerations

### API Key Storage

Currently, API keys are stored in plain text in the configuration file. While the file is in a protected system directory, this is not the most secure approach.

**Mitigation**: We recommend:

- Setting appropriate file permissions (600 on Unix-like systems)
- Using OS-level encryption (FileVault on macOS, BitLocker on Windows, LUKS on Linux)
- Rotating API keys regularly

**Future Enhancement**: We plan to implement OS keychain integration for more secure credential storage.

### Functional Tests

Functional tests interact with real Monday.com data. To prevent accidental data modification:

- Tests use unique identifiers with timestamps
- Tests automatically clean up created entries
- Tests are skipped in CI environments by default
- Use `SKIP_FUNCTIONAL_TESTS=1` to disable functional tests

## Security Advisories

Security advisories will be published in the following locations:

1. [GitHub Security Advisories](../../security/advisories)
2. Project README.md
3. Release notes for patched versions

## Disclosure Policy

We follow a **coordinated disclosure** policy:

1. Security issues are fixed privately
2. Fixes are released in a new version
3. Security advisory is published after the fix is available
4. We coordinate with reporters on disclosure timing

## Contact

For security-related questions or concerns that are not vulnerabilities, please open a regular GitHub issue or discussion.

## Attribution

We would like to thank the following individuals for responsibly disclosing security vulnerabilities:

- (No vulnerabilities reported yet)

---

**Last Updated**: January 2026
