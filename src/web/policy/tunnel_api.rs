use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::app::TunnelStatusView;

use super::{forbidden, validate_host, WebApiState, WebError, WebErrorBody, WebErrorResponse};

#[derive(Debug, Deserialize)]
pub(super) struct TunnelLoginRequest {
    password: String,
}

#[derive(Debug, Serialize)]
pub(super) struct TunnelLoginResponse {
    token: String,
}

pub(super) async fn status(
    State(state): State<WebApiState>,
    headers: HeaderMap,
) -> Result<Json<TunnelStatusView>, WebErrorResponse> {
    validate_host(&state, &headers).await?;
    Ok(Json(state.tunnel.status(Utc::now()).await.into()))
}

pub(super) async fn start(
    State(state): State<WebApiState>,
    headers: HeaderMap,
) -> Result<Json<TunnelStatusView>, WebErrorResponse> {
    validate_host(&state, &headers).await?;
    let status = state.tunnel.start(Utc::now()).await;
    let snapshot = state
        .events
        .subscribe()
        .bootstrap
        .snapshot
        .with_tunnel_status(status.clone());
    state.events.publish_snapshot(snapshot);
    Ok(Json(status.into()))
}

pub(super) async fn login(
    State(state): State<WebApiState>,
    headers: HeaderMap,
    Json(request): Json<TunnelLoginRequest>,
) -> Result<Json<TunnelLoginResponse>, WebErrorResponse> {
    validate_host(&state, &headers).await?;
    let config = state.config.read().await;
    let password = config.tunnel.config_password.clone();
    drop(config);
    match state
        .tunnel
        .issue_token(&password, &request.password, Utc::now())
        .await
    {
        Ok(Some(token)) => Ok(Json(TunnelLoginResponse {
            token: token.as_str().to_string(),
        })),
        Ok(None) => Err(forbidden(
            "tunnel_login_rejected",
            "Tunnel login credentials were rejected",
        )),
        Err(_error) => Err(WebErrorResponse {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: WebError {
                error: WebErrorBody {
                    code: "tunnel_login_failed",
                    message: "Tunnel login could not be completed".to_string(),
                },
            },
        }),
    }
}
