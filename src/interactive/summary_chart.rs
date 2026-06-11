//! Summary chart component for displaying activity type distribution

use chrono::Datelike;
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

/// Render the weekly summary chart
pub fn render_weekly(f: &mut Frame, app: &App, area: Rect) {
    let current_week_start = app.current_week_start;

    // ===== WEEKLY SUMMARY CALCULATION =====
    let mut activity_totals: HashMap<String, f64> = HashMap::new();
    let mut total_hours = 0.0;
    let mut days_with_entries = std::collections::HashSet::new();

    // Only count entries from the current week (Monday-Friday)
    for entry in &app.claims {
        // Check if entry is in current week
        let days_from_week_start = (entry.date - current_week_start).num_days();
        if days_from_week_start >= 0 && days_from_week_start < 5 {
            days_with_entries.insert(entry.date);

            // Handle vacation/illness/l104 without hours as 8 hours
            let hours = if entry.hours > 0.0 {
                entry.hours
            } else if entry.activity_type.to_lowercase().contains("vacation")
                || entry.activity_type.to_lowercase().contains("illness")
                || entry.activity_type.to_lowercase() == "l104"
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
    }

    // Calculate blank days in the week
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

/// Render the monthly summary chart
pub fn render_monthly(f: &mut Frame, app: &App, area: Rect) {
    let current_week_start = app.current_week_start;
    let current_month = current_week_start.month();
    let current_year = current_week_start.year();

    let mut lines = Vec::new();

    let month_name = match current_month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    };

    lines.push(Line::from(vec![Span::styled(
        format!("{} {}", month_name, current_year),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(""));

    // Calculate vacation, presales, and L104 for the entire month
    let mut vacation_days = 0.0;
    let mut presales_days = 0.0;
    let mut l104_days = 0.0;

    for entry in &app.monthly_claims {
        if entry.date.year() == current_year && entry.date.month() == current_month {
            let days = if entry.hours > 0.0 {
                entry.hours / 8.0
            } else {
                1.0 // Treat 0 hours as 1 full day
            };

            let activity_lower = entry.activity_type.to_lowercase();
            if activity_lower.contains("vacation") {
                vacation_days += days;
            } else if activity_lower.contains("presales") {
                presales_days += days;
            } else if activity_lower == "l104" {
                l104_days += days;
            }
        }
    }

    // Show vacation tracking
    if vacation_days > 0.0 {
        lines.push(Line::from(vec![
            Span::styled("Vacation: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{:.1} days", vacation_days),
                Style::default().fg(Color::Green),
            ),
        ]));
    }

    // Show presales tracking
    if presales_days > 0.0 {
        lines.push(Line::from(vec![
            Span::styled("Presales: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{:.1} days", presales_days),
                Style::default().fg(Color::Blue),
            ),
        ]));
    }

    // Show L104 tracking with limit
    if l104_days > 0.0 {
        let l104_color = if l104_days >= 3.0 {
            Color::Red
        } else if l104_days >= 2.5 {
            Color::Yellow
        } else {
            Color::LightMagenta
        };

        let l104_text = if l104_days >= 3.0 {
            format!("{:.1} / 3.0 days ⚠️ (limit reached)", l104_days)
        } else {
            format!("{:.1} / 3.0 days", l104_days)
        };

        lines.push(Line::from(vec![
            Span::styled("L104: ", Style::default().fg(Color::White)),
            Span::styled(l104_text, Style::default().fg(l104_color)),
        ]));
    }

    // If no vacation, presales, or L104 entries this month
    if vacation_days == 0.0 && presales_days == 0.0 && l104_days == 0.0 {
        lines.push(Line::from(vec![Span::styled(
            "No vacation, presales, or L104 entries this month",
            Style::default().fg(Color::Gray),
        )]));
    }

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Monthly Summary ")
            .border_style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(paragraph, area);
}

/// Legacy render function for backward compatibility
pub fn render(f: &mut Frame, app: &App, area: Rect) {
    render_weekly(f, app, area);
}

// Made with Bob
