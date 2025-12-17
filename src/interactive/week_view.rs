//! Week view component for displaying claims

use chrono::{Datelike, NaiveDate};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

use super::app::{App, ClaimEntry};
use super::utils::{format_hours, get_activity_color, get_weekday_name, truncate_str};

/// Render the week view
pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let current_week_start = app.current_week_start;

    // Create header with weekday names and dates
    let mut header_cells = vec![Cell::from("")]; // Empty cell for row labels
    let mut dates = Vec::new();

    for i in 0..5 {
        let date = current_week_start + chrono::Duration::days(i);
        dates.push(date);

        let weekday = get_weekday_name(date.weekday());
        let day = date.day();

        let is_selected = app.selected_day == Some(date);
        let is_today = date == chrono::Local::now().naive_local().date();

        let mut style = Style::default();
        if is_today {
            style = style.fg(Color::Cyan).add_modifier(Modifier::BOLD);
        }
        if is_selected {
            style = style.bg(Color::DarkGray);
        }

        header_cells.push(Cell::from(format!("{} {}", weekday, day)).style(style));
    }

    // Add total column
    header_cells.push(Cell::from("Total").style(Style::default().add_modifier(Modifier::BOLD)));

    let header = Row::new(header_cells)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .height(1);

    // Create rows for each unique time slot
    let max_entries_per_day = dates
        .iter()
        .map(|date| app.get_entries_for_date(*date).len())
        .max()
        .unwrap_or(0);

    let mut rows = Vec::new();

    for row_idx in 0..max_entries_per_day.max(1) {
        let mut cells = vec![Cell::from(format!("#{}", row_idx + 1))];
        let mut row_total = 0.0;

        for date in &dates {
            let entries = app.get_entries_for_date(*date);

            if let Some(entry) = entries.get(row_idx) {
                let is_selected =
                    app.selected_day == Some(*date) && app.selected_entry_index == Some(row_idx);

                let cell_content = format_entry_cell(entry, is_selected);
                cells.push(cell_content);
                row_total += entry.hours;
            } else {
                cells.push(Cell::from(""));
            }
        }

        // Add row total
        if row_total > 0.0 {
            cells.push(
                Cell::from(format_hours(row_total)).style(Style::default().fg(Color::Yellow)),
            );
        } else {
            cells.push(Cell::from(""));
        }

        rows.push(Row::new(cells).height(4)); // Increased height to accommodate activity type
    }

    // Add daily totals row
    let mut total_cells =
        vec![Cell::from("Daily Total").style(Style::default().add_modifier(Modifier::BOLD))];
    let mut week_total = 0.0;

    for date in &dates {
        let daily_total: f64 = app
            .get_entries_for_date(*date)
            .iter()
            .map(|e| e.hours)
            .sum();
        week_total += daily_total;

        let style = if daily_total >= 8.0 {
            Style::default().fg(Color::Green)
        } else if daily_total > 0.0 {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Red)
        };

        total_cells.push(Cell::from(format_hours(daily_total)).style(style));
    }

    // Add week total
    total_cells.push(
        Cell::from(format_hours(week_total)).style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
    );

    rows.push(
        Row::new(total_cells)
            .height(1)
            .style(Style::default().add_modifier(Modifier::BOLD)),
    );

    // Create the table
    let widths = vec![
        Constraint::Length(8),      // Row label
        Constraint::Percentage(16), // Monday
        Constraint::Percentage(16), // Tuesday
        Constraint::Percentage(16), // Wednesday
        Constraint::Percentage(16), // Thursday
        Constraint::Percentage(16), // Friday
        Constraint::Length(10),     // Total
    ];

    let title = format!(
        " Week of {} - {} ",
        current_week_start.format("%b %d"),
        (current_week_start + chrono::Duration::days(4)).format("%b %d, %Y")
    );

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .column_spacing(1);

    f.render_widget(table, area);
}

/// Format an entry for display in a cell
fn format_entry_cell(entry: &ClaimEntry, is_selected: bool) -> Cell {
    let activity_type = truncate_str(&entry.activity_type, 12);
    let customer = truncate_str(&entry.customer, 12);
    let work_item = truncate_str(&entry.work_item, 12);
    let hours = format_hours(entry.hours);

    let activity_color = get_activity_color(&entry.activity_type);

    // Show activity type, customer/work item (if present), and hours
    let mut lines = vec![Line::from(Span::styled(
        activity_type,
        Style::default()
            .fg(activity_color)
            .add_modifier(Modifier::BOLD),
    ))];

    // Only show customer/work item if they exist
    if !customer.is_empty() {
        lines.push(Line::from(Span::styled(
            customer,
            Style::default().fg(Color::White),
        )));
    }
    if !work_item.is_empty() {
        lines.push(Line::from(Span::styled(
            work_item,
            Style::default().fg(Color::Gray),
        )));
    }

    lines.push(Line::from(Span::styled(
        hours,
        Style::default().fg(Color::Yellow),
    )));

    let mut style = Style::default();
    if is_selected {
        style = style.bg(Color::DarkGray).add_modifier(Modifier::BOLD);
    }

    Cell::from(lines).style(style)
}

// Made with Bob
