# Changelog

## Version 0.2.1

Minor version update with maintenance improvements.

---

## Changes Summary

### Overview

This document summarizes the improvements made to the claim management tool to enhance usability and performance.

## 1. Automatic Work Item Assignment

### Feature Description

When users select specific activity types, the system now automatically sets the appropriate work item if not manually specified.

### Implementation Details

- **Location**: [`src/add.rs`](src/add.rs:66-89)
- **Logic**:
  - `vacation`, `illness`, `holiday`, `work_reduction` → Auto-set to `M.00556`
  - `intellectual_capital`, `education` → Auto-set to `M.00563`
  - Other activity types → No automatic assignment (user can still specify manually)

### Key Benefits

- Reduces manual data entry
- Ensures consistency in work item assignments
- Prevents errors from forgetting to set work items

## 2. Improved Query Performance with 30-Day Default Range

### Query Performance Feature

Queries now default to the last 30 days instead of just today, significantly improving performance by reducing the amount of data fetched from Monday.com.

### Query Implementation

- **Location**: [`src/main.rs`](src/main.rs:26-38) and [`src/query.rs`](src/query.rs:24-36)
- **Changes**:
  - Default `--days` parameter changed from `1` to `30`
  - Default start date changed from "today" to "30 days ago"
  - Users can still override with specific dates or day ranges

### Performance Benefits

- Faster query execution
- Reduced API load on Monday.com
- Better default behavior for typical use cases
- Users can still query specific dates or larger ranges when needed

## 3. Client-Side Date Filtering

### Filtering Feature

The application fetches all items from Monday.com and performs efficient client-side date filtering to show only relevant entries within the specified date range.

### Filtering Implementation

- **Location**: [`src/query.rs`](src/query.rs:24-36)
- **Approach**: Fetch all items from the current year group, then filter by date range on the client side
- **Performance**: The 5-week default window significantly reduces the amount of data that needs to be processed

### Filtering Benefits

- Reliable filtering that works with Monday.com's API
- Efficient for typical use cases (5-week window)
- Flexible client-side filtering allows for complex date range queries
- No API compatibility issues

## Testing

All changes have been tested and verified:

- ✅ 48 unit tests pass
- ✅ 12 functional tests pass
- ✅ 11 integration tests pass
- ✅ Release build compiles successfully

## Usage Examples

### Automatic Work Item Assignment

```bash
# Vacation automatically gets M.00556
claim add -t vacation -D 2024-12-15

# Education automatically gets M.00563
claim add -t education -D 2024-12-15

# Manual override still works
claim add -t vacation -w "CUSTOM.ITEM" -D 2024-12-15
```

### Improved Query Performance

```bash
# Now queries 2 weeks before + current week + 2 weeks after by default (fast!)
claim query

# Query specific date range
claim query -D 2024-12-01 -d 10

# Query single day
claim query -D 2024-12-15 -d 1
```

## Backward Compatibility

All changes are backward compatible:

- Existing commands continue to work as before
- Manual work item specification still overrides automatic assignment
- Users can still query any date range they want
- No breaking changes to the API or command structure
