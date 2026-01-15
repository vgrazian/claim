#!/bin/bash

# Automatic cleanup script for test entries (no confirmation)
# This script finds and deletes all test entries from the Monday.com board
# Uses work item filter to find test entries

echo "🧹 Cleaning up test entries automatically..."
echo ""

# Get all test entries using work item filter
echo "Querying for test entries with work item 'DELETE.ME'..."
OUTPUT=$(./target/release/claim query -D 2025-12-12 -d 1 -w "DELETE.ME" 2>&1)

# Extract IDs from the output (looking for "ID: <number>")
TEST_IDS=$(echo "$OUTPUT" | grep -o 'ID: [0-9]*' | sed 's/ID: //' | sort -u)

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

# Delete each entry
DELETED=0
FAILED=0

for ID in $TEST_IDS; do
    echo "Deleting entry ID: $ID"
    if ./target/release/claim delete -x "$ID" -y 2>&1 | grep -q "deleted successfully\|Item deleted"; then
        echo "  ✓ Deleted"
        ((DELETED++))
    else
        echo "  ✗ Failed"
        ((FAILED++))
    fi
    # Small delay to avoid rate limiting
    sleep 0.5
done

echo ""
echo "✅ Cleanup complete: $DELETED deleted, $FAILED failed"

# Verify cleanup
echo ""
echo "Verifying cleanup..."
REMAINING=$(./target/release/claim query -D 2025-12-12 -d 1 -w "DELETE.ME" 2>&1 | grep -o 'ID: [0-9]*' | wc -l | tr -d ' ')
if [ "$REMAINING" -eq 0 ]; then
    echo "✅ All test entries cleaned up successfully!"
else
    echo "⚠️  Warning: $REMAINING test entries still remain"
fi

# Made with Bob
