use chrono::Utc;
use crate::handlers::push::send_push_to_user;
use crate::app_state::AppState;
use std::sync::Arc;
use tokio::time::{interval, Duration};

/// Фоновая задача для проверки истечения времени бронирований
pub async fn start_booking_expiration_checker(state: Arc<AppState>) {
    // Проверяем каждые 5 минут для более быстрой реакции на истекшие бронирования
    let mut interval = interval(Duration::from_secs(60)); // 5 минут
    
    log::info!("Запущена фоновая задача проверки истечения времени бронирований (интервал: 5 минут)");
    
    // Выполняем первую проверку сразу при запуске
    if let Err(e) = check_expired_bookings(&state).await {
        log::error!("Ошибка при первой проверке истекших бронирований: {:?}", e);
    }
    
    loop {
        interval.tick().await;
        
        if let Err(e) = check_expired_bookings(&state).await {
            log::error!("Ошибка при проверке истекших бронирований: {:?}", e);
        }
    }
}

/// Проверяет истекшие бронирования и отправляет уведомления
async fn check_expired_bookings(state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    let now = Utc::now();
    
    // Находим все одобренные бронирования, у которых истекло время
    // Проверяем только те, которые истекли не более 2 часов назад (чтобы не обрабатывать старые повторно)
    // Исключаем те, для которых уже были отправлены уведомления за последние 10 минут
    let expired_bookings: Vec<(uuid::Uuid, uuid::Uuid, Option<uuid::Uuid>, i32, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)> = sqlx::query_as::<sqlx::Postgres, _>(
        r#"
        SELECT b.id, b.user_id, b.equipment_id, b.quantity, b.start_date, b.end_date
        FROM bookings b
        WHERE b.status = 'approved' 
          AND b.end_date < $1
          AND b.end_date > $1 - INTERVAL '2 hours'
          AND NOT EXISTS (
              SELECT 1 
              FROM notifications n
              WHERE n.user_id = b.user_id
                AND n.notification_type = 'booking_expired'
                AND n.created_at > $1 - INTERVAL '10 minutes'
          )
        ORDER BY b.end_date ASC
        "#
    )
    .bind(now)
    .fetch_all(&state.db.pool)
    .await?;

    log::info!("Найдено {} истекших бронирований", expired_bookings.len());

    for (booking_id, user_id, equipment_id, quantity, start_date, end_date) in expired_bookings {
        // Обновляем статус бронирования на "completed"
        sqlx::query::<sqlx::Postgres>(
            "UPDATE bookings SET status = 'completed', updated_at = CURRENT_TIMESTAMP WHERE id = $1"
        )
        .bind(booking_id)
        .execute(&state.db.pool)
        .await?;

        // Возвращаем оборудование
        if let Some(eq_id) = equipment_id {
            sqlx::query::<sqlx::Postgres>(
                "UPDATE equipment SET available_quantity = available_quantity + $1 WHERE id = $2"
            )
            .bind(quantity)
            .bind(eq_id)
            .execute(&state.db.pool)
            .await?;

            // Получаем информацию об оборудовании для уведомления
            let equipment_name: Option<(String,)> = sqlx::query_as::<sqlx::Postgres, (String,)>(
                "SELECT name FROM equipment WHERE id = $1"
            )
            .bind(eq_id)
            .fetch_optional(&state.db.pool)
            .await
            .ok()
            .flatten();

            let equipment_name_str = equipment_name
                .map(|(name,)| name)
                .unwrap_or_else(|| "Оборудование".to_string());

            // Форматируем даты для уведомления
            let start_date_str = start_date.format("%d.%m.%Y %H:%M").to_string();
            let end_date_str = end_date.format("%d.%m.%Y %H:%M").to_string();

            let notification_message = format!(
                "Время бронирования истекло.\nОборудование: {}\nКоличество: {} шт.\nПериод: с {} по {}",
                equipment_name_str,
                quantity,
                start_date_str,
                end_date_str
            );

            // Создаём уведомление для пользователя
            let _ = sqlx::query::<sqlx::Postgres>(
                "INSERT INTO notifications (id, user_id, title, message, notification_type)
                 VALUES (gen_random_uuid(), $1, 'Время бронирования истекло', $2, 'booking_expired')"
            )
            .bind(user_id)
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
                serde_json::Value::String(eq_id.to_string()),
            );
            data.insert(
                "equipment_name".to_string(),
                serde_json::Value::String(equipment_name_str.clone()),
            );
            data.insert(
                "quantity".to_string(),
                serde_json::Value::Number(serde_json::Number::from(quantity)),
            );
            data.insert(
                "start_date".to_string(),
                serde_json::Value::String(start_date.to_rfc3339()),
            );
            data.insert(
                "end_date".to_string(),
                serde_json::Value::String(end_date.to_rfc3339()),
            );
            data.insert(
                "status".to_string(),
                serde_json::Value::String("completed".to_string()),
            );

            let state_clone = state.clone();
            let user_id_for_push = user_id;
            let title = "Время бронирования истекло".to_string();
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

            log::info!("Отправлено уведомление об истечении времени бронирования {} пользователю {}", booking_id, user_id);
        }
    }

    Ok(())
}
