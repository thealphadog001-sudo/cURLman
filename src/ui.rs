use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Clear},
    Frame,
};
use crate::app::App;
use crate::events::UiState;

#[derive(PartialEq, Clone, Copy)]
pub enum ActivePane {
    Url,
    Method,
    RequestHeaders,
    RequestBody,
    Response,
    Environments,
    DownloadPrompt,
}

impl ActivePane {
    pub fn next(&self) -> Self {
        match self {
            ActivePane::Url => ActivePane::Method,
            ActivePane::Method => ActivePane::RequestHeaders,
            ActivePane::RequestHeaders => ActivePane::RequestBody,
            ActivePane::RequestBody => ActivePane::Response,
            ActivePane::Response => ActivePane::Url,
            _ => ActivePane::Url,
        }
    }
    pub fn prev(&self) -> Self {
        match self {
            ActivePane::Url => ActivePane::Response,
            ActivePane::Method => ActivePane::Url,
            ActivePane::RequestHeaders => ActivePane::Method,
            ActivePane::RequestBody => ActivePane::RequestHeaders,
            ActivePane::Response => ActivePane::RequestBody,
            _ => ActivePane::Url,
        }
    }
}

pub fn draw(f: &mut Frame, app: &App, active_pane: ActivePane, ui_state: &mut UiState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Top bar
            Constraint::Percentage(50), // Middle pane
            Constraint::Percentage(50), // Bottom pane
        ])
        .split(f.area());

    draw_top_bar(f, app, chunks[0], active_pane, ui_state);
    draw_middle_pane(f, app, chunks[1], active_pane, ui_state);
    draw_bottom_pane(f, app, chunks[2], active_pane);

    // Set cursor for active pane
    match active_pane {
        ActivePane::Url => {
            let inner_area = chunks[0].inner(ratatui::layout::Margin { vertical: 1, horizontal: 1 });
            let cursor_x = inner_area.x + 10 + ui_state.url_input.cursor() as u16 + 2; // Offset for method block + border
            if cursor_x < inner_area.right() {
                f.set_cursor_position(ratatui::layout::Position::new(cursor_x, inner_area.y));
            }
        }
        ActivePane::Method => {
            let inner_area = chunks[0].inner(ratatui::layout::Margin { vertical: 1, horizontal: 1 });
            let cursor_x = inner_area.x + ui_state.method_input.cursor() as u16;
            if cursor_x < inner_area.right() {
                f.set_cursor_position(ratatui::layout::Position::new(cursor_x, inner_area.y));
            }
        }
        ActivePane::RequestHeaders => {
            let inner_area = chunks[1].inner(ratatui::layout::Margin { vertical: 1, horizontal: 1 });
            let cursor_x = inner_area.x + ui_state.headers_input.cursor() as u16;
            if cursor_x < inner_area.right() {
                f.set_cursor_position(ratatui::layout::Position::new(cursor_x, inner_area.y));
            }
        }
        // tui-textarea handles its own cursor rendering
        _ => {}
    }

    if active_pane == ActivePane::Environments {
        draw_environments(f, ui_state);
    }

    if active_pane == ActivePane::DownloadPrompt {
        draw_download_prompt(f, ui_state);
    }
}

fn draw_top_bar(f: &mut Frame, _app: &App, area: Rect, active_pane: ActivePane, ui_state: &UiState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(10), // Method
            Constraint::Min(0),     // URL
        ])
        .split(area);

    let method_style = if active_pane == ActivePane::Method { Style::default().fg(Color::Yellow) } else { Style::default() };
    let method_block = Block::default().borders(Borders::ALL).title("Method").border_style(method_style);
    let method_text = Paragraph::new(ui_state.method_input.value()).block(method_block);
    f.render_widget(method_text, chunks[0]);

    let url_style = if active_pane == ActivePane::Url { Style::default().fg(Color::Yellow) } else { Style::default() };
    let url_block = Block::default().borders(Borders::ALL).title("URL (Enter to Send)").border_style(url_style);
    let url_text = Paragraph::new(ui_state.url_input.value()).block(url_block);
    f.render_widget(url_text, chunks[1]);
}

fn draw_middle_pane(f: &mut Frame, _app: &App, area: Rect, active_pane: ActivePane, ui_state: &mut UiState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // Headers
            Constraint::Percentage(70), // Body
        ])
        .split(area);

    let headers_style = if active_pane == ActivePane::RequestHeaders { Style::default().fg(Color::Yellow) } else { Style::default() };
    let headers_block = Block::default().borders(Borders::ALL).title("Headers (k:v || k:v)").border_style(headers_style);
    let headers_text = Paragraph::new(ui_state.headers_input.value()).block(headers_block);
    f.render_widget(headers_text, chunks[0]);

    let body_style = if active_pane == ActivePane::RequestBody { Style::default().fg(Color::Yellow) } else { Style::default() };
    ui_state.body_textarea.set_block(Block::default().borders(Borders::ALL).title("Body (Esc to exit)").border_style(body_style));
    f.render_widget(&ui_state.body_textarea, chunks[1]);
}

fn draw_bottom_pane(f: &mut Frame, app: &App, area: Rect, active_pane: ActivePane) {
    let response_style = if active_pane == ActivePane::Response { Style::default().fg(Color::Yellow) } else { Style::default() };
    let block = Block::default().borders(Borders::ALL).title("Response (Ctrl+S to save)").border_style(response_style);

    let mut lines = vec![];
    if let Some(resp) = &app.response {
        lines.push(Line::from(format!("Status: {} | Time: {}ms", resp.status_code, resp.time_ms)));
        lines.push(Line::from("--- Headers ---"));
        for (k, v) in &resp.headers {
            lines.push(Line::from(format!("{}: {}", k, v)));
        }
        lines.push(Line::from("--- Body ---"));
        for line in resp.body.lines() {
            lines.push(Line::from(line.to_string()));
        }
    } else {
        lines.push(Line::from("No response yet."));
    }

    let p = Paragraph::new(lines).block(block);
    f.render_widget(p, area);
}

fn draw_environments(f: &mut Frame, ui_state: &mut UiState) {
    let area = centered_rect(80, 80, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title("Environment Variables JSON (Ctrl+S to save, Esc to cancel)")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    ui_state.env_input.set_block(block);
    f.render_widget(&ui_state.env_input, area);
}

fn draw_download_prompt(f: &mut Frame, ui_state: &UiState) {
    let area = centered_rect(60, 20, f.area());
    f.render_widget(Clear, area);

    let block = Block::default().title("Save Response To File (Enter to save, Esc to cancel)").borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow));
    let p = Paragraph::new(ui_state.download_input.value()).block(block);
    f.render_widget(p, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
