use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::models::{
    Equipment, EquipmentWithDetails, CreateEquipmentRequest, UpdateEquipmentRequest,
};
use crate::auth::{AuthService, Claims};
use crate::errors::AppError;
use crate::app_state::AppState;

pub async fn create_equipment(
    state: web::Data<AppState>,
    claims: Claims,
    req: web::Json<CreateEquipmentRequest>,
) -> Result<HttpResponse, AppError> {
    AuthService::require_any_role(&claims, &["admin", "responsible"])?;

    let equipment_id = uuid::Uuid::new_v4();
    let qr_code = format!("EQ-{}", equipment_id.to_string().replace("-", "").chars().take(12).collect::<String>());

    let (quantity, is_unique) = if req.is_unique {
        (1i32, true)
    } else {
        (req.quantity.unwrap_or(1), false)
    };

    sqlx::query::<sqlx::Postgres>(
        "INSERT INTO equipment (id, name, description, category_id, quantity, available_quantity, is_unique, location, qr_code, responsible_user_id, status)
         VALUES ($1, $2, $3, $4, $5, $5, $6, $7, $8, $9, 'available')"
    )
    .bind(equipment_id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.category_id)
    .bind(quantity)
    .bind(is_unique)
    .bind(&req.location)
    .bind(&qr_code)
    .bind(&req.responsible_user_id)
    .execute(&state.db.pool)
    .await?;

    // Логирование
    let _ = sqlx::query::<sqlx::Postgres>(
        "INSERT INTO activity_logs (user_id, action, entity_type, entity_id, details)
         VALUES ($1, 'create_equipment', 'equipment', $2, $3)"
    )
    .bind(uuid::Uuid::parse_str(&claims.sub).unwrap())
    .bind(equipment_id)
    .bind(serde_json::json!({"name": req.name}))
    .execute(&state.db.pool)
    .await;

    let equipment: Equipment = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT id, name, description, category_id, quantity, available_quantity, COALESCE(is_unique, false) as is_unique, location, qr_code, responsible_user_id, status, created_at, updated_at 
         FROM equipment WHERE id = $1"
    )
    .bind(equipment_id)
    .fetch_one(&state.db.pool)
    .await?;

    Ok(HttpResponse::Created().json(equipment))
}

pub async fn get_equipment_list(
    state: web::Data<AppState>,
    _claims: Claims,
) -> Result<HttpResponse, AppError> {
    let equipment: Vec<EquipmentWithDetails> = sqlx::query_as::<sqlx::Postgres, _>(
        r#"
        SELECT 
            e.id, e.name, e.description, e.category_id, 
            c.name as category_name,
            e.quantity, e.available_quantity, COALESCE(e.is_unique, false) as is_unique, e.location, e.qr_code, 
            e.responsible_user_id, u.full_name as responsible_name,
            e.status, e.created_at, e.updated_at
        FROM equipment e
        LEFT JOIN equipment_categories c ON e.category_id = c.id
        LEFT JOIN users u ON e.responsible_user_id = u.id
        ORDER BY e.created_at DESC
        "#
    )
    .fetch_all(&state.db.pool)
    .await?;

    Ok(HttpResponse::Ok().json(equipment))
}

pub async fn get_equipment(
    state: web::Data<AppState>,
    _claims: Claims,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let equipment_id = path.into_inner();

    let equipment: Option<EquipmentWithDetails> = sqlx::query_as::<sqlx::Postgres, _>(
        r#"
        SELECT 
            e.id, e.name, e.description, e.category_id, 
            c.name as category_name,
            e.quantity, e.available_quantity, COALESCE(e.is_unique, false) as is_unique, e.location, e.qr_code, 
            e.responsible_user_id, u.full_name as responsible_name,
            e.status, e.created_at, e.updated_at
        FROM equipment e
        LEFT JOIN equipment_categories c ON e.category_id = c.id
        LEFT JOIN users u ON e.responsible_user_id = u.id
        WHERE e.id = $1
        "#
    )
    .bind(equipment_id)
    .fetch_optional(&state.db.pool)
    .await?;

    let equipment = equipment.ok_or_else(|| AppError::NotFound("Equipment not found".to_string()))?;

    Ok(HttpResponse::Ok().json(equipment))
}

pub async fn get_equipment_by_qr(
    state: web::Data<AppState>,
    _claims: Claims,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let qr_code = path.into_inner();

    let equipment: Option<EquipmentWithDetails> = sqlx::query_as::<sqlx::Postgres, _>(
        r#"
        SELECT 
            e.id, e.name, e.description, e.category_id, 
            c.name as category_name,
            e.quantity, e.available_quantity, COALESCE(e.is_unique, false) as is_unique, e.location, e.qr_code, 
            e.responsible_user_id, u.full_name as responsible_name,
            e.status, e.created_at, e.updated_at
        FROM equipment e
        LEFT JOIN equipment_categories c ON e.category_id = c.id
        LEFT JOIN users u ON e.responsible_user_id = u.id
        WHERE e.qr_code = $1
        "#
    )
    .bind(&qr_code)
    .fetch_optional(&state.db.pool)
    .await?;

    let equipment = equipment.ok_or_else(|| AppError::NotFound("Equipment not found".to_string()))?;

    Ok(HttpResponse::Ok().json(equipment))
}

pub async fn update_equipment(
    state: web::Data<AppState>,
    claims: Claims,
    path: web::Path<Uuid>,
    req: web::Json<UpdateEquipmentRequest>,
) -> Result<HttpResponse, AppError> {
    AuthService::require_any_role(&claims, &["admin", "responsible"])?;

    let equipment_id = path.into_inner();

    // Проверяем, что оборудование существует
    let exists: Option<(bool,)> = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT EXISTS(SELECT 1 FROM equipment WHERE id = $1)"
    )
    .bind(equipment_id)
    .fetch_optional(&state.db.pool)
    .await?;

    if exists.is_none() || !exists.unwrap().0 {
        return Err(AppError::NotFound("Equipment not found".to_string()));
    }

    // Обновляем поля
    if let Some(name) = &req.name {
        sqlx::query::<sqlx::Postgres>("UPDATE equipment SET name = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2")
            .bind(name)
            .bind(equipment_id)
            .execute(&state.db.pool)
            .await?;
    }

    if req.description.is_some() {
        sqlx::query::<sqlx::Postgres>("UPDATE equipment SET description = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2")
            .bind(&req.description)
            .bind(equipment_id)
            .execute(&state.db.pool)
            .await?;
    }

    if req.category_id.is_some() {
        sqlx::query::<sqlx::Postgres>("UPDATE equipment SET category_id = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2")
            .bind(&req.category_id)
            .bind(equipment_id)
            .execute(&state.db.pool)
            .await?;
    }

    // Уникальное оборудование: всегда 1 экземпляр
    if req.is_unique == Some(true) {
        sqlx::query::<sqlx::Postgres>("UPDATE equipment SET quantity = 1, available_quantity = 1, is_unique = true, updated_at = CURRENT_TIMESTAMP WHERE id = $1")
            .bind(equipment_id)
            .execute(&state.db.pool)
            .await?;
    } else {
        if let Some(quantity) = req.quantity {
            sqlx::query::<sqlx::Postgres>("UPDATE equipment SET quantity = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2")
                .bind(quantity)
                .bind(equipment_id)
                .execute(&state.db.pool)
                .await?;
        }
        if let Some(available_quantity) = req.available_quantity {
            sqlx::query::<sqlx::Postgres>("UPDATE equipment SET available_quantity = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2")
                .bind(available_quantity)
                .bind(equipment_id)
                .execute(&state.db.pool)
                .await?;
        }
        if req.is_unique.is_some() {
            sqlx::query::<sqlx::Postgres>("UPDATE equipment SET is_unique = false, updated_at = CURRENT_TIMESTAMP WHERE id = $1")
                .bind(equipment_id)
                .execute(&state.db.pool)
                .await?;
        }
    }

    if req.location.is_some() {
        sqlx::query::<sqlx::Postgres>("UPDATE equipment SET location = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2")
            .bind(&req.location)
            .bind(equipment_id)
            .execute(&state.db.pool)
            .await?;
    }

    if req.responsible_user_id.is_some() {
        sqlx::query::<sqlx::Postgres>("UPDATE equipment SET responsible_user_id = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2")
            .bind(&req.responsible_user_id)
            .bind(equipment_id)
            .execute(&state.db.pool)
            .await?;
    }

    if let Some(status) = &req.status {
        sqlx::query::<sqlx::Postgres>("UPDATE equipment SET status = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2")
            .bind(status)
            .bind(equipment_id)
            .execute(&state.db.pool)
            .await?;
    }

    // Получаем обновлённое оборудование из БД
    let equipment: Equipment = sqlx::query_as::<sqlx::Postgres, Equipment>(
        "SELECT id, name, description, category_id, quantity, available_quantity, COALESCE(is_unique, false) as is_unique, location, qr_code, responsible_user_id, status, created_at, updated_at 
         FROM equipment WHERE id = $1"
    )
    .bind(equipment_id)
    .fetch_one(&state.db.pool)
    .await?;

    // Логирование
    let _ = sqlx::query::<sqlx::Postgres>(
        "INSERT INTO activity_logs (user_id, action, entity_type, entity_id, details)
         VALUES ($1, 'update_equipment', 'equipment', $2, $3)"
    )
    .bind(uuid::Uuid::parse_str(&claims.sub).unwrap())
    .bind(equipment_id)
    .bind(serde_json::json!({"updated": true}))
    .execute(&state.db.pool)
    .await;

    Ok(HttpResponse::Ok().json(equipment))
}

pub async fn delete_equipment(
    state: web::Data<AppState>,
    claims: Claims,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    AuthService::require_any_role(&claims, &["admin", "responsible"])?;

    let equipment_id = path.into_inner();

    let result = sqlx::query::<sqlx::Postgres>("DELETE FROM equipment WHERE id = $1")
        .bind(equipment_id)
        .execute(&state.db.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Equipment not found".to_string()));
    }

    // Логирование
    let _ = sqlx::query::<sqlx::Postgres>(
        "INSERT INTO activity_logs (user_id, action, entity_type, entity_id)
         VALUES ($1, 'delete_equipment', 'equipment', $2)"
    )
    .bind(uuid::Uuid::parse_str(&claims.sub).unwrap())
    .bind(equipment_id)
    .execute(&state.db.pool)
    .await;

    Ok(HttpResponse::NoContent().finish())
}

