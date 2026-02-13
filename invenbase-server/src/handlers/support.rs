use actix_web::{web, HttpResponse};

use crate::models::{
    SupportRequest, SupportRequestWithUser, SupportRequestWithMessages, SupportRequestMessage,
    CreateSupportRequestRequest, UpdateSupportRequestRequest, AddSupportMessageRequest,
};
use crate::auth::{AuthService, Claims};
use crate::errors::AppError;
use crate::app_state::AppState;
use crate::handlers::push::send_push_to_user;

pub async fn create_support_request(
    state: web::Data<AppState>,
    claims: Claims,
    req: web::Json<CreateSupportRequestRequest>,
) -> Result<HttpResponse, AppError> {
    // Создавать заявки могут только пользователи и ответственные, не админы
    if claims.role == "admin" {
        return Err(AppError::BadRequest(
            "Администратор не может создавать заявки. Заявки создают пользователи или ответственные.".to_string(),
        ));
    }

    let user_id = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

    let subject = req.subject.trim();
    let message = req.message.trim();
    if subject.is_empty() {
        return Err(AppError::BadRequest("Тема заявки не может быть пустой.".to_string()));
    }
    if message.is_empty() {
        return Err(AppError::BadRequest("Сообщение не может быть пустым.".to_string()));
    }
    if subject.len() > 500 {
        return Err(AppError::BadRequest("Тема заявки слишком длинная.".to_string()));
    }

    let id = uuid::Uuid::new_v4();
    sqlx::query::<sqlx::Postgres>(
        "INSERT INTO support_requests (id, user_id, subject, message, status)
         VALUES ($1, $2, $3, $4, 'open')"
    )
    .bind(id)
    .bind(user_id)
    .bind(subject)
    .bind(message)
    .execute(&state.db.pool)
    .await?;

    let row: SupportRequest = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT id, user_id, subject, message, status, created_at, updated_at, admin_comment
         FROM support_requests WHERE id = $1"
    )
    .bind(id)
    .fetch_one(&state.db.pool)
    .await?;

    // Уведомления только админам о новой заявке (никогда не отправляем автору заявки)
    let notif_title = "Новая заявка в поддержку";
    let notif_message = format!(
        "Пользователь {} оставил заявку: {}",
        claims.username,
        if subject.len() > 80 { format!("{}…", &subject[..80]) } else { subject.to_string() }
    );
    let _ = sqlx::query::<sqlx::Postgres>(
        "INSERT INTO notifications (user_id, title, message, notification_type)
         SELECT id, $1, $2, 'support_new'
         FROM users WHERE role = 'admin' AND id != $3"
    )
    .bind(&notif_title)
    .bind(&notif_message)
    .bind(user_id)
    .execute(&state.db.pool)
    .await;

    let admin_ids: Vec<(uuid::Uuid,)> = sqlx::query_as(
        "SELECT id FROM users WHERE role = 'admin' AND id != $1"
    )
    .bind(user_id)
    .fetch_all(&state.db.pool)
    .await
    .unwrap_or_default();
    let mut data = serde_json::Map::new();
    data.insert("type".to_string(), serde_json::Value::String("support_new".to_string()));
    data.insert("support_request_id".to_string(), serde_json::Value::String(id.to_string()));
    for (admin_id,) in admin_ids {
        send_push_to_user(state.get_ref(), admin_id, notif_title, &notif_message, data.clone()).await;
    }

    Ok(HttpResponse::Created().json(row))
}

pub async fn get_support_requests(
    state: web::Data<AppState>,
    claims: Claims,
) -> Result<HttpResponse, AppError> {
    let user_id = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

    let is_admin = claims.role == "admin";

    let requests: Vec<SupportRequestWithUser> = if is_admin {
        sqlx::query_as::<sqlx::Postgres, _>(
            "SELECT sr.id, sr.user_id, u.full_name as user_name, sr.subject, sr.message,
                    sr.status, sr.created_at, sr.updated_at, sr.admin_comment
             FROM support_requests sr
             LEFT JOIN users u ON sr.user_id = u.id
             ORDER BY sr.created_at DESC"
        )
        .fetch_all(&state.db.pool)
        .await?
    } else {
        sqlx::query_as::<sqlx::Postgres, _>(
            "SELECT sr.id, sr.user_id, u.full_name as user_name, sr.subject, sr.message,
                    sr.status, sr.created_at, sr.updated_at, sr.admin_comment
             FROM support_requests sr
             LEFT JOIN users u ON sr.user_id = u.id
             WHERE sr.user_id = $1
             ORDER BY sr.created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&state.db.pool)
        .await?
    };

    let request_ids: Vec<uuid::Uuid> = requests.iter().map(|r| r.id).collect();
    let messages: Vec<SupportRequestMessage> = if request_ids.is_empty() {
        vec![]
    } else {
        // Загружаем все сообщения по заявкам (batch)
        let placeholders: String = request_ids
            .iter()
            .enumerate()
            .map(|(i, _)| format!("${}", i + 1))
            .collect::<Vec<_>>()
            .join(", ");
        let query = format!(
            "SELECT id, support_request_id, author_user_id, is_staff, message, created_at
             FROM support_request_messages
             WHERE support_request_id IN ({})
             ORDER BY created_at ASC",
            placeholders
        );
        let mut q = sqlx::query_as::<sqlx::Postgres, SupportRequestMessage>(&query);
        for id in &request_ids {
            q = q.bind(id);
        }
        q.fetch_all(&state.db.pool).await?
    };

    let mut messages_by_request: std::collections::HashMap<uuid::Uuid, Vec<SupportRequestMessage>> = {
        let mut map: std::collections::HashMap<uuid::Uuid, Vec<SupportRequestMessage>> =
            std::collections::HashMap::new();
        for m in messages {
            map.entry(m.support_request_id).or_default().push(m);
        }
        map
    };

    let result: Vec<SupportRequestWithMessages> = requests
        .into_iter()
        .map(|req| SupportRequestWithMessages {
            messages: messages_by_request.remove(&req.id).unwrap_or_default(),
            request: req,
        })
        .collect();

    Ok(HttpResponse::Ok().json(result))
}

const ALLOWED_STATUSES: &[&str] = &["open", "in_progress", "answered", "closed"];

pub async fn update_support_request(
    state: web::Data<AppState>,
    claims: Claims,
    path: web::Path<uuid::Uuid>,
    req: web::Json<UpdateSupportRequestRequest>,
) -> Result<HttpResponse, AppError> {
    AuthService::require_role(&claims, "admin")?;
    let request_id = path.into_inner();

    let current: Option<(String,)> = sqlx::query_as(
        "SELECT status FROM support_requests WHERE id = $1"
    )
    .bind(request_id)
    .fetch_optional(&state.db.pool)
    .await?;
    let current_status = current.as_ref().map(|c| c.0.as_str());
    if current_status.is_none() {
        return Err(AppError::NotFound("Заявка не найдена".to_string()));
    }
    let current_status = current_status.unwrap();

    // Закрытую заявку нельзя редактировать (ответить) — только удалить
    if current_status == "closed" {
        return Err(AppError::BadRequest(
            "Заявка закрыта. Ответить нельзя, можно только удалить заявку.".to_string(),
        ));
    }

    let new_status = req.status.as_deref().unwrap_or(current_status);
    // При переводе в «закрыта» не принимаем ответ в этом запросе — только смена статуса
    let allow_comment = req.admin_comment.as_ref().map(|s| !s.is_empty()).unwrap_or(false)
        && new_status != "closed";

    let mut conditions = Vec::new();
    let mut bind_count = 1;
    if req.status.is_some() {
        let status = req.status.as_deref().unwrap_or("open");
        if !ALLOWED_STATUSES.contains(&status) {
            return Err(AppError::BadRequest(
                "Недопустимый статус. Допустимы: open, in_progress, answered, closed.".to_string(),
            ));
        }
        conditions.push(format!("status = ${}", bind_count));
        bind_count += 1;
    }
    if allow_comment {
        conditions.push(format!("admin_comment = ${}", bind_count));
        bind_count += 1;
    }
    if conditions.is_empty() {
        return Err(AppError::BadRequest("Нет полей для обновления".to_string()));
    }
    conditions.push("updated_at = CURRENT_TIMESTAMP".to_string());
    let update_sql = format!(
        "UPDATE support_requests SET {} WHERE id = ${}",
        conditions.join(", "),
        bind_count
    );
    let mut q = sqlx::query::<sqlx::Postgres>(&update_sql);
    if let Some(ref s) = req.status {
        q = q.bind(s);
    }
    if allow_comment {
        if let Some(ref c) = req.admin_comment {
            q = q.bind(c);
        }
    }
    q = q.bind(request_id);
    q.execute(&state.db.pool).await?;

    // Сохраняем ответ в переписку (для истории)
    if allow_comment {
        if let Some(ref comment) = req.admin_comment {
            let admin_id = uuid::Uuid::parse_str(&claims.sub).unwrap_or_default();
            let _ = sqlx::query::<sqlx::Postgres>(
                "INSERT INTO support_request_messages (support_request_id, author_user_id, is_staff, message)
                 VALUES ($1, $2, TRUE, $3)"
            )
            .bind(request_id)
            .bind(admin_id)
            .bind(comment.as_str())
            .execute(&state.db.pool)
            .await;
        }
    }

    let updated: SupportRequestWithUser = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT sr.id, sr.user_id, u.full_name as user_name, sr.subject, sr.message,
                sr.status, sr.created_at, sr.updated_at, sr.admin_comment
         FROM support_requests sr
         LEFT JOIN users u ON sr.user_id = u.id
         WHERE sr.id = $1"
    )
    .bind(request_id)
    .fetch_one(&state.db.pool)
    .await?;

    // Уведомление только автору заявки об ответе (никогда не отправляем тому, кто ответил — админу)
    let responder_id = uuid::Uuid::parse_str(&claims.sub).unwrap_or_default();
    if allow_comment && updated.user_id != responder_id {
        let notif_title = "По заявке дан ответ";
        let subject_preview = if updated.subject.len() > 50 {
            format!("{}…", &updated.subject[..50])
        } else {
            updated.subject.clone()
        };
        let notif_message = format!(
            "По вашей заявке «{}» дан ответ. Перейдите в раздел «Поддержка».",
            subject_preview
        );
        let _ = sqlx::query::<sqlx::Postgres>(
            "INSERT INTO notifications (user_id, title, message, notification_type)
             VALUES ($1, $2, $3, 'support_reply')"
        )
        .bind(updated.user_id)
        .bind(&notif_title)
        .bind(&notif_message)
        .execute(&state.db.pool)
        .await;

        let mut data = serde_json::Map::new();
        data.insert("type".to_string(), serde_json::Value::String("support_reply".to_string()));
        data.insert("support_request_id".to_string(), serde_json::Value::String(request_id.to_string()));
        send_push_to_user(
            state.get_ref(),
            updated.user_id,
            &notif_title,
            &notif_message,
            data,
        ).await;
    }

    Ok(HttpResponse::Ok().json(updated))
}

/// Добавить сообщение в заявку (автор заявки, пока статус не «закрыта»).
pub async fn add_support_message(
    state: web::Data<AppState>,
    claims: Claims,
    path: web::Path<uuid::Uuid>,
    req: web::Json<AddSupportMessageRequest>,
) -> Result<HttpResponse, AppError> {
    let request_id = path.into_inner();
    let user_id = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

    let message = req.message.trim();
    if message.is_empty() {
        return Err(AppError::BadRequest("Сообщение не может быть пустым.".to_string()));
    }

    let row: Option<(uuid::Uuid, String)> = sqlx::query_as(
        "SELECT user_id, status FROM support_requests WHERE id = $1"
    )
    .bind(request_id)
    .fetch_optional(&state.db.pool)
    .await?;

    let (owner_id, status) = row
        .ok_or_else(|| AppError::NotFound("Заявка не найдена".to_string()))?;
    if owner_id != user_id {
        return Err(AppError::Unauthorized(
            "Добавлять сообщения может только автор заявки.".to_string(),
        ));
    }
    if status == "closed" {
        return Err(AppError::BadRequest(
            "Нельзя добавить сообщение в закрытую заявку. Создайте новую заявку.".to_string(),
        ));
    }

    let msg_id = uuid::Uuid::new_v4();
    sqlx::query::<sqlx::Postgres>(
        "INSERT INTO support_request_messages (id, support_request_id, author_user_id, is_staff, message)
         VALUES ($1, $2, $3, FALSE, $4)"
    )
    .bind(msg_id)
    .bind(request_id)
    .bind(user_id)
    .bind(message)
    .execute(&state.db.pool)
    .await?;

    // Вернуть заявку в статус «открыта», если была «ответ дан»
    if status == "answered" {
        let _ = sqlx::query::<sqlx::Postgres>(
            "UPDATE support_requests SET status = 'open', updated_at = CURRENT_TIMESTAMP WHERE id = $1"
        )
        .bind(request_id)
        .execute(&state.db.pool)
        .await;
    }

    let created: SupportRequestMessage = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT id, support_request_id, author_user_id, is_staff, message, created_at
         FROM support_request_messages WHERE id = $1"
    )
    .bind(msg_id)
    .fetch_one(&state.db.pool)
    .await?;

    Ok(HttpResponse::Created().json(created))
}

/// Удалить заявку (только админ, только закрытую заявку).
pub async fn delete_support_request(
    state: web::Data<AppState>,
    claims: Claims,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, AppError> {
    AuthService::require_role(&claims, "admin")?;
    let request_id = path.into_inner();

    let row: Option<(String,)> = sqlx::query_as(
        "SELECT status FROM support_requests WHERE id = $1"
    )
    .bind(request_id)
    .fetch_optional(&state.db.pool)
    .await?;

    let (status,) = row.ok_or_else(|| AppError::NotFound("Заявка не найдена".to_string()))?;
    if status != "closed" {
        return Err(AppError::BadRequest(
            "Удалить можно только закрытую заявку. Сначала закройте заявку.".to_string(),
        ));
    }

    sqlx::query::<sqlx::Postgres>("DELETE FROM support_requests WHERE id = $1")
        .bind(request_id)
        .execute(&state.db.pool)
        .await?;

    Ok(HttpResponse::NoContent().finish())
}
