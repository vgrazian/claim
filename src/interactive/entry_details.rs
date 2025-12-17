//! Entry details panel rendering

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::app::{App, ClaimEntry};

/// Render entry details panel
pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let entry = get_selected_entry(app);

    let lines = if let Some(entry) = entry {
        vec![
            Line::from(vec![
                Span::styled(
                    "Date: ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    entry.date.format("%Y-%m-%d (%A)").to_string(),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "Activity: ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    &entry.activity_type,
                    get_activity_color(&entry.activity_type),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "Customer: ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    if entry.customer.is_empty() {
                        "-"
                    } else {
                        &entry.customer
                    },
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "Work Item: ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    if entry.work_item.is_empty() {
                        "-"
                    } else {
                        &entry.work_item
                    },
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "Hours: ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{:.1}", entry.hours),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "Comment: ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    entry.comment.as_deref().unwrap_or("-"),
                    Style::default().fg(Color::Gray),
                ),
            ]),
        ]
    } else {
        vec![
            Line::from(Span::styled(
                "No entry selected",
                Style::default().fg(Color::Gray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Use arrow keys to select an entry",
                Style::default().fg(Color::DarkGray),
            )),
        ]
    };

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Entry Details ")
            .style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(paragraph, area);
}

/// Get the currently selected entry
fn get_selected_entry(app: &App) -> Option<&ClaimEntry> {
    if let Some(day) = app.selected_day {
        if let Some(idx) = app.selected_entry_index {
            let entries_on_day: Vec<_> = app.claims.iter().filter(|e| e.date == day).collect();
            return entries_on_day.get(idx).copied();
        }
    }
    None
}

/// Get color for activity type
fn get_activity_color(activity_type: &str) -> Style {
    let color = match activity_type.to_lowercase().as_str() {
        "billable" => Color::Green,
        "vacation" => Color::Blue,
        "presales" => Color::Cyan,
        "overhead" => Color::Yellow,
        "illness" => Color::Red,
        "holiday" => Color::Magenta,
        "education" => Color::LightBlue,
        "work_reduction" => Color::LightYellow,
        "intellectual_capital" => Color::LightCyan,
        "business_development" => Color::LightGreen,
        _ => Color::White,
    };
    Style::default().fg(color).add_modifier(Modifier::BOLD)
}

// Made with Bob
