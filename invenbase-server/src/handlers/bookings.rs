use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::models::{Booking, BookingWithDetails, CreateBookingRequest, CreateBulkBookingsRequest, UpdateBookingRequest};
use crate::auth::{AuthService, Claims};
use crate::errors::AppError;
use crate::app_state::AppState;
use crate::handlers::push::send_push_to_user;

pub async fn create_booking(
    state: web::Data<AppState>,
    claims: Claims,
    req: web::Json<CreateBookingRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;
    let mut tx = state.db.pool.begin().await?;

    // Проверяем доступность оборудования
    if let Some(equipment_id) = req.equipment_id {
        let available: Option<(i32,)> = sqlx::query_as::<sqlx::Postgres, _>(
            "SELECT available_quantity FROM equipment WHERE id = $1 FOR UPDATE"
        )
        .bind(equipment_id)
        .fetch_optional(&mut *tx)
        .await?;

        if let Some((available_qty,)) = available {
            if available_qty < req.quantity {
                return Err(AppError::BadRequest(
                    format!("Недостаточно оборудования. Доступно: {}", available_qty)
                ));
            }
        } else {
            return Err(AppError::NotFound("Equipment not found".to_string()));
        }
    }

    let booking_id = uuid::Uuid::new_v4();
    let permission_type = req.permission_type.as_deref().unwrap_or("internal").to_string();
    sqlx::query::<sqlx::Postgres>(
        "INSERT INTO bookings (id, user_id, equipment_id, group_id, quantity, start_date, end_date, purpose, permission_type, status)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'pending')"
    )
    .bind(booking_id)
    .bind(user_id)
    .bind(&req.equipment_id)
    .bind(&req.group_id)
    .bind(req.quantity)
    .bind(req.start_date)
    .bind(req.end_date)
    .bind(&req.purpose)
    .bind(&permission_type)
    .execute(&mut *tx)
    .await?;

    // Обновляем доступное количество оборудования
    if let Some(equipment_id) = req.equipment_id {
        sqlx::query::<sqlx::Postgres>(
            "UPDATE equipment SET available_quantity = available_quantity - $1 WHERE id = $2"
        )
        .bind(req.quantity)
        .bind(equipment_id)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    let notification_message = format!("Пользователь {} создал новое бронирование", claims.username);

    // Создаём уведомление в БД для каждого администратора и ответственного
    let _ = sqlx::query::<sqlx::Postgres>(
        "INSERT INTO notifications (user_id, title, message, notification_type)
         SELECT id, 'Новое бронирование', $1, 'booking_request'
         FROM users WHERE role IN ('admin', 'responsible')"
    )
    .bind(&notification_message)
    .execute(&state.db.pool)
    .await;

    // Отправляем push-уведомление администраторам и ответственным
    let admin_user_ids: Vec<(Uuid,)> = sqlx::query_as::<sqlx::Postgres, (Uuid,)>(
        "SELECT id FROM users WHERE role IN ('admin', 'responsible')"
    )
    .fetch_all(&state.db.pool)
    .await
    .unwrap_or_default();
    log::info!("Booking created: sending push to {} admin/responsible user(s) about new booking from {}", admin_user_ids.len(), claims.username);
    let mut data = serde_json::Map::new();
    data.insert("type".to_string(), serde_json::Value::String("booking_request".to_string()));
    data.insert("booking_id".to_string(), serde_json::Value::String(booking_id.to_string()));
    for (admin_id,) in admin_user_ids {
        send_push_to_user(
            state.get_ref(),
            admin_id,
            "Новое бронирование",
            &notification_message,
            data.clone(),
        ).await;
    }

    // Логирование
    let _ = sqlx::query::<sqlx::Postgres>(
        "INSERT INTO activity_logs (user_id, action, entity_type, entity_id, details)
         VALUES ($1, 'create_booking', 'booking', $2, $3)"
    )
    .bind(user_id)
    .bind(booking_id)
    .bind(serde_json::json!({"equipment_id": req.equipment_id, "quantity": req.quantity}))
    .execute(&state.db.pool)
    .await;

    let booking: Booking = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT id, user_id, equipment_id, group_id, quantity, start_date, end_date, purpose, status, permission_type, created_at, updated_at 
         FROM bookings WHERE id = $1"
    )
    .bind(booking_id)
    .fetch_one(&state.db.pool)
    .await?;

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

    if req.bookings.is_empty() {
        return Err(AppError::BadRequest("Список бронирований пуст".to_string()));
    }

    let mut created: Vec<Booking> = Vec::with_capacity(req.bookings.len());

    let mut tx = state.db.pool.begin().await?;

    for req_item in &req.bookings {
        if let Some(equipment_id) = req_item.equipment_id {
            let available: Option<(i32,)> = sqlx::query_as::<sqlx::Postgres, _>(
                "SELECT available_quantity FROM equipment WHERE id = $1 FOR UPDATE"
            )
            .bind(equipment_id)
            .fetch_optional(&mut *tx)
            .await?;

            if let Some((available_qty,)) = available {
                if available_qty < req_item.quantity {
                    return Err(AppError::BadRequest(
                        format!("Недостаточно оборудования. Доступно: {}", available_qty)
                    ));
                }
            } else {
                return Err(AppError::NotFound("Equipment not found".to_string()));
            }
        }

        let booking_id = uuid::Uuid::new_v4();
        let permission_type = req_item.permission_type.as_deref().unwrap_or("internal").to_string();

        sqlx::query::<sqlx::Postgres>(
            "INSERT INTO bookings (id, user_id, equipment_id, group_id, quantity, start_date, end_date, purpose, permission_type, status)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'pending')"
        )
        .bind(booking_id)
        .bind(user_id)
        .bind(&req_item.equipment_id)
        .bind(&req_item.group_id)
        .bind(req_item.quantity)
        .bind(req_item.start_date)
        .bind(req_item.end_date)
        .bind(&req_item.purpose)
        .bind(&permission_type)
        .execute(&mut *tx)
        .await?;

        if let Some(equipment_id) = req_item.equipment_id {
            sqlx::query::<sqlx::Postgres>(
                "UPDATE equipment SET available_quantity = available_quantity - $1 WHERE id = $2"
            )
            .bind(req_item.quantity)
            .bind(equipment_id)
            .execute(&mut *tx)
            .await?;
        }

        let booking: Booking = sqlx::query_as::<sqlx::Postgres, _>(
            "SELECT id, user_id, equipment_id, group_id, quantity, start_date, end_date, purpose, status, permission_type, created_at, updated_at 
             FROM bookings WHERE id = $1"
        )
        .bind(booking_id)
        .fetch_one(&mut *tx)
        .await?;
        created.push(booking);
    }

    tx.commit().await?;

    let count = created.len();
    let notification_message = format!(
        "Пользователь {} создал {} бронирований",
        claims.username,
        count
    );

    let _ = sqlx::query::<sqlx::Postgres>(
        "INSERT INTO notifications (user_id, title, message, notification_type)
         SELECT id, 'Новые бронирования', $1, 'booking_request'
         FROM users WHERE role IN ('admin', 'responsible')"
    )
    .bind(&notification_message)
    .execute(&state.db.pool)
    .await;

    let admin_user_ids: Vec<(Uuid,)> = sqlx::query_as::<sqlx::Postgres, (Uuid,)>(
        "SELECT id FROM users WHERE role IN ('admin', 'responsible')"
    )
    .fetch_all(&state.db.pool)
    .await
    .unwrap_or_default();

    let mut data = serde_json::Map::new();
    data.insert("type".to_string(), serde_json::Value::String("booking_request".to_string()));
    data.insert("count".to_string(), serde_json::Value::Number(serde_json::Number::from(count)));
    for (admin_id,) in admin_user_ids {
        send_push_to_user(
            state.get_ref(),
            admin_id,
            "Новые бронирования",
            &notification_message,
            data.clone(),
        ).await;
    }

    log::info!("Bulk booking: {} bookings created by {}, one notification sent", count, claims.username);

    Ok(HttpResponse::Created().json(created))
}

pub async fn get_bookings(
    state: web::Data<AppState>,
    claims: Claims,
) -> Result<HttpResponse, AppError> {
    let user_id = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

    let bookings: Vec<BookingWithDetails> = if claims.role == "admin" || claims.role == "responsible" {
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
            "#
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
            "#
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
        "#
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

    // Получаем текущее бронирование
    let booking: Option<Booking> = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT id, user_id, equipment_id, group_id, quantity, start_date, end_date, purpose, status, permission_type, created_at, updated_at 
         FROM bookings WHERE id = $1"
    )
    .bind(booking_id)
    .fetch_optional(&state.db.pool)
    .await?;

    let booking = booking.ok_or_else(|| AppError::NotFound("Booking not found".to_string()))?;

    // Только админы и ответственные могут менять статус
    if req.status.is_some() && claims.role != "admin" && claims.role != "responsible" {
        return Err(AppError::Unauthorized("Only admins can change booking status".to_string()));
    }

    // Обновляем статус
    if let Some(status) = &req.status {
        let old_status = booking.status.clone();
        
        sqlx::query::<sqlx::Postgres>("UPDATE bookings SET status = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2")
            .bind(status)
            .bind(booking_id)
            .execute(&state.db.pool)
            .await?;

        // Если бронирование отменено или отклонено, возвращаем оборудование
        if (old_status == "approved" || old_status == "pending") && (status == "cancelled" || status == "rejected") {
            if let Some(equipment_id) = booking.equipment_id {
                sqlx::query::<sqlx::Postgres>(
                    "UPDATE equipment SET available_quantity = available_quantity + $1 WHERE id = $2"
                )
                .bind(booking.quantity)
                .bind(equipment_id)
                .execute(&state.db.pool)
                .await?;
            }
        }

        // Отправляем уведомление при отмене бронирования
        if status == "cancelled" && old_status != "cancelled" {
            if let Some(equipment_id) = booking.equipment_id {
                // Получаем информацию об оборудовании для уведомления
                let equipment_name: Option<(String,)> = sqlx::query_as::<sqlx::Postgres, (String,)>(
                    "SELECT name FROM equipment WHERE id = $1"
                )
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

                let notification_message = format!(
                    "Оборудование: {}\nКоличество: {} шт.\nПериод: с {} по {}",
                    equipment_name_str,
                    booking.quantity,
                    start_date_str,
                    end_date_str
                );

                // Создаём уведомление для пользователя
                let _ = sqlx::query::<sqlx::Postgres>(
                    "INSERT INTO notifications (id, user_id, title, message, notification_type)
                     VALUES (gen_random_uuid(), $1, 'Бронирование отменено', $2, 'booking_cancelled')"
                )
                .bind(booking.user_id)
                .bind(&notification_message)
                .execute(&state.db.pool)
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
                    serde_json::Value::String("cancelled".to_string()),
                );

                let state_clone = state.get_ref().clone();
                let user_id_for_push = booking.user_id;
                let title = "Бронирование отменено".to_string();
                let body = notification_message.clone();
                tokio::spawn(async move {
                    send_push_to_user(
                        &state_clone,
                        user_id_for_push,
                        &title,
                        &body,
                        data,
                    )
                    .await;
                });
            }
        }

        // Если бронирование одобрено, создаём разрешение
        if status == "approved" && old_status == "pending" {
            let _ = sqlx::query::<sqlx::Postgres>(
                "INSERT INTO permissions (booking_id, equipment_id, permission_type, issued_by, status)
                 VALUES ($1, $2, $3, $4, 'active')"
            )
            .bind(booking_id)
            .bind(booking.equipment_id)
            .bind(booking.permission_type.clone())
            .bind(current_user_id)
            .execute(&state.db.pool)
            .await;
        }
    }

    // Обновляем даты
    if req.start_date.is_some() || req.end_date.is_some() {
        let start_date = req.start_date.unwrap_or(booking.start_date);
        let end_date = req.end_date.unwrap_or(booking.end_date);
        
        sqlx::query::<sqlx::Postgres>("UPDATE bookings SET start_date = $1, end_date = $2, updated_at = CURRENT_TIMESTAMP WHERE id = $3")
            .bind(start_date)
            .bind(end_date)
            .bind(booking_id)
            .execute(&state.db.pool)
            .await?;
    }

    let updated_booking: Booking = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT id, user_id, equipment_id, group_id, quantity, start_date, end_date, purpose, status, permission_type, created_at, updated_at 
         FROM bookings WHERE id = $1"
    )
    .bind(booking_id)
    .fetch_one(&state.db.pool)
    .await?;

    // Логирование
    let _ = sqlx::query::<sqlx::Postgres>(
        "INSERT INTO activity_logs (user_id, action, entity_type, entity_id, details)
         VALUES ($1, 'update_booking', 'booking', $2, $3)"
    )
    .bind(current_user_id)
    .bind(booking_id)
    .bind(serde_json::json!({"status": req.status, "updated": true}))
    .execute(&state.db.pool)
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

    let booking: Option<Booking> = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT id, user_id, equipment_id, group_id, quantity, start_date, end_date, purpose, status, permission_type, created_at, updated_at 
         FROM bookings WHERE id = $1"
    )
    .bind(booking_id)
    .fetch_optional(&state.db.pool)
    .await?;

    let booking = booking.ok_or_else(|| AppError::NotFound("Booking not found".to_string()))?;

    if booking.status != "pending" {
        return Err(AppError::BadRequest("Only pending bookings can be approved".to_string()));
    }

    // Обновляем статус
    sqlx::query::<sqlx::Postgres>("UPDATE bookings SET status = 'approved', updated_at = CURRENT_TIMESTAMP WHERE id = $1")
        .bind(booking_id)
        .execute(&state.db.pool)
        .await?;

    // Создаём разрешение
    if let Some(equipment_id) = booking.equipment_id {
        // Получаем информацию об оборудовании для уведомления
        let equipment_name: Option<(String,)> = sqlx::query_as::<sqlx::Postgres, (String,)>(
            "SELECT name FROM equipment WHERE id = $1"
        )
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
            equipment_name_str,
            booking.quantity,
            start_date_str,
            end_date_str
        );

        let permission_id = uuid::Uuid::new_v4();
        let _ = sqlx::query::<sqlx::Postgres>(
            "INSERT INTO permissions (id, booking_id, equipment_id, permission_type, issued_by, status)
             VALUES ($1, $2, $3, $4, $5, 'active')"
        )
        .bind(permission_id)
        .bind(booking_id)
        .bind(equipment_id)
        .bind(booking.permission_type.clone())
        .bind(current_user_id)
        .execute(&state.db.pool)
        .await;

        // Создаём уведомление для пользователя
        let _ = sqlx::query::<sqlx::Postgres>(
            "INSERT INTO notifications (id, user_id, title, message, notification_type)
             VALUES (gen_random_uuid(), $1, 'Бронирование одобрено', $2, 'booking_approved')"
        )
        .bind(booking.user_id)
        .bind(&notification_message)
        .execute(&state.db.pool)
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
            send_push_to_user(
                &state_clone,
                user_id_for_push,
                &title,
                &body,
                data,
            )
            .await;
        });
    }

    // Логирование
    let _ = sqlx::query::<sqlx::Postgres>(
        "INSERT INTO activity_logs (user_id, action, entity_type, entity_id, details)
         VALUES ($1, 'approve_booking', 'booking', $2, $3)"
    )
    .bind(current_user_id)
    .bind(booking_id)
    .bind(serde_json::json!({"status": "approved"}))
    .execute(&state.db.pool)
    .await;

    let updated_booking: Booking = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT id, user_id, equipment_id, group_id, quantity, start_date, end_date, purpose, status, permission_type, created_at, updated_at 
         FROM bookings WHERE id = $1"
    )
    .bind(booking_id)
    .fetch_one(&state.db.pool)
    .await?;

    Ok(HttpResponse::Ok().json(updated_booking))
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

    let booking: Option<Booking> = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT id, user_id, equipment_id, group_id, quantity, start_date, end_date, purpose, status, permission_type, created_at, updated_at 
         FROM bookings WHERE id = $1"
    )
    .bind(booking_id)
    .fetch_optional(&state.db.pool)
    .await?;

    let booking = booking.ok_or_else(|| AppError::NotFound("Booking not found".to_string()))?;

    if booking.status != "pending" {
        return Err(AppError::BadRequest("Only pending bookings can be rejected".to_string()));
    }

    // Обновляем статус
    sqlx::query::<sqlx::Postgres>("UPDATE bookings SET status = 'rejected', updated_at = CURRENT_TIMESTAMP WHERE id = $1")
        .bind(booking_id)
        .execute(&state.db.pool)
        .await?;

    // Возвращаем оборудование
    if let Some(equipment_id) = booking.equipment_id {
        // Получаем информацию об оборудовании для уведомления
        let equipment_name: Option<(String,)> = sqlx::query_as::<sqlx::Postgres, (String,)>(
            "SELECT name FROM equipment WHERE id = $1"
        )
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
            equipment_name_str,
            booking.quantity,
            start_date_str,
            end_date_str
        );

        sqlx::query::<sqlx::Postgres>(
            "UPDATE equipment SET available_quantity = available_quantity + $1 WHERE id = $2"
        )
        .bind(booking.quantity)
        .bind(equipment_id)
        .execute(&state.db.pool)
        .await?;

        // Создаём уведомление для пользователя
        let _ = sqlx::query::<sqlx::Postgres>(
            "INSERT INTO notifications (id, user_id, title, message, notification_type)
             VALUES (gen_random_uuid(), $1, 'Бронирование отклонено', $2, 'booking_rejected')"
        )
        .bind(booking.user_id)
        .bind(&notification_message)
        .execute(&state.db.pool)
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
            send_push_to_user(
                &state_clone,
                user_id_for_push,
                &title,
                &body,
                data,
            )
            .await;
        });
    }

    // Логирование
    let _ = sqlx::query::<sqlx::Postgres>(
        "INSERT INTO activity_logs (user_id, action, entity_type, entity_id, details)
         VALUES ($1, 'reject_booking', 'booking', $2, $3)"
    )
    .bind(current_user_id)
    .bind(booking_id)
    .bind(serde_json::json!({"status": "rejected"}))
    .execute(&state.db.pool)
    .await;

    let updated_booking: Booking = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT id, user_id, equipment_id, group_id, quantity, start_date, end_date, purpose, status, permission_type, created_at, updated_at 
         FROM bookings WHERE id = $1"
    )
    .bind(booking_id)
    .fetch_one(&state.db.pool)
    .await?;

    Ok(HttpResponse::Ok().json(updated_booking))
}

pub async fn delete_booking(
    state: web::Data<AppState>,
    claims: Claims,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let booking_id = path.into_inner();
    let user_id = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

    // Получаем бронирование
    let booking: Option<Booking> = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT id, user_id, equipment_id, group_id, quantity, start_date, end_date, purpose, status, permission_type, created_at, updated_at 
         FROM bookings WHERE id = $1"
    )
    .bind(booking_id)
    .fetch_optional(&state.db.pool)
    .await?;

    let booking = booking.ok_or_else(|| AppError::NotFound("Booking not found".to_string()))?;

    // Пользователь может удалить только своё бронирование, если оно ещё не одобрено
    if booking.user_id != user_id && claims.role != "admin" && claims.role != "responsible" {
        return Err(AppError::Unauthorized("Access denied".to_string()));
    }

    if booking.status == "approved" && claims.role != "admin" && claims.role != "responsible" {
        return Err(AppError::BadRequest("Cannot delete approved booking".to_string()));
    }

    // Возвращаем оборудование только если бронирование всё ещё "удерживает" его
    // Т.е. доступное количество было уменьшено при создании (pending) и ещё не было возвращено
    // ни истечением срока (completed), ни отменой/отклонением
    if let Some(equipment_id) = booking.equipment_id {
        if booking.status == "pending" || booking.status == "approved" {
            sqlx::query::<sqlx::Postgres>(
                "UPDATE equipment SET available_quantity = available_quantity + $1 WHERE id = $2"
            )
            .bind(booking.quantity)
            .bind(equipment_id)
            .execute(&state.db.pool)
            .await?;
        }
    }

    sqlx::query::<sqlx::Postgres>("DELETE FROM bookings WHERE id = $1")
        .bind(booking_id)
        .execute(&state.db.pool)
        .await?;

    Ok(HttpResponse::NoContent().finish())
}

