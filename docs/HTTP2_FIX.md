# HTTP/2 Configuration Fix

## Issue

When running `cargo run -- query --limit 10 --verbose`, the application failed with:

```
Error: Failed to send request to Monday.com: error sending request for url (https://api.monday.com/v2): http2 error: connection error detected: frame with invalid size
```

## Root Cause

The `http2_prior_knowledge()` configuration in `MondayClient::new()` forced HTTP/2 without proper negotiation, which Monday.com's API doesn't support correctly.

## Fix Applied

Removed `.http2_prior_knowledge()` from the HTTP client builder in `src/monday.rs` line 232.

The client will now automatically negotiate HTTP/2 via ALPN if the server supports it, providing better compatibility.

## Status

✅ Fixed - Application now works correctly
✅ All tests pass (107 tests)
✅ Connection pooling still active and working
