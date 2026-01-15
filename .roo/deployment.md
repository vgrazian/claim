# Deployment Instructions

## Executable Location

The `claim` executable is installed at: `/Users/<username>/.local/bin/claim`

## Build and Deploy Process

After making code changes:

1. **Build the release version:**

   ```bash
   cargo build --release
   ```

2. **Copy to installation directory:**

   ```bash
   cp target/release/claim /Users/valer/.local/bin/claim
   ```

3. **Commit and push changes:**

   ```bash
   git add -A
   git commit -m "your commit message"
   git push
   ```

## Quick Deploy Script

You can use this one-liner to build and deploy:

```bash
cargo build --release && cp target/release/claim /Users/valer/.local/bin/claim
```

## Verification

To verify the installation:

```bash
which claim
# Should output: /Users/valer/.local/bin/claim

claim --version
# Should show the current version
