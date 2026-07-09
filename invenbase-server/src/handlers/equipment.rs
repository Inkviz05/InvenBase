use actix_web::{web, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::auth::{AuthService, Claims};
use crate::errors::AppError;
use crate::models::{
    CreateEquipmentMovementRequest, CreateEquipmentRequest, Equipment, EquipmentMovement,
    EquipmentWithDetails, UpdateEquipmentRequest,
};

pub async fn create_equipment(
    state: web::Data<AppState>,
    claims: Claims,
    req: web::Json<CreateEquipmentRequest>,
) -> Result<HttpResponse, AppError> {
    AuthService::require_any_role(&claims, &["admin", "responsible"])?;

    let equipment_id = uuid::Uuid::new_v4();
    let qr_code = format!(
        "EQ-{}",
        equipment_id
            .to_string()
            .replace("-", "")
            .chars()
            .take(12)
            .collect::<String>()
    );

    let (quantity, is_unique) = if req.is_unique {
        (1i32, true)
    } else {
        (req.quantity.unwrap_or(1), false)
    };

    sqlx::query::<sqlx::Postgres>(
        "INSERT INTO equipment (id, name, description, category_id, squad_id, quantity, available_quantity, is_unique, location, qr_code, responsible_user_id, status)
         VALUES ($1, $2, $3, $4, $5, $6, $6, $7, $8, $9, $10, 'available')"
    )
    .bind(equipment_id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.category_id)
    .bind(&req.squad_id)
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
         VALUES ($1, 'create_equipment', 'equipment', $2, $3)",
    )
    .bind(uuid::Uuid::parse_str(&claims.sub).unwrap())
    .bind(equipment_id)
    .bind(serde_json::json!({"name": req.name}))
    .execute(&state.db.pool)
    .await;

    let equipment: Equipment = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT id, name, description, category_id, squad_id, quantity, available_quantity, COALESCE(is_unique, false) as is_unique, location, qr_code, responsible_user_id, status, created_at, updated_at
         FROM equipment WHERE id = $1"
    )
    .bind(equipment_id)
    .fetch_one(&state.db.pool)
    .await?;

    Ok(HttpResponse::Created().json(equipment))
}

#[derive(Debug, Deserialize)]
pub struct EquipmentListQuery {
    pub squad_id: Option<Uuid>,
}

pub async fn get_equipment_list(
    state: web::Data<AppState>,
    _claims: Claims,
    query: web::Query<EquipmentListQuery>,
) -> Result<HttpResponse, AppError> {
    let sql = r#"
        SELECT
            e.id, e.name, e.description, e.category_id,
            c.name as category_name,
            e.squad_id, s.name as squad_name,
            e.quantity, e.available_quantity, COALESCE(e.is_unique, false) as is_unique, e.location, e.qr_code,
            e.responsible_user_id, u.full_name as responsible_name,
            e.status, e.created_at, e.updated_at
        FROM equipment e
        LEFT JOIN equipment_categories c ON e.category_id = c.id
        LEFT JOIN squads s ON e.squad_id = s.id
        LEFT JOIN users u ON e.responsible_user_id = u.id
        "#;
    let order = " ORDER BY e.created_at DESC";

    let equipment: Vec<EquipmentWithDetails> = if let Some(squad_id) = query.squad_id {
        sqlx::query_as::<sqlx::Postgres, _>(&format!("{} WHERE e.squad_id = $1{}", sql, order))
            .bind(squad_id)
            .fetch_all(&state.db.pool)
            .await?
    } else {
        sqlx::query_as::<sqlx::Postgres, _>(&format!("{}{}", sql, order))
            .fetch_all(&state.db.pool)
            .await?
    };

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
            e.squad_id, s.name as squad_name,
            e.quantity, e.available_quantity, COALESCE(e.is_unique, false) as is_unique, e.location, e.qr_code,
            e.responsible_user_id, u.full_name as responsible_name,
            e.status, e.created_at, e.updated_at
        FROM equipment e
        LEFT JOIN equipment_categories c ON e.category_id = c.id
        LEFT JOIN squads s ON e.squad_id = s.id
        LEFT JOIN users u ON e.responsible_user_id = u.id
        WHERE e.id = $1
        "#
    )
    .bind(equipment_id)
    .fetch_optional(&state.db.pool)
    .await?;

    let equipment =
        equipment.ok_or_else(|| AppError::NotFound("Equipment not found".to_string()))?;

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
            e.squad_id, s.name as squad_name,
            e.quantity, e.available_quantity, COALESCE(e.is_unique, false) as is_unique, e.location, e.qr_code,
            e.responsible_user_id, u.full_name as responsible_name,
            e.status, e.created_at, e.updated_at
        FROM equipment e
        LEFT JOIN equipment_categories c ON e.category_id = c.id
        LEFT JOIN squads s ON e.squad_id = s.id
        LEFT JOIN users u ON e.responsible_user_id = u.id
        WHERE e.qr_code = $1
        "#
    )
    .bind(&qr_code)
    .fetch_optional(&state.db.pool)
    .await?;

    let equipment =
        equipment.ok_or_else(|| AppError::NotFound("Equipment not found".to_string()))?;

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
    let exists: Option<(bool,)> =
        sqlx::query_as::<sqlx::Postgres, _>("SELECT EXISTS(SELECT 1 FROM equipment WHERE id = $1)")
            .bind(equipment_id)
            .fetch_optional(&state.db.pool)
            .await?;

    if exists.is_none() || !exists.unwrap().0 {
        return Err(AppError::NotFound("Equipment not found".to_string()));
    }

    // Обновляем поля
    if let Some(name) = &req.name {
        sqlx::query::<sqlx::Postgres>(
            "UPDATE equipment SET name = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2",
        )
        .bind(name)
        .bind(equipment_id)
        .execute(&state.db.pool)
        .await?;
    }

    if req.description.is_some() {
        sqlx::query::<sqlx::Postgres>(
            "UPDATE equipment SET description = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2",
        )
        .bind(&req.description)
        .bind(equipment_id)
        .execute(&state.db.pool)
        .await?;
    }

    if req.category_id.is_some() {
        sqlx::query::<sqlx::Postgres>(
            "UPDATE equipment SET category_id = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2",
        )
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
            sqlx::query::<sqlx::Postgres>(
                "UPDATE equipment SET quantity = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2",
            )
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
        sqlx::query::<sqlx::Postgres>(
            "UPDATE equipment SET location = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2",
        )
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

    if req.squad_id.is_some() {
        sqlx::query::<sqlx::Postgres>(
            "UPDATE equipment SET squad_id = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2",
        )
        .bind(&req.squad_id)
        .bind(equipment_id)
        .execute(&state.db.pool)
        .await?;
    }

    if let Some(status) = &req.status {
        sqlx::query::<sqlx::Postgres>(
            "UPDATE equipment SET status = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2",
        )
        .bind(status)
        .bind(equipment_id)
        .execute(&state.db.pool)
        .await?;
    }

    // Получаем обновлённое оборудование из БД
    let equipment: Equipment = sqlx::query_as::<sqlx::Postgres, Equipment>(
        "SELECT id, name, description, category_id, squad_id, quantity, available_quantity, COALESCE(is_unique, false) as is_unique, location, qr_code, responsible_user_id, status, created_at, updated_at
         FROM equipment WHERE id = $1"
    )
    .bind(equipment_id)
    .fetch_one(&state.db.pool)
    .await?;

    // Логирование
    let _ = sqlx::query::<sqlx::Postgres>(
        "INSERT INTO activity_logs (user_id, action, entity_type, entity_id, details)
         VALUES ($1, 'update_equipment', 'equipment', $2, $3)",
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
         VALUES ($1, 'delete_equipment', 'equipment', $2)",
    )
    .bind(uuid::Uuid::parse_str(&claims.sub).unwrap())
    .bind(equipment_id)
    .execute(&state.db.pool)
    .await;

    Ok(HttpResponse::NoContent().finish())
}

/// Перенос оборудования в другой сквад / кабинет с записью истории перемещений.
pub async fn move_equipment(
    state: web::Data<AppState>,
    claims: Claims,
    path: web::Path<Uuid>,
    req: web::Json<CreateEquipmentMovementRequest>,
) -> Result<HttpResponse, AppError> {
    // Только админ или ответственный
    AuthService::require_any_role(&claims, &["admin", "responsible"])?;

    let equipment_id = path.into_inner();
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid user id in token".to_string()))?;

    // Получаем текущее состояние оборудования и ответственного сквада
    let row = sqlx::query!(
        r#"
        SELECT e.name as equipment_name, e.squad_id, e.location, s.responsible_user_id
        FROM equipment e
        LEFT JOIN squads s ON e.squad_id = s.id
        WHERE e.id = $1
        "#,
        equipment_id
    )
    .fetch_optional(&state.db.pool)
    .await?;

    let row = match row {
        Some(r) => r,
        None => return Err(AppError::NotFound("Equipment not found".to_string())),
    };

    // Ответственный может перемещать только оборудование своего сквада
    if claims.role == "responsible" {
        if let Some(responsible_id) = row.responsible_user_id {
            if responsible_id != user_id {
                return Err(AppError::BadRequest(
                    "Вы можете перемещать только оборудование своего сквада".to_string(),
                ));
            }
        } else {
            return Err(AppError::BadRequest(
                "У оборудования не указан ответственный сквада".to_string(),
            ));
        }
    }

    let equipment_name = row.equipment_name.clone();
    let from_squad_id = row.squad_id;
    let from_location = row.location.clone();

    // Обновляем оборудование
    sqlx::query::<sqlx::Postgres>(
        r#"
        UPDATE equipment
        SET squad_id = $1,
            location = COALESCE($2, location),
            updated_at = CURRENT_TIMESTAMP
        WHERE id = $3
        "#,
    )
    .bind(&req.to_squad_id)
    .bind(&req.to_location)
    .bind(equipment_id)
    .execute(&state.db.pool)
    .await?;

    // Записываем движение
    let movement_id = Uuid::new_v4();
    sqlx::query::<sqlx::Postgres>(
        r#"
        INSERT INTO equipment_movements (
            id,
            equipment_id,
            from_squad_id,
            to_squad_id,
            from_location,
            to_location,
            moved_by,
            comment
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(movement_id)
    .bind(equipment_id)
    .bind(from_squad_id)
    .bind(&req.to_squad_id)
    .bind(&from_location)
    .bind(&req.to_location)
    .bind(user_id)
    .bind(&req.comment)
    .execute(&state.db.pool)
    .await?;

    // Названия сквадов для отчёта (без ID в логе)
    let squad_ids: Vec<Uuid> = [from_squad_id, req.to_squad_id]
        .into_iter()
        .flatten()
        .collect();
    let squad_names: std::collections::HashMap<Uuid, String> = if squad_ids.is_empty() {
        std::collections::HashMap::new()
    } else {
        sqlx::query_as::<_, (Uuid, String)>("SELECT id, name FROM squads WHERE id = ANY($1)")
            .bind(&squad_ids)
            .fetch_all(&state.db.pool)
            .await
            .ok()
            .unwrap_or_default()
            .into_iter()
            .map(|(id, name)| (id, name))
            .collect()
    };
    let from_squad_name = from_squad_id.and_then(|id| squad_names.get(&id).cloned());
    let to_squad_name = req.to_squad_id.and_then(|id| squad_names.get(&id).cloned());

    let details = serde_json::json!({
        "from_location": from_location,
        "to_location": req.to_location,
        "from_squad_name": from_squad_name,
        "to_squad_name": to_squad_name,
        "equipment_name": equipment_name,
        "comment": req.comment
    });
    let _ = sqlx::query::<sqlx::Postgres>(
        "INSERT INTO activity_logs (user_id, action, entity_type, entity_id, details)
         VALUES ($1, 'equipment_move', 'equipment', $2, $3)",
    )
    .bind(user_id)
    .bind(equipment_id)
    .bind(details)
    .execute(&state.db.pool)
    .await;

    let movement: EquipmentMovement = sqlx::query_as::<sqlx::Postgres, _>(
        r#"
        SELECT
            m.id,
            m.equipment_id,
            m.from_squad_id,
            m.to_squad_id,
            m.from_location,
            m.to_location,
            m.moved_by,
            u.full_name as moved_by_name,
            m.comment,
            m.moved_at
        FROM equipment_movements m
        LEFT JOIN users u ON m.moved_by = u.id
        WHERE m.id = $1
        "#,
    )
    .bind(movement_id)
    .fetch_one(&state.db.pool)
    .await?;

    Ok(HttpResponse::Created().json(movement))
}

/// История перемещений оборудования.
pub async fn get_equipment_movements(
    state: web::Data<AppState>,
    _claims: Claims,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let equipment_id = path.into_inner();

    let movements: Vec<EquipmentMovement> = sqlx::query_as::<sqlx::Postgres, _>(
        r#"
        SELECT
            m.id,
            m.equipment_id,
            m.from_squad_id,
            m.to_squad_id,
            m.from_location,
            m.to_location,
            m.moved_by,
            u.full_name as moved_by_name,
            m.comment,
            m.moved_at
        FROM equipment_movements m
        LEFT JOIN users u ON m.moved_by = u.id
        WHERE m.equipment_id = $1
        ORDER BY m.moved_at DESC
        "#,
    )
    .bind(equipment_id)
    .fetch_all(&state.db.pool)
    .await?;

    Ok(HttpResponse::Ok().json(movements))
}
