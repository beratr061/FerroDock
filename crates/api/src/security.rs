use axum::extract::{Request, State};
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use ferrodock_core::AppError;
use tower::ServiceBuilder;
use tower::layer::util::{Identity, Stack};
use tower_http::set_header::SetResponseHeaderLayer;

use crate::error::ApiError;
use crate::session_cookie;
use crate::state::AppState;

const CONTENT_SECURITY_POLICY: &str = "default-src 'self'; script-src 'self'; \
    style-src 'self'; img-src 'self' data:; font-src 'self'; connect-src 'self'; \
    frame-ancestors 'none'; base-uri 'self'; form-action 'self'; object-src 'none'";

type HeaderLayer = SetResponseHeaderLayer<HeaderValue>;

fn static_header(name: HeaderName, value: &'static str) -> HeaderLayer {
    SetResponseHeaderLayer::if_not_present(name, HeaderValue::from_static(value))
}

pub type SecurityHeaders = ServiceBuilder<
    Stack<
        HeaderLayer,
        Stack<HeaderLayer, Stack<HeaderLayer, Stack<HeaderLayer, Stack<HeaderLayer, Identity>>>>,
    >,
>;

pub fn headers() -> SecurityHeaders {
    ServiceBuilder::new()
        .layer(static_header(
            header::CONTENT_SECURITY_POLICY,
            CONTENT_SECURITY_POLICY,
        ))
        .layer(static_header(header::X_FRAME_OPTIONS, "DENY"))
        .layer(static_header(header::X_CONTENT_TYPE_OPTIONS, "nosniff"))
        .layer(static_header(
            header::REFERRER_POLICY,
            "strict-origin-when-cross-origin",
        ))
        .layer(static_header(
            HeaderName::from_static("permissions-policy"),
            "camera=(), microphone=(), geolocation=(), payment=()",
        ))
}

pub fn hsts(enabled: bool) -> Option<HeaderLayer> {
    enabled.then(|| {
        static_header(
            header::STRICT_TRANSPORT_SECURITY,
            "max-age=31536000; includeSubDomains",
        )
    })
}

fn is_safe_method(method: &Method) -> bool {
    matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS)
}

fn header_str<'a>(headers: &'a HeaderMap, name: &HeaderName) -> Option<&'a str> {
    headers.get(name).and_then(|value| value.to_str().ok())
}

fn uses_session_cookie(headers: &HeaderMap, secure: bool) -> bool {
    let has_bearer = header_str(headers, &header::AUTHORIZATION)
        .is_some_and(|value| value.starts_with("Bearer "));
    let cookie_name = session_cookie::name(secure);
    let has_cookie = headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .filter_map(|pair| pair.split_once('='))
        .any(|(name, _)| name.trim() == cookie_name);
    has_cookie && !has_bearer
}

pub(crate) fn origin_allowed(headers: &HeaderMap, public_origin: Option<&str>) -> bool {
    if header_str(headers, &HeaderName::from_static("sec-fetch-site")) == Some("same-origin") {
        return true;
    }
    let Some(origin) = header_str(headers, &header::ORIGIN) else {
        return false;
    };
    if public_origin.is_some_and(|expected| expected.trim_end_matches('/') == origin) {
        return true;
    }
    let origin_host = origin.split_once("://").map(|(_, host)| host);
    origin_host.is_some() && origin_host == header_str(headers, &header::HOST)
}

pub async fn csrf_guard(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let settings = &state.settings;
    let needs_check = !is_safe_method(request.method())
        && uses_session_cookie(request.headers(), settings.secure_cookies);
    if needs_check && !origin_allowed(request.headers(), settings.public_origin.as_deref()) {
        let error = AppError::forbidden().with_param("reason", "CSRF_ORIGIN_MISMATCH");
        return ApiError(error).into_response();
    }
    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with(pairs: &[(&'static str, &'static str)]) -> HeaderMap {
        let mut map = HeaderMap::new();
        for (name, value) in pairs {
            map.insert(
                HeaderName::from_static(name),
                HeaderValue::from_static(value),
            );
        }
        map
    }

    #[test]
    fn matches_session_cookie_by_exact_name() {
        assert!(uses_session_cookie(
            &with(&[("cookie", "theme=dark; fd_session=abc")]),
            false
        ));
        assert!(!uses_session_cookie(
            &with(&[("cookie", "old_fd_session=abc")]),
            false
        ));
        assert!(!uses_session_cookie(
            &with(&[("cookie", "__Host-fd_session=abc")]),
            false
        ));
        assert!(uses_session_cookie(
            &with(&[("cookie", "__Host-fd_session=abc")]),
            true
        ));
        let bearer = with(&[
            ("cookie", "fd_session=abc"),
            ("authorization", "Bearer fd_live_x"),
        ]);
        assert!(!uses_session_cookie(&bearer, false));
    }

    #[test]
    fn accepts_same_origin_and_rejects_cross_site() {
        assert!(origin_allowed(
            &with(&[("sec-fetch-site", "same-origin")]),
            None
        ));
        assert!(origin_allowed(
            &with(&[("origin", "https://panel.test"), ("host", "panel.test")]),
            None
        ));
        assert!(origin_allowed(
            &with(&[("origin", "https://panel.test")]),
            Some("https://panel.test/")
        ));
        assert!(!origin_allowed(
            &with(&[("origin", "https://evil.test"), ("host", "panel.test")]),
            None
        ));
        assert!(!origin_allowed(&HeaderMap::new(), None));
    }
}
