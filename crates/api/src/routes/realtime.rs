use std::time::Duration;

use axum::extract::State;
use axum::extract::ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade};
use axum::response::Response;
use ferrodock_core::Id;
use ferrodock_infra::realtime::{Envelope, Signal};
use tokio::sync::broadcast::Receiver;
use tokio::sync::broadcast::error::RecvError;

use crate::error::{ApiResult, ErrorEnvelope};
use crate::socket_auth::SocketAuth;
use crate::state::AppState;

const PING_INTERVAL: Duration = Duration::from_secs(20);
const MAX_LIFETIME: Duration = Duration::from_mins(15);
const SESSION_RENEW_CODE: u16 = 4000;

#[utoipa::path(
    get,
    path = "/api/v1/ws",
    tag = "realtime",
    description = "WebSocket that pushes change signals (deployment log, deployment status, containers) scoped to the caller's organization; data is then read through the REST API",
    responses((status = 101), (status = 401, body = ErrorEnvelope), (status = 403, body = ErrorEnvelope))
)]
pub async fn realtime(
    upgrade: WebSocketUpgrade,
    State(state): State<AppState>,
    SocketAuth(principal): SocketAuth,
) -> ApiResult<Response> {
    let receiver = state.hub.subscribe();
    let organization_id = principal.organization_id;
    Ok(upgrade.on_upgrade(move |socket| serve(socket, receiver, organization_id)))
}

fn encode(signal: &Signal) -> Option<Message> {
    serde_json::to_string(signal)
        .ok()
        .map(|text| Message::Text(text.into()))
}

fn forward(
    envelope: Result<Envelope, RecvError>,
    organization_id: Id,
) -> Result<Option<Message>, ()> {
    match envelope {
        Ok(envelope) if envelope.visible_to(organization_id) => Ok(encode(&envelope.signal)),
        Ok(_) => Ok(None),
        Err(RecvError::Lagged(_)) => Ok(encode(&Signal::Resync)),
        Err(RecvError::Closed) => Err(()),
    }
}

async fn serve(mut socket: WebSocket, mut receiver: Receiver<Envelope>, organization_id: Id) {
    let lifetime = tokio::time::sleep(MAX_LIFETIME);
    tokio::pin!(lifetime);
    let mut ping = tokio::time::interval(PING_INTERVAL);
    loop {
        let outgoing = tokio::select! {
            () = &mut lifetime => {
                let frame = CloseFrame { code: SESSION_RENEW_CODE, reason: "renew".into() };
                let _ = socket.send(Message::Close(Some(frame))).await;
                return;
            }
            _ = ping.tick() => Some(Message::Ping(Vec::new().into())),
            envelope = receiver.recv() => match forward(envelope, organization_id) {
                Ok(message) => message,
                Err(()) => return,
            },
            incoming = socket.recv() => match incoming {
                Some(Ok(Message::Close(_)) | Err(_)) | None => return,
                Some(Ok(_)) => None,
            },
        };
        if let Some(message) = outgoing
            && socket.send(message).await.is_err()
        {
            return;
        }
    }
}
