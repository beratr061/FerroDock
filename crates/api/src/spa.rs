use std::borrow::Cow;

use axum::body::Body;
use axum::http::{StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use ferrodock_core::crypto::random_bytes;
use rust_embed::RustEmbed;

use crate::security::content_security_policy;

#[derive(RustEmbed)]
#[folder = "../../web/dist"]
#[allow_missing = true]
struct Assets;

const INDEX: &str = "index.html";
const IMMUTABLE: &str = "public, max-age=31536000, immutable";
const REVALIDATE: &str = "no-cache";
const NONCE_PLACEHOLDER: &str = "FERRODOCK_CSP_NONCE";
const NONCE_LEN: usize = 16;

fn asset_response(path: &str, data: Cow<'static, [u8]>) -> Response {
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    let cache = if path.starts_with("assets/") {
        IMMUTABLE
    } else {
        REVALIDATE
    };
    (
        [
            (header::CONTENT_TYPE, mime.as_ref().to_owned()),
            (header::CACHE_CONTROL, cache.to_owned()),
        ],
        Body::from(data),
    )
        .into_response()
}

fn index_response(template: &[u8]) -> Response {
    let nonce = STANDARD.encode(random_bytes::<NONCE_LEN>());
    let html = String::from_utf8_lossy(template).replace(NONCE_PLACEHOLDER, &nonce);
    (
        [
            (header::CONTENT_TYPE, "text/html; charset=utf-8".to_owned()),
            (header::CACHE_CONTROL, REVALIDATE.to_owned()),
            (
                header::CONTENT_SECURITY_POLICY,
                content_security_policy(Some(&nonce)),
            ),
        ],
        html,
    )
        .into_response()
}

pub async fn serve(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    if path.starts_with("api/") {
        return StatusCode::NOT_FOUND.into_response();
    }
    if let Some(file) = Assets::get(path).filter(|_| !path.is_empty()) {
        return asset_response(path, file.data);
    }
    match Assets::get(INDEX) {
        Some(index) => index_response(&index.data),
        None => (
            StatusCode::SERVICE_UNAVAILABLE,
            "web interface is not built",
        )
            .into_response(),
    }
}
