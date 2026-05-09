use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Environment {
    pub name: String,
    pub variables: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestState {
    pub url: String,
    pub method: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ResponseState {
    pub status_code: u32,
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub time_ms: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub requests: Vec<RequestState>,
    pub environments: Vec<Environment>,
    pub active_env: Option<usize>,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            requests: vec![RequestState {
                url: String::new(),
                method: "GET".to_string(),
                headers: vec![],
                body: String::new(),
            }],
            environments: vec![],
            active_env: None,
        }
    }
}

pub struct App {
    pub config: AppConfig,
    pub current_request_idx: usize,
    pub response: Option<ResponseState>,
    // Add UI states here later (e.g. active pane, tui-input instances, etc.)
}

impl App {
    pub fn new() -> Self {
        let config = load_config().unwrap_or_default();
        let config = if config.requests.is_empty() {
            AppConfig::default()
        } else {
            config
        };
        App {
            config,
            current_request_idx: 0,
            response: None,
        }
    }

    pub fn save(&self) {
        save_config(&self.config).unwrap_or(());
    }

    pub fn get_active_request(&self) -> &RequestState {
        &self.config.requests[self.current_request_idx]
    }

    pub fn get_active_request_mut(&mut self) -> &mut RequestState {
        &mut self.config.requests[self.current_request_idx]
    }

    pub fn substitute_vars(&self, input: &str) -> String {
        let env_vars = self.config.active_env
            .and_then(|idx| self.config.environments.get(idx))
            .map(|env| &env.variables);

        if let Some(vars) = env_vars {
            let mut result = input.to_string();
            for (key, val) in vars {
                let pattern = format!("{{{{{}}}}}", key); // matches {{key}}
                result = result.replace(&pattern, val);
            }
            result
        } else {
            input.to_string()
        }
    }
}

fn get_config_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("postman_tui");
    if !path.exists() {
        fs::create_dir_all(&path).unwrap_or(());
    }
    path.push("state.json");
    path
}

fn load_config() -> Result<AppConfig, Box<dyn std::error::Error>> {
    let path = get_config_path();
    let data = fs::read_to_string(path)?;
    let config: AppConfig = serde_json::from_str(&data)?;
    Ok(config)
}

fn save_config(config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    let path = get_config_path();
    let data = serde_json::to_string_pretty(config)?;
    fs::write(path, data)?;
    Ok(())
}
