use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;
use serde::Serialize;

use crate::app::{AppLiveUpdate, AppSnapshot, EventFeedItem};
use crate::text::line_safe;

use super::{
    snapshot_for_surface, validate_host, RequestHost, WebApiState, WebErrorBody, WebErrorResponse,
};

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(super) enum WebSocketEnvelope {
    Hello {
        version: u8,
        snapshot: AppSnapshot,
        event_feed: Vec<EventFeedItem>,
    },
    Snapshot {
        version: u8,
        snapshot: AppSnapshot,
    },
    Event {
        version: u8,
        item: EventFeedItem,
    },
    Error {
        version: u8,
        error: WebErrorBody,
    },
}

pub(super) async fn websocket(
    State(state): State<WebApiState>,
    headers: HeaderMap,
    upgrade: WebSocketUpgrade,
) -> Result<Response, WebErrorResponse> {
    let surface = validate_host(&state, &headers).await?;
    Ok(upgrade.on_upgrade(move |socket| websocket_session(socket, state, surface)))
}

async fn websocket_session(mut socket: WebSocket, state: WebApiState, surface: RequestHost) {
    let subscriber = state.events.subscribe();
    let snapshot = snapshot_for_surface(subscriber.bootstrap.snapshot, &state, surface).await;
    let hello = WebSocketEnvelope::Hello {
        version: 1,
        snapshot,
        event_feed: subscriber.bootstrap.recent_events,
    };
    if send_envelope(&mut socket, &hello).await.is_err() {
        return;
    }

    let stop = Arc::new(AtomicBool::new(false));
    let receiver_stop = Arc::clone(&stop);
    let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
    std::thread::spawn(move || {
        while !receiver_stop.load(Ordering::Relaxed) {
            match subscriber.live.recv_timeout(Duration::from_millis(200)) {
                Ok(update) => {
                    if sender.send(update).is_err() {
                        break;
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    });

    while let Some(update) = receiver.recv().await {
        let envelope = match update {
            AppLiveUpdate::Snapshot { snapshot } => {
                let snapshot = snapshot_for_surface(*snapshot, &state, surface).await;
                WebSocketEnvelope::Snapshot {
                    version: 1,
                    snapshot,
                }
            }
            AppLiveUpdate::Event { item } => WebSocketEnvelope::Event { version: 1, item },
        };
        if send_envelope(&mut socket, &envelope).await.is_err() {
            stop.store(true, Ordering::Relaxed);
            return;
        }
    }
    stop.store(true, Ordering::Relaxed);
}

async fn send_envelope(
    socket: &mut WebSocket,
    envelope: &WebSocketEnvelope,
) -> Result<(), axum::Error> {
    let text = match serde_json::to_string(envelope) {
        Ok(text) => text,
        Err(error) => {
            let fallback = WebSocketEnvelope::Error {
                version: 1,
                error: WebErrorBody {
                    code: "serialization_error",
                    message: line_safe(&error.to_string()),
                },
            };
            serde_json::to_string(&fallback).unwrap_or_else(|_| {
                "{\"type\":\"error\",\"version\":1,\"error\":{\"code\":\"serialization_error\",\"message\":\"serialization failed\"}}".to_string()
            })
        }
    };
    socket.send(Message::Text(text.into())).await
}
