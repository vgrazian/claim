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

    for entry in &app.claims {
        *activity_totals
            .entry(entry.activity_type.clone())
            .or_insert(0.0) += entry.hours;
        total_hours += entry.hours;
    }

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
    if total_hours > 0.0 {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Total: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format_hours(total_hours),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
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
