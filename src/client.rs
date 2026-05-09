use crate::app::{RequestState, ResponseState};
use crossbeam_channel::{Receiver, Sender, unbounded};
use curl::easy::{Easy, List};
use std::thread;
use std::time::Instant;

pub enum ClientEvent {
    Success(ResponseState),
    Error(String),
}

pub struct HttpClient {
    rx: Receiver<ClientEvent>,
}

impl HttpClient {
    pub fn new() -> (Self, Sender<ClientEvent>) {
        let (tx, rx) = unbounded();
        (Self { rx }, tx)
    }

    pub fn try_recv(&self) -> Option<ClientEvent> {
        self.rx.try_recv().ok()
    }
}

pub fn spawn_request(req: RequestState, tx: Sender<ClientEvent>) {
    thread::spawn(move || {
        let mut easy = Easy::new();
        let start_time = Instant::now();

        // Basic Setup
        if let Err(e) = easy.url(&req.url) {
            let _ = tx.send(ClientEvent::Error(format!("Invalid URL: {}", e)));
            return;
        }

        let _ = easy.custom_request(&req.method);

        // Headers
        let mut list = List::new();
        for (k, v) in &req.headers {
            let header = format!("{}: {}", k, v);
            if let Err(e) = list.append(&header) {
                let _ = tx.send(ClientEvent::Error(format!("Invalid header {}: {}", header, e)));
                return;
            }
        }
        let _ = easy.http_headers(list);

        // Body
        if !req.body.is_empty() {
            let _ = easy.post_fields_copy(req.body.as_bytes());
        }

        // Response Data collection
        let mut response_body = Vec::new();
        let mut response_headers = Vec::new();

        {
            let mut transfer = easy.transfer();

            let _ = transfer.write_function(|data| {
                response_body.extend_from_slice(data);
                Ok(data.len())
            });

            let _ = transfer.header_function(|data| {
                if let Ok(header_str) = std::str::from_utf8(data) {
                    if let Some(idx) = header_str.find(':') {
                        let key = header_str[..idx].trim().to_string();
                        let value = header_str[idx + 1..].trim().to_string();
                        response_headers.push((key, value));
                    }
                }
                true
            });

            if let Err(e) = transfer.perform() {
                let _ = tx.send(ClientEvent::Error(format!("Request failed: {}", e)));
                return;
            }
        }

        let status_code = easy.response_code().unwrap_or(0);
        let time_ms = start_time.elapsed().as_millis() as u64;
        let body_str = String::from_utf8_lossy(&response_body).to_string();

        let response_state = ResponseState {
            status_code,
            headers: response_headers,
            body: body_str,
            time_ms,
        };

        let _ = tx.send(ClientEvent::Success(response_state));
    });
}
