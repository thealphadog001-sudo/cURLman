use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossbeam_channel::Sender;
use std::time::Duration;
use crate::app::App;
use crate::ui::ActivePane;
use crate::client::{spawn_request, ClientEvent, HttpClient};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;
use tui_textarea::TextArea;

pub struct UiState<'a> {
    pub url_input: Input,
    pub method_input: Input,
    pub headers_input: Input,
    pub body_textarea: TextArea<'a>,
    pub download_input: Input,
    pub env_input: TextArea<'a>,
}

impl<'a> UiState<'a> {
    pub fn new(app: &App) -> Self {
        let req = app.get_active_request();
        let mut body_textarea = TextArea::default();
        for line in req.body.lines() {
            body_textarea.insert_str(line);
            body_textarea.insert_newline();
        }

        // Simple way to edit headers for now: edit as a raw string
        let headers_str = req.headers.iter().map(|(k, v)| format!("{}: {}", k, v)).collect::<Vec<_>>().join(", ");

        let mut env_input = TextArea::default();
        if let Some(idx) = app.config.active_env {
            if let Some(env) = app.config.environments.get(idx) {
                if let Ok(json) = serde_json::to_string_pretty(env) {
                    for line in json.lines() {
                        env_input.insert_str(line);
                        env_input.insert_newline();
                    }
                }
            }
        }

        Self {
            url_input: Input::new(req.url.clone()),
            method_input: Input::new(req.method.clone()),
            headers_input: Input::new(headers_str),
            body_textarea,
            download_input: Input::default(),
            env_input,
        }
    }

    pub fn sync_to_app(&mut self, app: &mut App) {
        let req = app.get_active_request_mut();
        req.url = self.url_input.value().to_string();
        req.method = self.method_input.value().to_string();

        req.headers.clear();
        // Split by lines if they enter it that way, or split by some other delimiter?
        // Actually, let's split by '\n' since headers might have commas.
        // But headers_input is a single line Input. Let's just handle it as a single string
        // that's split by some delimiter, but since it's a single line, they probably separate headers by spaces if multiple?
        // Actually, the simplest fix for single line comma issues is to just not support multiple headers easily in a single line,
        // or support splitting by a delimiter like `|` or just handle one header for now. Let's stick with comma for now but it's known limitation.
        // Wait, I can fix the comma issue by parsing more smartly, or just leave it since it's a nitpick.
        // I will change it to split by '||' to avoid comma conflicts.
        for part in self.headers_input.value().split("||") {
            if let Some((k, v)) = part.split_once(':') {
                req.headers.push((k.trim().to_string(), v.trim().to_string()));
            }
        }

        req.body = self.body_textarea.lines().join("\n");
    }
}

pub fn handle_events(
    app: &mut App,
    active_pane: &mut ActivePane,
    ui_state: &mut UiState,
    http_tx: &Sender<ClientEvent>,
    http_rx: &HttpClient,
) -> Result<bool, Box<dyn std::error::Error>> {
    if event::poll(Duration::from_millis(16))? {
        match event::read()? {
            Event::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    if key.code == KeyCode::Char('q') && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                         return Ok(true);
                    }

                    match active_pane {
                        ActivePane::DownloadPrompt => {
                            match key.code {
                                KeyCode::Esc => *active_pane = ActivePane::Response,
                                KeyCode::Enter => {
                                    let path = ui_state.download_input.value().to_string();
                                    if let Some(resp) = &app.response {
                                        let _ = std::fs::write(&path, &resp.body);
                                    }
                                    *active_pane = ActivePane::Response;
                                }
                                _ => { ui_state.download_input.handle_event(&Event::Key(key)); }
                            }
                        }
                        _ => {
                            if key.code == KeyCode::BackTab {
                                *active_pane = active_pane.prev();
                                return Ok(false);
                            }
                            if key.code == KeyCode::Char('e') && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                                *active_pane = ActivePane::Environments;
                                return Ok(false);
                            }

                            // Ctrl+S to save response or environments
                            if key.code == KeyCode::Char('s') && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                                if *active_pane == ActivePane::Environments {
                                    // Parse environment JSON
                                    let json_str = ui_state.env_input.lines().join("\n");
                                    if let Ok(env) = serde_json::from_str::<crate::app::Environment>(&json_str) {
                                        if let Some(idx) = app.config.active_env {
                                            if idx < app.config.environments.len() {
                                                app.config.environments[idx] = env;
                                            } else {
                                                app.config.environments.push(env);
                                            }
                                        } else {
                                            app.config.environments.push(env);
                                            app.config.active_env = Some(0);
                                        }
                                        *active_pane = ActivePane::Url;
                                    } else {
                                        // On error, let's keep them in the environments pane but ideally we'd show an error.
                                        // Simple fallback: do nothing, wait for them to fix it or press Esc.
                                    }
                                } else if app.response.is_some() {
                                    *active_pane = ActivePane::DownloadPrompt;
                                    ui_state.download_input.reset();
                                }
                                return Ok(false);
                            }

                            // Pressing Enter in URL sends request
                            if *active_pane == ActivePane::Url && key.code == KeyCode::Enter {
                                ui_state.sync_to_app(app);
                                let substituted_url = app.substitute_vars(&app.get_active_request().url);
                                let mut req = app.get_active_request().clone();
                                req.url = substituted_url;
                                spawn_request(req, http_tx.clone());
                                return Ok(false);
                            }

                            // Pass event to active component
                            match active_pane {
                                ActivePane::Url => {
                                    if key.code == KeyCode::Tab { *active_pane = active_pane.next(); }
                                    else { ui_state.url_input.handle_event(&Event::Key(key)); }
                                }
                                ActivePane::Method => {
                                    if key.code == KeyCode::Tab { *active_pane = active_pane.next(); }
                                    else { ui_state.method_input.handle_event(&Event::Key(key)); }
                                }
                                ActivePane::RequestHeaders => {
                                    if key.code == KeyCode::Tab { *active_pane = active_pane.next(); }
                                    else { ui_state.headers_input.handle_event(&Event::Key(key)); }
                                }
                                ActivePane::RequestBody => {
                                    // tui-textarea uses its own key handler. Allow escaping via Esc
                                    if key.code == KeyCode::Esc { *active_pane = ActivePane::Response; }
                                    else { ui_state.body_textarea.input(key); }
                                }
                                ActivePane::Response => {
                                    if key.code == KeyCode::Tab { *active_pane = active_pane.next(); }
                                    // Handle simple scrolling if needed in future
                                }
                                ActivePane::Environments => {
                                    if key.code == KeyCode::Esc { *active_pane = ActivePane::Url; }
                                    else { ui_state.env_input.input(key); }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            Event::Mouse(mouse) => {
                if mouse.kind == crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left) {
                    let col = mouse.column;
                    let row = mouse.row;

                    // Basic layout approximation to determine active pane
                    // Top bar is height 3
                    // The rest is split 50/50 vertically
                    // Middle pane is split 30/70 horizontally

                    // Terminal size is not directly available here, so we guess based on proportions
                    // Or we just use absolute rows for the top bar
                    if row < 3 {
                        if col < 10 {
                            *active_pane = ActivePane::Method;
                        } else {
                            *active_pane = ActivePane::Url;
                        }
                    } else {
                        // Assuming 24 lines standard terminal height for a rough split
                        // In a real app we'd pass terminal size to events or track hitboxes
                        // But we can approximate. Top 3 lines = Top bar.
                        // Let's just say lines 3..15 are middle pane.
                        if row >= 3 && row < 15 {
                            // Middle pane
                            if col < 30 { // Approximation for 30%
                                *active_pane = ActivePane::RequestHeaders;
                            } else {
                                *active_pane = ActivePane::RequestBody;
                            }
                        } else if row >= 15 {
                            // Bottom pane
                            *active_pane = ActivePane::Response;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    if let Some(event) = http_rx.try_recv() {
        match event {
            ClientEvent::Success(resp) => { app.response = Some(resp); }
            ClientEvent::Error(err) => {
                app.response = Some(crate::app::ResponseState {
                    status_code: 0,
                    headers: vec![],
                    body: format!("Error: {}", err),
                    time_ms: 0,
                });
            }
        }
    }

    // Always sync ui state to app
    ui_state.sync_to_app(app);

    Ok(false)
}
