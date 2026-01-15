# Test Scripts

This directory contains various test scripts for the claim application.

## Optimization Tests

### test_all_optimizations.sh

Comprehensive test suite for all database query optimizations (#1-5).

**Usage:**

```bash
./tests/scripts/test_all_optimizations.sh
```

**Tests:**

- Optimization #1: Combined board+items query
- Optimization #2: HTTP connection pooling
- Optimization #3: In-memory cache (cache hits/misses)
- Optimization #4: Smart pagination with adaptive delays
- Optimization #5: Performance metrics tracking

**Expected Output:**

- Cache MISS on first queries
- Cache HIT on repeated queries
- Significant speedup for cached queries (5-50x faster)
- All metrics displayed in verbose mode

---

## Functional Tests

### run_functional_tests.sh

Runs the full functional test suite.

**Usage:**

```bash
./tests/scripts/run_functional_tests.sh
```

---

### run_tests.sh

Runs all unit and integration tests.

**Usage:**

```bash
./tests/scripts/run_tests.sh
```

---

## Cleanup Scripts

### cleanup_test_entries.sh

Manually clean up test entries from Monday.com board.

**Usage:**

```bash
./tests/scripts/cleanup_test_entries.sh
```

---

### cleanup_test_entries_auto.sh

Automatically clean up test entries (use with caution).

**Usage:**

```bash
./tests/scripts/cleanup_test_entries_auto.sh
```

---

## Quick Start

To test all optimizations:

```bash
cd /path/to/claim
./tests/scripts/test_all_optimizations.sh
```

To run all tests:

```bash
cargo test
./tests/scripts/run_functional_tests.sh
```

---

## Notes

- All scripts should be run from the project root directory
- Ensure you have a valid API key configured before running tests
- The optimization tests will make real API calls to Monday.com
- Cache tests require running queries in sequence to observe cache behavior
