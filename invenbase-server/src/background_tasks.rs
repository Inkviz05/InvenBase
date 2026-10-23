use crate::app_state::AppState;
use crate::services::bookings as booking_service;
use crate::services::notification_outbox;
use chrono::Utc;
use std::sync::Arc;
use tokio::time::{interval, Duration};

pub async fn start_notification_outbox_worker(state: Arc<AppState>) {
    let mut ticker = interval(Duration::from_secs(5));
    log::info!("Started notification outbox worker");

    loop {
        ticker.tick().await;
        for _ in 0..20 {
            let message = match notification_outbox::claim_next(&state.db.pool).await {
                Ok(Some(message)) => message,
                Ok(None) => break,
                Err(error) => {
                    log::error!("Failed to claim notification outbox message: {:?}", error);
                    break;
                }
            };

            match crate::handlers::push::deliver_outbox_push(&state, &message).await {
                Ok(()) => {
                    if let Err(error) =
                        notification_outbox::mark_delivered(&state.db.pool, message.id).await
                    {
                        log::error!(
                            "Failed to complete outbox message {}: {:?}",
                            message.id,
                            error
                        );
                    }
                }
                Err(error) => {
                    log::warn!(
                        "Outbox delivery {} failed on attempt {}/{}: {}",
                        message.id,
                        message.attempts,
                        message.max_attempts,
                        error
                    );
                    if let Err(database_error) =
                        notification_outbox::mark_delivery_failed(&state.db.pool, &message, &error)
                            .await
                    {
                        log::error!(
                            "Failed to record outbox error for {}: {:?}",
                            message.id,
                            database_error
                        );
                    }
                }
            }
        }
    }
}

pub async fn start_booking_expiration_checker(state: Arc<AppState>) {
    let mut interval = interval(Duration::from_secs(120));

    log::info!("Started booking expiration checker");

    if let Err(e) = check_expired_bookings(&state).await {
        log::error!("Failed to check expired bookings on startup: {:?}", e);
    }

    loop {
        interval.tick().await;

        if let Err(e) = check_expired_bookings(&state).await {
            log::error!("Failed to check expired bookings: {:?}", e);
        }
    }
}

type ExpiredBookingRow = (
    uuid::Uuid,
    uuid::Uuid,
    Option<uuid::Uuid>,
    i32,
    chrono::DateTime<chrono::Utc>,
    chrono::DateTime<chrono::Utc>,
    String,
);

async fn check_expired_bookings(state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    let now = Utc::now();

    let expired_bookings: Vec<ExpiredBookingRow> =
        sqlx::query_as::<sqlx::Postgres, ExpiredBookingRow>(
            r#"
        SELECT b.id, b.user_id, b.equipment_id, b.quantity, b.start_date, b.end_date, b.status
        FROM bookings b
        WHERE b.status IN ('approved', 'pending')
          AND b.end_date < $1
        ORDER BY b.end_date ASC
        "#,
        )
        .bind(now)
        .fetch_all(&state.db.pool)
        .await?;

    log::info!("Found {} expired bookings", expired_bookings.len());

    for (booking_id, user_id, equipment_id, quantity, start_date, end_date, status) in
        expired_bookings
    {
        let new_status = if status == booking_service::STATUS_APPROVED {
            booking_service::mark_awaiting_return(&state.db.pool, booking_id).await?;
            booking_service::STATUS_AWAITING_RETURN
        } else {
            booking_service::expire_pending_booking(&state.db.pool, booking_id).await?;
            booking_service::STATUS_EXPIRED
        };

        let equipment_name_str = if let Some(eq_id) = equipment_id {
            let name: Option<(String,)> = sqlx::query_as::<sqlx::Postgres, (String,)>(
                "SELECT name FROM equipment WHERE id = $1",
            )
            .bind(eq_id)
            .fetch_optional(&state.db.pool)
            .await
            .ok()
            .flatten();
            name.map(|(n,)| n)
                .unwrap_or_else(|| "Оборудование".to_string())
        } else {
            "Оборудование".to_string()
        };

        let start_date_str = start_date.format("%d.%m.%Y %H:%M").to_string();
        let end_date_str = end_date.format("%d.%m.%Y %H:%M").to_string();

        let (title, notification_type, notification_message) = if status
            == booking_service::STATUS_APPROVED
        {
            (
                "Нужно подтвердить возврат оборудования".to_string(),
                "booking_return_due",
                format!(
                    "Период бронирования завершился.\nОборудование: {}\nКоличество: {} шт.\nПериод: с {} по {}\nПосле фактического возврата подтвердите его в системе.",
                    equipment_name_str, quantity, start_date_str, end_date_str
                ),
            )
        } else {
            (
                "Бронирование истекло".to_string(),
                "booking_expired",
                format!(
                    "Бронирование истекло и не было одобрено.\nОборудование: {}\nКоличество: {} шт.\nПериод: с {} по {}",
                    equipment_name_str, quantity, start_date_str, end_date_str
                ),
            )
        };

        if status == booking_service::STATUS_APPROVED {
            let mut data = serde_json::Map::new();
            data.insert(
                "type".to_string(),
                serde_json::Value::String("notification".to_string()),
            );
            data.insert(
                "booking_id".to_string(),
                serde_json::Value::String(booking_id.to_string()),
            );
            if let Some(eq_id) = equipment_id {
                data.insert(
                    "equipment_id".to_string(),
                    serde_json::Value::String(eq_id.to_string()),
                );
            }
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
                serde_json::Value::String(new_status.to_string()),
            );

            notification_outbox::create_notification_with_push(
                &state.db.pool,
                user_id,
                &title,
                &notification_message,
                notification_type,
                data,
                &[],
            )
            .await?;
        } else {
            sqlx::query::<sqlx::Postgres>(
                "INSERT INTO notifications (id, user_id, title, message, notification_type)
                 VALUES (gen_random_uuid(), $1, $2, $3, $4)",
            )
            .bind(user_id)
            .bind(&title)
            .bind(&notification_message)
            .bind(notification_type)
            .execute(&state.db.pool)
            .await?;
        }

        log::info!("Expired booking {} moved to {}", booking_id, new_status);
    }

    Ok(())
}
