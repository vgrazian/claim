# Test Suite Improvements

## Overview

Enhanced the functional test suite to ensure safe testing without damaging existing production data.

## Key Improvements

### 1. Unique Test Identifiers

- **Before**: Tests used static values like "TEST" and "DELETE.ME"
- **After**: Tests generate unique identifiers using timestamps
  - Customer: `TEST_<timestamp>` (e.g., `TEST_1702371234`)
  - Work Item: `TEST.DELETE.ME.<timestamp>` (e.g., `TEST.DELETE.ME.1702371234`)

### 2. Improved Query Range

- **Before**: Tests queried only 1 day (`-d 1`)
- **After**: Tests query 35 days (`-d 35`) to match the new default behavior
- **Benefit**: Tests now validate the improved 5-week query window feature

### 3. Enhanced Safety Documentation

Added comprehensive safety documentation at the top of the test file:

- Explains the cleanup strategy
- Documents safety measures
- Clarifies that tests skip in CI environments

### 4. Better Test Output

- Tests now print the unique identifiers being used
- Clearer indication of test entry creation
- Warning messages when test entries are found in queries

## Safety Measures

### Automatic Cleanup

- Each test tracks created entry IDs using thread-local storage
- Cleanup runs at the end of each test, even if the test fails
- Multiple entries (e.g., from multi-day tests) are all tracked and cleaned up

### CI Environment Protection

- All tests that modify data skip execution in CI environments
- Prevents accidental data modification in automated builds
- Only help and read-only tests run in CI

### Unique Identifiers

- Timestamp-based identifiers ensure no collision with real data
- Easy to identify test entries in the Monday.com board
- Pattern matching allows for manual cleanup if needed

## Test Coverage

The test suite validates:

1. ✅ Adding single entries
2. ✅ Adding multi-day entries
3. ✅ Querying with various parameters
4. ✅ Deleting entries
5. ✅ Verbose output
6. ✅ Help commands
7. ✅ Date-specific queries
8. ✅ Query limits
9. ✅ Workflow operations

## Running Tests

### Run all tests

```bash
cargo test
```

### Run only functional tests

```bash
cargo test --test functional_tests
```

### Run with output

```bash
cargo test --test functional_tests -- --nocapture
```

## Manual Cleanup

### Automatic Cleanup Script

The easiest way to clean up test entries is to use the provided script:

```bash
chmod +x cleanup_test_entries_auto.sh
./cleanup_test_entries_auto.sh
```

This script will:

1. Query for all entries with work item "DELETE.ME"
2. Extract their IDs
3. Delete each entry automatically
4. Verify the cleanup was successful

### Manual Cleanup Steps

If you prefer to clean up manually:

1. **Query for test entries using work item filter:**

```bash
./target/release/claim query -D 2025-12-12 -d 1 -w "DELETE.ME"
```

2. **Extract IDs from the output and delete each one:**

```bash
# For each ID found, run:
./target/release/claim delete -x <ENTRY_ID> -y
```

3. **Verify cleanup:**

```bash
./target/release/claim query -D 2025-12-12 -d 1 -w "DELETE.ME"
# Should return no entries
```

### Why Cleanup Fails

The test cleanup can fail for several reasons:

- Thread-local storage doesn't persist across test boundaries
- Entry IDs may not be properly extracted from command output
- Tests may crash before cleanup runs
- Rate limiting may cause delete operations to fail

**Solution:** The improved test suite now includes:

- Orphaned entry detection and cleanup
- Work item filter-based queries for reliable ID extraction
- Rate limiting delays between delete operations
- Verification step after cleanup

## Best Practices

1. **Always run tests in a test environment** if possible
2. **Check for leftover test entries** after running tests
3. **Use the `-y` flag** in tests to skip confirmation prompts
4. **Track all created entries** for proper cleanup
5. **Use unique identifiers** to avoid conflicts with real data

## Future Improvements

Potential enhancements for the test suite:

- Add integration tests with a dedicated test board
- Implement test fixtures for common scenarios
- Add performance benchmarks
- Create end-to-end workflow tests
- Add tests for error handling and edge cases
