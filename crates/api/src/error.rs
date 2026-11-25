use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use ferrodock_core::{AppError, ErrorKind};
use serde::Serialize;
use serde_json::{Map, Value};
use utoipa::ToSchema;

#[derive(Debug)]
pub struct ApiError(pub AppError);

pub type ApiResult<T> = Result<T, ApiError>;

impl From<AppError> for ApiError {
    fn from(error: AppError) -> Self {
        Self(error)
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    #[schema(value_type = Object)]
    pub params: Map<String, Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorEnvelope {
    pub error: ErrorBody,
}

fn status_for(kind: ErrorKind) -> StatusCode {
    match kind {
        ErrorKind::Validation => StatusCode::UNPROCESSABLE_ENTITY,
        ErrorKind::Unauthorized => StatusCode::UNAUTHORIZED,
        ErrorKind::Forbidden => StatusCode::FORBIDDEN,
        ErrorKind::NotFound => StatusCode::NOT_FOUND,
        ErrorKind::Conflict => StatusCode::CONFLICT,
        ErrorKind::RateLimited => StatusCode::TOO_MANY_REQUESTS,
        ErrorKind::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
        ErrorKind::Internal => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn fallback_message(kind: ErrorKind) -> &'static str {
    match kind {
        ErrorKind::Validation => "The request contains invalid data",
        ErrorKind::Unauthorized => "Authentication is required",
        ErrorKind::Forbidden => "You do not have permission for this action",
        ErrorKind::NotFound => "The requested resource was not found",
        ErrorKind::Conflict => "The resource is in a conflicting state",
        ErrorKind::RateLimited => "Too many requests",
        ErrorKind::Unavailable => "A dependency is temporarily unavailable",
        ErrorKind::Internal => "An unexpected error occurred",
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let kind = self.0.kind();
        if matches!(kind, ErrorKind::Internal | ErrorKind::Unavailable) {
            tracing::error!(code = self.0.code(), error = ?self.0, "request failed");
        }
        let params = if kind == ErrorKind::Internal {
            Map::new()
        } else {
            self.0.params().clone()
        };
        let body = ErrorEnvelope {
            error: ErrorBody {
                code: self.0.code().to_owned(),
                message: fallback_message(kind).to_owned(),
                params,
            },
        };
        (status_for(kind), Json(body)).into_response()
    }
}
