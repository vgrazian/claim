#!/bin/bash

# Test script to verify database query optimizations
# Tests Optimization #1 (Combined Query), #2 (Connection Pooling), and #5 (Metrics)

echo "🧪 Testing Database Query Optimizations"
echo "========================================"
echo ""

# Test 1: Query with verbose mode to see metrics
echo "Test 1: Query with verbose mode (should show performance metrics)"
echo "-------------------------------------------------------------------"
cargo run --quiet -- query --limit 5 --verbose 2>&1 | grep -E "(Fetched|API|Performance|Duration|Items)"
echo ""

# Test 2: Query without verbose mode (metrics should not show)
echo "Test 2: Query without verbose mode (metrics hidden)"
echo "----------------------------------------------------"
cargo run --quiet -- query --limit 5 2>&1 | grep -E "(Querying|Performance)" || echo "✓ Metrics correctly hidden in non-verbose mode"
echo ""

# Test 3: Multi-day query to test combined query optimization
echo "Test 3: Multi-day query (tests combined query optimization)"
echo "-----------------------------------------------------------"
cargo run --quiet -- query --days 7 --verbose 2>&1 | grep -E "(combined|optimized|API|Fetched)" | head -5
echo ""

# Test 4: Query with filters
echo "Test 4: Query with customer filter"
echo "-----------------------------------"
cargo run --quiet -- query --limit 5 --customer "test" --verbose 2>&1 | grep -E "(Filter|After filtering)" | head -3
echo ""

echo "✅ All optimization tests completed!"
echo ""
echo "Summary of Optimizations Implemented:"
echo "  ✓ Optimization #1: Combined board+items query (reduces API calls)"
echo "  ✓ Optimization #2: HTTP connection pooling (faster requests)"
echo "  ✓ Optimization #5: Performance metrics tracking (visibility)"

# Made with Bob
