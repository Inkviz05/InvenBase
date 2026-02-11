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

/// Запись истёкшего бронирования: id, user_id, equipment_id, quantity, start_date, end_date, status
type ExpiredBookingRow = (
    uuid::Uuid,
    uuid::Uuid,
    Option<uuid::Uuid>,
    i32,
    chrono::DateTime<chrono::Utc>,
    chrono::DateTime<chrono::Utc>,
    String,
);

/// Проверяет истекшие бронирования и возвращает оборудование в доступные
async fn check_expired_bookings(state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    let now = Utc::now();

    // Все истёкшие бронирования (approved и pending): обрабатываем все, без окна 2 часа
    let expired_bookings: Vec<ExpiredBookingRow> = sqlx::query_as::<sqlx::Postgres, ExpiredBookingRow>(
        r#"
        SELECT b.id, b.user_id, b.equipment_id, b.quantity, b.start_date, b.end_date, b.status
        FROM bookings b
        WHERE (b.status = 'approved' OR b.status = 'pending')
          AND b.end_date < $1
        ORDER BY b.end_date ASC
        "#
    )
    .bind(now)
    .fetch_all(&state.db.pool)
    .await?;

    log::info!("Найдено {} истекших бронирований", expired_bookings.len());

    for (booking_id, user_id, equipment_id, quantity, start_date, end_date, status) in expired_bookings {
        let new_status = if status == "approved" {
            "completed"
        } else {
            "cancelled"
        };

        sqlx::query::<sqlx::Postgres>(
            "UPDATE bookings SET status = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2"
        )
        .bind(new_status)
        .bind(booking_id)
        .execute(&state.db.pool)
        .await?;

        if let Some(eq_id) = equipment_id {
            sqlx::query::<sqlx::Postgres>(
                "UPDATE equipment SET available_quantity = available_quantity + $1 WHERE id = $2"
            )
            .bind(quantity)
            .bind(eq_id)
            .execute(&state.db.pool)
            .await?;
        }

        let equipment_name_str = if let Some(eq_id) = equipment_id {
            let name: Option<(String,)> = sqlx::query_as::<sqlx::Postgres, (String,)>(
                "SELECT name FROM equipment WHERE id = $1"
            )
            .bind(eq_id)
            .fetch_optional(&state.db.pool)
            .await
            .ok()
            .flatten();
            name.map(|(n,)| n).unwrap_or_else(|| "Оборудование".to_string())
        } else {
            "Оборудование".to_string()
        };

        let start_date_str = start_date.format("%d.%m.%Y %H:%M").to_string();
        let end_date_str = end_date.format("%d.%m.%Y %H:%M").to_string();

        let (title, notification_type, notification_message) = if status == "approved" {
            (
                "Время бронирования истекло".to_string(),
                "booking_expired",
                format!(
                    "Время бронирования истекло.\nОборудование: {}\nКоличество: {} шт.\nПериод: с {} по {}",
                    equipment_name_str, quantity, start_date_str, end_date_str
                ),
            )
        } else {
            (
                "Бронирование истекло".to_string(),
                "booking_expired",
                format!(
                    "Бронирование истекло (не было одобрено).\nОборудование: {}\nКоличество: {} шт.\nПериод: с {} по {}",
                    equipment_name_str, quantity, start_date_str, end_date_str
                ),
            )
        };

        let _ = sqlx::query::<sqlx::Postgres>(
            "INSERT INTO notifications (id, user_id, title, message, notification_type)
             VALUES (gen_random_uuid(), $1, $2, $3, $4)"
        )
        .bind(user_id)
        .bind(&title)
        .bind(&notification_message)
        .bind(notification_type)
        .execute(&state.db.pool)
        .await;

        if status == "approved" {
            let mut data = serde_json::Map::new();
            data.insert("type".to_string(), serde_json::Value::String("notification".to_string()));
            data.insert("booking_id".to_string(), serde_json::Value::String(booking_id.to_string()));
            if let Some(eq_id) = equipment_id {
                data.insert("equipment_id".to_string(), serde_json::Value::String(eq_id.to_string()));
            }
            data.insert("equipment_name".to_string(), serde_json::Value::String(equipment_name_str.clone()));
            data.insert("quantity".to_string(), serde_json::Value::Number(serde_json::Number::from(quantity)));
            data.insert("start_date".to_string(), serde_json::Value::String(start_date.to_rfc3339()));
            data.insert("end_date".to_string(), serde_json::Value::String(end_date.to_rfc3339()));
            data.insert("status".to_string(), serde_json::Value::String(new_status.to_string()));

            let state_clone = state.clone();
            let user_id_for_push = user_id;
            let title_push = title.clone();
            let body_push = notification_message.clone();
            tokio::spawn(async move {
                send_push_to_user(&state_clone, user_id_for_push, &title_push, &body_push, data).await;
            });
        }

        log::info!(
            "Истекшее бронирование {} переведено в {}, оборудование возвращено",
            booking_id, new_status
        );
    }

    Ok(())
}
