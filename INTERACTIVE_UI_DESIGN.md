# Interactive UI Feature Design

## Overview

A terminal-based interactive UI that launches when the user runs `claim` without parameters. Provides an intuitive, week-based view with panes, charts, and messages for managing Monday.com claims.

## Architecture

### Components

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

pub struct ClaimEntry {
    id: String,
    date: NaiveDate,
    activity_type: String,
    customer: String,
    work_item: String,
    hours: f64,
    comment: Option<String>,
}
```

## User Interactions

### Keyboard Controls

**Navigation:**

- `←/→` or `h/l`: Navigate between weeks
- `↑/↓` or `j/k`: Navigate between entries in current day
- `Tab`: Switch between panes
- `Home/End`: Jump to current week / first week

**Actions:**

- `a`: Add new entry (opens dialog)
- `e`: Edit selected entry
- `d`: Delete selected entry (with confirmation)
- `r`: Refresh data from Monday.com
- `c`: Copy entry to another day
- `Space`: Toggle entry selection

**View:**

- `1-5`: Jump to specific day of week
- `w`: Toggle week/month view
- `s`: Show statistics
- `f`: Filter by customer/work item

**General:**

- `q`: Quit application
- `?` or `h`: Show help
- `Esc`: Cancel current operation

### Workflows

#### 1. Adding an Entry

1. Press `a` to open add dialog
2. Select date (defaults to current day)
3. Enter/select customer (autocomplete from cache)
4. Enter/select work item (autocomplete from cache)
5. Enter hours (default: 8)
6. Select activity type (default: billable)
7. Optional: Add comment
8. Confirm to create

#### 2. Editing an Entry

1. Navigate to entry with arrow keys
2. Press `e` to edit
3. Modify fields (pre-filled with current values)
4. Confirm to update

#### 3. Deleting an Entry

1. Navigate to entry
2. Press `d` to delete
3. Confirm deletion (shows entry details)

#### 4. Week Navigation

1. Use `←/→` to move between weeks
2. Current week highlighted
3. Data loads automatically

## Data Flow

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

## Technical Implementation

### Dependencies

```toml
[dependencies]
ratatui = "0.26"           # TUI framework
crossterm = "0.27"         # Terminal manipulation
tui-input = "0.8"          # Input handling
unicode-width = "0.1"      # Text width calculation
```

### Module Structure

```
src/
├── interactive/
│   ├── mod.rs              # Main interactive module
│   ├── app.rs              # Application state and logic
│   ├── ui.rs               # UI rendering
│   ├── events.rs           # Event handling
│   ├── week_view.rs        # Week view component
│   ├── summary_chart.rs    # Chart component
│   ├── messages.rs         # Message pane
│   ├── dialogs.rs          # Input dialogs
│   └── utils.rs            # UI utilities
```

### Error Handling

- Network errors: Show in message pane, allow retry
- Validation errors: Inline feedback in dialogs
- API errors: Detailed message with suggested actions
- Graceful degradation: Continue with cached data if API unavailable

## Visual Design

### Color Scheme

- **Primary**: Cyan (headers, selected items)
- **Success**: Green (completed actions, full weeks)
- **Warning**: Yellow (missing entries, low hours)
- **Error**: Red (errors, validation failures)
- **Info**: Blue (help text, tips)
- **Muted**: Gray (borders, secondary text)

### Activity Type Colors

- Billable: Green
- Vacation: Blue
- Presales: Cyan
- Overhead: Yellow
- Illness: Red
- Others: White

### Borders and Styling

- Use rounded corners for modern look
- Double borders for main panes
- Single borders for sub-components
- Bold text for emphasis
- Italic for hints/tips

## Performance Considerations

1. **Lazy Loading**: Only load visible week data
2. **Debouncing**: Debounce API calls during rapid navigation
3. **Caching**: Aggressive caching of query results
4. **Async Operations**: Non-blocking API calls with loading indicators
5. **Efficient Rendering**: Only re-render changed components

## Future Enhancements

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

## Testing Strategy

1. **Unit Tests**: Test state management and business logic
2. **Integration Tests**: Test API interactions
3. **Manual Testing**: UI/UX testing with real data
4. **Performance Tests**: Test with large datasets
5. **Accessibility**: Ensure keyboard-only navigation works perfectly

## Implementation Phases

### Phase 1: Foundation (Current)

- Add dependencies
- Create module structure
- Implement basic app state
- Set up event loop

### Phase 2: Core UI

- Implement week view
- Add navigation controls
- Display existing claims
- Basic styling

### Phase 3: Interactions

- Add entry dialog
- Edit entry dialog
- Delete confirmation
- Data refresh

### Phase 4: Polish

- Summary charts
- Message pane
- Help screen
- Error handling

### Phase 5: Enhancement

- Autocomplete
- Keyboard shortcuts
- Performance optimization
- Documentation
