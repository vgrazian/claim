//! Form UI rendering with context-aware right panel

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use super::activity_types;
use super::app::{App, AppMode};
use super::form::FormField;

/// Render the form editor
pub fn render_form(f: &mut Frame, app: &App, area: Rect) {
    if let Some(form) = &app.form_data {
        let title = match app.mode {
            AppMode::AddEntry => " Add Entry ",
            AppMode::EditEntry => " Edit Entry ",
            _ => " Form ",
        };

        // Create form fields
        let mut lines = vec![];

        // Instructions
        lines.push(Line::from(Span::styled(
            "Tab: Next | Shift+Tab: Prev | ←→: Move cursor | Home/End | Backspace/Del | Enter: Save | Esc: Cancel",
            Style::default().fg(Color::Gray),
        )));
        lines.push(Line::from(""));

        // Render each field with clear labels and values
        for field in FormField::all() {
            let is_current = form.current_field == field && !form.focus_on_cache;
            let value = form.get_field_value(field);

            let label_style = if is_current {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let value_style = if is_current {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };

            // Format the field display with cursor at the correct position
            let display_value = if is_current {
                // Insert cursor at the cursor position
                let cursor_pos = form.cursor_position.min(value.len());
                if value.is_empty() {
                    "█".to_string()
                } else {
                    let mut display = value.to_string();
                    display.insert(cursor_pos, '█');
                    display
                }
            } else if value.is_empty() {
                "<empty>".to_string()
            } else {
                value.to_string()
            };

            lines.push(Line::from(vec![
                Span::styled(format!("{:15}", field.label()), label_style),
                Span::raw(": "),
                Span::styled(display_value, value_style),
            ]));
        }

        // Add spacing
        lines.push(Line::from(""));

        // Show validation hints for current field
        if !form.focus_on_cache {
            let hint = match form.current_field {
                FormField::Date => "Format: YYYY-MM-DD (e.g., 2024-01-15)",
                FormField::ActivityType => "Press 0-9 to select from list →",
                FormField::Customer => "Enter customer name or press 0-9 to select from cache →",
                FormField::WorkItem => "Enter work item code or press 0-9 to select from cache →",
                FormField::Hours => "Enter hours (e.g., 8, 4.5)",
                FormField::Comment => "Optional comment",
            };

            lines.push(Line::from(Span::styled(
                format!("💡 {}", hint),
                Style::default().fg(Color::Blue),
            )));
        }

        let paragraph =
            Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(title));

        f.render_widget(paragraph, area);
    }
}

/// Render context-aware right panel based on current field
pub fn render_context_panel(f: &mut Frame, app: &App, area: Rect) {
    if let Some(form) = &app.form_data {
        // Determine what to show based on current field
        match form.current_field {
            FormField::ActivityType => {
                render_activity_type_panel(f, form.activity_type.as_str(), area);
            }
            FormField::Customer | FormField::WorkItem => {
                render_cache_panel_with_selection(f, app, area);
            }
            _ => {
                // For other fields, show cache as reference
                render_cache_panel_with_selection(f, app, area);
            }
        }
    }
}

/// Render activity type selection panel
fn render_activity_type_panel(f: &mut Frame, current_type: &str, area: Rect) {
    let activity_types = activity_types::get_all_activity_types();

    let items: Vec<ListItem> = activity_types
        .iter()
        .map(|at| {
            let is_selected = at.name == current_type;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let prefix = if is_selected { "▶ " } else { "  " };
            let content = format!("{}{} - {}", prefix, at.number, at.display_name);

            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Activity Types (press 0-9) ")
            .style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(list, area);
}

/// Render cache panel with selection highlighting
pub fn render_cache_panel_with_selection(f: &mut Frame, app: &App, area: Rect) {
    let entries = app.cache.get_unique_entries();
    let selected_index = if let Some(form) = &app.form_data {
        if form.focus_on_cache {
            Some(form.selected_cache_index)
        } else {
            None
        }
    } else {
        None
    };

    // Only show 9 billable entries (already filtered during cache refresh)
    let items: Vec<ListItem> = entries
        .iter()
        .take(9)
        .enumerate()
        .map(|(i, entry)| {
            let is_selected = selected_index == Some(i);
            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let prefix = if is_selected { "▶ " } else { "  " };
            let number = if i < 10 {
                format!("{}", i)
            } else {
                " ".to_string()
            };
            let content = format!(
                "{}{} {} | {}",
                prefix, number, entry.customer, entry.work_item
            );

            ListItem::new(content).style(style)
        })
        .collect();

    let title = if selected_index.is_some() {
        " Recent Entries (↑↓ to select, Enter to use) "
    } else {
        " Recent Entries (press 0-9 to select) "
    };

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(list, area);
}

// Made with Bob
