use axum::http;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Json;
use jwt_simple::reexports::serde_json::json;
use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;
use utoipa::ToSchema;
#[derive(Debug, ToSchema, Serialize, Deserialize)]
pub struct ErrorOutput {
    pub error: String,
}
impl ErrorOutput {
    pub fn new(error: &str) -> Self {
        Self {
            error: error.to_string(),
        }
    }
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Email already exists: {0}")]
    EmailAlreadyExists(String),
    #[error("sqlx error: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("password hashing error: {0}")]
    PasswordHashError(#[from] argon2::password_hash::Error),
    #[error("general error: {0}")]
    AnyError(#[from] anyhow::Error),
    #[error("http header parse error: {0}")]
    HttpHeaderError(#[from] http::header::InvalidHeaderValue),
    #[error("create chat error: {0}")]
    CreateChatError(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Update chat error: {0}")]
    UpdateChatError(String),
    #[error("{0}")]
    ChatFileError(String),
    #[error("create message error: {0}")]
    CreateMessageError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response<axum::body::Body> {
        let status = match self {
            AppError::SqlxError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::PasswordHashError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::AnyError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::EmailAlreadyExists(_) => StatusCode::CONFLICT,
            AppError::HttpHeaderError(_) => StatusCode::BAD_REQUEST,
            Self::CreateChatError(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::IoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::UpdateChatError(_) => StatusCode::BAD_REQUEST,
            Self::ChatFileError(_) => StatusCode::BAD_REQUEST,
            Self::CreateMessageError(_) => StatusCode::BAD_REQUEST,
        };

        (status, Json(json!({ "error": self.to_string() }))).into_response()
    }
}
