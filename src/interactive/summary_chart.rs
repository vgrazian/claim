//! Summary chart component for displaying activity type distribution

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::collections::HashMap;

use super::app::App;
use super::utils::{format_hours, get_activity_color};

/// Render the summary chart
pub fn render(f: &mut Frame, app: &App, area: Rect) {
    // Calculate activity type distribution
    let mut activity_totals: HashMap<String, f64> = HashMap::new();
    let mut total_hours = 0.0;
    // Track which days have entries
    let mut days_with_entries = std::collections::HashSet::new();

    for entry in &app.claims {
        days_with_entries.insert(entry.date);

        // Handle vacation/illness without hours as 8 hours
        let hours = if entry.hours > 0.0 {
            entry.hours
        } else if entry.activity_type.to_lowercase().contains("vacation")
            || entry.activity_type.to_lowercase().contains("illness")
        {
            8.0
        } else {
            entry.hours
        };

        *activity_totals
            .entry(entry.activity_type.clone())
            .or_insert(0.0) += hours;
        total_hours += hours;
    }

    // Calculate total with blank days as 8 hours
    let current_week_start = app.current_week_start;
    let mut blank_days = 0;
    for i in 0..5 {
        let date = current_week_start + chrono::Duration::days(i);
        if !days_with_entries.contains(&date) {
            blank_days += 1;
        }
    }
    let total_hours_with_blanks = total_hours + (blank_days as f64 * 8.0);

    // Sort by hours (descending), then by activity type name for stable ordering
    let mut activities: Vec<_> = activity_totals.into_iter().collect();
    activities.sort_by(|a, b| {
        // First compare by hours (descending)
        match b.1.partial_cmp(&a.1).unwrap() {
            std::cmp::Ordering::Equal => {
                // If hours are equal, sort by activity type name (ascending) for stability
                a.0.cmp(&b.0)
            }
            other => other,
        }
    });

    // Create chart lines
    let mut lines = Vec::new();

    if total_hours > 0.0 {
        for (activity_type, hours) in activities {
            let percentage = (hours / total_hours) * 100.0;
            let bar_width = ((percentage / 100.0) * 30.0) as usize; // Max 30 chars for bar

            let color = get_activity_color(&activity_type);
            let bar = "█".repeat(bar_width);

            let line = Line::from(vec![
                Span::styled(
                    format!("{:15} ", activity_type),
                    Style::default().fg(Color::White),
                ),
                Span::styled(bar, Style::default().fg(color)),
                Span::styled(
                    format!(" {} ({:.0}%)", format_hours(hours), percentage),
                    Style::default().fg(color),
                ),
            ]);

            lines.push(line);
        }
    } else {
        lines.push(Line::from(Span::styled(
            "No entries for this week",
            Style::default().fg(Color::Gray),
        )));
    }

    // Add total line
    if total_hours > 0.0 || blank_days > 0 {
        lines.push(Line::from(""));

        // Show warning if over 40 hours
        let total_color = if total_hours_with_blanks > 40.0 {
            Color::Red
        } else {
            Color::Cyan
        };

        let total_text = if total_hours_with_blanks > 40.0 {
            format!("{} ⚠️ (exceeds 40h)", format_hours(total_hours_with_blanks))
        } else {
            format_hours(total_hours_with_blanks)
        };

        lines.push(Line::from(vec![
            Span::styled("Total: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                total_text,
                Style::default()
                    .fg(total_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        // Show breakdown if there are blank days
        if blank_days > 0 {
            lines.push(Line::from(vec![Span::styled(
                format!(
                    "  ({} logged + {} blank days × 8h)",
                    format_hours(total_hours),
                    blank_days
                ),
                Style::default().fg(Color::Gray),
            )]));
        }
    }

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Weekly Summary ")
            .border_style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(paragraph, area);
}

// Made with Bob
