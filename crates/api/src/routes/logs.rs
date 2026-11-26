use std::time::Duration;

use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Path, Query, State, WebSocketUpgrade};
use axum::response::Response;
use ferrodock_core::AppResult;
use ferrodock_infra::docker::{LogLine, LogStream};
use futures_util::StreamExt;
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

use crate::error::{ApiResult, ErrorEnvelope};
use crate::socket_auth::SocketAuth;
use crate::state::AppState;

const DEFAULT_TAIL: u32 = 300;
const PING_INTERVAL: Duration = Duration::from_secs(20);

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct LogsQuery {
    pub container: String,
    pub tail: Option<u32>,
}

#[utoipa::path(
    get,
    path = "/api/v1/applications/{id}/logs",
    tag = "applications",
    description = "Streams the output of one of the application's containers over WebSocket; each text frame is a JSON object with source (stdout or stderr) and text",
    params(("id" = Uuid, Path, description = "Application id"), LogsQuery),
    responses(
        (status = 101),
        (status = 403, body = ErrorEnvelope),
        (status = 404, body = ErrorEnvelope)
    )
)]
pub async fn container_logs(
    upgrade: WebSocketUpgrade,
    State(state): State<AppState>,
    SocketAuth(principal): SocketAuth,
    Path(id): Path<Uuid>,
    Query(query): Query<LogsQuery>,
) -> ApiResult<Response> {
    let tail = query.tail.unwrap_or(DEFAULT_TAIL);
    let stream = state
        .applications
        .container_logs(&principal, id.into(), &query.container, tail)
        .await?;
    Ok(upgrade.on_upgrade(move |socket| relay(socket, stream)))
}

async fn forward(socket: &mut WebSocket, line: Option<AppResult<LogLine>>) -> bool {
    let Some(Ok(line)) = line else {
        let _ = socket.send(Message::Close(None)).await;
        return false;
    };
    match serde_json::to_string(&line) {
        Ok(text) => socket.send(Message::Text(text.into())).await.is_ok(),
        Err(_) => true,
    }
}

pub(super) async fn relay(mut socket: WebSocket, mut stream: LogStream) {
    let mut ping = tokio::time::interval(PING_INTERVAL);
    loop {
        let open = tokio::select! {
            line = stream.next() => forward(&mut socket, line).await,
            incoming = socket.recv() => matches!(incoming, Some(Ok(message)) if !matches!(message, Message::Close(_))),
            _ = ping.tick() => socket.send(Message::Ping(Vec::new().into())).await.is_ok(),
        };
        if !open {
            return;
        }
    }
}
