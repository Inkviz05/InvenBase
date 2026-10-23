use serde_json::Value;
use sqlx::{PgPool, Postgres};
use uuid::Uuid;

use super::notification_outbox;

pub async fn notify_reviewers_about_booking_request(
    pool: &PgPool,
    title: &str,
    message: &str,
    data: serde_json::Map<String, Value>,
) -> Result<usize, sqlx::Error> {
    let reviewer_ids: Vec<(Uuid,)> =
        sqlx::query_as("SELECT id FROM users WHERE role IN ('admin', 'responsible')")
            .fetch_all(pool)
            .await?;

    for (reviewer_id,) in &reviewer_ids {
        notification_outbox::create_notification_with_push(
            pool,
            *reviewer_id,
            title,
            message,
            "booking_request",
            data.clone(),
            &[],
        )
        .await?;
    }
    Ok(reviewer_ids.len())
}

pub async fn notify_user(
    pool: &PgPool,
    user_id: Uuid,
    title: &str,
    message: &str,
    notification_type: &str,
    data: serde_json::Map<String, Value>,
) -> Result<Uuid, sqlx::Error> {
    notification_outbox::create_notification_with_push(
        pool,
        user_id,
        title,
        message,
        notification_type,
        data,
        &[],
    )
    .await
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
