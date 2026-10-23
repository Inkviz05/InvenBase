use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::auth::{AuthService, Claims};
use crate::errors::AppError;
use crate::models::{CreateNotificationRequest, Notification};
use crate::services::notification_outbox;

pub async fn get_notifications(
    state: web::Data<AppState>,
    claims: Claims,
) -> Result<HttpResponse, AppError> {
    let user_id = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

    let notifications: Vec<Notification> = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT id, user_id, title, message, notification_type, is_read, created_at
         FROM notifications WHERE user_id = $1
         ORDER BY created_at DESC
         LIMIT 50",
    )
    .bind(user_id)
    .fetch_all(&state.db.pool)
    .await?;

    Ok(HttpResponse::Ok().json(notifications))
}

pub async fn get_unread_count(
    state: web::Data<AppState>,
    claims: Claims,
) -> Result<HttpResponse, AppError> {
    let user_id = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

    let count: (i64,) = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND is_read = FALSE",
    )
    .bind(user_id)
    .fetch_one(&state.db.pool)
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "count": count.0 })))
}

pub async fn mark_as_read(
    state: web::Data<AppState>,
    claims: Claims,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let notification_id = path.into_inner();
    let user_id = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

    sqlx::query::<sqlx::Postgres>(
        "UPDATE notifications SET is_read = TRUE WHERE id = $1 AND user_id = $2",
    )
    .bind(notification_id)
    .bind(user_id)
    .execute(&state.db.pool)
    .await?;

    Ok(HttpResponse::NoContent().finish())
}

pub async fn mark_all_as_read(
    state: web::Data<AppState>,
    claims: Claims,
) -> Result<HttpResponse, AppError> {
    let user_id = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

    sqlx::query::<sqlx::Postgres>("UPDATE notifications SET is_read = TRUE WHERE user_id = $1")
        .bind(user_id)
        .execute(&state.db.pool)
        .await?;

    Ok(HttpResponse::NoContent().finish())
}

pub async fn create_notification(
    state: web::Data<AppState>,
    claims: Claims,
    req: web::Json<CreateNotificationRequest>,
) -> Result<HttpResponse, AppError> {
    AuthService::require_any_role(&claims, &["admin", "responsible"])?;

    let mut data = serde_json::Map::new();
    data.insert(
        "type".to_string(),
        serde_json::Value::String(req.notification_type.clone()),
    );
    let notification_id = notification_outbox::create_notification_with_push(
        &state.db.pool,
        req.user_id,
        &req.title,
        &req.message,
        &req.notification_type,
        data,
        &[],
    )
    .await?;

    let notification: Notification = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT id, user_id, title, message, notification_type, is_read, created_at
         FROM notifications WHERE id = $1",
    )
    .bind(notification_id)
    .fetch_one(&state.db.pool)
    .await?;

    Ok(HttpResponse::Created().json(notification))
}
