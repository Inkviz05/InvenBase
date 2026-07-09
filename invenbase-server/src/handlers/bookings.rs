use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::auth::{AuthService, Claims};
use crate::errors::AppError;
use crate::handlers::push::send_push_to_user;
use crate::models::{
    Booking, BookingWithDetails, CreateBookingRequest, CreateBulkBookingsRequest,
    UpdateBookingRequest,
};
use crate::services::booking_events;
use crate::services::bookings as booking_service;

pub async fn create_booking(
    state: web::Data<AppState>,
    claims: Claims,
    req: web::Json<CreateBookingRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;
    let booking = booking_service::create_reserved_booking(&state.db.pool, user_id, &req).await?;
    let booking_id = booking.id;

    let notification_message =
        format!("Пользователь {} создал новое бронирование", claims.username);

    // Создаём уведомление в БД для каждого администратора и ответственного
    booking_events::notify_reviewers_about_booking_request(
        &state.db.pool,
        "Новое бронирование",
        &notification_message,
    )
    .await;

    // Отправляем push-уведомление администраторам и ответственным
    let admin_user_ids: Vec<(Uuid,)> = sqlx::query_as::<sqlx::Postgres, (Uuid,)>(
        "SELECT id FROM users WHERE role IN ('admin', 'responsible')",
    )
    .fetch_all(&state.db.pool)
    .await
    .unwrap_or_default();
    log::info!(
        "Booking created: sending push to {} admin/responsible user(s) about new booking from {}",
        admin_user_ids.len(),
        claims.username
    );
    let mut data = serde_json::Map::new();
    data.insert(
        "type".to_string(),
        serde_json::Value::String("booking_request".to_string()),
    );
    data.insert(
        "booking_id".to_string(),
        serde_json::Value::String(booking_id.to_string()),
    );
    for (admin_id,) in admin_user_ids {
        send_push_to_user(
            state.get_ref(),
            admin_id,
            "Новое бронирование",
            &notification_message,
            data.clone(),
        )
        .await;
    }

    // Логирование
    booking_events::record_activity(
        &state.db.pool,
        user_id,
        booking_id,
        "create_booking",
        serde_json::json!({"equipment_id": req.equipment_id, "quantity": req.quantity}),
    )
    .await;
    Ok(HttpResponse::Created().json(booking))
}

/// Массовое создание бронирований (из корзины): одно уведомление администраторам вместо N.
pub async fn create_bulk_bookings(
    state: web::Data<AppState>,
    claims: Claims,
    req: web::Json<CreateBulkBookingsRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;
    let created =
        booking_service::create_reserved_bookings(&state.db.pool, user_id, &req.bookings).await?;

    let count = created.len();
    let notification_message = format!(
        "Пользователь {} создал {} бронирований",
        claims.username, count
    );

    booking_events::notify_reviewers_about_booking_request(
        &state.db.pool,
        "Новые бронирования",
        &notification_message,
    )
    .await;

    let admin_user_ids: Vec<(Uuid,)> = sqlx::query_as::<sqlx::Postgres, (Uuid,)>(
        "SELECT id FROM users WHERE role IN ('admin', 'responsible')",
    )
    .fetch_all(&state.db.pool)
    .await
    .unwrap_or_default();

    let mut data = serde_json::Map::new();
    data.insert(
        "type".to_string(),
        serde_json::Value::String("booking_request".to_string()),
    );
    data.insert(
        "count".to_string(),
        serde_json::Value::Number(serde_json::Number::from(count)),
    );
    for (admin_id,) in admin_user_ids {
        send_push_to_user(
            state.get_ref(),
            admin_id,
            "Новые бронирования",
            &notification_message,
            data.clone(),
        )
        .await;
    }

    log::info!(
        "Bulk booking: {} bookings created by {}, one notification sent",
        count,
        claims.username
    );

    Ok(HttpResponse::Created().json(created))
}

pub async fn get_bookings(
    state: web::Data<AppState>,
    claims: Claims,
) -> Result<HttpResponse, AppError> {
    let user_id = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

    let bookings: Vec<BookingWithDetails> =
        if claims.role == "admin" || claims.role == "responsible" {
            // Админы и ответственные видят все бронирования
            sqlx::query_as::<sqlx::Postgres, _>(
                r#"
            SELECT
                b.id, b.user_id, u.username, u.full_name, b.equipment_id, e.name as equipment_name,
                b.group_id, g.name as group_name, b.quantity, b.start_date, b.end_date,
                b.purpose, b.status, b.permission_type, b.created_at, b.updated_at
            FROM bookings b
            LEFT JOIN users u ON b.user_id = u.id
            LEFT JOIN equipment e ON b.equipment_id = e.id
            LEFT JOIN equipment_groups g ON b.group_id = g.id
            ORDER BY b.created_at DESC
            "#,
            )
            .fetch_all(&state.db.pool)
            .await?
        } else {
            // Обычные пользователи видят только свои бронирования
            sqlx::query_as::<sqlx::Postgres, _>(
                r#"
            SELECT
                b.id, b.user_id, u.username, u.full_name, b.equipment_id, e.name as equipment_name,
                b.group_id, g.name as group_name, b.quantity, b.start_date, b.end_date,
                b.purpose, b.status, b.permission_type, b.created_at, b.updated_at
            FROM bookings b
            LEFT JOIN users u ON b.user_id = u.id
            LEFT JOIN equipment e ON b.equipment_id = e.id
            LEFT JOIN equipment_groups g ON b.group_id = g.id
            WHERE b.user_id = $1
            ORDER BY b.created_at DESC
            "#,
            )
            .bind(user_id)
            .fetch_all(&state.db.pool)
            .await?
        };

    Ok(HttpResponse::Ok().json(bookings))
}

pub async fn get_booking(
    state: web::Data<AppState>,
    claims: Claims,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let booking_id = path.into_inner();
    let user_id = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

    let booking: Option<BookingWithDetails> = sqlx::query_as::<sqlx::Postgres, _>(
        r#"
        SELECT
            b.id, b.user_id, u.username, u.full_name, b.equipment_id, e.name as equipment_name,
            b.group_id, g.name as group_name, b.quantity, b.start_date, b.end_date,
            b.purpose, b.status, b.permission_type, b.created_at, b.updated_at
        FROM bookings b
        LEFT JOIN users u ON b.user_id = u.id
        LEFT JOIN equipment e ON b.equipment_id = e.id
        LEFT JOIN equipment_groups g ON b.group_id = g.id
        WHERE b.id = $1
        "#,
    )
    .bind(booking_id)
    .fetch_optional(&state.db.pool)
    .await?;

    let booking = booking.ok_or_else(|| AppError::NotFound("Booking not found".to_string()))?;

    // Проверяем права доступа
    if claims.role != "admin" && claims.role != "responsible" && booking.user_id != user_id {
        return Err(AppError::Unauthorized("Access denied".to_string()));
    }

    Ok(HttpResponse::Ok().json(booking))
}

pub async fn update_booking(
    state: web::Data<AppState>,
    claims: Claims,
    path: web::Path<Uuid>,
    req: web::Json<UpdateBookingRequest>,
) -> Result<HttpResponse, AppError> {
    let booking_id = path.into_inner();
    let current_user_id = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

    if req.status.is_some() {
        return Err(AppError::BadRequest(
            "Статус бронирования меняется только через отдельные действия: approve, reject, return или cancel".to_string(),
        ));
    }

    let booking: Booking = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT id, user_id, equipment_id, group_id, quantity, start_date, end_date, purpose, status, permission_type, created_at, updated_at
         FROM bookings WHERE id = $1"
    )
    .bind(booking_id)
    .fetch_optional(&state.db.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Booking not found".to_string()))?;

    if booking.user_id != current_user_id && claims.role != "admin" && claims.role != "responsible"
    {
        return Err(AppError::Unauthorized("Access denied".to_string()));
    }

    if booking.status != booking_service::STATUS_PENDING {
        return Err(AppError::BadRequest(
            "Даты можно менять только у бронирования в статусе pending".to_string(),
        ));
    }

    let start_date = req.start_date.unwrap_or(booking.start_date);
    let end_date = req.end_date.unwrap_or(booking.end_date);

    if start_date >= end_date {
        return Err(AppError::BadRequest(
            "Дата начала должна быть раньше даты окончания".to_string(),
        ));
    }

    let updated_booking: Booking = sqlx::query_as::<sqlx::Postgres, _>(
        "UPDATE bookings
         SET start_date = $1, end_date = $2, updated_at = CURRENT_TIMESTAMP
         WHERE id = $3
         RETURNING id, user_id, equipment_id, group_id, quantity, start_date, end_date, purpose, status, permission_type, created_at, updated_at"
    )
    .bind(start_date)
    .bind(end_date)
    .bind(booking_id)
    .fetch_one(&state.db.pool)
    .await?;

    booking_events::record_activity(
        &state.db.pool,
        current_user_id,
        booking_id,
        "update_booking_dates",
        serde_json::json!({"start_date": start_date, "end_date": end_date}),
    )
    .await;

    Ok(HttpResponse::Ok().json(updated_booking))
}
pub async fn approve_booking(
    state: web::Data<AppState>,
    claims: Claims,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    AuthService::require_any_role(&claims, &["admin", "responsible"])?;

    let booking_id = path.into_inner();
    let current_user_id = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

    let booking =
        booking_service::approve_booking(&state.db.pool, booking_id, current_user_id).await?;

    // Создаём уведомление
    if let Some(equipment_id) = booking.equipment_id {
        // Получаем информацию об оборудовании для уведомления
        let equipment_name: Option<(String,)> =
            sqlx::query_as::<sqlx::Postgres, (String,)>("SELECT name FROM equipment WHERE id = $1")
                .bind(equipment_id)
                .fetch_optional(&state.db.pool)
                .await
                .ok()
                .flatten();

        let equipment_name_str = equipment_name
            .map(|(name,)| name)
            .unwrap_or_else(|| "Оборудование".to_string());

        // Форматируем даты для уведомления
        let start_date_str = booking.start_date.format("%d.%m.%Y %H:%M").to_string();
        let end_date_str = booking.end_date.format("%d.%m.%Y %H:%M").to_string();

        // Улучшенное тело уведомления с подробной информацией
        let notification_message = format!(
            "Оборудование: {}\nКоличество: {} шт.\nПериод: с {} по {}",
            equipment_name_str, booking.quantity, start_date_str, end_date_str
        );

        // Создаём уведомление для пользователя
        booking_events::notify_user(
            &state.db.pool,
            booking.user_id,
            "Бронирование одобрено",
            &notification_message,
            "booking_approved",
        )
        .await;

        // Отправляем push-уведомление пользователю
        let mut data = serde_json::Map::new();
        data.insert(
            "type".to_string(),
            serde_json::Value::String("notification".to_string()),
        );
        data.insert(
            "booking_id".to_string(),
            serde_json::Value::String(booking_id.to_string()),
        );
        data.insert(
            "equipment_id".to_string(),
            serde_json::Value::String(equipment_id.to_string()),
        );
        data.insert(
            "equipment_name".to_string(),
            serde_json::Value::String(equipment_name_str.clone()),
        );
        data.insert(
            "quantity".to_string(),
            serde_json::Value::Number(serde_json::Number::from(booking.quantity)),
        );
        data.insert(
            "start_date".to_string(),
            serde_json::Value::String(booking.start_date.to_rfc3339()),
        );
        data.insert(
            "end_date".to_string(),
            serde_json::Value::String(booking.end_date.to_rfc3339()),
        );
        data.insert(
            "status".to_string(),
            serde_json::Value::String("approved".to_string()),
        );

        // fire-and-forget, не блокируем ответ
        let state_clone = state.get_ref().clone();
        let user_id_for_push = booking.user_id;
        let title = "Бронирование одобрено".to_string();
        let body = notification_message.clone();
        tokio::spawn(async move {
            send_push_to_user(&state_clone, user_id_for_push, &title, &body, data).await;
        });
    }

    // Логирование
    booking_events::record_activity(
        &state.db.pool,
        current_user_id,
        booking_id,
        "approve_booking",
        serde_json::json!({"status": "approved"}),
    )
    .await;
    Ok(HttpResponse::Ok().json(booking))
}

pub async fn reject_booking(
    state: web::Data<AppState>,
    claims: Claims,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    AuthService::require_any_role(&claims, &["admin", "responsible"])?;

    let booking_id = path.into_inner();
    let current_user_id = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

    let booking = booking_service::reject_booking(&state.db.pool, booking_id).await?;

    // Создаём уведомление
    if let Some(equipment_id) = booking.equipment_id {
        // Получаем информацию об оборудовании для уведомления
        let equipment_name: Option<(String,)> =
            sqlx::query_as::<sqlx::Postgres, (String,)>("SELECT name FROM equipment WHERE id = $1")
                .bind(equipment_id)
                .fetch_optional(&state.db.pool)
                .await
                .ok()
                .flatten();

        let equipment_name_str = equipment_name
            .map(|(name,)| name)
            .unwrap_or_else(|| "Оборудование".to_string());

        // Форматируем даты для уведомления
        let start_date_str = booking.start_date.format("%d.%m.%Y %H:%M").to_string();
        let end_date_str = booking.end_date.format("%d.%m.%Y %H:%M").to_string();

        // Улучшенное тело уведомления с подробной информацией
        let notification_message = format!(
            "Оборудование: {}\nКоличество: {} шт.\nПериод: с {} по {}",
            equipment_name_str, booking.quantity, start_date_str, end_date_str
        );

        // Создаём уведомление для пользователя
        booking_events::notify_user(
            &state.db.pool,
            booking.user_id,
            "Бронирование отклонено",
            &notification_message,
            "booking_rejected",
        )
        .await;

        // Отправляем push-уведомление пользователю
        let mut data = serde_json::Map::new();
        data.insert(
            "type".to_string(),
            serde_json::Value::String("notification".to_string()),
        );
        data.insert(
            "booking_id".to_string(),
            serde_json::Value::String(booking_id.to_string()),
        );
        data.insert(
            "equipment_id".to_string(),
            serde_json::Value::String(equipment_id.to_string()),
        );
        data.insert(
            "equipment_name".to_string(),
            serde_json::Value::String(equipment_name_str.clone()),
        );
        data.insert(
            "quantity".to_string(),
            serde_json::Value::Number(serde_json::Number::from(booking.quantity)),
        );
        data.insert(
            "start_date".to_string(),
            serde_json::Value::String(booking.start_date.to_rfc3339()),
        );
        data.insert(
            "end_date".to_string(),
            serde_json::Value::String(booking.end_date.to_rfc3339()),
        );
        data.insert(
            "status".to_string(),
            serde_json::Value::String("rejected".to_string()),
        );

        let state_clone = state.get_ref().clone();
        let user_id_for_push = booking.user_id;
        let title = "Бронирование отклонено".to_string();
        let body = notification_message.clone();
        tokio::spawn(async move {
            send_push_to_user(&state_clone, user_id_for_push, &title, &body, data).await;
        });
    }

    // Логирование
    booking_events::record_activity(
        &state.db.pool,
        current_user_id,
        booking_id,
        "reject_booking",
        serde_json::json!({"status": "rejected"}),
    )
    .await;
    Ok(HttpResponse::Ok().json(booking))
}

pub async fn confirm_booking_return(
    state: web::Data<AppState>,
    claims: Claims,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    AuthService::require_any_role(&claims, &["admin", "responsible"])?;

    let booking_id = path.into_inner();
    let current_user_id = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

    let booking = booking_service::confirm_return(&state.db.pool, booking_id).await?;

    booking_events::record_activity(
        &state.db.pool,
        current_user_id,
        booking_id,
        "confirm_booking_return",
        serde_json::json!({"status": "returned"}),
    )
    .await;

    booking_events::notify_user(
        &state.db.pool,
        booking.user_id,
        "Возврат оборудования подтвержден",
        "Оборудование возвращено и снова доступно для бронирования.",
        "booking_returned",
    )
    .await;

    Ok(HttpResponse::Ok().json(booking))
}
pub async fn delete_booking(
    state: web::Data<AppState>,
    claims: Claims,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let booking_id = path.into_inner();
    cancel_booking_for_claims(&state, &claims, booking_id).await?;

    Ok(HttpResponse::NoContent().finish())
}

pub async fn cancel_booking(
    state: web::Data<AppState>,
    claims: Claims,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let booking_id = path.into_inner();
    let booking = cancel_booking_for_claims(&state, &claims, booking_id).await?;

    Ok(HttpResponse::Ok().json(booking))
}

async fn cancel_booking_for_claims(
    state: &web::Data<AppState>,
    claims: &Claims,
    booking_id: Uuid,
) -> Result<Booking, AppError> {
    let user_id = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

    let booking: Booking = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT id, user_id, equipment_id, group_id, quantity, start_date, end_date, purpose, status, permission_type, created_at, updated_at
         FROM bookings WHERE id = $1"
    )
    .bind(booking_id)
    .fetch_optional(&state.db.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Booking not found".to_string()))?;

    if booking.user_id != user_id && claims.role != "admin" && claims.role != "responsible" {
        return Err(AppError::Unauthorized("Access denied".to_string()));
    }

    if booking.status == "approved" && claims.role != "admin" && claims.role != "responsible" {
        return Err(AppError::BadRequest(
            "Cannot delete approved booking".to_string(),
        ));
    }

    let booking = booking_service::cancel_booking(&state.db.pool, booking_id).await?;

    booking_events::record_activity(
        &state.db.pool,
        user_id,
        booking_id,
        "cancel_booking",
        serde_json::json!({"status": "cancelled"}),
    )
    .await;

    Ok(booking)
}
