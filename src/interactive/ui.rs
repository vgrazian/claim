//! Main UI rendering logic

use chrono::Datelike;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row},
    Frame,
};

use super::app::{App, AppMode};
use super::messages::MessageType;
use super::utils::get_message_style;
use super::{entry_details, form_ui, summary_chart, week_view};

/// Main draw function
pub fn draw(f: &mut Frame, app: &App) {
    let size = f.size();

    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Main content
            Constraint::Length(5), // Messages
            Constraint::Length(3), // Footer
        ])
        .split(size);

    // Render header
    render_header(f, app, chunks[0]);

    // Render main content based on mode
    match app.mode {
        AppMode::Help => render_help(f, chunks[1]),
        AppMode::Report => render_report(f, app, chunks[1]),
        _ => render_main_content(f, app, chunks[1]),
    }

    // Render messages
    render_messages(f, app, chunks[2]);

    // Render footer
    render_footer(f, app, chunks[3]);

    // Render loading overlay if loading
    if app.loading {
        render_loading_overlay(f, app, size);
    }
}
/// Render the header
fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let title = format!(
        " Claim Manager - {} ({}) - Year: {} ",
        app.user.name, app.user.email, app.current_year
    );
    let header = Paragraph::new(title)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );

    f.render_widget(header, area);
}

/// Render the main content area
fn render_main_content(f: &mut Frame, app: &App, area: Rect) {
    // Check if we're in form mode
    let in_form_mode = matches!(app.mode, AppMode::AddEntry | AppMode::EditEntry);

    if in_form_mode {
        // Form mode layout: form on bottom, week view on top, cache on right
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(70), // Main content
                Constraint::Percentage(30), // Side panel (cache)
            ])
            .split(area);

        // Split main content into week view and form
        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40), // Week view (smaller)
                Constraint::Percentage(60), // Form editor
            ])
            .split(main_chunks[0]);

        // Render week view (smaller)
        week_view::render(f, app, content_chunks[0]);

        // Render form editor
        form_ui::render_form(f, app, content_chunks[1]);

        // Render context-aware panel (activity types or cache based on current field)
        form_ui::render_context_panel(f, app, main_chunks[1]);
    } else {
        // Normal mode layout - no side panel, use full width
        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50), // Week view
                Constraint::Percentage(25), // Entry details
                Constraint::Percentage(25), // Summary chart
            ])
            .split(area);

        // Render week view
        week_view::render(f, app, content_chunks[0]);

        // Render entry details panel
        entry_details::render(f, app, content_chunks[1]);

        // Render summary chart
        summary_chart::render(f, app, content_chunks[2]);
    }
}

/// Render the help screen
fn render_help(f: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(Span::styled(
            "Keyboard Shortcuts",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Cyan),
        )),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Navigation:",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from("  ←/→ or h/l    Navigate between weeks"),
        Line::from("  ↑/↓ or j/k    Navigate between entries"),
        Line::from("  1-5           Jump to specific day of week"),
        Line::from("  Home          Jump to current week"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Actions:",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from("  a             Add new entry"),
        Line::from("  e             Edit selected entry"),
        Line::from("  d             Delete selected entry"),
        Line::from("  r             Refresh data from Monday.com"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "General:",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from("  q             Quit application"),
        Line::from("  ? or h        Show this help"),
        Line::from("  Esc           Cancel current operation"),
        Line::from(""),
        Line::from(Span::styled(
            "Press any key to return...",
            Style::default().fg(Color::Gray),
        )),
    ];

    let paragraph = Paragraph::new(help_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Help ")
            .border_style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(paragraph, area);
}

/// Render the messages pane
fn render_messages(f: &mut Frame, app: &App, area: Rect) {
    let mut lines = Vec::new();

    // Show recent messages (last 3)
    let recent_messages: Vec<_> = app.messages.iter().rev().take(3).collect();

    for msg in recent_messages.iter().rev() {
        let mut style = get_message_style(msg.message_type);

        // Add blinking effect for delete confirmation in DeleteEntry mode
        if app.mode == AppMode::DeleteEntry && msg.message_type == MessageType::Warning {
            style = style.add_modifier(Modifier::SLOW_BLINK);
        }

        lines.push(Line::from(vec![
            Span::styled(format!("{} ", msg.icon()), style),
            Span::styled(&msg.text, style),
        ]));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "Ready",
            Style::default().fg(Color::Gray),
        )));
    }

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Messages ")
            .border_style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(paragraph, area);
}

/// Render the footer with keyboard shortcuts
fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let shortcuts = match app.mode {
        AppMode::Normal => {
            "[Tab] Next week  [Shift+Tab] Prev week  [←→] Days  [↑↓] Entries  [Enter/e] Edit  [a]dd  [d]elete  [u]pdate  [p]rint  [?] help  [q]uit"
        }
        AppMode::AddEntry => "[Esc] Cancel add",
        AppMode::EditEntry => "[Esc] Cancel edit",
        AppMode::DeleteEntry => "[y] Confirm  [n/Esc] Cancel",
        AppMode::Help => "Press any key to return",
        AppMode::Report => "[↑↓] Select row  [Tab] Next week  [Shift+Tab] Prev week  [Esc/p/q] Return to normal view",
    };

    let footer = Paragraph::new(shortcuts)
        .style(Style::default().fg(Color::Gray))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

    f.render_widget(footer, area);
}

/// Render the cache panel showing recent entries
#[allow(dead_code)]
fn render_cache_panel(f: &mut Frame, app: &App, area: Rect) {
    let entries = app.cache.get_unique_entries(app.user.id);

    // Take the most recent 9 billable entries (already filtered during cache refresh)
    let recent_entries: Vec<_> = entries.iter().take(9).collect();

    let mut lines = vec![
        Line::from(Span::styled(
            "Recent Entries",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Cyan),
        )),
        Line::from(""),
    ];

    if recent_entries.is_empty() {
        lines.push(Line::from(Span::styled(
            "No cached entries",
            Style::default().fg(Color::Gray),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Press 'r' to refresh cache",
            Style::default().fg(Color::Yellow),
        )));
    } else {
        for entry in recent_entries {
            lines.push(Line::from(vec![Span::styled(
                format!("• {}", entry.customer),
                Style::default().fg(Color::White),
            )]));
            lines.push(Line::from(vec![Span::styled(
                format!("  {}", entry.work_item),
                Style::default().fg(Color::Gray),
            )]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("Total: {} entries", entries.len()),
            Style::default().fg(Color::Cyan),
        )));
    }

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Quick Select ")
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(paragraph, area);
}

/// Render loading overlay with spinner
fn render_loading_overlay(f: &mut Frame, app: &App, area: Rect) {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Create a centered popup
    let popup_width = 50;
    let popup_height = 5;
    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;

    let popup_area = Rect {
        x: popup_x,
        y: popup_y,
        width: popup_width,
        height: popup_height,
    };

    // Animated spinner characters
    let spinner_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let frame_idx = ((now / 100) % spinner_frames.len() as u128) as usize;
    let spinner = spinner_frames[frame_idx];

    let loading_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!("{} ", spinner),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(&app.loading_message, Style::default().fg(Color::White)),
        ]),
        Line::from(""),
    ];

    let paragraph = Paragraph::new(loading_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Loading ")
                .border_style(Style::default().fg(Color::Cyan))
                .style(Style::default().bg(Color::Black)),
        )
        .alignment(ratatui::layout::Alignment::Center);

    // Clear the area first
    f.render_widget(ratatui::widgets::Clear, popup_area);
    f.render_widget(paragraph, popup_area);
}

// Made with Bob

/// Format hours without .00 unless the decimal is non-zero
fn format_hours(hours: f64) -> String {
    if hours == 0.0 {
        String::from("0")
    } else if hours % 1.0 == 0.0 {
        format!("{:.0}", hours)
    } else {
        format!("{:.2}", hours)
    }
}

/// Render the report view
fn render_report(f: &mut Frame, app: &App, area: Rect) {
    use ratatui::widgets::Table;
    use std::collections::HashMap;

    // Group entries by activity type and customer/work_item combination
    // Key: (activity_value, customer, work_item)
    let mut report_data: HashMap<(i32, String, String), [f64; 5]> = HashMap::new();

    for entry in &app.claims {
        let key = (
            entry.activity_value,
            entry.customer.clone(),
            entry.work_item.clone(),
        );
        let day_index = entry.date.weekday().num_days_from_monday() as usize;

        if day_index < 5 {
            report_data.entry(key).or_insert([0.0; 5])[day_index] += entry.hours;
        }
    }

    // Separate billable (activity_value == 1) from non-billable entries
    let mut billable_data: Vec<_> = report_data
        .iter()
        .filter(|((activity_value, _, _), _)| *activity_value == 1)
        .map(|((_, customer, work_item), hours)| ((customer.clone(), work_item.clone()), *hours))
        .collect();

    let mut non_billable_data: Vec<_> = report_data
        .iter()
        .filter(|((activity_value, _, _), _)| *activity_value != 1)
        .map(|((activity_value, customer, work_item), hours)| {
            (
                (*activity_value, customer.clone(), work_item.clone()),
                *hours,
            )
        })
        .collect();

    // Sort billable by customer/work_item
    billable_data.sort_by(|a, b| match a.0 .0.cmp(&b.0 .0) {
        std::cmp::Ordering::Equal => a.0 .1.cmp(&b.0 .1),
        other => other,
    });

    // Sort non-billable by activity type, then customer/work_item
    non_billable_data.sort_by(|a, b| match a.0 .0.cmp(&b.0 .0) {
        std::cmp::Ordering::Equal => match a.0 .1.cmp(&b.0 .1) {
            std::cmp::Ordering::Equal => a.0 .2.cmp(&b.0 .2),
            other => other,
        },
        other => other,
    });

    // Calculate column totals (sum of each day)
    let mut day_totals = [0.0; 5];
    for hours in report_data.values() {
        for i in 0..5 {
            day_totals[i] += hours[i];
        }
    }

    // Create table rows
    let mut rows = Vec::new();
    let mut current_row_index = 0;

    // Calculate dates for the week
    let monday = app.current_week_start;
    let dates = [
        monday,
        monday + chrono::Duration::days(1),
        monday + chrono::Duration::days(2),
        monday + chrono::Duration::days(3),
        monday + chrono::Duration::days(4),
    ];

    // Header row with dates above weekday names (dd/MMM format)
    rows.push(
        Row::new(vec![
            Cell::from("Work Item / Customer"),
            Cell::from(format!("{}\nMonday", dates[0].format("%d/%b"))),
            Cell::from(format!("{}\nTuesday", dates[1].format("%d/%b"))),
            Cell::from(format!("{}\nWednesday", dates[2].format("%d/%b"))),
            Cell::from(format!("{}\nThursday", dates[3].format("%d/%b"))),
            Cell::from(format!("{}\nFriday", dates[4].format("%d/%b"))),
            Cell::from("Total"),
        ])
        .style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Cyan),
        )
        .height(2), // Make header 2 rows tall to show both date and weekday
    );

    // Billable entries
    for ((customer, work_item), hours) in billable_data {
        let label = if !work_item.is_empty() && !customer.is_empty() {
            format!("{} - {}", work_item, customer)
        } else if !work_item.is_empty() {
            work_item
        } else {
            customer
        };

        let row_total: f64 = hours.iter().sum();

        // Determine row style based on selection
        let row_style = if app.selected_report_row == Some(current_row_index) {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        } else {
            Style::default()
        };

        rows.push(
            Row::new(vec![
                Cell::from(label),
                Cell::from(if hours[0] == 0.0 {
                    String::new()
                } else {
                    format_hours(hours[0])
                }),
                Cell::from(if hours[1] == 0.0 {
                    String::new()
                } else {
                    format_hours(hours[1])
                }),
                Cell::from(if hours[2] == 0.0 {
                    String::new()
                } else {
                    format_hours(hours[2])
                }),
                Cell::from(if hours[3] == 0.0 {
                    String::new()
                } else {
                    format_hours(hours[3])
                }),
                Cell::from(if hours[4] == 0.0 {
                    String::new()
                } else {
                    format_hours(hours[4])
                }),
                Cell::from(format_hours(row_total)),
            ])
            .style(row_style),
        );

        current_row_index += 1;
    }

    // Non-billable entries (if any)
    if !non_billable_data.is_empty() {
        // Add separator row
        rows.push(Row::new(vec![
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
        ]));
        current_row_index += 1;

        for ((activity_value, customer, work_item), hours) in non_billable_data {
            // Get activity type name
            let activity_name = crate::interactive::activity_types::get_activity_type_by_number(
                activity_value as u8,
            )
            .unwrap_or("unknown");

            let label = if !work_item.is_empty() && !customer.is_empty() {
                format!("{} - {} ({})", work_item, customer, activity_name)
            } else if !work_item.is_empty() {
                format!("{} ({})", work_item, activity_name)
            } else if !customer.is_empty() {
                format!("{} ({})", customer, activity_name)
            } else {
                format!("({})", activity_name)
            };

            let row_total: f64 = hours.iter().sum();

            // Determine row style based on selection
            let row_style = if app.selected_report_row == Some(current_row_index) {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default().fg(Color::Gray)
            };

            rows.push(
                Row::new(vec![
                    Cell::from(label),
                    Cell::from(if hours[0] == 0.0 {
                        String::new()
                    } else {
                        format_hours(hours[0])
                    }),
                    Cell::from(if hours[1] == 0.0 {
                        String::new()
                    } else {
                        format_hours(hours[1])
                    }),
                    Cell::from(if hours[2] == 0.0 {
                        String::new()
                    } else {
                        format_hours(hours[2])
                    }),
                    Cell::from(if hours[3] == 0.0 {
                        String::new()
                    } else {
                        format_hours(hours[3])
                    }),
                    Cell::from(if hours[4] == 0.0 {
                        String::new()
                    } else {
                        format_hours(hours[4])
                    }),
                    Cell::from(format_hours(row_total)),
                ])
                .style(row_style),
            );
            current_row_index += 1;
        }
    }

    // Total row
    let grand_total: f64 = day_totals.iter().sum();
    rows.push(
        Row::new(vec![
            Cell::from("Total"),
            Cell::from(format_hours(day_totals[0])),
            Cell::from(format_hours(day_totals[1])),
            Cell::from(format_hours(day_totals[2])),
            Cell::from(format_hours(day_totals[3])),
            Cell::from(format_hours(day_totals[4])),
            Cell::from(format_hours(grand_total)),
        ])
        .style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Yellow),
        ),
    );

    // Clamp selected row to valid range
    if let Some(selected) = app.selected_report_row {
        if selected >= current_row_index {
            // This will be handled by the app logic, but we note it here
        }
    }

    let widths = vec![
        Constraint::Percentage(35),
        Constraint::Percentage(10),
        Constraint::Percentage(10),
        Constraint::Percentage(10),
        Constraint::Percentage(10),
        Constraint::Percentage(10),
        Constraint::Percentage(15),
    ];

    let title = format!(
        " Weekly Report - {} to {} ",
        app.current_week_start.format("%b %d"),
        (app.current_week_start + chrono::Duration::days(4)).format("%b %d, %Y")
    );

    let table = Table::new(rows, widths)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .column_spacing(1);

    f.render_widget(table, area);
}
