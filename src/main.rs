pub mod app;
pub mod client;
pub mod events;
pub mod ui;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

use app::App;
use client::HttpClient;
use events::UiState;
use ui::ActivePane;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Setup app state
    let mut app = App::new();
    let mut active_pane = ActivePane::Url;
    let mut ui_state = UiState::new(&app);
    let (http_rx, http_tx) = HttpClient::new();

    // Main loop
    loop {
        terminal.draw(|f| {
            ui::draw(f, &app, active_pane, &mut ui_state);
        })?;

        let should_quit = events::handle_events(
            &mut app,
            &mut active_pane,
            &mut ui_state,
            &http_tx,
            &http_rx,
        )?;

        if should_quit {
            break;
        }
    }

    // Save state on exit
    app.save();

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
