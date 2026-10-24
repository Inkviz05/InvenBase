use actix_web::{web, HttpRequest, HttpResponse};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration as StdDuration, Instant};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::auth::Claims;
use crate::errors::AppError;

const API_PROTOCOL_VERSION: u16 = 1;
const SETUP_CODE_LENGTH: usize = 24;
const DEFAULT_TTL_MINUTES: i64 = 10;
const MAX_TTL_MINUTES: i64 = 60;
const PAIR_RATE_LIMIT: usize = 10;
const PAIR_RATE_WINDOW: StdDuration = StdDuration::from_secs(60);

static PAIR_ATTEMPTS: OnceLock<Mutex<HashMap<String, VecDeque<Instant>>>> = OnceLock::new();

#[derive(Debug, Serialize)]
pub struct InstanceResponse {
    pub product: &'static str,
    pub instance_id: Uuid,
    pub instance_name: String,
    pub api_protocol_version: u16,
    pub api_base_path: &'static str,
}

#[derive(Debug, Deserialize)]
pub struct CreateSetupCodeRequest {
    pub expires_in_minutes: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct CreateSetupCodeResponse {
    pub id: Uuid,
    pub setup_code: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct PairInstanceRequest {
    pub setup_code: String,
}

#[derive(Debug, Serialize)]
pub struct PairInstanceResponse {
    pub instance: InstanceResponse,
    pub paired_at: DateTime<Utc>,
}

async fn instance_response(state: &AppState) -> Result<InstanceResponse, AppError> {
    let instance_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT instance_id FROM app_instance WHERE singleton = TRUE",
    )
    .fetch_one(&state.db.pool)
    .await?;

    Ok(InstanceResponse {
        product: "InvenBase",
        instance_id,
        instance_name: state.config.instance_name.clone(),
        api_protocol_version: API_PROTOCOL_VERSION,
        api_base_path: "/api",
    })
}

pub async fn get_instance(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(instance_response(&state).await?))
}

pub async fn create_setup_code(
    state: web::Data<AppState>,
    claims: &Claims,
    request: web::Json<CreateSetupCodeRequest>,
) -> Result<HttpResponse, AppError> {
    let ttl_minutes = request.expires_in_minutes.unwrap_or(DEFAULT_TTL_MINUTES);
    if !(1..=MAX_TTL_MINUTES).contains(&ttl_minutes) {
        return Err(AppError::BadRequest(format!(
            "expires_in_minutes must be between 1 and {MAX_TTL_MINUTES}"
        )));
    }

    let created_by = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid token subject".to_string()))?;
    let normalized_code = Uuid::new_v4()
        .simple()
        .to_string()
        .chars()
        .take(SETUP_CODE_LENGTH)
        .collect::<String>()
        .to_uppercase();
    let setup_code = format_setup_code(&normalized_code);
    let expires_at = Utc::now() + Duration::minutes(ttl_minutes);

    sqlx::query("DELETE FROM mobile_setup_codes WHERE expires_at < CURRENT_TIMESTAMP")
        .execute(&state.db.pool)
        .await?;
    let id = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO mobile_setup_codes (code_hash, created_by, expires_at)
         VALUES (digest($1, 'sha256'), $2, $3)
         RETURNING id",
    )
    .bind(&normalized_code)
    .bind(created_by)
    .bind(expires_at)
    .fetch_one(&state.db.pool)
    .await?;

    Ok(HttpResponse::Created().json(CreateSetupCodeResponse {
        id,
        setup_code,
        expires_at,
    }))
}

pub async fn revoke_setup_code(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let deleted = sqlx::query_scalar::<_, Uuid>(
        "DELETE FROM mobile_setup_codes
         WHERE id = $1 AND used_at IS NULL
         RETURNING id",
    )
    .bind(path.into_inner())
    .fetch_optional(&state.db.pool)
    .await?;

    if deleted.is_none() {
        return Err(AppError::NotFound(
            "Active setup code was not found".to_string(),
        ));
    }

    Ok(HttpResponse::NoContent().finish())
}

pub async fn pair_instance(
    state: web::Data<AppState>,
    http_request: HttpRequest,
    request: web::Json<PairInstanceRequest>,
) -> Result<HttpResponse, AppError> {
    enforce_pair_rate_limit(&http_request)?;
    let normalized_code = normalize_setup_code(&request.setup_code)?;
    let paired_at = Utc::now();
    let consumed = sqlx::query_scalar::<_, Uuid>(
        "UPDATE mobile_setup_codes
         SET used_at = $2
         WHERE code_hash = digest($1, 'sha256')
           AND used_at IS NULL
           AND expires_at > $2
         RETURNING id",
    )
    .bind(normalized_code)
    .bind(paired_at)
    .fetch_optional(&state.db.pool)
    .await?;

    if consumed.is_none() {
        return Err(AppError::Unauthorized(
            "Setup code is invalid, expired, or already used".to_string(),
        ));
    }

    Ok(HttpResponse::Ok().json(PairInstanceResponse {
        instance: instance_response(&state).await?,
        paired_at,
    }))
}

fn enforce_pair_rate_limit(request: &HttpRequest) -> Result<(), AppError> {
    let source = request
        .connection_info()
        .realip_remote_addr()
        .map(str::to_string)
        .or_else(|| request.peer_addr().map(|address| address.ip().to_string()))
        .unwrap_or_else(|| "unknown".to_string());
    let now = Instant::now();
    let attempts = PAIR_ATTEMPTS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut attempts = attempts
        .lock()
        .map_err(|_| AppError::InternalError("Pair rate limiter lock poisoned".to_string()))?;
    attempts.retain(|_, entries| {
        while entries
            .front()
            .is_some_and(|at| now.duration_since(*at) >= PAIR_RATE_WINDOW)
        {
            entries.pop_front();
        }
        !entries.is_empty()
    });
    let entries = attempts.entry(source).or_default();
    if entries.len() >= PAIR_RATE_LIMIT {
        return Err(AppError::TooManyRequests(
            "Too many pairing attempts; retry in one minute".to_string(),
        ));
    }
    entries.push_back(now);
    Ok(())
}

fn normalize_setup_code(value: &str) -> Result<String, AppError> {
    let normalized = value
        .chars()
        .filter(|character| !character.is_ascii_whitespace() && *character != '-')
        .collect::<String>()
        .to_uppercase();
    if normalized.len() != SETUP_CODE_LENGTH
        || !normalized
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return Err(AppError::BadRequest(
            "setup_code must contain 24 hexadecimal characters".to_string(),
        ));
    }
    Ok(normalized)
}

fn format_setup_code(value: &str) -> String {
    value
        .as_bytes()
        .chunks(4)
        .map(|chunk| std::str::from_utf8(chunk).expect("setup code is ASCII"))
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setup_code_normalization_accepts_display_format() {
        assert_eq!(
            normalize_setup_code("abcd-1234-EF00-5678-abcd-9999").unwrap(),
            "ABCD1234EF005678ABCD9999"
        );
    }

    #[test]
    fn setup_code_normalization_rejects_wrong_shape() {
        assert!(normalize_setup_code("short").is_err());
        assert!(normalize_setup_code("ZZZZ-1234-EF00-5678-ABCD-9999").is_err());
    }
}
