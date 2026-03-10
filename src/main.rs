use std::{collections::BTreeMap, env, fs, net::SocketAddr, sync::Arc};

use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};
use futures::future::join_all;
use reqwest::Client;
use serde::Deserialize;

#[derive(Clone)]
struct AppState {
    client: Client,
    settings: Arc<Settings>,
}

#[derive(Debug, Deserialize)]
struct Settings {
    services: BTreeMap<String, ServiceCheck>,
}

#[derive(Debug, Deserialize)]
struct ServiceCheck {
    service: String,
    checkforstatus: u16,
    shouldcontain: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config.yaml".to_string());
    let port = env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8080);

    let settings: Settings = serde_yaml::from_str(&fs::read_to_string(config_path)?)?;
    let client = Client::builder().build()?;
    let state = AppState {
        client,
        settings: Arc::new(settings),
    };

    let app = Router::new()
        .route("/health", get(health))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health(State(state): State<AppState>) -> impl IntoResponse {
    let checks = state
        .settings
        .services
        .iter()
        .map(|(alias, check)| run_check(&state.client, alias, check));

    let results = join_all(checks)
        .await
        .into_iter()
        .collect::<BTreeMap<_, _>>();

    Json(results)
}

async fn run_check(client: &Client, alias: &str, check: &ServiceCheck) -> (String, &'static str) {
    let url = normalize_url(&check.service);
    let result = client.get(url).send().await;

    let status = match result {
        Ok(response) if response.status().as_u16() == check.checkforstatus => {
            if let Some(expected) = &check.shouldcontain {
                match response.text().await {
                    Ok(body) if body.contains(expected) => "ok",
                    _ => "error",
                }
            } else {
                "ok"
            }
        }
        _ => "error",
    };

    (alias.to_string(), status)
}

fn normalize_url(service: &str) -> String {
    if service.starts_with("http://") || service.starts_with("https://") {
        service.to_string()
    } else {
        format!("http://{service}/")
    }
}

#[cfg(test)]
mod tests {
    use super::{normalize_url, Settings};

    #[test]
    fn parses_yaml_config() {
        let config = r#"
services:
  alias1:
    service: "service:80"
    checkforstatus: 200
    shouldcontain: "somestring"
  alias2:
    service: "anotherservice:80"
    checkforstatus: 200
"#;

        let settings: Settings = serde_yaml::from_str(config).expect("config should parse");

        assert_eq!(settings.services.len(), 2);
        assert_eq!(
            settings.services["alias1"].shouldcontain.as_deref(),
            Some("somestring")
        );
        assert_eq!(settings.services["alias2"].checkforstatus, 200);
    }

    #[test]
    fn adds_http_scheme_when_missing() {
        assert_eq!(normalize_url("service:80"), "http://service:80/");
        assert_eq!(
            normalize_url("https://service:443/health"),
            "https://service:443/health"
        );
    }
}
