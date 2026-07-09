use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::auth::{AuthService, Claims};
use crate::errors::AppError;
use crate::models::ActivityLog;

pub async fn get_logs(
    state: web::Data<AppState>,
    claims: Claims,
    query: web::Query<LogQuery>,
) -> Result<HttpResponse, AppError> {
    AuthService::require_any_role(&claims, &["admin", "responsible"])?;

    let limit = query.limit.unwrap_or(100).min(1000);
    let offset = query.offset.unwrap_or(0);

    let logs: Vec<ActivityLog> = if let Some(user_id) = query.user_id {
        sqlx::query_as::<sqlx::Postgres, _>(
            "SELECT id, user_id, action, entity_type, entity_id, details, created_at
             FROM activity_logs
             WHERE user_id = $1
             ORDER BY created_at DESC
             LIMIT $2 OFFSET $3",
        )
        .bind(user_id)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&state.db.pool)
        .await?
    } else if let Some(entity_type) = &query.entity_type {
        sqlx::query_as::<sqlx::Postgres, _>(
            "SELECT id, user_id, action, entity_type, entity_id, details, created_at
             FROM activity_logs
             WHERE entity_type = $1
             ORDER BY created_at DESC
             LIMIT $2 OFFSET $3",
        )
        .bind(entity_type)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&state.db.pool)
        .await?
    } else {
        sqlx::query_as::<sqlx::Postgres, _>(
            "SELECT id, user_id, action, entity_type, entity_id, details, created_at
             FROM activity_logs
             ORDER BY created_at DESC
             LIMIT $1 OFFSET $2",
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&state.db.pool)
        .await?
    };

    Ok(HttpResponse::Ok().json(logs))
}

#[derive(serde::Deserialize)]
pub struct LogQuery {
    pub user_id: Option<Uuid>,
    pub entity_type: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}
