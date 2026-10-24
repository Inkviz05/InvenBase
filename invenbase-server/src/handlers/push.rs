use actix_web::{web, HttpResponse};
use serde::Deserialize;
use serde_json::json;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::auth::Claims;
use crate::errors::AppError;
use crate::services::notification_outbox;

#[derive(Debug, Deserialize)]
pub struct RegisterPushTokenRequest {
    pub token: String,
    pub platform: Option<String>,
    pub device_id: Option<String>,
    pub provider: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UnregisterPushDeviceRequest {
    pub device_id: String,
    pub provider: Option<String>,
}

trait PushTransport {
    async fn send(
        &self,
        title: &str,
        body: &str,
        data: &serde_json::Map<String, serde_json::Value>,
        devices: &[(String,)],
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>>;
}

struct RuStoreTransport<'a> {
    state: &'a AppState,
    user_id: Uuid,
}

struct HuaweiTransport<'a> {
    state: &'a AppState,
    user_id: Uuid,
}

impl PushTransport for HuaweiTransport<'_> {
    async fn send(
        &self,
        title: &str,
        body: &str,
        data: &serde_json::Map<String, serde_json::Value>,
        devices: &[(String,)],
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let client_id = self
            .state
            .config
            .huawei_client_id
            .as_deref()
            .ok_or("Huawei client ID is not configured")?;
        let client_secret = self
            .state
            .config
            .huawei_client_secret
            .as_deref()
            .ok_or("Huawei client secret is not configured")?;
        send_push_huawei(
            &self.state.db.pool,
            client_id,
            client_secret,
            title,
            body,
            data,
            devices,
            self.user_id,
        )
        .await
    }
}

impl PushTransport for RuStoreTransport<'_> {
    async fn send(
        &self,
        title: &str,
        body: &str,
        data: &serde_json::Map<String, serde_json::Value>,
        devices: &[(String,)],
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let project_id = self
            .state
            .config
            .rustore_project_id
            .as_deref()
            .ok_or("RuStore project ID is not configured")?;
        let service_token = self
            .state
            .config
            .rustore_service_token
            .as_deref()
            .ok_or("RuStore service token is not configured")?;
        send_push_rustore(
            &self.state.db.pool,
            project_id,
            service_token,
            title,
            body,
            data,
            devices,
            self.user_id,
        )
        .await
    }
}

/// Регистрация push-токена устройства для текущего пользователя.
///
/// POST /api/push/register-token
pub async fn register_push_token(
    state: web::Data<AppState>,
    claims: Claims,
    body: web::Json<RegisterPushTokenRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

    let token = body.token.trim();
    if token.is_empty() {
        return Err(AppError::BadRequest(
            "Push token must not be empty".to_string(),
        ));
    }

    let platform = body.platform.as_deref().unwrap_or("android").to_string();
    let provider = body
        .provider
        .as_deref()
        .ok_or_else(|| AppError::BadRequest("Push provider is required".to_string()))?
        .to_lowercase();
    if !matches!(provider.as_str(), "rustore" | "huawei") {
        return Err(AppError::BadRequest(
            "Unsupported push provider".to_string(),
        ));
    }
    let device_id = body
        .device_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("legacy:{token}"));

    let mut transaction = state.db.pool.begin().await?;

    sqlx::query(
        "DELETE FROM user_devices
         WHERE provider = $1 AND push_token = $2
           AND (user_id <> $3 OR device_id <> $4)",
    )
    .bind(&provider)
    .bind(token)
    .bind(user_id)
    .bind(&device_id)
    .execute(&mut *transaction)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO user_devices
            (id, user_id, device_id, provider, push_token, platform, active, last_seen_at)
        VALUES ($1, $2, $3, $4, $5, $6, TRUE, NOW())
        ON CONFLICT (user_id, provider, device_id)
        DO UPDATE SET
            push_token = EXCLUDED.push_token,
            platform = EXCLUDED.platform,
            active = TRUE,
            last_seen_at = NOW(),
            updated_at = NOW()
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(&device_id)
    .bind(&provider)
    .bind(token)
    .bind(&platform)
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;

    log::info!(
        "Push device registered for user {} (provider: {}, platform: {})",
        user_id,
        provider,
        platform
    );
    Ok(HttpResponse::NoContent().finish())
}

/// Деактивирует только текущее физическое устройство пользователя.
pub async fn unregister_push_device(
    state: web::Data<AppState>,
    claims: Claims,
    body: web::Json<UnregisterPushDeviceRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;
    let device_id = body.device_id.trim();
    if device_id.is_empty() {
        return Err(AppError::BadRequest(
            "Device ID must not be empty".to_string(),
        ));
    }
    let provider = body
        .provider
        .as_deref()
        .ok_or_else(|| AppError::BadRequest("Push provider is required".to_string()))?
        .to_lowercase();
    if !matches!(provider.as_str(), "rustore" | "huawei") {
        return Err(AppError::BadRequest(
            "Unsupported push provider".to_string(),
        ));
    }

    sqlx::query(
        "UPDATE user_devices
         SET active = FALSE, updated_at = NOW()
         WHERE user_id = $1 AND provider = $2 AND device_id = $3",
    )
    .bind(user_id)
    .bind(&provider)
    .bind(device_id)
    .execute(&state.db.pool)
    .await?;

    Ok(HttpResponse::NoContent().finish())
}

/// Ставит push-уведомление в надёжную очередь доставки.
pub async fn send_push_to_user(
    state: &AppState,
    user_id: Uuid,
    title: &str,
    body: &str,
    data: serde_json::Map<String, serde_json::Value>,
) {
    if let Err(error) =
        notification_outbox::enqueue(&state.db.pool, user_id, title, body, data, &[]).await
    {
        log::error!("Failed to enqueue push for user {}: {:?}", user_id, error);
    }
}

/// Ставит push в очередь, исключая указанные токены при фактической доставке.
pub async fn send_push_to_user_excluding_tokens(
    state: &AppState,
    user_id: Uuid,
    title: &str,
    body: &str,
    data: serde_json::Map<String, serde_json::Value>,
    exclude_tokens: &[String],
) {
    if let Err(error) =
        notification_outbox::enqueue(&state.db.pool, user_id, title, body, data, exclude_tokens)
            .await
    {
        log::error!("Failed to enqueue push for user {}: {:?}", user_id, error);
    }
}

/// Выполняет одну попытку доставки сообщения из outbox.
pub async fn deliver_outbox_push(
    state: &AppState,
    message: &notification_outbox::OutboxMessage,
) -> Result<(), String> {
    let devices: Vec<(String, String)> = sqlx::query_as::<sqlx::Postgres, (String, String)>(
        "SELECT provider, push_token FROM user_devices
         WHERE user_id = $1 AND active = TRUE",
    )
    .bind(message.user_id)
    .fetch_all(&state.db.pool)
    .await
    .map_err(|error| format!("failed to load devices: {error}"))?;

    let excluded: HashSet<&str> = message.exclude_tokens.iter().map(String::as_str).collect();
    let devices: Vec<(String, String)> = devices
        .into_iter()
        .filter(|(_, token)| !excluded.contains(token.as_str()))
        .collect();
    if devices.is_empty() {
        return Ok(());
    }

    let mut data = message.data.clone();
    data.insert("title".to_string(), json!(message.title));
    data.insert("body".to_string(), json!(message.body));
    for provider in ["rustore", "huawei"] {
        let provider_devices: Vec<(String,)> = devices
            .iter()
            .filter(|(device_provider, _)| device_provider == provider)
            .map(|(_, token)| (token.clone(),))
            .collect();
        if provider_devices.is_empty() {
            continue;
        }

        let sent = match provider {
            "rustore" => {
                RuStoreTransport {
                    state,
                    user_id: message.user_id,
                }
                .send(&message.title, &message.body, &data, &provider_devices)
                .await
            }
            "huawei" => {
                HuaweiTransport {
                    state,
                    user_id: message.user_id,
                }
                .send(&message.title, &message.body, &data, &provider_devices)
                .await
            }
            _ => unreachable!(),
        }
        .map_err(|error| format!("{provider} transport failed: {error}"))?;

        if sent != provider_devices.len() {
            return Err(format!(
                "{provider} transport delivered {sent} of {} devices",
                provider_devices.len()
            ));
        }
    }

    Ok(())
}

/// Отправка через официальный RuStore Push API.
async fn send_push_rustore(
    db_pool: &sqlx::PgPool,
    project_id: &str,
    service_token: &str,
    title: &str,
    body: &str,
    data: &serde_json::Map<String, serde_json::Value>,
    devices: &[(String,)],
    user_id: Uuid,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let url = format!("https://vkpns.rustore.ru/v1/projects/{project_id}/messages:send");
    let string_data = stringify_push_data(data);
    let mut sent_count = 0;
    let mut invalid_tokens = Vec::new();

    for (token,) in devices {
        let payload = json!({
            "message": {
                "token": token,
                "data": string_data,
                "notification": { "title": title, "body": body },
                "android": {
                    "ttl": "86400s",
                    "notification": {
                        "title": title,
                        "body": body,
                        "channel_id": "invenbase_notifications"
                    }
                }
            }
        });
        let response = client
            .post(&url)
            .bearer_auth(service_token)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            sent_count += 1;
            continue;
        }

        let status = response.status();
        let error_body = response.text().await.unwrap_or_default();
        let provider_status = serde_json::from_str::<serde_json::Value>(&error_body)
            .ok()
            .and_then(|value| {
                value
                    .get("error")
                    .and_then(|error| error.get("status"))
                    .and_then(|status| status.as_str())
                    .map(ToOwned::to_owned)
            });
        if provider_status.as_deref() == Some("NOT_FOUND") {
            invalid_tokens.push(token.clone());
        }
        log::error!(
            "RuStore push failed for user {}: status={}, provider_status={:?}",
            user_id,
            status,
            provider_status
        );
    }

    for token in invalid_tokens {
        sqlx::query(
            "UPDATE user_devices
             SET active = FALSE, updated_at = NOW()
             WHERE user_id = $1 AND provider = 'rustore' AND push_token = $2",
        )
        .bind(user_id)
        .bind(token)
        .execute(db_pool)
        .await?;
    }

    Ok(sent_count)
}

#[derive(Deserialize)]
struct HuaweiOauthResponse {
    access_token: String,
}

fn build_huawei_payload(
    token: &str,
    title: &str,
    body: &str,
    data: &serde_json::Map<String, serde_json::Value>,
) -> serde_json::Value {
    let data_json =
        serde_json::to_string(&stringify_push_data(data)).unwrap_or_else(|_| "{}".into());
    json!({
        "validate_only": false,
        "message": {
            "data": data_json,
            "notification": {
                "title": title,
                "body": body
            },
            "android": {
                "urgency": "HIGH",
                "ttl": "86400s",
                "notification": {
                    "title": title,
                    "body": body,
                    "default_sound": true,
                    "channel_id": "invenbase_notifications"
                }
            },
            "token": [token]
        }
    })
}

async fn get_huawei_oauth_token(
    client: &reqwest::Client,
    client_id: &str,
    client_secret: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let response = client
        .post("https://oauth-login.cloud.huawei.com/oauth2/v3/token")
        .form(&[
            ("grant_type", "client_credentials"),
            ("client_id", client_id),
            ("client_secret", client_secret),
        ])
        .send()
        .await?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("Huawei OAuth request failed with status {status}").into());
    }

    Ok(response.json::<HuaweiOauthResponse>().await?.access_token)
}

/// Отправка через Huawei Push Kit REST API для Android/HMS Core.
async fn send_push_huawei(
    db_pool: &sqlx::PgPool,
    client_id: &str,
    client_secret: &str,
    title: &str,
    body: &str,
    data: &serde_json::Map<String, serde_json::Value>,
    devices: &[(String,)],
    user_id: Uuid,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let access_token = get_huawei_oauth_token(&client, client_id, client_secret).await?;
    let url = format!("https://push-api.cloud.huawei.com/v1/{client_id}/messages:send");
    let mut sent_count = 0;
    let mut invalid_tokens = Vec::new();

    for (token,) in devices {
        let response = client
            .post(&url)
            .bearer_auth(&access_token)
            .json(&build_huawei_payload(token, title, body, data))
            .send()
            .await?;
        let status = response.status();
        let response_body = response.text().await.unwrap_or_default();
        let provider_code = serde_json::from_str::<serde_json::Value>(&response_body)
            .ok()
            .and_then(|value| value.get("code")?.as_str().map(ToOwned::to_owned));

        if status.is_success() && provider_code.as_deref() == Some("80000000") {
            sent_count += 1;
            continue;
        }
        if provider_code.as_deref() == Some("80300007") {
            invalid_tokens.push(token.clone());
        }
        log::error!(
            "Huawei push failed for user {}: status={}, provider_code={:?}",
            user_id,
            status,
            provider_code
        );
    }

    for token in invalid_tokens {
        sqlx::query(
            "UPDATE user_devices
             SET active = FALSE, updated_at = NOW()
             WHERE user_id = $1 AND provider = 'huawei' AND push_token = $2",
        )
        .bind(user_id)
        .bind(token)
        .execute(db_pool)
        .await?;
    }

    Ok(sent_count)
}

fn stringify_push_data(
    data: &serde_json::Map<String, serde_json::Value>,
) -> HashMap<String, String> {
    data.iter()
        .map(|(key, value)| {
            let string_value = match value {
                serde_json::Value::String(value) => value.clone(),
                _ => serde_json::to_string(value).unwrap_or_else(|_| value.to_string()),
            };
            (key.clone(), string_value)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{build_huawei_payload, stringify_push_data};

    #[test]
    fn push_data_is_converted_to_provider_string_values() {
        let mut data = serde_json::Map::new();
        data.insert("text".to_string(), serde_json::json!("value"));
        data.insert("count".to_string(), serde_json::json!(3));
        data.insert("nested".to_string(), serde_json::json!({"ok": true}));

        let result = stringify_push_data(&data);
        assert_eq!(result.get("text").map(String::as_str), Some("value"));
        assert_eq!(result.get("count").map(String::as_str), Some("3"));
        assert_eq!(
            result.get("nested").map(String::as_str),
            Some("{\"ok\":true}")
        );
    }

    #[test]
    fn huawei_payload_contains_notification_tokens_and_stringified_data() {
        let mut data = serde_json::Map::new();
        data.insert("notification_id".to_string(), serde_json::json!(42));

        let payload = build_huawei_payload("device-token", "Title", "Body", &data);
        assert_eq!(payload["message"]["token"][0], "device-token");
        assert_eq!(payload["message"]["notification"]["title"], "Title");
        assert_eq!(
            payload["message"]["android"]["notification"]["channel_id"],
            "invenbase_notifications"
        );
        let data: serde_json::Value =
            serde_json::from_str(payload["message"]["data"].as_str().unwrap()).unwrap();
        assert_eq!(data["notification_id"], "42");
    }
}
