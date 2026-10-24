use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub code: String,
    pub message: String,
    pub details: Option<Value>,
    pub request_id: Option<String>,
}

#[derive(Debug)]
pub enum AppError {
    DatabaseError(sqlx::Error),
    NotFound(String),
    Unauthorized(String),
    InvalidCredentials,
    Forbidden(String),
    Conflict(String),
    TooManyRequests(String),
    BadRequest(String),
    InternalError(String),
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            AppError::DatabaseError(_) => "DATABASE_ERROR",
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::Unauthorized(_) => "UNAUTHORIZED",
            AppError::InvalidCredentials => "INVALID_CREDENTIALS",
            AppError::Forbidden(_) => "FORBIDDEN",
            AppError::Conflict(_) => "CONFLICT",
            AppError::TooManyRequests(_) => "RATE_LIMITED",
            AppError::BadRequest(_) => "VALIDATION_ERROR",
            AppError::InternalError(_) => "INTERNAL_ERROR",
        }
    }

    fn public_message(&self) -> String {
        match self {
            AppError::DatabaseError(_) => {
                "An error occurred while processing your request".to_string()
            }
            AppError::InvalidCredentials => "Invalid credentials".to_string(),
            AppError::InternalError(_) => "An internal error occurred".to_string(),
            AppError::NotFound(message)
            | AppError::Unauthorized(message)
            | AppError::Forbidden(message)
            | AppError::Conflict(message)
            | AppError::TooManyRequests(message)
            | AppError::BadRequest(message) => message.clone(),
        }
    }

    pub fn response_body(&self) -> ApiErrorResponse {
        ApiErrorResponse {
            code: self.code().to_string(),
            message: self.public_message(),
            details: None,
            request_id: None,
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::DatabaseError(error) => write!(f, "Database error: {error}"),
            AppError::NotFound(message) => write!(f, "Not found: {message}"),
            AppError::Unauthorized(message) => write!(f, "Unauthorized: {message}"),
            AppError::InvalidCredentials => write!(f, "Invalid credentials"),
            AppError::Forbidden(message) => write!(f, "Forbidden: {message}"),
            AppError::Conflict(message) => write!(f, "Conflict: {message}"),
            AppError::TooManyRequests(message) => write!(f, "Rate limited: {message}"),
            AppError::BadRequest(message) => write!(f, "Bad request: {message}"),
            AppError::InternalError(message) => write!(f, "Internal error: {message}"),
        }
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::DatabaseError(_) | AppError::InternalError(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Unauthorized(_) | AppError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::TooManyRequests(_) => StatusCode::TOO_MANY_REQUESTS,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::DatabaseError(error) => log::error!("Database error: {error:?}"),
            AppError::InternalError(message) => log::error!("Internal error: {message}"),
            _ => log::info!("API error {}: {}", self.code(), self),
        }

        HttpResponse::build(self.status_code()).json(self.response_body())
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        AppError::DatabaseError(error)
    }
}

impl std::error::Error for AppError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variants_map_to_stable_statuses_and_codes() {
        for (error, status, code) in [
            (
                AppError::NotFound("missing".to_string()),
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
            ),
            (
                AppError::Unauthorized("token missing".to_string()),
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
            ),
            (
                AppError::Forbidden("role required".to_string()),
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
            ),
            (
                AppError::Conflict("duplicate".to_string()),
                StatusCode::CONFLICT,
                "CONFLICT",
            ),
            (
                AppError::TooManyRequests("slow down".to_string()),
                StatusCode::TOO_MANY_REQUESTS,
                "RATE_LIMITED",
            ),
            (
                AppError::BadRequest("invalid".to_string()),
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
            ),
        ] {
            assert_eq!(error.status_code(), status);
            assert_eq!(error.code(), code);
        }
    }

    #[test]
    fn internal_details_are_not_exposed() {
        let error = AppError::InternalError("secret diagnostic".to_string());
        let body = error.response_body();

        assert_eq!(body.code, "INTERNAL_ERROR");
        assert_eq!(body.message, "An internal error occurred");
        assert!(!body.message.contains("secret"));
    }
}
