use super::models::*;
use std::{sync::OnceLock, time::Duration};

mod cli;
mod copilot;
mod enterprise;
mod evidence;
mod exports;
mod org_access;
mod policy;
mod team;
mod telemetry;

pub struct ControlPlaneClient {
    config: ServerConfig,
    client: reqwest::blocking::Client,
}

fn shared_http_client() -> &'static reqwest::blocking::Client {
    static CLIENT: OnceLock<reqwest::blocking::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::blocking::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(30))
            .pool_idle_timeout(Duration::from_secs(90))
            .tcp_keepalive(Duration::from_secs(30))
            .build()
            .expect("failed to build shared control plane HTTP client")
    })
}

fn normalize_loopback_url(url: &str) -> String {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let Ok(mut parsed) = reqwest::Url::parse(trimmed) else {
        return trimmed.to_string();
    };

    if parsed.host_str() == Some("localhost") && parsed.set_host(Some("127.0.0.1")).is_ok() {
        return parsed.to_string();
    }

    trimmed.to_string()
}

fn server_error_from_response(response: reqwest::blocking::Response) -> ServerError {
    let status = response.status();
    let body = response.text().unwrap_or_default();
    let message = serde_json::from_str::<serde_json::Value>(&body)
        .ok()
        .and_then(|value| {
            value
                .get("error")
                .or_else(|| value.get("message"))
                .and_then(|error| error.as_str())
                .map(str::to_string)
        })
        .filter(|message| !message.trim().is_empty())
        .unwrap_or_else(|| {
            let body = body.trim();
            if body.is_empty() {
                format!("Server returned status: {}", status)
            } else {
                let snippet = body.chars().take(240).collect::<String>();
                format!("Server returned status: {} ({})", status, snippet)
            }
        });

    ServerError::ServerError(message)
}

impl ControlPlaneClient {
    pub fn new(mut config: ServerConfig) -> Self {
        config.url = normalize_loopback_url(&config.url);
        Self {
            config,
            client: shared_http_client().clone(),
        }
    }

    fn endpoint_url(&self, segments: &[&str]) -> Result<reqwest::Url, ServerError> {
        let mut url = reqwest::Url::parse(&self.config.url)
            .map_err(|e| ServerError::ServerError(format!("Invalid server URL: {}", e)))?;

        let mut path = url.path_segments_mut().map_err(|_| {
            ServerError::ServerError("Server URL cannot be used as base URL".to_string())
        })?;
        path.pop_if_empty();
        for segment in segments {
            path.push(segment);
        }
        drop(path);

        Ok(url)
    }
}
