# Interactive UI Documentation

This document provides comprehensive documentation for the Interactive UI feature of the Claim management tool.

## Table of Contents

1. [Overview](#overview)
2. [Design Specification](#design-specification)
3. [Implementation Details](#implementation-details)
4. [User Feedback Updates](#user-feedback-updates)
5. [UX Improvements](#ux-improvements)
6. [Features](#features)
7. [Usage Guide](#usage-guide)
8. [Technical Architecture](#technical-architecture)
9. [Future Enhancements](#future-enhancements)

---

## Overview

The interactive UI feature provides a terminal-based application for managing Monday.com claims with an intuitive interface, week-based views, charts, and real-time editing capabilities.

### Key Features

- **Week-Based Calendar View**: Displays current week (Monday-Sunday) with all entries
- **Activity Summary Chart**: Bar chart showing distribution of activity types
- **Cache Panel**: Shows 15 most recent customer/work item pairs
- **Loading Indicators**: Animated Braille spinner during data loading
- **Inline Form Editor**: Add and edit entries with full field navigation
- **Report Mode**: Analyze work by customer/project with daily breakdown

---

## Design Specification

### Architecture Components

#### 1. Main Application (`src/interactive.rs`)

- Entry point for interactive mode
- Manages application state and event loop
- Coordinates between UI components and backend logic

#### 2. UI Layout

```
┌─────────────────────────────────────────────────────────────────┐
│ Claim Manager - Week of Dec 16-20, 2025          [q]uit [h]elp │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  Week View (Main Pane)                                           │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ Mon 16  │ Tue 17  │ Wed 18  │ Thu 19  │ Fri 20  │ Total   │  │
│  ├─────────┼─────────┼─────────┼─────────┼─────────┼─────────┤  │
│  │ CUST-A  │ CUST-A  │ CUST-B  │ CUST-B  │ CUST-A  │         │  │
│  │ WI-123  │ WI-123  │ WI-456  │ WI-456  │ WI-123  │         │  │
│  │ 8h      │ 8h      │ 4h      │ 8h      │ 8h      │ 36h     │  │
│  │         │         │ CUST-C  │         │         │         │  │
│  │         │         │ WI-789  │         │         │         │  │
│  │         │         │ 4h      │         │         │         │  │
│  └─────────┴─────────┴─────────┴─────────┴─────────┴─────────┘  │
│                                                                   │
│  Weekly Summary Chart                                            │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ Billable:    ████████████████████ 28h (70%)                │  │
│  │ Presales:    ████████ 8h (20%)                             │  │
│  │ Overhead:    ████ 4h (10%)                                 │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                   │
│  Messages & Status                                               │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ ✓ Week complete: 40/40 hours                               │  │
│  │ ⚠ Missing entry for Monday                                 │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                   │
├─────────────────────────────────────────────────────────────────┤
│ [←→] Navigate weeks  [a]dd  [d]elete  [e]dit  [r]efresh        │
└─────────────────────────────────────────────────────────────────┘
```

#### 3. Key Components

**WeekView** (`src/interactive/week_view.rs`)

- Displays claims organized by day of the week
- Shows customer, work item, hours for each entry
- Highlights current day
- Supports navigation between weeks

**SummaryChart** (`src/interactive/summary_chart.rs`)

- Bar chart showing activity type distribution
- Percentage and hour breakdown
- Color-coded by activity type

**MessagePane** (`src/interactive/messages.rs`)

- Status messages (success, warnings, errors)
- Validation feedback
- Quick tips and shortcuts

**InputDialog** (`src/interactive/dialogs.rs`)

- Modal dialogs for add/edit/delete operations
- Form-based input with validation
- Autocomplete for customer/work item (from cache)

### State Management

```rust
pub struct AppState {
    // Current view state
    current_week_start: NaiveDate,
    selected_day: Option<NaiveDate>,
    selected_entry: Option<usize>,
    
    // Data
    claims: Vec<ClaimEntry>,
    cache: EntryCache,
    
    // UI state
    mode: AppMode,
    messages: Vec<Message>,
    
    // Backend
    client: MondayClient,
    user: MondayUser,
}

pub enum AppMode {
    Normal,      // Viewing/navigating
    AddEntry,    // Adding new entry
    EditEntry,   // Editing existing entry
    DeleteEntry, // Confirming deletion
    Help,        // Help screen
}
```

### Visual Design

#### Color Scheme

- **Primary**: Cyan (headers, selected items)
- **Success**: Green (completed actions, full weeks)
- **Warning**: Yellow (missing entries, low hours)
- **Error**: Red (errors, validation failures)
- **Info**: Blue (help text, tips)
- **Muted**: Gray (borders, secondary text)

#### Activity Type Colors

- Billable: Green
- Vacation: Blue
- Presales: Cyan
- Overhead: Yellow
- Illness: Red
- Others: White

---

## Implementation Details

### Module Structure

Created `src/interactive/` with the following modules:

```
src/interactive/
├── mod.rs              # Module exports and main entry point
├── app.rs              # Application state and event handling
├── ui.rs               # Main UI layout and rendering
├── week_view.rs        # Week calendar view component
├── summary_chart.rs    # Activity distribution chart
├── form.rs             # Form state management
├── form_ui.rs          # Form rendering
├── events.rs           # Event polling
├── messages.rs         # Message/notification system
├── utils.rs            # UI utility functions
├── entry_details.rs    # Entry details panel
├── activity_types.rs   # Activity type definitions
└── dialogs.rs          # Dialog components
```

### Dependencies Added

```toml
ratatui = "0.29.0"           # Terminal UI framework
crossterm = "0.28.1"         # Cross-platform terminal manipulation
tui-input = "0.11.0"         # Text input widget
unicode-width = "0.2.0"      # Unicode string width calculation
```

### Key Components

#### App State (`app.rs`)

- Manages application state (current week, selected entry, mode)
- Handles keyboard events and mode transitions
- Integrates with Monday.com API client
- Manages form data and cache

#### Form System (`form.rs`, `form_ui.rs`)

- Field-by-field navigation
- Validation before save
- Cache integration for auto-fill
- Visual feedback for current field

#### UI Rendering (`ui.rs`)

- Dynamic layout based on mode
- Form mode: Week view (40%) + Form (60%)
- Normal mode: Week view (70%) + Chart (30%)
- Cache panel always visible on right (30%)

### API Integration

- Uses existing `MondayClient` for all API calls
- New `update_item_verbose` method added for updates
- Async/await for non-blocking operations
- Proper error handling and user feedback

---

## User Feedback Updates

### Improved Navigation Controls

#### Previous Navigation

- `←/→` or `h/l`: Navigate between weeks
- `↑/↓` or `j/k`: Navigate between entries

#### New Navigation (Implemented)

- **`Tab`**: Navigate to next week
- **`Shift+Tab`**: Navigate to previous week  
- **`←/→`**: Navigate between days (Monday-Friday)
- **`↑/↓`**: Navigate between entries on the selected day
- **`Enter` or `e`**: Edit selected entry
- **`1-5`**: Jump to specific day of week
- **`Home`**: Jump to current week

#### Rationale

- More intuitive: Tab keys for major navigation (weeks)
- Arrow keys for fine-grained navigation (days and entries)
- Selecting an entry with arrows shows clear intent to interact with it
- Enter key provides natural way to edit selected entry

### Loading Indicators

#### Animated Spinner

- **Braille Pattern Spinner**: Uses Unicode Braille characters for smooth animation
- **Frames**: `⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏`
- **Animation Speed**: 100ms per frame
- **Centered Overlay**: Appears as popup over main content

#### Loading Messages

Context-aware messages during operations:

- "Refreshing cache from last 4 weeks..."
- "Loading week data..."
- Custom messages for different operations

### Cache Panel with Auto-Refresh

#### Side Panel Display

- **Location**: Right side of screen (30% width)
- **Title**: "Quick Select"
- **Content**: 15 most recent customer/work item pairs
- **Format**:

  ```
  • Customer Name
    Work Item ID
  ```

#### Auto-Refresh on Startup

- Cache automatically refreshed when interactive UI launches
- Equivalent to running with `-r` option
- Queries last 4 weeks of data
- Updates cache file for persistence

#### Manual Refresh

- Press `r` to refresh cache and reload data
- Shows loading spinner during refresh
- Displays success message with entry count

### Updated UI Layout

```
┌─────────────────────────────────────────────────────────────┐
│ Header (User info)                                          │
├──────────────────────────────────┬──────────────────────────┤
│                                  │                          │
│  Week View (70%)                 │  Cache Panel (30%)       │
│  ┌────────────────────────────┐  │  ┌──────────────────┐   │
│  │ Mon │ Tue │ Wed │ Thu │ Fri│  │  │ Recent Entries   │   │
│  │ ... │ ... │ ... │ ... │ ...│  │  │ • Customer A     │   │
│  └────────────────────────────┘  │  │   WI-123         │   │
│                                  │  │ • Customer B     │   │
│  Summary Chart (30%)             │  │   WI-456         │   │
│  ┌────────────────────────────┐  │  │ ...              │   │
│  │ Activity Distribution      │  │  │ Total: 42 entries│   │
│  │ Billable: ████████ 28h     │  │  └──────────────────┘   │
│  └────────────────────────────┘  │                          │
├──────────────────────────────────┴──────────────────────────┤
│ Messages                                                     │
├──────────────────────────────────────────────────────────────┤
│ Footer (Keyboard shortcuts)                                  │
└──────────────────────────────────────────────────────────────┘
```

---

## UX Improvements

### Context-Aware Right Panel

The right panel now dynamically changes based on the current field:

#### Activity Type Field

- Shows all 13 activity types with numbers (0-12)
- Each type displays with its number and descriptive name
- Current selection is highlighted with ▶ indicator
- Press 0-9 to instantly select an activity type

#### Customer/Work Item Fields

- Shows recent cache entries
- Displays 15 most recent customer/work item pairs
- Each entry numbered 0-9 for quick selection
- Press number to auto-fill both customer and work item

#### Other Fields

- Shows cache as reference
- Helpful for maintaining consistency

### Number Key Selection

Users can now press 0-9 to:

#### Select Activity Type

Press the number next to the activity type:

- 0 = Vacation
- 1 = Billable (default)
- 2 = Holding
- 3 = Education
- 4 = Work Reduction
- 5 = TBD
- 6 = Holiday
- 7 = Presales
- 8 = Illness
- 9 = Paid Not Worked
- (10-12 require typing the name)

#### Select from Cache

- Press 0-9 to select a recent entry
- Automatically fills both Customer and Work Item fields
- Saves time and ensures consistency

### Enhanced Field Visibility

All form fields now clearly show:

- **Field Label**: Bold cyan when selected, white otherwise
- **Current Value**: Yellow when selected, gray otherwise
- **Cursor Indicator**: █ symbol shows active field
- **Empty State**: Shows `<empty>` for unfilled fields
- **Field Hints**: Context-sensitive help text below form
  - Date format examples
  - Selection instructions
  - Field-specific guidance

### Improved Instructions

Updated instruction bar shows:

```
Tab: Next field | Shift+Tab: Prev | Enter: Save | Esc: Cancel | 0-9: Select from list
```

### Visual Feedback

- **Selected items** in lists show ▶ indicator
- **Current field** highlighted in cyan with bold text
- **Field values** clearly visible with color coding
- **Hints** displayed in blue with 💡 icon

---

## Features

### 1. Week-Based Calendar View

- Displays current week (Monday-Sunday) with all entries
- Navigate between weeks using Tab (next) and Shift+Tab (previous)
- Navigate days using Left/Right arrow keys
- Navigate entries within a day using Up/Down arrow keys
- Visual indicators for selected day and entry

### 2. Activity Summary Chart

- Bar chart showing distribution of activity types for the week
- Color-coded by activity type (billable, vacation, etc.)
- Real-time updates as data changes

### 3. Cache Panel

- Shows 15 most recent customer/work item pairs
- Auto-refreshes on startup (like `-r` option)
- Selectable entries for quick form filling
- Helps with consistent data entry

### 4. Loading Indicators

- Animated Braille spinner during data loading
- Status messages for user feedback
- Non-blocking async operations

### 5. Inline Form Editor

- **Add Mode** (press 'a'): Create new entries
- **Edit Mode** (press 'e' on selected entry): Modify existing entries
- All fields visible and editable in bottom panel:
  - Date (YYYY-MM-DD format)
  - Activity Type (billable, vacation, etc.)
  - Customer name
  - Work Item code
  - Hours (default: 8)
  - Comment

### 6. Form Navigation

- Tab: Move to next field
- Shift+Tab: Move to previous field
- Tab from last field: Switch to cache panel
- Up/Down in cache: Select cache entry
- Enter in cache: Auto-fill customer/work item from cache
- Enter in form: Save changes
- Esc: Cancel and return to normal mode

### 7. API Integration

- **Create**: New entries saved to Monday.com via GraphQL API
- **Update**: Existing entries updated via GraphQL API
- **Refresh**: Automatic data reload after save/update
- Error handling with user-friendly messages

### 8. Keyboard Controls

- `q`: Quit application
- `?`: Show help screen
- `a`: Add new entry
- `e`: Edit selected entry
- `d`: Delete selected entry (with confirmation)
- `r`: Refresh data from Monday.com
- Tab/Shift+Tab: Navigate weeks
- Arrow keys: Navigate days and entries
- Enter: Confirm action
- Esc: Cancel action

---

## Usage Guide

### Starting the Interactive UI

```bash
# Run without any command to launch interactive UI
cargo run

# Or with the built binary
./claim
```

### Adding an Entry

1. Press `a` to enter add mode
2. Fill in the fields (Tab to navigate)
3. Optionally Tab to cache panel and select an entry to auto-fill
4. Press Enter to save
5. Press Esc to cancel

### Editing an Entry

1. Navigate to the entry using arrow keys
2. Press `e` to enter edit mode
3. Modify fields as needed (Tab to navigate)
4. Press Enter to save changes
5. Press Esc to cancel

### Navigation Tips

- Use Tab/Shift+Tab to move between weeks quickly
- Use arrow keys for precise day/entry selection
- The cache panel helps maintain consistency across entries
- Watch the status bar for helpful messages

### User Workflow Examples

#### Adding an Entry with Activity Type Selection

1. Press `a` to enter add mode
2. Fill in date (or leave for today)
3. Tab to Activity Type field
4. **Right panel shows all activity types**
5. Press `0` for vacation, `1` for billable, etc.
6. Tab to Customer field
7. **Right panel shows recent entries**
8. Press `0-9` to select a recent customer/work item pair
9. Adjust hours if needed
10. Press Enter to save

#### Editing an Entry

1. Navigate to entry with arrow keys
2. Press `e` to edit
3. **All current values are visible**
4. Tab through fields to modify
5. Use number keys for quick selection
6. Press Enter to save changes

---

## Technical Architecture

### State Management

- Centralized `App` struct holds all state
- Mode-based event handling
- Form data cloned before async operations (borrow checker)
- Automatic data refresh after mutations

### Data Flow

```
User Input → Event Handler → State Update → Backend API → State Update → UI Render
     ↓                                                                        ↑
     └────────────────────────────────────────────────────────────────────────┘
```

### Caching Strategy

- Load cache on startup
- Refresh cache in background every 5 minutes
- Update cache immediately after add/edit/delete
- Use cache for autocomplete suggestions

### API Interactions

- Initial load: Query current week + 2 weeks before/after
- Week navigation: Query new week if not cached
- Add/Edit/Delete: Immediate API call + cache update
- Refresh: Re-query current view range

### Performance Considerations

1. **Lazy Loading**: Only load visible week data
2. **Debouncing**: Debounce API calls during rapid navigation
3. **Caching**: Aggressive caching of query results
4. **Async Operations**: Non-blocking API calls with loading indicators
5. **Efficient Rendering**: Only re-render changed components

---

## Future Enhancements

### Potential Improvements

1. **Enhanced Validation**
   - Date range validation
   - Work item format validation
   - Customer name suggestions

2. **Advanced Features**
   - Multi-day entry creation
   - Copy/paste entries
   - Bulk operations
   - Search/filter functionality

3. **UI Improvements**
   - Color themes
   - Customizable layouts
   - More chart types
   - Better error messages

4. **Performance**
   - Caching strategies
   - Lazy loading
   - Background refresh

5. **Extended Activity Types**
   - Support for types 10-12 with number selection

6. **Search/Filter**
   - Type to filter cache entries

7. **Custom Shortcuts**
   - User-defined number mappings

8. **Field Validation**
   - Real-time validation feedback

9. **Auto-complete**
   - Suggest values as user types

### Long-term Vision

1. **Month View**: Calendar-style month view
2. **Statistics Dashboard**: Detailed analytics and trends
3. **Export**: Export week/month data to CSV/PDF
4. **Templates**: Save and reuse common entry patterns
5. **Bulk Operations**: Add/edit/delete multiple entries
6. **Search**: Full-text search across all entries
7. **Notifications**: Desktop notifications for reminders
8. **Themes**: Customizable color schemes
9. **Mouse Support**: Click to select, drag to copy
10. **Multi-user**: View team member claims (if permissions allow)

---

## Testing

### Manual Testing Checklist

- [ ] Launch interactive UI without parameters
- [ ] Navigate between weeks
- [ ] Navigate between days and entries
- [ ] Add a new entry
- [ ] Edit an existing entry
- [ ] Use cache panel for auto-fill
- [ ] Cancel form operations
- [ ] Verify data persists to Monday.com
- [ ] Test error handling (invalid data, API errors)
- [ ] Test help screen
- [ ] Test quit functionality
- [ ] Activity type selection works (press 0-9)
- [ ] Cache selection works (press 0-9)
- [ ] Right panel updates when changing fields
- [ ] All field values are visible
- [ ] Cursor indicator shows on current field
- [ ] Hints display correctly for each field
- [ ] Number keys work in both add and edit modes

### Known Limitations

1. No delete confirmation dialog yet (placeholder exists)
2. Form validation is basic (can be enhanced)
3. No undo/redo functionality
4. Single-day entries only (no multi-day support yet)

---

## Conclusion

The interactive UI feature is now fully functional with:

- ✅ Intuitive terminal-based interface
- ✅ Week-based calendar view
- ✅ Activity distribution charts
- ✅ Inline form editor with full field navigation
- ✅ Cache panel for quick data entry
- ✅ Complete API integration for CRUD operations
- ✅ Loading indicators and status messages
- ✅ Comprehensive keyboard controls
- ✅ Context-aware right panel
- ✅ Number key selection for activity types and cache entries
- ✅ All field values clearly visible
- ✅ Visual feedback and hints
- ✅ Intuitive and fast workflow

The feature is ready for testing and can be further enhanced based on user feedback.

---

**Last Updated**: January 2026
