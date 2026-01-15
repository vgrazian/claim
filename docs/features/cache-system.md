# Cache Feature for Client and Work Item Pairs

## Overview

The command line tool now includes a caching feature that stores client and work item pairs from the last 4 weeks **for the current user only**. When adding entries interactively, users can select from previously used combinations instead of typing them manually.

**Important**: The cache only stores entries that belong to the current user and have both a customer name and work item filled in.

## Features

### 1. Automatic Cache Management

- **Cache Location**: Stored in your system's cache directory (e.g., `~/.cache/claim/entries_cache.json` on Linux/macOS)
- **Auto-refresh**: Cache is automatically refreshed if it's older than 24 hours when running `claim add` interactively
- **Data Stored**: Customer name, work item, and last used date for each unique pair

### 2. Manual Cache Refresh

You can manually refresh the cache using the `-r` or `--refresh-cache` flag:

```bash
claim add --refresh-cache
```

This will:

- Query the last 4 weeks (28 days) of entries
- Extract all unique customer + work item pairs
- Store them with their most recent usage date
- Display the number of unique entries cached

### 3. Interactive Selection

When running `claim add` without parameters, the tool will:

1. Show up to 10 most recent entries from the cache
2. Display them with their last used date
3. Allow you to select an entry by number
4. Pre-fill the customer and work item fields
5. Continue prompting for other details (date, activity type, hours, etc.)

#### Example Interactive Session

```
=== Add New Claim ===

📋 Recent entries (from last 4 weeks):
  1. Acme Corp | WI-12345 (last used: 2025-12-15)
  2. Beta Inc | WI-67890 (last used: 2025-12-14)
  3. Gamma LLC | WI-11111 (last used: 2025-12-10)
  4. Delta Co | WI-22222 (last used: 2025-12-08)
  5. Epsilon Ltd | WI-33333 (last used: 2025-12-05)

You can select an entry by number, or enter details manually.
Select entry number (or press Enter to enter manually): 1

✅ Selected: Acme Corp | WI-12345

Date (YYYY-MM-DD, YYYY.MM.DD, or YYYY/MM/DD, optional - default: today): 
Activity type (enter number or name, optional - default: billable): 
Comment (optional): Working on feature X
Number of hours (optional): 8
Number of working days (optional, default: 1, skips weekends): 
```

## Command Line Usage

### Add with Cache Refresh

```bash
# Refresh cache and add entry interactively
claim add --refresh-cache

# Refresh cache and add entry with parameters
claim add --refresh-cache -D 2025-12-16 -c "Acme Corp" -w "WI-12345" -H 8
```

### Add Interactively (uses cache if available)

```bash
# Simply run add without parameters
claim add
```

### Add with Parameters (bypasses interactive mode)

```bash
# Standard add command (cache is still refreshed if stale)
claim add -D 2025-12-16 -c "Acme Corp" -w "WI-12345" -H 8
```

## Cache Behavior

### When Cache is Refreshed

1. **Automatic refresh**: When cache is older than 24 hours and you run `claim add` interactively
2. **Manual refresh**: When you use the `--refresh-cache` flag
3. **First run**: When no cache exists yet

### What Gets Cached

- Only entries with both customer name AND work item are cached
- Empty or null values are filtered out
- Duplicate pairs are merged, keeping the most recent date
- Entries are sorted by most recent usage

### Cache Staleness

The cache is considered "stale" after 24 hours. When stale:

- It will be automatically refreshed on the next interactive add
- You can force a refresh with `--refresh-cache`
- The cache is still usable even when stale

## Technical Details

### Cache Structure

```json
{
  "entries": [
    {
      "customer": "Acme Corp",
      "work_item": "WI-12345",
      "last_used": "2025-12-15"
    }
  ],
  "last_updated": "2025-12-16T14:30:00+01:00"
}
```

### Implementation

- **Module**: `src/cache.rs`
- **Storage**: JSON file in system cache directory
- **Query Range**: Last 28 days (4 weeks)
- **Max Display**: 10 most recent entries in interactive mode
- **Deduplication**: Automatic based on customer + work item pair

## Benefits

1. **Faster Entry**: Select from recent entries instead of typing
2. **Consistency**: Reduces typos in customer names and work items
3. **Convenience**: See what you've been working on recently
4. **Efficiency**: Cache is automatically managed and refreshed

## Troubleshooting

### Cache Not Showing Entries

- Run `claim add --refresh-cache` to manually refresh
- Check that you have entries in the last 4 weeks with both customer and work item filled

### Cache Location

To find your cache file location, it's typically:

- **Linux**: `~/.cache/claim/entries_cache.json`
- **macOS**: `~/Library/Caches/claim/entries_cache.json`
- **Windows**: `%LOCALAPPDATA%\claim\cache\entries_cache.json`

### Clear Cache

To clear the cache, simply delete the cache file. It will be recreated on the next refresh.

## Examples

### Example 1: Quick Entry with Recent Project

```bash
$ claim add
=== Add New Claim ===

📋 Recent entries (from last 4 weeks):
  1. Project Alpha | M.12345 (last used: 2025-12-15)
  2. Project Beta | M.67890 (last used: 2025-12-14)

Select entry number (or press Enter to enter manually): 1
✅ Selected: Project Alpha | M.12345

Date: [Enter for today]
Activity type: [Enter for billable]
Comment: Daily standup and code review
Hours: 8
Days: [Enter for 1]
```

### Example 2: Force Cache Refresh

```bash
$ claim add --refresh-cache
🔄 Refreshing cache from last 4 weeks...
✅ Cache refreshed with 15 unique entries

=== Add New Claim ===
[... continues with interactive prompts ...]
```

### Example 3: Verbose Mode with Cache

```bash
$ claim add --refresh-cache -v
🔄 Refreshing cache from last 4 weeks...
Querying board 6500270039 for user 'John Doe'...
✅ Cache refreshed with 15 unique entries

=== Add New Claim ===
[... shows cached entries ...]
