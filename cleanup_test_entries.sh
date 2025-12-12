#!/bin/bash

# Cleanup script for test entries
# This script finds and deletes all test entries from the Monday.com board

echo "🧹 Cleaning up test entries..."
echo ""

# Get all test entries for today
OUTPUT=$(./target/release/claim query -D 2025-12-12 -d 1 2>&1)

# Extract IDs of TEST entries
TEST_IDS=$(echo "$OUTPUT" | grep -A 10 "Customer.*TEST" | grep "ID:" | sed 's/.*ID: \([0-9]*\).*/\1/' | sort -u)

if [ -z "$TEST_IDS" ]; then
    echo "✅ No test entries found to clean up"
    exit 0
fi

echo "Found test entries to delete:"
echo "$TEST_IDS"
echo ""

# Count entries
COUNT=$(echo "$TEST_IDS" | wc -l | tr -d ' ')
echo "Total test entries to delete: $COUNT"
echo ""

# Ask for confirmation
read -p "Do you want to delete these entries? (y/N) " -n 1 -r
echo ""

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ Cleanup cancelled"
    exit 0
fi

# Delete each entry
DELETED=0
FAILED=0

for ID in $TEST_IDS; do
    echo "Deleting entry ID: $ID"
    if ./target/release/claim delete -x "$ID" -y > /dev/null 2>&1; then
        echo "  ✓ Deleted"
        ((DELETED++))
    else
        echo "  ✗ Failed"
        ((FAILED++))
    fi
done

echo ""
echo "✅ Cleanup complete: $DELETED deleted, $FAILED failed"

# Made with Bob
