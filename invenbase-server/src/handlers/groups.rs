use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::models::{EquipmentGroup, CreateEquipmentGroupRequest};
use crate::auth::{AuthService, Claims};
use crate::errors::AppError;
use crate::app_state::AppState;

pub async fn create_group(
    state: web::Data<AppState>,
    claims: Claims,
    req: web::Json<CreateEquipmentGroupRequest>,
) -> Result<HttpResponse, AppError> {
    AuthService::require_any_role(&claims, &["admin", "responsible"])?;

    let group_id = uuid::Uuid::new_v4();

    sqlx::query::<sqlx::Postgres>(
        "INSERT INTO equipment_groups (id, name, description)
         VALUES ($1, $2, $3)"
    )
    .bind(group_id)
    .bind(&req.name)
    .bind(&req.description)
    .execute(&state.db.pool)
    .await?;

    // Добавляем оборудование в группу
    for item in &req.equipment_items {
        sqlx::query::<sqlx::Postgres>(
            "INSERT INTO equipment_group_items (group_id, equipment_id, quantity)
             VALUES ($1, $2, $3)"
        )
        .bind(group_id)
        .bind(item.equipment_id)
        .bind(item.quantity)
        .execute(&state.db.pool)
        .await?;
    }

    let group: EquipmentGroup = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT id, name, description, created_at FROM equipment_groups WHERE id = $1"
    )
    .bind(group_id)
    .fetch_one(&state.db.pool)
    .await?;

    Ok(HttpResponse::Created().json(group))
}

pub async fn get_groups(
    state: web::Data<AppState>,
    _claims: Claims,
) -> Result<HttpResponse, AppError> {
    let groups: Vec<EquipmentGroup> = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT id, name, description, created_at FROM equipment_groups ORDER BY created_at DESC"
    )
    .fetch_all(&state.db.pool)
    .await?;

    Ok(HttpResponse::Ok().json(groups))
}

pub async fn get_group(
    state: web::Data<AppState>,
    _claims: Claims,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let group_id = path.into_inner();

    let group: Option<EquipmentGroup> = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT id, name, description, created_at FROM equipment_groups WHERE id = $1"
    )
    .bind(group_id)
    .fetch_optional(&state.db.pool)
    .await?;

    let group = group.ok_or_else(|| AppError::NotFound("Group not found".to_string()))?;

    Ok(HttpResponse::Ok().json(group))
}

pub async fn update_group(
    state: web::Data<AppState>,
    claims: Claims,
    path: web::Path<Uuid>,
    req: web::Json<CreateEquipmentGroupRequest>,
) -> Result<HttpResponse, AppError> {
    AuthService::require_any_role(&claims, &["admin", "responsible"])?;

    let group_id = path.into_inner();

    // Проверяем, что группа существует
    let exists: Option<(bool,)> = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT EXISTS(SELECT 1 FROM equipment_groups WHERE id = $1)"
    )
    .bind(group_id)
    .fetch_optional(&state.db.pool)
    .await?;

    if exists.is_none() || !exists.unwrap().0 {
        return Err(AppError::NotFound("Group not found".to_string()));
    }

    // Обновляем название и описание группы
    sqlx::query::<sqlx::Postgres>(
        "UPDATE equipment_groups SET name = $1, description = $2 WHERE id = $3"
    )
    .bind(&req.name)
    .bind(&req.description)
    .bind(group_id)
    .execute(&state.db.pool)
    .await?;

    // Удаляем старые элементы группы
    sqlx::query::<sqlx::Postgres>(
        "DELETE FROM equipment_group_items WHERE group_id = $1"
    )
    .bind(group_id)
    .execute(&state.db.pool)
    .await?;

    // Добавляем новые элементы группы
    for item in &req.equipment_items {
        sqlx::query::<sqlx::Postgres>(
            "INSERT INTO equipment_group_items (group_id, equipment_id, quantity)
             VALUES ($1, $2, $3)"
        )
        .bind(group_id)
        .bind(item.equipment_id)
        .bind(item.quantity)
        .execute(&state.db.pool)
        .await?;
    }

    let group: EquipmentGroup = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT id, name, description, created_at FROM equipment_groups WHERE id = $1"
    )
    .bind(group_id)
    .fetch_one(&state.db.pool)
    .await?;

    Ok(HttpResponse::Ok().json(group))
}

pub async fn delete_group(
    state: web::Data<AppState>,
    claims: Claims,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    AuthService::require_any_role(&claims, &["admin", "responsible"])?;

    let group_id = path.into_inner();

    let result = sqlx::query::<sqlx::Postgres>("DELETE FROM equipment_groups WHERE id = $1")
        .bind(group_id)
        .execute(&state.db.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Group not found".to_string()));
    }

    Ok(HttpResponse::NoContent().finish())
}

