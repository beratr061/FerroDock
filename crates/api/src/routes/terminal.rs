use std::time::Duration;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::response::Response;
use ferrodock_domain::terminal::TerminalService;
use ferrodock_infra::docker::{ShellSession, ShellSize};
use serde::Deserialize;
use serde_json::json;
use tokio::io::AsyncWriteExt;
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
    pub container: String,
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
    description = "Opens an interactive shell in a running container over WebSocket. Binary frames start with 0x00 for terminal bytes or 0x01 for JSON control messages (resize, exit)",
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
    let size = size(query.cols, query.rows);
    let session = state
        .terminal
        .open(actor(&principal, &client), &query.container, size)
        .await?;
    let service = state.terminal.clone();
    Ok(upgrade
        .max_message_size(MAX_FRAME)
        .on_upgrade(move |socket| relay(socket, session, service)))
}

fn framed(kind: u8, payload: &[u8]) -> Message {
    let mut frame = Vec::with_capacity(payload.len() + 1);
    frame.push(kind);
    frame.extend_from_slice(payload);
    Message::Binary(frame.into())
}

async fn apply_input(session: &mut ShellSession, service: &TerminalService, frame: &[u8]) -> bool {
    match frame.split_first() {
        Some((&DATA, bytes)) => {
            session.input.write_all(bytes).await.is_ok() && session.input.flush().await.is_ok()
        }
        Some((&CONTROL, json)) => {
            if let Ok(Control::Resize { cols, rows }) = serde_json::from_slice(json) {
                let _ = service
                    .resize(&session.exec_id, size(Some(cols), Some(rows)))
                    .await;
            }
            true
        }
        _ => true,
    }
}

async fn finish(socket: &mut WebSocket, exec_id: &str, service: &TerminalService) {
    let code = service.exit_code(exec_id).await;
    let exit = json!({ "type": "exit", "code": code }).to_string();
    let _ = socket.send(framed(CONTROL, exit.as_bytes())).await;
    let _ = socket.send(Message::Close(None)).await;
}

async fn relay(mut socket: WebSocket, mut session: ShellSession, service: TerminalService) {
    let mut ping = tokio::time::interval(PING_INTERVAL);
    loop {
        tokio::select! {
            chunk = futures_util::StreamExt::next(&mut session.output) => match chunk {
                Some(Ok(bytes)) => {
                    if socket.send(framed(DATA, &bytes)).await.is_err() {
                        return;
                    }
                }
                _ => return finish(&mut socket, &session.exec_id, &service).await,
            },
            incoming = socket.recv() => match incoming {
                Some(Ok(Message::Binary(frame))) => {
                    if !apply_input(&mut session, &service, &frame).await {
                        return;
                    }
                }
                Some(Ok(Message::Close(_)) | Err(_)) | None => return,
                Some(Ok(_)) => {}
            },
            _ = ping.tick() => {
                if socket.send(Message::Ping(Vec::new().into())).await.is_err() {
                    return;
                }
            }
        }
    }
}
