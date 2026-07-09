use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::auth::{AuthService, Claims};
use crate::errors::AppError;
use crate::models::{CreatePermissionRequest, Permission};

pub async fn create_permission(
    state: web::Data<AppState>,
    claims: Claims,
    req: web::Json<CreatePermissionRequest>,
) -> Result<HttpResponse, AppError> {
    AuthService::require_any_role(&claims, &["admin", "responsible"])?;

    let issued_by = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

    let permission_id = uuid::Uuid::new_v4();

    sqlx::query::<sqlx::Postgres>(
        "INSERT INTO permissions (id, booking_id, equipment_id, permission_type, issued_by, expires_at, status)
         VALUES ($1, $2, $3, $4, $5, $6, 'active')"
    )
    .bind(permission_id)
    .bind(req.booking_id)
    .bind(&req.equipment_id)
    .bind(&req.permission_type)
    .bind(issued_by)
    .bind(&req.expires_at)
    .execute(&state.db.pool)
    .await?;

    let permission: Permission = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT id, booking_id, equipment_id, permission_type, issued_by, issued_at, expires_at, status
         FROM permissions WHERE id = $1"
    )
    .bind(permission_id)
    .fetch_one(&state.db.pool)
    .await?;

    Ok(HttpResponse::Created().json(permission))
}

pub async fn get_permissions(
    state: web::Data<AppState>,
    claims: Claims,
) -> Result<HttpResponse, AppError> {
    let user_id = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

    let permissions: Vec<Permission> = if claims.role == "admin" || claims.role == "responsible" {
        sqlx::query_as::<sqlx::Postgres, _>(
            "SELECT id, booking_id, equipment_id, permission_type, issued_by, issued_at, expires_at, status
             FROM permissions ORDER BY issued_at DESC"
        )
        .fetch_all(&state.db.pool)
        .await?
    } else {
        // Пользователи видят только свои разрешения через бронирования
        sqlx::query_as::<sqlx::Postgres, _>(
            r#"
            SELECT p.id, p.booking_id, p.equipment_id, p.permission_type, p.issued_by, p.issued_at, p.expires_at, p.status
            FROM permissions p
            JOIN bookings b ON p.booking_id = b.id
            WHERE b.user_id = $1
            ORDER BY p.issued_at DESC
            "#
        )
        .bind(user_id)
        .fetch_all(&state.db.pool)
        .await?
    };

    Ok(HttpResponse::Ok().json(permissions))
}

pub async fn revoke_permission(
    state: web::Data<AppState>,
    claims: Claims,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    AuthService::require_any_role(&claims, &["admin", "responsible"])?;

    let permission_id = path.into_inner();

    sqlx::query::<sqlx::Postgres>("UPDATE permissions SET status = 'revoked' WHERE id = $1")
        .bind(permission_id)
        .execute(&state.db.pool)
        .await?;

    Ok(HttpResponse::NoContent().finish())
}
