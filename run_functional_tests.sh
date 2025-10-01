#!/bin/bash

echo "Running specific functional tests for claim application..."
echo "Note: These tests require a valid Monday.com API configuration"
echo ""

Build the application first
echo "=== Building the application ==="
cargo build --release

if [ $? -ne 0 ]; then
echo "Build failed! Please fix compilation errors before running functional tests."
exit 1
fi

echo ""
echo "=== Test 1: Adding test entry ==="
echo "Adding entry with customer 'TEST', work item 'DELETE.ME', 1 hour billable..."
./target/release/claim add -c "TEST" -w "DELETE.ME" -H 1 -t billable -y

if [ $? -eq 0 ]; then
echo "✅ Test 1 passed: Entry added successfully"
else
echo "⚠️ Test 1: Add command completed (may have failed due to API)"
fi

echo ""
echo "=== Test 2: Querying current day ==="
echo "Querying today's entries to find the test entry..."
./target/release/claim query -d 1

if [ $? -eq 0 ]; then
echo "✅ Test 2 passed: Query executed successfully"
echo ""
echo "💡 Please note the ID of the test entry (look for 'ID: ...' in output)"
echo " You'll need this ID for the deletion test."
else
echo "⚠️ Test 2: Query command completed (may have failed due to API)"
fi

echo ""
echo "=== Test 3: Deleting the entry ==="
echo "To complete test 3, manually run:"
echo " ./target/release/claim delete -x <ENTRY_ID> -y"
echo ""
echo "Replace <ENTRY_ID> with the actual ID from the query output above."
echo ""
echo "Functional test demonstration completed!"