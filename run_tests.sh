#!/bin/bash

echo "Running cargo tests for claim application..."

# Run unit tests
echo "=== Running unit tests ==="
cargo test --lib -- --test-threads=1

# Run integration tests
echo "=== Running integration tests ==="
cargo test --test integration_tests -- --test-threads=1

# Run all tests
echo "=== Running all tests ==="
cargo test -- --test-threads=1

echo "Tests completed!"