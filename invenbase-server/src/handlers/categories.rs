use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::models::{EquipmentCategory, CreateCategoryRequest};
use crate::auth::{AuthService, Claims};
use crate::errors::AppError;
use crate::app_state::AppState;

pub async fn create_category(
    state: web::Data<AppState>,
    claims: Claims,
    req: web::Json<CreateCategoryRequest>,
) -> Result<HttpResponse, AppError> {
    AuthService::require_any_role(&claims, &["admin", "responsible"])?;

    let category_id = uuid::Uuid::new_v4();

    sqlx::query::<sqlx::Postgres>(
        "INSERT INTO equipment_categories (id, name, description)
         VALUES ($1, $2, $3)"
    )
    .bind(category_id)
    .bind(&req.name)
    .bind(&req.description)
    .execute(&state.db.pool)
    .await?;

    let category: EquipmentCategory = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT id, name, description, created_at FROM equipment_categories WHERE id = $1"
    )
    .bind(category_id)
    .fetch_one(&state.db.pool)
    .await?;

    Ok(HttpResponse::Created().json(category))
}

pub async fn get_categories(
    state: web::Data<AppState>,
    _claims: Claims,
) -> Result<HttpResponse, AppError> {
    let categories: Vec<EquipmentCategory> = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT id, name, description, created_at FROM equipment_categories ORDER BY name"
    )
    .fetch_all(&state.db.pool)
    .await?;

    Ok(HttpResponse::Ok().json(categories))
}

pub async fn get_category(
    state: web::Data<AppState>,
    _claims: Claims,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let category_id = path.into_inner();

    let category: Option<EquipmentCategory> = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT id, name, description, created_at FROM equipment_categories WHERE id = $1"
    )
    .bind(category_id)
    .fetch_optional(&state.db.pool)
    .await?;

    let category = category.ok_or_else(|| AppError::NotFound("Category not found".to_string()))?;

    Ok(HttpResponse::Ok().json(category))
}

pub async fn update_category(
    state: web::Data<AppState>,
    claims: Claims,
    path: web::Path<Uuid>,
    req: web::Json<CreateCategoryRequest>,
) -> Result<HttpResponse, AppError> {
    AuthService::require_any_role(&claims, &["admin", "responsible"])?;

    let category_id = path.into_inner();

    sqlx::query::<sqlx::Postgres>(
        "UPDATE equipment_categories SET name = $1, description = $2 WHERE id = $3"
    )
    .bind(&req.name)
    .bind(&req.description)
    .bind(category_id)
    .execute(&state.db.pool)
    .await?;

    let category: Option<EquipmentCategory> = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT id, name, description, created_at FROM equipment_categories WHERE id = $1"
    )
    .bind(category_id)
    .fetch_optional(&state.db.pool)
    .await?;

    let category = category.ok_or_else(|| AppError::NotFound("Category not found".to_string()))?;

    Ok(HttpResponse::Ok().json(category))
}

pub async fn delete_category(
    state: web::Data<AppState>,
    claims: Claims,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    AuthService::require_any_role(&claims, &["admin", "responsible"])?;

    let category_id = path.into_inner();

    let result = sqlx::query::<sqlx::Postgres>("DELETE FROM equipment_categories WHERE id = $1")
        .bind(category_id)
        .execute(&state.db.pool)
        .await
        .map_err(|e| {
            // В категории ещё есть оборудование — возвращаем понятную ошибку вместо 500
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.code().as_deref() == Some("23503")
                    && db_err
                        .constraint()
                        .map(|c| c.to_string())
                        == Some("equipment_category_id_fkey".to_string())
                {
                    return AppError::BadRequest(
                        "Нельзя удалить категорию, пока в ней есть оборудование. Сначала удалите или перенесите оборудование."
                            .to_string(),
                    );
                }
            }
            AppError::from(e)
        })?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Category not found".to_string()));
    }

    Ok(HttpResponse::NoContent().finish())
}

