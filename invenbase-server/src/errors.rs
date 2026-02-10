use actix_web::{HttpResponse, ResponseError};
use serde_json::json;
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    DatabaseError(sqlx::Error),
    NotFound(String),
    Unauthorized(String),
    BadRequest(String),
    InternalError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::DatabaseError(e) => write!(f, "Database error: {}", e),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            AppError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            AppError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        log::error!("AppError occurred: {:?}", self);
        match self {
            AppError::DatabaseError(e) => {
                log::error!("Database error details: {:?}", e);
                HttpResponse::InternalServerError().json(json!({
                    "error": "Database error",
                    "message": "An error occurred while processing your request"
                }))
            }
            AppError::NotFound(msg) => {
                HttpResponse::NotFound().json(json!({
                    "error": "Not found",
                    "message": msg
                }))
            }
            AppError::Unauthorized(msg) => {
                HttpResponse::Unauthorized().json(json!({
                    "error": "Unauthorized",
                    "message": msg
                }))
            }
            AppError::BadRequest(msg) => {
                HttpResponse::BadRequest().json(json!({
                    "error": "Bad request",
                    "message": msg
                }))
            }
            AppError::InternalError(msg) => {
                log::error!("Internal error: {}", msg);
                HttpResponse::InternalServerError().json(json!({
                    "error": "Internal error",
                    "message": msg
                }))
            }
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::DatabaseError(err)
    }
}

impl std::error::Error for AppError {}

