use anyhow::{bail, Context, Result};
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{body::Incoming, Request, Response};
use hyper_util::rt::TokioIo;
use reqwest::Client;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;

use crate::config::ProfileConfig;
use crate::rewrite::rewrite_model;

pub struct ProxyState {
    pub config: RwLock<ProfileConfig>,
    pub client: Client,
}

impl ProxyState {
    pub fn new(config: ProfileConfig) -> Self {
        Self {
            config: RwLock::new(config),
            client: Client::new(),
        }
    }

    pub async fn reload_config(&self) -> Result<()> {
        let new_config = ProfileConfig::load()?;
        let mut config = self.config.write().await;
        *config = new_config;
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct RouteResolution {
    provider_name: String,
    provider: crate::config::Provider,
    upstream_path: String,
}

fn resolve_route(path: &str, config: &ProfileConfig) -> Result<RouteResolution> {
    if let Some(rest) = path.strip_prefix("/p/") {
        let mut split = rest.splitn(2, '/');
        let provider_name = split.next().unwrap_or_default();
        if provider_name.is_empty() {
            bail!("Missing provider in route. Expected /p/<provider>/...");
        }

        let provider = config.provider(provider_name)?.clone();
        let suffix = split.next().unwrap_or_default();
        let upstream_path = format!("/{}", suffix);
        return Ok(RouteResolution {
            provider_name: provider_name.to_string(),
            provider,
            upstream_path,
        });
    }

    Ok(RouteResolution {
        provider_name: config.active.clone(),
        provider: config.active_provider()?.clone(),
        upstream_path: path.to_string(),
    })
}

async fn handle_request(
    req: Request<Incoming>,
    state: Arc<ProxyState>,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    match proxy_request(req, state).await {
        Ok(resp) => Ok(resp),
        Err(e) => {
            let body = serde_json::json!({
                "error": {
                    "type": "proxy_error",
                    "message": format!("{:#}", e)
                }
            });
            Ok(Response::builder()
                .status(502)
                .header("content-type", "application/json")
                .body(Full::new(Bytes::from(serde_json::to_vec(&body).unwrap())))
                .unwrap())
        }
    }
}

async fn proxy_request(
    req: Request<Incoming>,
    state: Arc<ProxyState>,
) -> Result<Response<Full<Bytes>>> {
    let path = req.uri().path().to_string();
    let query = req.uri().query().map(ToString::to_string);
    let method = req.method().clone();
    let headers = req.headers().clone();
    let route = {
        let config = state.config.read().await;
        resolve_route(&path, &config)?
    };

    // Read request body
    let body_bytes = req
        .collect()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to read request body: {}", e))?
        .to_bytes();

    // Rewrite model in JSON body
    let body_bytes = if !body_bytes.is_empty() {
        if let Ok(mut json) = serde_json::from_slice::<serde_json::Value>(&body_bytes) {
            if let Some(model) = json.get("model").and_then(|m| m.as_str()) {
                let rewritten = rewrite_model(model, &route.provider);
                json["model"] = serde_json::Value::String(rewritten);
            }
            Bytes::from(serde_json::to_vec(&json)?)
        } else {
            body_bytes
        }
    } else {
        body_bytes
    };

    // Build upstream URL
    // Strip /v1 prefix from upstream_path if base_url already ends with /v1
    let base = route.provider.base_url.trim_end_matches('/');
    let upstream_path = if base.ends_with("/v1") {
        route
            .upstream_path
            .strip_prefix("/v1")
            .unwrap_or(&route.upstream_path)
    } else {
        &route.upstream_path
    };
    let mut upstream_url = format!("{}{}", base, upstream_path);
    if let Some(query) = query {
        upstream_url.push('?');
        upstream_url.push_str(&query);
    }

    // Build upstream request
    let mut upstream_req = state.client.request(method, &upstream_url);

    // Copy relevant headers (skip hop-by-hop; forward inbound auth unless provider has explicit auth configured).
    let provider_has_explicit_auth =
        route.provider.api_key.is_some() || route.provider.auth_token.is_some();
    for (name, value) in headers.iter() {
        let name_str = name.as_str().to_lowercase();
        if matches!(
            name_str.as_str(),
            "host" | "connection" | "transfer-encoding" | "keep-alive"
        ) {
            continue;
        }
        if provider_has_explicit_auth && (name_str == "authorization" || name_str == "x-api-key") {
            continue;
        }
        upstream_req = upstream_req.header(name.clone(), value.clone());
    }

    // Set provider auth
    if let Some(ref key) = route.provider.api_key {
        upstream_req = upstream_req.header("x-api-key", key);
        upstream_req = upstream_req.header("Authorization", format!("Bearer {}", key));
    }
    if let Some(ref token) = route.provider.auth_token {
        upstream_req = upstream_req.header("Authorization", format!("Bearer {}", token));
    }

    if !body_bytes.is_empty() {
        upstream_req = upstream_req.header("content-type", "application/json");
    }

    // Send
    let upstream_resp = upstream_req
        .body(body_bytes.to_vec())
        .send()
        .await
        .with_context(|| format!("Failed to reach upstream: {}", upstream_url))?;

    let status = upstream_resp.status();
    let resp_headers = upstream_resp.headers().clone();
    let resp_body = upstream_resp.bytes().await?;

    let mut response = Response::builder().status(status.as_u16());
    for (name, value) in resp_headers.iter() {
        let name_str = name.as_str().to_lowercase();
        if matches!(name_str.as_str(), "transfer-encoding" | "connection") {
            continue;
        }
        response = response.header(name.clone(), value.clone());
    }
    response = response.header("x-claude-model-switch-provider", route.provider_name);

    Ok(response.body(Full::new(resp_body)).unwrap())
}

pub async fn run_proxy(port: u16) -> Result<()> {
    let config = ProfileConfig::load()?;
    let state = Arc::new(ProxyState::new(config));

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr).await?;
    println!("Proxy listening on http://127.0.0.1:{}", port);

    #[cfg(unix)]
    {
        let reload_state = state.clone();
        tokio::spawn(async move {
            let mut sig = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::hangup())
                .expect("Failed to register SIGHUP handler");
            loop {
                sig.recv().await;
                eprintln!("Received SIGHUP, reloading config...");
                if let Err(e) = reload_state.reload_config().await {
                    eprintln!("Failed to reload config: {:#}", e);
                } else {
                    let config = reload_state.config.read().await;
                    eprintln!("Reloaded. Active provider: {}", config.active);
                }
            }
        });
    }

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let state = state.clone();

        tokio::spawn(async move {
            if let Err(e) = http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(move |req| {
                        let state = state.clone();
                        handle_request(req, state)
                    }),
                )
                .await
            {
                eprintln!("Connection error: {}", e);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ModelMapping, ProfileConfig, Provider};
    use std::collections::HashMap;

    fn config_fixture() -> ProfileConfig {
        ProfileConfig {
            active: "claude".to_string(),
            providers: HashMap::from([
                (
                    "claude".to_string(),
                    Provider {
                        base_url: "https://api.anthropic.com".to_string(),
                        api_key: None,
                        auth_token: None,
                        models: None,
                    },
                ),
                (
                    "glm".to_string(),
                    Provider {
                        base_url: "https://open.z.ai/api/paas/v4".to_string(),
                        api_key: Some("k".to_string()),
                        auth_token: None,
                        models: Some(ModelMapping {
                            haiku: "glm-4.5-air".to_string(),
                            sonnet: "glm-4.7".to_string(),
                            opus: "glm-4.7".to_string(),
                        }),
                    },
                ),
            ]),
        }
    }

    #[test]
    fn resolve_active_route() {
        let config = config_fixture();
        let route = resolve_route("/v1/messages", &config).unwrap();
        assert_eq!(route.provider_name, "claude");
        assert_eq!(route.upstream_path, "/v1/messages");
    }

    #[test]
    fn resolve_profile_route() {
        let config = config_fixture();
        let route = resolve_route("/p/glm/v1/messages", &config).unwrap();
        assert_eq!(route.provider_name, "glm");
        assert_eq!(route.upstream_path, "/v1/messages");
    }

    #[test]
    fn resolve_profile_route_root_suffix() {
        let config = config_fixture();
        let route = resolve_route("/p/glm", &config).unwrap();
        assert_eq!(route.upstream_path, "/");
    }

    #[test]
    fn resolve_profile_route_missing_provider() {
        let config = config_fixture();
        assert!(resolve_route("/p/missing/v1/messages", &config).is_err());
    }
}
