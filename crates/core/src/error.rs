use std::error::Error as StdError;

use serde::Serialize;
use serde_json::{Map, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    Validation,
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict,
    RateLimited,
    Unavailable,
    Internal,
}

type BoxedSource = Box<dyn StdError + Send + Sync + 'static>;

#[derive(Debug, thiserror::Error)]
#[error("{code}")]
pub struct AppError {
    kind: ErrorKind,
    code: &'static str,
    params: Map<String, Value>,
    #[source]
    source: Option<BoxedSource>,
}

pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    pub fn new(kind: ErrorKind, code: &'static str) -> Self {
        Self {
            kind,
            code,
            params: Map::new(),
            source: None,
        }
    }

    pub fn validation(code: &'static str) -> Self {
        Self::new(ErrorKind::Validation, code)
    }

    pub fn unauthorized() -> Self {
        Self::new(ErrorKind::Unauthorized, "UNAUTHORIZED")
    }

    pub fn forbidden() -> Self {
        Self::new(ErrorKind::Forbidden, "FORBIDDEN")
    }

    pub fn not_found(code: &'static str) -> Self {
        Self::new(ErrorKind::NotFound, code)
    }

    pub fn conflict(code: &'static str) -> Self {
        Self::new(ErrorKind::Conflict, code)
    }

    pub fn internal(source: impl Into<BoxedSource>) -> Self {
        Self::new(ErrorKind::Internal, "INTERNAL").with_source(source)
    }

    #[must_use]
    pub fn with_param(mut self, key: &str, value: impl Into<Value>) -> Self {
        self.params.insert(key.to_owned(), value.into());
        self
    }

    #[must_use]
    pub fn with_source(mut self, source: impl Into<BoxedSource>) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    pub fn code(&self) -> &'static str {
        self.code
    }

    pub fn params(&self) -> &Map<String, Value> {
        &self.params
    }
}
