use chrono::{DateTime, Utc};
use serde_json::{Map, Value};
use sqlx::PgPool;
use uuid::Uuid;

pub struct OutboxMessage {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub body: String,
    pub data: Map<String, Value>,
    pub exclude_tokens: Vec<String>,
    pub attempts: i32,
    pub max_attempts: i32,
}

type OutboxRow = (Uuid, Uuid, String, String, Value, Value, i32, i32);

pub async fn enqueue(
    pool: &PgPool,
    user_id: Uuid,
    title: &str,
    body: &str,
    data: Map<String, Value>,
    exclude_tokens: &[String],
) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO notification_outbox
            (id, user_id, title, body, data, exclude_tokens)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(id)
    .bind(user_id)
    .bind(title)
    .bind(body)
    .bind(Value::Object(data))
    .bind(serde_json::to_value(exclude_tokens).unwrap_or_else(|_| Value::Array(Vec::new())))
    .execute(pool)
    .await?;

    Ok(id)
}

pub async fn claim_next(pool: &PgPool) -> Result<Option<OutboxMessage>, sqlx::Error> {
    let row = sqlx::query_as::<sqlx::Postgres, OutboxRow>(
        r#"
        WITH candidate AS (
            SELECT id
            FROM notification_outbox
            WHERE attempts < max_attempts
              AND (
                  (status = 'pending' AND next_attempt_at <= NOW())
                  OR (status = 'processing' AND locked_at < NOW() - INTERVAL '5 minutes')
              )
            ORDER BY next_attempt_at, created_at
            FOR UPDATE SKIP LOCKED
            LIMIT 1
        )
        UPDATE notification_outbox AS outbox
        SET status = 'processing', attempts = outbox.attempts + 1, locked_at = NOW()
        FROM candidate
        WHERE outbox.id = candidate.id
        RETURNING outbox.id, outbox.user_id, outbox.title, outbox.body,
                  outbox.data, outbox.exclude_tokens, outbox.attempts, outbox.max_attempts
        "#,
    )
    .fetch_optional(pool)
    .await?;

    row.map(|row| {
        let data = row.4.as_object().cloned().unwrap_or_default();
        let exclude_tokens = serde_json::from_value(row.5).unwrap_or_default();
        Ok(OutboxMessage {
            id: row.0,
            user_id: row.1,
            title: row.2,
            body: row.3,
            data,
            exclude_tokens,
            attempts: row.6,
            max_attempts: row.7,
        })
    })
    .transpose()
}

pub async fn mark_delivered(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE notification_outbox
         SET status = 'delivered', processed_at = NOW(), locked_at = NULL, last_error = NULL
         WHERE id = $1",
    )
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn mark_delivery_failed(
    pool: &PgPool,
    message: &OutboxMessage,
    error: &str,
) -> Result<(), sqlx::Error> {
    let terminal = message.attempts >= message.max_attempts;
    let next_attempt_at: DateTime<Utc> = Utc::now() + retry_delay(message.attempts);
    sqlx::query(
        "UPDATE notification_outbox
         SET status = $2, next_attempt_at = $3, locked_at = NULL,
             last_error = $4, processed_at = CASE WHEN $2 = 'failed' THEN NOW() ELSE NULL END
         WHERE id = $1",
    )
    .bind(message.id)
    .bind(if terminal { "failed" } else { "pending" })
    .bind(next_attempt_at)
    .bind(error)
    .execute(pool)
    .await?;
    Ok(())
}

fn retry_delay(attempts: i32) -> chrono::Duration {
    let exponent = attempts.saturating_sub(1).clamp(0, 6) as u32;
    chrono::Duration::seconds(30 * 2_i64.pow(exponent))
}

#[cfg(test)]
mod tests {
    use super::retry_delay;

    #[test]
    fn retry_backoff_is_exponential_and_bounded() {
        assert_eq!(retry_delay(1).num_seconds(), 30);
        assert_eq!(retry_delay(2).num_seconds(), 60);
        assert_eq!(retry_delay(3).num_seconds(), 120);
        assert_eq!(retry_delay(20).num_seconds(), 1_920);
    }
}
