use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::header::HOST;
use axum::http::{HeaderMap, Method, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use serde::Serialize;
use tokio::sync::RwLock;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::services::ServeDir;

use crate::app::{
    AppEventStore, AppSnapshot, ConfigEndpointPolicy, EventFeedItem, JournalSourceView,
    MatrixStartupStatus, MatrixStatusView, WebStartupStatus, WebStatusView,
};
use crate::config::{ConfigSource, RuntimeConfig};
use crate::web::tunnel_state::WebTunnelState;

mod config_api;
mod host;
mod tunnel_api;
mod ws;

pub(crate) use config_api::default_runtime_for_web_config;
pub(crate) use host::RequestHost;

pub type WebEndpointPolicy = ConfigEndpointPolicy;

impl WebEndpointPolicy {
    pub fn new(remote_bind: bool) -> Self {
        Self {
            state_changing_enabled: true,
            state_changing_reason: "enabled for trusted WebUI clients".to_string(),
            remote_bind,
        }
    }
}

#[derive(Clone)]
pub struct WebApiState {
    asset_root: PathBuf,
    bind_host: String,
    policy: WebEndpointPolicy,
    events: AppEventStore,
    config: Arc<RwLock<RuntimeConfig>>,
    config_source: Arc<RwLock<ConfigSource>>,
    web_status: WebStartupStatus,
    tunnel: WebTunnelState,
}

#[derive(Debug, Serialize)]
pub struct WebError {
    error: WebErrorBody,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct WebErrorBody {
    code: &'static str,
    message: String,
}

impl WebApiState {
    pub fn new(
        asset_root: PathBuf,
        bind_host: String,
        runtime_config: RuntimeConfig,
        events: AppEventStore,
        web_status: WebStartupStatus,
        tunnel: WebTunnelState,
    ) -> Self {
        let remote_bind = !is_loopback_host(&bind_host);
        let config_source = runtime_config.config_source.clone();
        Self {
            asset_root,
            bind_host,
            policy: WebEndpointPolicy::new(remote_bind),
            events,
            config: Arc::new(RwLock::new(runtime_config)),
            config_source: Arc::new(RwLock::new(config_source)),
            web_status,
            tunnel,
        }
    }
}

pub(crate) fn router(state: WebApiState) -> Router {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT])
        .allow_origin(AllowOrigin::predicate(|origin, _parts| {
            origin.to_str().map(is_trusted_origin).unwrap_or(false)
        }));
    let asset_root = state.asset_root.clone();
    let host_policy_state = state.clone();
    Router::new()
        .route("/api/health", get(health))
        .route("/api/snapshot", get(snapshot))
        .route(
            "/api/config",
            get(config_api::config_view).put(config_api::update_config),
        )
        .route("/api/web/status", get(web_status))
        .route("/api/matrix/status", get(matrix_status))
        .route("/api/web/policy", get(web_policy))
        .route("/api/tunnel/status", get(tunnel_api::status))
        .route("/api/tunnel/start", post(tunnel_api::start))
        .route("/api/tunnel/login", post(tunnel_api::login))
        .route("/api/events", get(ws::websocket))
        .route("/ws", get(ws::websocket))
        .fallback_service(ServeDir::new(asset_root).append_index_html_on_directories(true))
        .layer(cors)
        .layer(middleware::from_fn_with_state(
            host_policy_state,
            validate_host_middleware,
        ))
        .with_state(state)
}

async fn health(State(state): State<WebApiState>) -> Result<Json<HealthView>, WebErrorResponse> {
    Ok(Json(HealthView {
        status: "ok",
        web: WebStatusView::from(state.web_status),
        checked_at: Utc::now().to_rfc3339(),
    }))
}

async fn snapshot(
    State(state): State<WebApiState>,
    headers: HeaderMap,
) -> Result<Json<SnapshotApiView>, WebErrorResponse> {
    let surface = validate_host(&state, &headers).await?;
    let snapshot = state.events.subscribe().bootstrap.snapshot;
    let snapshot = snapshot_for_surface(snapshot, &state, surface).await;
    Ok(Json(SnapshotApiView {
        events: snapshot.event_feed.clone(),
        snapshot,
    }))
}

async fn web_status(
    State(state): State<WebApiState>,
    headers: HeaderMap,
) -> Result<Json<WebStatusView>, WebErrorResponse> {
    validate_host(&state, &headers).await?;
    Ok(Json(WebStatusView::from(state.web_status)))
}

async fn matrix_status(
    State(state): State<WebApiState>,
    headers: HeaderMap,
) -> Result<Json<MatrixStatusView>, WebErrorResponse> {
    validate_host(&state, &headers).await?;
    let config = state.config.read().await;
    Ok(Json(
        MatrixStartupStatus::from_runtime_config(&config).into(),
    ))
}

async fn web_policy(
    State(state): State<WebApiState>,
    headers: HeaderMap,
) -> Result<Json<WebEndpointPolicy>, WebErrorResponse> {
    validate_host(&state, &headers).await?;
    Ok(Json(state.policy))
}

async fn authorize_state_change(
    state: &WebApiState,
    headers: &HeaderMap,
) -> Result<(), WebErrorResponse> {
    validate_host(state, headers).await?;
    Ok(())
}

async fn validate_host_middleware(
    State(state): State<WebApiState>,
    request: Request,
    next: Next,
) -> Result<Response, WebErrorResponse> {
    validate_host(&state, request.headers()).await?;
    Ok(next.run(request).await)
}

pub(super) async fn validate_host(
    state: &WebApiState,
    headers: &HeaderMap,
) -> Result<RequestHost, WebErrorResponse> {
    let Some(host) = headers.get(HOST).and_then(|value| value.to_str().ok()) else {
        return Err(forbidden("host_required", "Host header is required"));
    };
    let host = host_without_port(host);
    if let Some(active_tunnel) = state.tunnel.active_tunnel(Utc::now()).await {
        if let Some(surface) = host::classify_host(host, &state.bind_host, Some(&active_tunnel)) {
            return Ok(surface);
        }
    }
    if let Some(surface) = host::classify_host(host, &state.bind_host, None) {
        return Ok(surface);
    }
    Err(forbidden("host_rejected", "Host header is not trusted"))
}

pub(super) async fn snapshot_for_surface(
    snapshot: AppSnapshot,
    state: &WebApiState,
    surface: RequestHost,
) -> AppSnapshot {
    if surface == RequestHost::LocalLoopback {
        return snapshot;
    }
    let config = state.config.read().await;
    redact_journal_folder(snapshot, &config)
}

fn redact_journal_folder(mut snapshot: AppSnapshot, config: &RuntimeConfig) -> AppSnapshot {
    snapshot.journal_source = JournalSourceView {
        folder: if config.journal.folder.is_empty() {
            "Default journal folder".to_string()
        } else {
            "Configured journal folder".to_string()
        },
        ..snapshot.journal_source
    };
    snapshot
}

fn host_without_port(host: &str) -> &str {
    let trimmed = host.trim();
    if let Some(without_bracket) = trimmed.strip_prefix('[') {
        return without_bracket.split(']').next().unwrap_or(without_bracket);
    }
    trimmed.split(':').next().unwrap_or(trimmed)
}

fn is_trusted_origin(origin: &str) -> bool {
    origin.starts_with("http://127.0.0.1:")
        || origin.starts_with("http://localhost:")
        || origin.starts_with("http://[::1]:")
}

fn is_loopback_host(host: &str) -> bool {
    matches!(host.trim(), "127.0.0.1" | "localhost" | "::1" | "[::1]")
}

fn is_wildcard_bind_host(host: &str) -> bool {
    matches!(host.trim(), "0.0.0.0" | "::" | "[::]")
}

fn forbidden(code: &'static str, message: impl Into<String>) -> WebErrorResponse {
    WebErrorResponse {
        status: StatusCode::FORBIDDEN,
        body: WebError {
            error: WebErrorBody {
                code,
                message: message.into(),
            },
        },
    }
}

fn unprocessable(code: &'static str, message: impl Into<String>) -> WebErrorResponse {
    WebErrorResponse {
        status: StatusCode::UNPROCESSABLE_ENTITY,
        body: WebError {
            error: WebErrorBody {
                code,
                message: message.into(),
            },
        },
    }
}

#[derive(Debug)]
pub struct WebErrorResponse {
    status: StatusCode,
    body: WebError,
}

impl IntoResponse for WebErrorResponse {
    fn into_response(self) -> Response {
        (self.status, Json(self.body)).into_response()
    }
}

#[derive(Debug, Serialize)]
struct HealthView {
    status: &'static str,
    web: WebStatusView,
    checked_at: String,
}

#[derive(Clone, Debug, Serialize)]
struct SnapshotApiView {
    #[serde(flatten)]
    snapshot: AppSnapshot,
    events: Vec<EventFeedItem>,
}

#[cfg(test)]
mod tests {
    use super::WebEndpointPolicy;

    #[test]
    fn remote_bind_keeps_config_state_changes_enabled() {
        let policy = WebEndpointPolicy::new(true);

        assert!(policy.state_changing_enabled);
        assert!(policy.remote_bind);
    }
}
