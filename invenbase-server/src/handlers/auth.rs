use actix_web::{web, HttpResponse};

use crate::app_state::AppState;
use crate::auth::{AuthService, Claims};
use crate::errors::AppError;
use crate::models::{LoginRequest, LoginResponse, User, UserResponse};

pub async fn login(
    state: web::Data<AppState>,
    req: web::Json<LoginRequest>,
) -> Result<HttpResponse, AppError> {
    log::info!("Login attempt for username: {}", req.username);

    let user: Option<User> = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT id, username, password_hash, email, full_name, role, created_at, updated_at
         FROM users WHERE username = $1",
    )
    .bind(&req.username)
    .fetch_optional(&state.db.pool)
    .await
    .map_err(|e| {
        log::error!("Database error during login: {:?}", e);
        AppError::DatabaseError(e)
    })?;

    let user = user.ok_or_else(|| {
        log::info!("Login failed: user not found - {}", req.username);
        AppError::Unauthorized("Invalid credentials".to_string())
    })?;

    log::info!("User found: {} (role: {})", user.username, user.role);

    // Проверяем, что JWT_SECRET не пустой
    if state.config.jwt_secret.is_empty() {
        log::error!("JWT_SECRET is empty!");
        return Err(AppError::InternalError(
            "JWT_SECRET is not configured".to_string(),
        ));
    }

    let auth_service = AuthService::new(&state.config.jwt_secret, state.config.jwt_expiration);

    if !AuthService::verify_password(&req.password, &user.password_hash).map_err(|e| {
        log::error!("Password verification error: {:?}", e);
        AppError::InternalError(format!("Password verification failed: {}", e))
    })? {
        log::info!("Login failed: invalid password for user - {}", req.username);
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    let token = auth_service.generate_token(&user).map_err(|e| {
        log::error!("Token generation error: {:?}", e);
        AppError::InternalError(format!("Token generation failed: {}", e))
    })?;

    // Логирование входа
    let _ = sqlx::query::<sqlx::Postgres>(
        "INSERT INTO activity_logs (user_id, action, entity_type, details)
         VALUES ($1, 'login', 'user', $2)",
    )
    .bind(user.id)
    .bind(serde_json::json!({"username": user.username}))
    .execute(&state.db.pool)
    .await;

    Ok(HttpResponse::Ok().json(LoginResponse {
        token,
        user: UserResponse::from(user),
    }))
}

pub async fn get_current_user(
    state: web::Data<AppState>,
    claims: Claims,
) -> Result<HttpResponse, AppError> {
    let user: Option<User> = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT id, username, password_hash, email, full_name, role, created_at, updated_at
         FROM users WHERE id = $1",
    )
    .bind(
        claims
            .sub
            .parse::<uuid::Uuid>()
            .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?,
    )
    .fetch_optional(&state.db.pool)
    .await?;

    let user = user.ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(HttpResponse::Ok().json(UserResponse::from(user)))
}
