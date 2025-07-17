use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum ApiError {
    ValidationError(String),
    DatabaseError(String),
    AuthenticationError(String),
    UserNotFound,
    UsernameAlreadyExists,
    EmailAlreadyExists,
    InvalidCredentials,
    InternalServerError,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::AuthenticationError(msg) => (StatusCode::UNAUTHORIZED, msg),
            ApiError::UserNotFound => (StatusCode::NOT_FOUND, "User not found".to_string()),
            ApiError::UsernameAlreadyExists => (StatusCode::CONFLICT, "Username already exists".to_string()),
            ApiError::EmailAlreadyExists => (StatusCode::CONFLICT, "Email already exists".to_string()),
            ApiError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()),
            ApiError::InternalServerError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}

impl From<crate::models::DatabaseError> for ApiError {
    fn from(error: crate::models::DatabaseError) -> Self {
        match error {
            crate::models::DatabaseError::UsernameAlreadyExists => ApiError::UsernameAlreadyExists,
            crate::models::DatabaseError::EmailAlreadyExists => ApiError::EmailAlreadyExists,
            crate::models::DatabaseError::UserNotFound => ApiError::UserNotFound,
            crate::models::DatabaseError::DatabaseOperationFailed(msg) => ApiError::DatabaseError(msg),
            
            // Map validation errors to ValidationError with their display message
            crate::models::DatabaseError::UsernameTooShort => ApiError::ValidationError(error.to_string()),
            crate::models::DatabaseError::PasswordTooWeak => ApiError::ValidationError(error.to_string()),
            crate::models::DatabaseError::InvalidEmail => ApiError::ValidationError(error.to_string()),
            
            // Map query errors appropriately
            crate::models::DatabaseError::QueryNotFound => ApiError::UserNotFound, // or create new variant
            crate::models::DatabaseError::CohortNotFound => ApiError::UserNotFound, // or create new variant
        }
    }
}

pub type ApiResult<T> = Result<T, ApiError>;