use std::time::Duration;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::response::Response;
use ferrodock_core::AppError;
use ferrodock_domain::terminal::{Attachment, TerminalEvent, TerminalService};
use ferrodock_infra::docker::ShellSize;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::broadcast::error::RecvError;
use utoipa::IntoParams;

use super::actor;
use crate::error::{ApiResult, ErrorEnvelope};
use crate::extract::ClientMeta;
use crate::socket_auth::SocketAuth;
use crate::state::AppState;

const DATA: u8 = 0;
const CONTROL: u8 = 1;
const MAX_FRAME: usize = 64 * 1024;
const PING_INTERVAL: Duration = Duration::from_secs(20);

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct TerminalQuery {
    pub container: Option<String>,
    pub session: Option<String>,
    pub cols: Option<u16>,
    pub rows: Option<u16>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Control {
    Resize { cols: u16, rows: u16 },
}

fn size(columns: Option<u16>, rows: Option<u16>) -> ShellSize {
    ShellSize {
        columns: columns.unwrap_or(80).clamp(20, 500),
        rows: rows.unwrap_or(24).clamp(5, 200),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/terminal",
    tag = "terminal",
    description = "Opens an interactive shell in a running container over WebSocket, or resumes one of your shells with session. Binary frames start with 0x00 for terminal bytes or 0x01 for JSON control messages (session, resize, exit). A shell keeps running for ten minutes after the connection drops and replays its last 64 KiB on resume",
    params(TerminalQuery),
    responses(
        (status = 101),
        (status = 403, body = ErrorEnvelope),
        (status = 404, body = ErrorEnvelope),
        (status = 409, body = ErrorEnvelope)
    )
)]
pub async fn terminal(
    upgrade: WebSocketUpgrade,
    State(state): State<AppState>,
    SocketAuth(principal): SocketAuth,
    client: ClientMeta,
    Query(query): Query<TerminalQuery>,
) -> ApiResult<Response> {
    let wanted = size(query.cols, query.rows);
    let attachment = match (&query.session, &query.container) {
        (Some(session), _) => {
            let attachment = state.terminal.resume(&principal, session)?;
            let _ = state.terminal.resize(&attachment.session, wanted).await;
            attachment
        }
        (None, Some(container)) => {
            state
                .terminal
                .open(actor(&principal, &client), container, wanted)
                .await?
        }
        (None, None) => return Err(AppError::validation("TERMINAL_TARGET_MISSING").into()),
    };
    let service = state.terminal.clone();
    Ok(upgrade
        .max_message_size(MAX_FRAME)
        .on_upgrade(move |socket| relay(socket, attachment, service)))
}

fn framed(kind: u8, payload: &[u8]) -> Message {
    let mut frame = Vec::with_capacity(payload.len() + 1);
    frame.push(kind);
    frame.extend_from_slice(payload);
    Message::Binary(frame.into())
}

fn control(value: &serde_json::Value) -> Message {
    framed(CONTROL, value.to_string().as_bytes())
}

async fn apply_input(attachment: &Attachment, service: &TerminalService, frame: &[u8]) -> bool {
    match frame.split_first() {
        Some((&DATA, bytes)) => service.write(&attachment.session, bytes).await.is_ok(),
        Some((&CONTROL, json)) => {
            if let Ok(Control::Resize { cols, rows }) = serde_json::from_slice(json) {
                let _ = service
                    .resize(&attachment.session, size(Some(cols), Some(rows)))
                    .await;
            }
            true
        }
        _ => true,
    }
}

async fn finish(socket: &mut WebSocket, code: Option<i64>) {
    let _ = socket
        .send(control(&json!({ "type": "exit", "code": code })))
        .await;
    let _ = socket.send(Message::Close(None)).await;
}

async fn greet(socket: &mut WebSocket, attachment: &Attachment) -> bool {
    let hello = json!({ "type": "session", "id": attachment.session.id });
    if socket.send(control(&hello)).await.is_err() {
        return false;
    }
    attachment.replay.is_empty() || socket.send(framed(DATA, &attachment.replay)).await.is_ok()
}

async fn forward_event(socket: &mut WebSocket, event: Result<TerminalEvent, RecvError>) -> bool {
    match event {
        Ok(TerminalEvent::Output(bytes)) => socket.send(framed(DATA, &bytes)).await.is_ok(),
        Ok(TerminalEvent::Exit(code)) => {
            finish(socket, code).await;
            false
        }
        Err(RecvError::Lagged(_)) => true,
        Err(RecvError::Closed) => false,
    }
}

async fn forward_input(
    incoming: Option<Result<Message, axum::Error>>,
    attachment: &Attachment,
    service: &TerminalService,
) -> bool {
    match incoming {
        Some(Ok(Message::Binary(frame))) => apply_input(attachment, service, &frame).await,
        Some(Ok(Message::Close(_)) | Err(_)) | None => false,
        Some(Ok(_)) => true,
    }
}

async fn relay(mut socket: WebSocket, mut attachment: Attachment, service: TerminalService) {
    if !greet(&mut socket, &attachment).await {
        return;
    }
    if let Some(exit) = attachment.exit {
        return finish(&mut socket, exit.code).await;
    }
    let mut ping = tokio::time::interval(PING_INTERVAL);
    loop {
        let open = tokio::select! {
            event = attachment.events.recv() => forward_event(&mut socket, event).await,
            incoming = socket.recv() => forward_input(incoming, &attachment, &service).await,
            _ = ping.tick() => socket.send(Message::Ping(Vec::new().into())).await.is_ok(),
        };
        if !open {
            return;
        }
    }
}
