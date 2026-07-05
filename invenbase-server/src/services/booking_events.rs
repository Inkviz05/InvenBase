use serde_json::Value;
use sqlx::{PgPool, Postgres};
use uuid::Uuid;

pub async fn notify_reviewers_about_booking_request(pool: &PgPool, title: &str, message: &str) {
    let _ = sqlx::query::<Postgres>(
        "INSERT INTO notifications (user_id, title, message, notification_type)
         SELECT id, $1, $2, 'booking_request'
         FROM users WHERE role IN ('admin', 'responsible')",
    )
    .bind(title)
    .bind(message)
    .execute(pool)
    .await;
}

pub async fn notify_user(
    pool: &PgPool,
    user_id: Uuid,
    title: &str,
    message: &str,
    notification_type: &str,
) {
    let _ = sqlx::query::<Postgres>(
        "INSERT INTO notifications (id, user_id, title, message, notification_type)
         VALUES (gen_random_uuid(), $1, $2, $3, $4)",
    )
    .bind(user_id)
    .bind(title)
    .bind(message)
    .bind(notification_type)
    .execute(pool)
    .await;
}

pub async fn record_activity(
    pool: &PgPool,
    actor_id: Uuid,
    booking_id: Uuid,
    action: &str,
    details: Value,
) {
    let _ = sqlx::query::<Postgres>(
        "INSERT INTO activity_logs (user_id, action, entity_type, entity_id, details)
         VALUES ($1, $2, 'booking', $3, $4)",
    )
    .bind(actor_id)
    .bind(action)
    .bind(booking_id)
    .bind(details)
    .execute(pool)
    .await;
}
