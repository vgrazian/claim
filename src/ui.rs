use anyhow::Result;
use chrono::{Datelike, Duration, Local, NaiveDate, Weekday};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame, Terminal,
};
use std::io;

use crate::monday::{Item, MondayClient, MondayUser};

/// Get the start of the business week (Monday) for a given date
fn get_week_start(date: NaiveDate) -> NaiveDate {
    let weekday = date.weekday();
    let days_from_monday = match weekday {
        Weekday::Mon => 0,
        Weekday::Tue => 1,
        Weekday::Wed => 2,
        Weekday::Thu => 3,
        Weekday::Fri => 4,
        Weekday::Sat => 5,
        Weekday::Sun => 6,
    };
    date - Duration::days(days_from_monday)
}

/// Get all business days (Mon-Fri) for a given week start
fn get_business_week_dates(week_start: NaiveDate) -> Vec<NaiveDate> {
    (0..5)
        .map(|i| week_start + Duration::days(i))
        .collect()
}

/// Application state
struct App {
    current_week_start: NaiveDate,
    items: Vec<Item>,
    loading: bool,
    error_message: Option<String>,
}

impl App {
    fn new() -> Self {
        let today = Local::now().date_naive();
        let week_start = get_week_start(today);
        
        Self {
            current_week_start: week_start,
            items: Vec::new(),
            loading: false,
            error_message: None,
        }
    }

    fn previous_week(&mut self) {
        self.current_week_start = self.current_week_start - Duration::days(7);
        self.loading = true;
        self.error_message = None;
    }

    fn next_week(&mut self) {
        self.current_week_start = self.current_week_start + Duration::days(7);
        self.loading = true;
        self.error_message = None;
    }

    fn get_week_range_string(&self) -> String {
        let week_end = self.current_week_start + Duration::days(4);
        format!(
            "{} - {}",
            self.current_week_start.format("%Y-%m-%d"),
            week_end.format("%Y-%m-%d")
        )
    }
}

/// Extract date from item
fn extract_item_date(item: &Item) -> Option<NaiveDate> {
    for col in &item.column_values {
        if let Some(col_id) = &col.id {
            if col_id == "date4" {
                if let Some(value) = &col.value {
                    if value != "null" && !value.is_empty() {
                        if let Ok(parsed_value) = serde_json::from_str::<serde_json::Value>(value) {
                            if let Some(date_obj) = parsed_value.get("date") {
                                if let Some(date_str) = date_obj.as_str() {
                                    if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                                        return Some(date);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

/// Extract column value from item
fn extract_column_value(item: &Item, column_id: &str) -> String {
    for col in &item.column_values {
        if let Some(col_id) = &col.id {
            if col_id == column_id {
                if let Some(text) = &col.text {
                    if !text.is_empty() && text != "null" {
                        return text.clone();
                    }
                }
            }
        }
    }
    String::new()
}

/// Extract hours from item
fn extract_hours(item: &Item) -> f64 {
    let hours_str = extract_column_value(item, "numbers__1");
    hours_str.parse::<f64>().unwrap_or(0.0)
}

/// Fetch items for the current week
async fn fetch_week_items(
    client: &MondayClient,
    user: &MondayUser,
    week_dates: &[NaiveDate],
) -> Result<Vec<Item>> {
    let current_year = Local::now().year().to_string();
    let board_id = "6500270039";
    
    // Get board with groups to find the year group
    let board = client.get_board_with_groups(board_id, false).await?;
    let group_id = crate::utils::get_year_group_id(&board, &current_year);
    
    // Convert week dates to strings for the query
    let date_strings: Vec<String> = week_dates
        .iter()
        .map(|d| d.format("%Y-%m-%d").to_string())
        .collect();
    
    // Use query_items_with_filters for server-side filtering
    let filtered_items = client
        .query_items_with_filters(
            board_id,
            &group_id,
            user.id,
            &date_strings,
            500,  // limit
            false, // verbose
        )
        .await?;
    
    Ok(filtered_items)
}

/// Render the UI
fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Table
            Constraint::Length(3),  // Footer
        ])
        .split(f.area());

    // Header
    let header_text = if app.loading {
        format!("Loading week: {} ...", app.get_week_range_string())
    } else {
        format!("Claim Items - Week: {}", app.get_week_range_string())
    };
    
    let header = Paragraph::new(header_text)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).title("Claims TUI"));
    f.render_widget(header, chunks[0]);

    // Table
    if let Some(error) = &app.error_message {
        let error_widget = Paragraph::new(error.as_str())
            .style(Style::default().fg(Color::Red))
            .block(Block::default().borders(Borders::ALL).title("Error"));
        f.render_widget(error_widget, chunks[1]);
    } else if app.loading {
        let loading_widget = Paragraph::new("Loading...")
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(loading_widget, chunks[1]);
    } else {
        render_items_table(f, chunks[1], app);
    }

    // Footer
    let footer = Paragraph::new("← Previous Week | → Next Week | q Quit")
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL).title("Controls"));
    f.render_widget(footer, chunks[2]);
}

/// Render the items table
fn render_items_table(f: &mut Frame, area: Rect, app: &App) {
    let week_dates = get_business_week_dates(app.current_week_start);
    
    // Group items by date
    let mut items_by_date: Vec<Vec<&Item>> = vec![Vec::new(); 5];
    for item in &app.items {
        if let Some(item_date) = extract_item_date(item) {
            if let Some(pos) = week_dates.iter().position(|d| d == &item_date) {
                items_by_date[pos].push(item);
            }
        }
    }
    
    // Calculate daily totals
    let daily_totals: Vec<f64> = items_by_date
        .iter()
        .map(|items| items.iter().map(|item| extract_hours(item)).sum())
        .collect();
    
    let week_total: f64 = daily_totals.iter().sum();
    
    // Create header
    let header_cells = ["Date", "Customer", "Work Item", "Hours", "Activity", "Comment"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);
    
    // Create rows
    let mut rows = Vec::new();
    
    for (day_idx, date) in week_dates.iter().enumerate() {
        let date_str = date.format("%a %m/%d").to_string();
        let day_items = &items_by_date[day_idx];
        
        if day_items.is_empty() {
            // Empty day
            rows.push(Row::new(vec![
                Cell::from(date_str).style(Style::default().fg(Color::DarkGray)),
                Cell::from("-"),
                Cell::from("-"),
                Cell::from("0.0"),
                Cell::from("-"),
                Cell::from("-"),
            ]));
        } else {
            // First item for the day
            let first_item = day_items[0];
            rows.push(Row::new(vec![
                Cell::from(date_str.clone()).style(Style::default().fg(Color::Green)),
                Cell::from(extract_column_value(first_item, "text__1")),
                Cell::from(extract_column_value(first_item, "text8__1")),
                Cell::from(format!("{:.1}", extract_hours(first_item))),
                Cell::from(extract_column_value(first_item, "status")),
                Cell::from(extract_column_value(first_item, "text2__1")),
            ]));
            
            // Additional items for the same day
            for item in day_items.iter().skip(1) {
                rows.push(Row::new(vec![
                    Cell::from(""),
                    Cell::from(extract_column_value(item, "text__1")),
                    Cell::from(extract_column_value(item, "text8__1")),
                    Cell::from(format!("{:.1}", extract_hours(item))),
                    Cell::from(extract_column_value(item, "status")),
                    Cell::from(extract_column_value(item, "text2__1")),
                ]));
            }
        }
        
        // Add daily total
        rows.push(Row::new(vec![
            Cell::from(""),
            Cell::from(""),
            Cell::from("Day Total:").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::ITALIC)),
            Cell::from(format!("{:.1}", daily_totals[day_idx])).style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Cell::from(""),
            Cell::from(""),
        ]));
    }
    
    // Add week total
    rows.push(Row::new(vec![
        Cell::from(""),
        Cell::from(""),
        Cell::from("Week Total:").style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        Cell::from(format!("{:.1}", week_total)).style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        Cell::from(""),
        Cell::from(""),
    ]));
    
    let widths = [
        Constraint::Length(10),  // Date
        Constraint::Percentage(20), // Customer
        Constraint::Percentage(20), // Work Item
        Constraint::Length(8),   // Hours
        Constraint::Percentage(15), // Activity
        Constraint::Percentage(25), // Comment
    ];
    
    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Items"))
        .column_spacing(1);
    
    f.render_widget(table, area);
}

/// Run the TUI application
pub async fn run_tui(client: &MondayClient, user: &MondayUser) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    
    // Initial data fetch
    let week_dates = get_business_week_dates(app.current_week_start);
    match fetch_week_items(client, user, &week_dates).await {
        Ok(items) => {
            app.items = items;
            app.loading = false;
        }
        Err(e) => {
            app.error_message = Some(format!("Failed to fetch items: {}", e));
            app.loading = false;
        }
    }

    loop {
        terminal.draw(|f| ui(f, &app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Left => {
                        app.previous_week();
                        let week_dates = get_business_week_dates(app.current_week_start);
                        match fetch_week_items(client, user, &week_dates).await {
                            Ok(items) => {
                                app.items = items;
                                app.loading = false;
                            }
                            Err(e) => {
                                app.error_message = Some(format!("Failed to fetch items: {}", e));
                                app.loading = false;
                            }
                        }
                    }
                    KeyCode::Right => {
                        app.next_week();
                        let week_dates = get_business_week_dates(app.current_week_start);
                        match fetch_week_items(client, user, &week_dates).await {
                            Ok(items) => {
                                app.items = items;
                                app.loading = false;
                            }
                            Err(e) => {
                                app.error_message = Some(format!("Failed to fetch items: {}", e));
                                app.loading = false;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

// Made with Bob
