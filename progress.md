# CURRENT STATUS

## Active Task
Clarify requirements for a new presales code validation and six-month hours aggregation feature.

## Current Phase
Inspecting the codebase to locate the timesheet UI, presales code rules, comment handling, and existing limit logic.

## Immediate Next Steps
1. Identify relevant source files.
2. Read the existing implementation for presales and L104-like constraints.
3. Ask targeted clarification questions grounded in the code.

## Blockers
- Requirement details are still ambiguous and need confirmation from the user.

# TECHNICAL CONTEXT

## Files Under Investigation
- To be populated during code inspection.

# ISSUES AND OBSERVATIONS

## Notes
- `progress.md` did not exist at session start and was created for tracking.

# INVESTIGATION TRAIL

## Session Start
- Verified workspace structure.
- Confirmed `progress.md` was missing and created it.

# USEFUL DEBUGGING INFORMATION

## Notes
- None yet.

# PROGRESS TRACKING

## Completed Milestones
- Created `progress.md` for session continuity.

## Remaining Work Items
1. Find presales-related code paths.
2. Inspect validation and UI display logic.
3. Ask clarification questions.

## Update
- Identified the relevant implementation points: quick selection option `0` comes from [`EntryCache::get_unique_entries()`](src/cache.rs:144), form validation lives in [`FormData::validate()`](src/interactive/form.rs:222), TUI add/edit handling and persistence live in [`App::handle_add_mode()`](src/interactive/app.rs:660) and [`App::save_new_entry()`](src/interactive/app.rs:1528), and monthly summary rendering lives in [`render_monthly()`](src/interactive/summary_chart.rs:157).
- Confirmed the current TUI already has an L104-specific monthly indicator, but there is no existing six-month presales aggregation and no validation tying `PRESALES` / `M.34212` to an opportunity number comment.
- Confirmed the current local cache only stores recent customer/work-item pairs in [`src/cache.rs`](src/cache.rs), not opportunity numbers or historical hour aggregates.

## Update 2
- Requirement clarified: the previous six-month scope changed to year-based aggregation keyed by the selected TUI year/week context.
- For [`PRESALES`](src/cache.rs:150) + [`M.34212`](src/cache.rs:151), the comment is expected to be an 18-character alphanumeric opportunity code; invalid values should warn but not block save.
- The 24-hour yearly cap per opportunity is informational only and should warn in UI rather than block add/edit.
- Summary UI should replace the current monthly-only layout from [`render_monthly()`](src/interactive/summary_chart.rs:157) with a combined Summary view: Month section for vacation/L104 and Yearly section for weekly visible presales opportunities.
- CLI needs a cache reset command in addition to TUI-triggered reset; command wiring lives in [`src/main.rs`](src/main.rs:24).
