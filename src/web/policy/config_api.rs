use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;

use crate::app::{ConfigApiView, EditableConfigUpdate, EditableConfigView};
use crate::config::{AppConfig, ConfigSource, ConfigWriteError, RuntimeConfig, WebConfig};

use super::{
    authorize_state_change, forbidden, unprocessable, validate_host, WebApiState, WebError,
    WebErrorBody, WebErrorResponse,
};

pub(super) async fn config_view(
    State(state): State<WebApiState>,
    headers: HeaderMap,
) -> Result<Json<ConfigApiView>, WebErrorResponse> {
    validate_host(&state, &headers)?;
    let config = state.config.read().await;
    Ok(Json(ConfigApiView {
        version: 1,
        config: EditableConfigView::from_runtime_config(&config),
        policy: state.policy.clone(),
    }))
}

pub(super) async fn update_config(
    State(state): State<WebApiState>,
    headers: HeaderMap,
    Json(update): Json<EditableConfigUpdate>,
) -> Result<Json<ConfigApiView>, WebErrorResponse> {
    authorize_state_change(&state, &headers)?;
    let source = state.config_source.read().await.clone();
    let outcome = AppConfig::write_update_to_source(&source, &update).map_err(write_error)?;
    let runtime = outcome
        .config
        .into_runtime_with_source(outcome.source.clone(), &Default::default());
    {
        let mut config = state.config.write().await;
        *config = runtime;
    }
    {
        let mut config_source = state.config_source.write().await;
        *config_source = outcome.source;
    }
    config_view(State(state), headers).await
}

fn write_error(error: ConfigWriteError) -> WebErrorResponse {
    match error {
        ConfigWriteError::UnsafeRemoteBind { .. } => {
            unprocessable("unsafe_remote_bind", "Invalid WebUI bind host.")
        }
        ConfigWriteError::NoWritableTarget | ConfigWriteError::Blocked { .. } => forbidden(
            "config_write_blocked",
            "This config source cannot be saved from the WebUI.",
        ),
        ConfigWriteError::MalformedToml { .. } => unprocessable(
            "malformed_config",
            "The config file is malformed. Fix the file before saving from the WebUI.",
        ),
        ConfigWriteError::Io { .. } => WebErrorResponse {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: WebError {
                error: WebErrorBody {
                    code: "config_write_failed",
                    message:
                        "The config file could not be saved. Check file permissions and try again."
                            .to_string(),
                },
            },
        },
    }
}

pub(crate) fn default_runtime_for_web_config(web: &WebConfig) -> RuntimeConfig {
    RuntimeConfig {
        web: web.clone(),
        config_source: ConfigSource::InMemory,
        ..RuntimeConfig::default()
    }
}
