//! Interactive terminal UI module for claim management
//!
//! This module provides a terminal-based user interface for managing Monday.com claims
//! with intuitive controls, week-based views, charts, and real-time feedback.

pub mod activity_types;
pub mod app;
pub mod dialogs;
pub mod entry_details;
pub mod events;
pub mod form;
pub mod form_ui;
pub mod messages;
pub mod summary_chart;
pub mod ui;
pub mod utils;
pub mod week_view;

pub use app::App;
pub use events::EventHandler;

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

use crate::config::Config;
use crate::monday::MondayClient;

/// Run the interactive UI application
pub async fn run_interactive() -> Result<()> {
    // Load configuration
    let config = Config::load()?;
    let client = MondayClient::new(config.api_key.clone());

    // Get current user
    let user = client.get_current_user_verbose(false).await?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let mut app = App::new(client, user).await?;
    let res = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

/// Main application loop
async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    let event_handler = EventHandler::new();

    loop {
        // Draw UI
        terminal.draw(|f| ui::draw(f, app))?;

        // Handle events
        if let Some(event) = event_handler.next()? {
            if !app.handle_event(event).await? {
                break;
            }
        }
    }

    Ok(())
}

// Made with Bob
