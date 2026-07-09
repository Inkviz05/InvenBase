use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::auth::{AuthService, Claims};
use crate::errors::AppError;
use crate::models::{
    CreateSquadRequest, EquipmentWithDetails, Squad, SquadWithDetails, UpdateSquadRequest,
};

pub async fn create_squad(
    state: web::Data<AppState>,
    claims: Claims,
    req: web::Json<CreateSquadRequest>,
) -> Result<HttpResponse, AppError> {
    AuthService::require_any_role(&claims, &["admin", "responsible"])?;

    if let Some(resp_id) = req.responsible_user_id {
        let role: Option<(String,)> = sqlx::query_as("SELECT role FROM users WHERE id = $1")
            .bind(resp_id)
            .fetch_optional(&state.db.pool)
            .await?;
        match role.as_ref().map(|r| r.0.as_str()) {
            Some("admin") | Some("responsible") => {}
            _ => return Err(AppError::BadRequest(
                "Ответственным сквада можно назначить только пользователя с ролью «Администратор» или «Ответственный».".to_string(),
            )),
        }
    }

    let squad_id = uuid::Uuid::new_v4();

    sqlx::query::<sqlx::Postgres>(
        "INSERT INTO squads (id, name, description, location, responsible_user_id)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(squad_id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.location)
    .bind(&req.responsible_user_id)
    .execute(&state.db.pool)
    .await?;

    let squad: Squad = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT id, name, description, location, responsible_user_id, created_at, updated_at
         FROM squads WHERE id = $1",
    )
    .bind(squad_id)
    .fetch_one(&state.db.pool)
    .await?;

    Ok(HttpResponse::Created().json(squad))
}

pub async fn get_squads(
    state: web::Data<AppState>,
    _claims: Claims,
) -> Result<HttpResponse, AppError> {
    let squads: Vec<SquadWithDetails> = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT s.id, s.name, s.description, s.location, s.responsible_user_id,
                u.full_name as responsible_name, s.created_at, s.updated_at
         FROM squads s
         LEFT JOIN users u ON s.responsible_user_id = u.id
         ORDER BY s.created_at DESC",
    )
    .fetch_all(&state.db.pool)
    .await?;

    Ok(HttpResponse::Ok().json(squads))
}

pub async fn get_squad(
    state: web::Data<AppState>,
    _claims: Claims,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let squad_id = path.into_inner();

    let squad: Option<SquadWithDetails> = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT s.id, s.name, s.description, s.location, s.responsible_user_id,
                u.full_name as responsible_name, s.created_at, s.updated_at
         FROM squads s
         LEFT JOIN users u ON s.responsible_user_id = u.id
         WHERE s.id = $1",
    )
    .bind(squad_id)
    .fetch_optional(&state.db.pool)
    .await?;

    let squad = squad.ok_or_else(|| AppError::NotFound("Squad not found".to_string()))?;

    Ok(HttpResponse::Ok().json(squad))
}

pub async fn update_squad(
    state: web::Data<AppState>,
    claims: Claims,
    path: web::Path<Uuid>,
    req: web::Json<UpdateSquadRequest>,
) -> Result<HttpResponse, AppError> {
    AuthService::require_any_role(&claims, &["admin", "responsible"])?;

    let squad_id = path.into_inner();

    // Проверяем, что сквад существует
    let exists: Option<(bool,)> =
        sqlx::query_as::<sqlx::Postgres, _>("SELECT EXISTS(SELECT 1 FROM squads WHERE id = $1)")
            .bind(squad_id)
            .fetch_optional(&state.db.pool)
            .await?;

    if exists.is_none() || !exists.unwrap().0 {
        return Err(AppError::NotFound("Squad not found".to_string()));
    }

    if let Some(resp_id) = req.responsible_user_id {
        let role: Option<(String,)> = sqlx::query_as("SELECT role FROM users WHERE id = $1")
            .bind(resp_id)
            .fetch_optional(&state.db.pool)
            .await?;
        match role.as_ref().map(|r| r.0.as_str()) {
            Some("admin") | Some("responsible") => {}
            _ => return Err(AppError::BadRequest(
                "Ответственным сквада можно назначить только пользователя с ролью «Администратор» или «Ответственный».".to_string(),
            )),
        }
    }

    // Обновляем только переданные поля
    // Формируем динамический запрос
    let mut conditions = Vec::new();
    let mut bind_count = 1;

    if req.name.is_some() {
        conditions.push(format!("name = ${}", bind_count));
        bind_count += 1;
    }
    if req.description.is_some() {
        conditions.push(format!("description = ${}", bind_count));
        bind_count += 1;
    }
    if req.location.is_some() {
        conditions.push(format!("location = ${}", bind_count));
        bind_count += 1;
    }
    if req.responsible_user_id.is_some() {
        conditions.push(format!("responsible_user_id = ${}", bind_count));
        bind_count += 1;
    }

    if conditions.is_empty() {
        return Err(AppError::BadRequest("No fields to update".to_string()));
    }

    conditions.push("updated_at = CURRENT_TIMESTAMP".to_string());
    let update_query = format!(
        "UPDATE squads SET {} WHERE id = ${}",
        conditions.join(", "),
        bind_count
    );

    let mut query_builder = sqlx::query::<sqlx::Postgres>(&update_query);

    if let Some(ref name) = req.name {
        query_builder = query_builder.bind(name);
    }
    if let Some(ref desc) = req.description {
        query_builder = query_builder.bind(desc);
    }
    if let Some(ref loc) = req.location {
        query_builder = query_builder.bind(loc);
    }
    if let Some(ref resp_id) = req.responsible_user_id {
        query_builder = query_builder.bind(resp_id);
    }
    query_builder = query_builder.bind(squad_id);

    query_builder.execute(&state.db.pool).await?;

    let squad: SquadWithDetails = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT s.id, s.name, s.description, s.location, s.responsible_user_id,
                u.full_name as responsible_name, s.created_at, s.updated_at
         FROM squads s
         LEFT JOIN users u ON s.responsible_user_id = u.id
         WHERE s.id = $1",
    )
    .bind(squad_id)
    .fetch_one(&state.db.pool)
    .await?;

    Ok(HttpResponse::Ok().json(squad))
}

pub async fn delete_squad(
    state: web::Data<AppState>,
    claims: Claims,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    AuthService::require_any_role(&claims, &["admin", "responsible"])?;

    let squad_id = path.into_inner();

    let result = sqlx::query::<sqlx::Postgres>("DELETE FROM squads WHERE id = $1")
        .bind(squad_id)
        .execute(&state.db.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Squad not found".to_string()));
    }

    Ok(HttpResponse::NoContent().finish())
}

/// Список оборудования сквада
pub async fn get_squad_equipment(
    state: web::Data<AppState>,
    _claims: Claims,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let squad_id = path.into_inner();

    let _exists: Option<(bool,)> =
        sqlx::query_as::<sqlx::Postgres, _>("SELECT EXISTS(SELECT 1 FROM squads WHERE id = $1)")
            .bind(squad_id)
            .fetch_optional(&state.db.pool)
            .await?;

    if _exists.map(|(e,)| e).unwrap_or(false) == false {
        return Err(AppError::NotFound("Squad not found".to_string()));
    }

    let equipment: Vec<EquipmentWithDetails> = sqlx::query_as::<sqlx::Postgres, _>(
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
        WHERE e.squad_id = $1
        ORDER BY e.created_at DESC
        "#
    )
    .bind(squad_id)
    .fetch_all(&state.db.pool)
    .await?;

    Ok(HttpResponse::Ok().json(equipment))
}
