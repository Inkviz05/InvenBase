use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::auth::{AuthService, Claims};
use crate::errors::AppError;
use crate::models::{CreateUserRequest, UpdateUserRequest, User, UserResponse};

pub async fn create_user(
    state: web::Data<AppState>,
    claims: Claims,
    req: web::Json<CreateUserRequest>,
) -> Result<HttpResponse, AppError> {
    // Только админы могут создавать пользователей
    AuthService::require_role(&claims, "admin")?;

    let password_hash = AuthService::hash_password(&req.password)?;
    let role = req.role.as_deref().unwrap_or("user").to_string();

    if !["admin", "responsible", "user"].contains(&role.as_str()) {
        return Err(AppError::BadRequest("Invalid role".to_string()));
    }

    let user_id = uuid::Uuid::new_v4();

    sqlx::query::<sqlx::Postgres>(
        "INSERT INTO users (id, username, password_hash, email, full_name, role)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(user_id)
    .bind(&req.username)
    .bind(password_hash)
    .bind(&req.email)
    .bind(&req.full_name)
    .bind(&role)
    .execute(&state.db.pool)
    .await?;

    // Логирование
    let _ = sqlx::query::<sqlx::Postgres>(
        "INSERT INTO activity_logs (user_id, action, entity_type, entity_id, details)
         VALUES ($1, 'create_user', 'user', $2, $3)",
    )
    .bind(uuid::Uuid::parse_str(&claims.sub).unwrap())
    .bind(user_id)
    .bind(serde_json::json!({"username": req.username, "role": role}))
    .execute(&state.db.pool)
    .await;

    let user: User = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT id, username, password_hash, email, full_name, role, created_at, updated_at
         FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_one(&state.db.pool)
    .await?;

    Ok(HttpResponse::Created().json(UserResponse::from(user)))
}

pub async fn get_users(
    state: web::Data<AppState>,
    claims: Claims,
) -> Result<HttpResponse, AppError> {
    AuthService::require_any_role(&claims, &["admin", "responsible"])?;

    let users: Vec<User> = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT id, username, password_hash, email, full_name, role, created_at, updated_at
         FROM users ORDER BY created_at DESC",
    )
    .fetch_all(&state.db.pool)
    .await?;

    let users_response: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();

    Ok(HttpResponse::Ok().json(users_response))
}

pub async fn get_user(
    state: web::Data<AppState>,
    claims: Claims,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let user_id = path.into_inner();

    // Пользователь может видеть только свой профиль, админы и ответственные - всех
    if claims.role != "admin" && claims.role != "responsible" {
        let current_user_id = uuid::Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

        if current_user_id != user_id {
            return Err(AppError::Unauthorized("Access denied".to_string()));
        }
    }

    let user: Option<User> = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT id, username, password_hash, email, full_name, role, created_at, updated_at
         FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(&state.db.pool)
    .await?;

    let user = user.ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(HttpResponse::Ok().json(UserResponse::from(user)))
}

pub async fn update_user(
    state: web::Data<AppState>,
    claims: Claims,
    path: web::Path<Uuid>,
    req: web::Json<UpdateUserRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id = path.into_inner();
    let current_user_id = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

    // Только админы могут изменять других пользователей
    if user_id != current_user_id && claims.role != "admin" {
        return Err(AppError::Unauthorized("Access denied".to_string()));
    }

    // Проверяем, что пользователь существует
    let existing_user: Option<User> = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT id, username, password_hash, email, full_name, role, created_at, updated_at
         FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(&state.db.pool)
    .await?;

    let existing_user =
        existing_user.ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    // Обновляем поля
    let mut updates = Vec::new();
    let mut param_count = 1;

    if let Some(ref username) = req.username {
        // Проверяем уникальность username, если он изменился
        if username != &existing_user.username {
            let exists: bool = sqlx::query_scalar::<sqlx::Postgres, _>(
                "SELECT EXISTS(SELECT 1 FROM users WHERE username = $1 AND id != $2)",
            )
            .bind(username)
            .bind(user_id)
            .fetch_one(&state.db.pool)
            .await?;

            if exists {
                return Err(AppError::BadRequest("Username already exists".to_string()));
            }
        }
        updates.push(format!("username = ${}", param_count));
        param_count += 1;
    }

    // Обрабатываем email - если Some, то обновляем (даже если None внутри)
    if req.email.is_some() {
        updates.push(format!("email = ${}", param_count));
        param_count += 1;
    }

    // Обрабатываем full_name - если Some, то обновляем (даже если None внутри)
    if req.full_name.is_some() {
        updates.push(format!("full_name = ${}", param_count));
        param_count += 1;
    }

    // Только админы могут менять роль
    if claims.role == "admin" {
        if let Some(ref role) = req.role {
            if !["admin", "responsible", "user"].contains(&role.as_str()) {
                return Err(AppError::BadRequest("Invalid role".to_string()));
            }
            updates.push(format!("role = ${}", param_count));
            param_count += 1;
        }
    }

    // Обновление пароля (будет обработано позже)
    let password_hash = if let Some(ref password) = req.password {
        Some(AuthService::hash_password(password)?)
    } else {
        None
    };

    if password_hash.is_some() {
        updates.push(format!("password_hash = ${}", param_count));
        param_count += 1;
    }

    // Если нет обновлений, возвращаем текущего пользователя
    if updates.is_empty() {
        log::info!("No fields to update for user {}", user_id);
        return Ok(HttpResponse::Ok().json(UserResponse::from(existing_user)));
    }

    updates.push("updated_at = CURRENT_TIMESTAMP".to_string());

    let query = format!(
        "UPDATE users SET {} WHERE id = ${} RETURNING id, username, password_hash, email, full_name, role, created_at, updated_at",
        updates.join(", "),
        param_count
    );

    log::debug!("Update user query: {}", query);
    log::debug!("Updating user {} with: username={:?}, email={:?}, full_name={:?}, role={:?}, password_changed={}",
        user_id, req.username, req.email, req.full_name, req.role, req.password.is_some());

    let mut query_builder = sqlx::query_as::<sqlx::Postgres, User>(&query);

    // Привязываем параметры в том же порядке, в котором они добавлены в updates
    if let Some(ref username) = req.username {
        query_builder = query_builder.bind(username);
    }
    if req.email.is_some() {
        query_builder = query_builder.bind(&req.email);
    }
    if req.full_name.is_some() {
        query_builder = query_builder.bind(&req.full_name);
    }
    if claims.role == "admin" {
        if let Some(ref role) = req.role {
            query_builder = query_builder.bind(role);
        }
    }
    if let Some(ref hash) = password_hash {
        query_builder = query_builder.bind(hash);
    }

    query_builder = query_builder.bind(user_id);

    let user: User = query_builder.fetch_one(&state.db.pool).await.map_err(|e| {
        log::error!("Error updating user {}: {:?}", user_id, e);
        AppError::DatabaseError(e)
    })?;

    log::info!("Successfully updated user {}", user_id);

    // Логирование
    let _ = sqlx::query::<sqlx::Postgres>(
        "INSERT INTO activity_logs (user_id, action, entity_type, entity_id, details)
         VALUES ($1, 'update_user', 'user', $2, $3)",
    )
    .bind(current_user_id)
    .bind(user_id)
    .bind(serde_json::json!({
        "username": req.username.as_ref().unwrap_or(&existing_user.username),
        "email": req.email.as_ref().or(existing_user.email.as_ref()),
        "full_name": req.full_name.as_ref().or(existing_user.full_name.as_ref()),
        "role": req.role.as_ref().unwrap_or(&existing_user.role),
        "password_changed": req.password.is_some()
    }))
    .execute(&state.db.pool)
    .await;

    Ok(HttpResponse::Ok().json(UserResponse::from(user)))
}

pub async fn delete_user(
    state: web::Data<AppState>,
    claims: Claims,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    AuthService::require_role(&claims, "admin")?;

    let user_id = path.into_inner();

    let result = sqlx::query::<sqlx::Postgres>("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&state.db.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("User not found".to_string()));
    }

    // Логирование
    let _ = sqlx::query::<sqlx::Postgres>(
        "INSERT INTO activity_logs (user_id, action, entity_type, entity_id)
         VALUES ($1, 'delete_user', 'user', $2)",
    )
    .bind(uuid::Uuid::parse_str(&claims.sub).unwrap())
    .bind(user_id)
    .execute(&state.db.pool)
    .await;

    Ok(HttpResponse::NoContent().finish())
}
