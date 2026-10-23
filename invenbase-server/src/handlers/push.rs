use actix_web::{web, HttpResponse};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::Deserialize;
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::fs;
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
    fn provider(&self) -> &'static str;

    async fn send(
        &self,
        title: &str,
        body: &str,
        data: &serde_json::Map<String, serde_json::Value>,
        devices: &[(String,)],
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>>;
}

struct FcmTransport<'a> {
    state: &'a AppState,
    user_id: Uuid,
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
    fn provider(&self) -> &'static str {
        "huawei"
    }

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
    fn provider(&self) -> &'static str {
        "rustore"
    }

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

impl PushTransport for FcmTransport<'_> {
    fn provider(&self) -> &'static str {
        "fcm"
    }

    async fn send(
        &self,
        title: &str,
        body: &str,
        data: &serde_json::Map<String, serde_json::Value>,
        devices: &[(String,)],
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        if let (Some(project_id), Some(service_account_path)) = (
            self.state.config.fcm_project_id.as_deref(),
            self.state.config.fcm_service_account_path.as_deref(),
        ) {
            let sent = send_push_v1(
                &self.state.db.pool,
                project_id,
                service_account_path,
                title,
                body,
                data,
                devices,
                self.user_id,
            )
            .await?;
            if sent > 0 || self.state.config.fcm_server_key.is_none() {
                return Ok(sent);
            }
        }

        if let Some(server_key) = self.state.config.fcm_server_key.as_deref() {
            return Ok(
                send_push_legacy(server_key, title, body, data, devices, self.user_id).await,
            );
        }

        Err("FCM transport is not configured".into())
    }
}

/// Регистрация FCM токена устройства для текущего пользователя.
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
    let provider = body.provider.as_deref().unwrap_or("fcm").to_lowercase();
    if !matches!(provider.as_str(), "fcm" | "rustore" | "huawei") {
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
    let provider = body.provider.as_deref().unwrap_or("fcm").to_lowercase();
    if !matches!(provider.as_str(), "fcm" | "rustore" | "huawei") {
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
    for provider in ["fcm", "rustore", "huawei"] {
        let provider_devices: Vec<(String,)> = devices
            .iter()
            .filter(|(device_provider, _)| device_provider == provider)
            .map(|(_, token)| (token.clone(),))
            .collect();
        if provider_devices.is_empty() {
            continue;
        }

        let sent = match provider {
            "fcm" => {
                FcmTransport {
                    state,
                    user_id: message.user_id,
                }
                .send(&message.title, &message.body, &data, &provider_devices)
                .await
            }
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

/// Старый синхронный путь доставки оставлен только как совместимый fallback-код.
///
/// Сначала пытается использовать HTTP v1 API (если настроен), затем fallback на Legacy API.
#[allow(dead_code)]
async fn send_push_to_user_direct_legacy(
    state: &AppState,
    user_id: Uuid,
    title: &str,
    body: &str,
    mut data: serde_json::Map<String, serde_json::Value>,
) {
    // Загружаем все устройства пользователя
    let transport = FcmTransport { state, user_id };
    let devices: Vec<(String,)> = match sqlx::query_as::<sqlx::Postgres, (String,)>(
        "SELECT push_token FROM user_devices
         WHERE user_id = $1 AND provider = $2 AND active = TRUE",
    )
    .bind(user_id)
    .bind(transport.provider())
    .fetch_all(&state.db.pool)
    .await
    {
        Ok(list) => list,
        Err(err) => {
            log::error!("Failed to load user devices for push: {:?}", err);
            return;
        }
    };

    if devices.is_empty() {
        log::warn!(
            "No devices found for user {}, push notification skipped",
            user_id
        );
        return;
    }

    log::info!(
        "Sending push notification to user {} ({} devices): title='{}', body='{}'",
        user_id,
        devices.len(),
        title,
        body
    );

    // Общие поля data
    data.insert(
        "title".to_string(),
        serde_json::Value::String(title.to_string()),
    );
    data.insert(
        "body".to_string(),
        serde_json::Value::String(body.to_string()),
    );

    match transport.send(title, body, &data, &devices).await {
        Ok(sent_count) => {
            log::info!(
                "Push transport completed for user {} ({} devices sent)",
                user_id,
                sent_count
            );
            return;
        }
        Err(error) => {
            log::error!(
                "FCM transport failed for user {}: {:?}; using compatibility fallback",
                user_id,
                error
            );
        }
    }

    // Пытаемся использовать HTTP v1 API, если настроен (приоритет)
    if let (Some(project_id), Some(service_account_path)) = (
        state.config.fcm_project_id.clone(),
        state.config.fcm_service_account_path.clone(),
    ) {
        match send_push_v1(
            &state.db.pool,
            &project_id,
            &service_account_path,
            title,
            body,
            &data,
            &devices,
            user_id,
        )
        .await
        {
            Ok(sent_count) => {
                if sent_count > 0 {
                    log::info!("Push notifications sent successfully via HTTP v1 API to user {} ({} devices)", user_id, sent_count);
                    return;
                } else {
                    log::warn!(
                        "No valid devices found for user {}, falling back to Legacy API",
                        user_id
                    );
                }
            }
            Err(e) => {
                log::error!(
                    "Failed to send push via HTTP v1 API: {:?}, falling back to Legacy API",
                    e
                );
                // Fallback to Legacy API
            }
        }
    }

    // Fallback: Legacy API (если HTTP v1 не настроен или не сработал)
    let server_key = match state.config.fcm_server_key.clone() {
        Some(key) => key,
        None => {
            log::warn!("Neither HTTP v1 API nor Legacy API is configured. FCM_PROJECT_ID and FCM_SERVICE_ACCOUNT_PATH (for v1) or FCM_SERVER_KEY (for Legacy) must be set.");
            return;
        }
    };

    // Fallback: Legacy API (может не работать, так как отключен в июле 2024)
    let client = reqwest::Client::new();

    for (token,) in devices {
        // Преобразуем data из Map в правильный формат для FCM Legacy API
        // В Legacy API все значения в data должны быть строками
        let mut fcm_data: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        for (key, value) in &data {
            let value_str = match value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                serde_json::Value::Null => "null".to_string(),
                serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
                    // Для сложных объектов сериализуем в JSON строку
                    serde_json::to_string(value).unwrap_or_else(|_| value.to_string())
                }
            };
            fcm_data.insert(key.clone(), value_str);
        }

        // Формат для Legacy FCM API
        // ВАЖНО: Legacy API был отключен в июле 2024, но попробуем правильный формат
        let payload = serde_json::json!({
            "to": token,
            "notification": {
                "title": title,
                "body": body,
                "sound": "default"
            },
            "data": fcm_data,
            "priority": "high",
            "time_to_live": 86400
        });

        log::debug!(
            "FCM Legacy API payload: {}",
            serde_json::to_string(&payload).unwrap_or_default()
        );

        let res = client
            .post("https://fcm.googleapis.com/fcm/send")
            .header("Authorization", format!("key={}", server_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await;

        match res {
            Ok(resp) => {
                if resp.status().is_success() {
                    log::info!(
                        "FCM push sent successfully to user {} (token: {}...)",
                        user_id,
                        &token[..std::cmp::min(20, token.len())]
                    );
                } else {
                    let status = resp.status();
                    let error_text = resp.text().await.unwrap_or_else(|_| "unknown".to_string());

                    // Если 404, Legacy API отключен
                    if status == 404 {
                        log::error!(
                            "FCM Legacy API returned 404 - API is disabled (deprecated July 2024). User: {}, token: {}...",
                            user_id,
                            &token[..std::cmp::min(20, token.len())]
                        );
                        log::error!("⚠️ Legacy FCM API (fcm/send) was disabled in July 2024.");
                        log::error!("📋 To fix this, you need to:");
                        log::error!("   1. Get your Firebase Project ID from Firebase Console → Project Settings → General");
                        log::error!("   2. Download Service Account JSON from Firebase Console → Project Settings → Service accounts → Generate new private key");
                        log::error!("   3. Add to .env: FCM_PROJECT_ID=your-project-id");
                        log::error!("   4. Add to .env: FCM_SERVICE_ACCOUNT_PATH=./path/to/service-account.json");
                        log::error!("   5. Update code to use HTTP v1 API (see PUSH_NOTIFICATIONS_SERVER.md)");
                    } else {
                        log::error!(
                            "FCM push failed for user {}: status={}, error={}, token={}...",
                            user_id,
                            status,
                            error_text,
                            &token[..std::cmp::min(20, token.len())]
                        );
                    }
                }
            }
            Err(err) => {
                log::error!("Error sending FCM push to user {}: {:?}", user_id, err);
            }
        }
    }
}

/// Отправка push админам, но не на устройства автора заявки (чтобы автор не получал уведомление о своей заявке).
#[allow(dead_code)]
async fn send_push_to_user_excluding_tokens_direct_legacy(
    state: &AppState,
    user_id: Uuid,
    title: &str,
    body: &str,
    mut data: serde_json::Map<String, serde_json::Value>,
    exclude_tokens: &[String],
) {
    let transport = FcmTransport { state, user_id };
    let devices: Vec<(String,)> = match sqlx::query_as::<sqlx::Postgres, (String,)>(
        "SELECT push_token FROM user_devices
         WHERE user_id = $1 AND provider = $2 AND active = TRUE",
    )
    .bind(user_id)
    .bind(transport.provider())
    .fetch_all(&state.db.pool)
    .await
    {
        Ok(list) => list,
        Err(err) => {
            log::error!("Failed to load user devices for push: {:?}", err);
            return;
        }
    };

    let exclude: HashSet<&str> = exclude_tokens.iter().map(String::as_str).collect();
    let devices: Vec<(String,)> = devices
        .into_iter()
        .filter(|(t,)| !exclude.contains(t.as_str()))
        .collect();

    if devices.is_empty() {
        log::info!(
            "Skipping push to user {}: all devices are excluded (author's devices)",
            user_id
        );
        return;
    }

    log::info!(
        "Sending push to user {} ({} devices, {} excluded): title='{}'",
        user_id,
        devices.len(),
        exclude_tokens.len(),
        title
    );

    data.insert(
        "title".to_string(),
        serde_json::Value::String(title.to_string()),
    );
    data.insert(
        "body".to_string(),
        serde_json::Value::String(body.to_string()),
    );

    match transport.send(title, body, &data, &devices).await {
        Ok(sent_count) => {
            log::info!(
                "Push transport completed for user {} ({} devices sent)",
                user_id,
                sent_count
            );
            return;
        }
        Err(error) => {
            log::error!(
                "FCM transport failed for user {}: {:?}; using compatibility fallback",
                user_id,
                error
            );
        }
    }

    if let (Some(project_id), Some(service_account_path)) = (
        state.config.fcm_project_id.clone(),
        state.config.fcm_service_account_path.clone(),
    ) {
        if let Ok(sent_count) = send_push_v1(
            &state.db.pool,
            &project_id,
            &service_account_path,
            title,
            body,
            &data,
            &devices,
            user_id,
        )
        .await
        {
            if sent_count > 0 {
                log::info!(
                    "Push sent via HTTP v1 to user {} ({} devices)",
                    user_id,
                    sent_count
                );
                return;
            }
        }
    }

    let server_key = match state.config.fcm_server_key.clone() {
        Some(key) => key,
        None => return,
    };
    let client = reqwest::Client::new();
    for (token,) in devices {
        let mut fcm_data: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        for (key, value) in &data {
            let value_str = match value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                serde_json::Value::Null => "null".to_string(),
                serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
                    serde_json::to_string(value).unwrap_or_else(|_| value.to_string())
                }
            };
            fcm_data.insert(key.clone(), value_str);
        }
        let payload = serde_json::json!({
            "to": token,
            "notification": { "title": title, "body": body, "sound": "default" },
            "data": fcm_data,
            "priority": "high",
            "time_to_live": 86400
        });
        let _ = client
            .post("https://fcm.googleapis.com/fcm/send")
            .header("Authorization", format!("key={}", server_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await;
    }
}

async fn send_push_legacy(
    server_key: &str,
    title: &str,
    body: &str,
    data: &serde_json::Map<String, serde_json::Value>,
    devices: &[(String,)],
    user_id: Uuid,
) -> usize {
    let client = reqwest::Client::new();
    let fcm_data: HashMap<String, String> = data
        .iter()
        .map(|(key, value)| {
            let value = match value {
                serde_json::Value::String(value) => value.clone(),
                _ => serde_json::to_string(value).unwrap_or_else(|_| value.to_string()),
            };
            (key.clone(), value)
        })
        .collect();
    let mut sent_count = 0;

    for (token,) in devices {
        let payload = json!({
            "to": token,
            "notification": { "title": title, "body": body, "sound": "default" },
            "data": fcm_data,
            "priority": "high",
            "time_to_live": 86400
        });
        match client
            .post("https://fcm.googleapis.com/fcm/send")
            .header("Authorization", format!("key={server_key}"))
            .json(&payload)
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => sent_count += 1,
            Ok(response) => log::error!(
                "Legacy FCM delivery failed for user {} with status {}",
                user_id,
                response.status()
            ),
            Err(error) => log::error!(
                "Legacy FCM request failed for user {}: {:?}",
                user_id,
                error
            ),
        }
    }

    sent_count
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

/// Отправка push-уведомления через FCM HTTP v1 API
/// Возвращает количество успешно отправленных уведомлений
async fn send_push_v1(
    db_pool: &sqlx::PgPool,
    project_id: &str,
    service_account_path: &str,
    _title: &str,
    _body: &str,
    data: &serde_json::Map<String, serde_json::Value>,
    devices: &[(String,)],
    user_id: Uuid,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    // Нормализуем путь (для Windows обрабатываем обратные слэши)
    let normalized_path = service_account_path.replace('\\', "/");
    log::debug!("Reading Service Account from: {}", normalized_path);

    // Проверяем существование файла
    if !std::path::Path::new(&normalized_path).exists() {
        return Err(format!("Service Account file not found: {}", normalized_path).into());
    }

    // Читаем Service Account JSON
    let service_account_json = fs::read_to_string(&normalized_path)?;
    let service_account: serde_json::Value = serde_json::from_str(&service_account_json)?;

    let client_email = service_account["client_email"]
        .as_str()
        .ok_or("Missing client_email in service account")?;
    let private_key_pem = service_account["private_key"]
        .as_str()
        .ok_or("Missing private_key in service account")?;

    // Получаем OAuth access token
    let access_token = get_oauth_token(client_email, private_key_pem).await?;

    let client = reqwest::Client::new();
    let url = format!(
        "https://fcm.googleapis.com/v1/projects/{}/messages:send",
        project_id
    );

    // Преобразуем data в формат для FCM v1 (все значения должны быть строками)
    let mut fcm_data = HashMap::new();
    for (key, value) in data {
        let value_str = match value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Null => "null".to_string(),
            _ => serde_json::to_string(value)?,
        };
        fcm_data.insert(key.clone(), value_str);
    }

    // Отправляем на каждое устройство
    let mut sent_count = 0;
    let mut invalid_tokens = Vec::new();

    for (token,) in devices {
        let payload = json!({
            "message": {
                "token": token,
                "data": fcm_data,
                "android": {
                    "priority": "high",
                    "ttl": "86400s"
                }
            }
        });

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            log::info!(
                "FCM v1 push sent successfully to user {} (token: {}...)",
                user_id,
                &token[..std::cmp::min(20, token.len())]
            );
            sent_count += 1;
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown".to_string());

            // Парсим ошибку для проверки на UNREGISTERED
            let error_json: Result<serde_json::Value, _> = serde_json::from_str(&error_text);
            let is_unregistered = error_json
                .as_ref()
                .ok()
                .and_then(|json| json.get("error"))
                .and_then(|err| err.get("details"))
                .and_then(|details| details.as_array())
                .map(|details| {
                    details.iter().any(|detail| {
                        detail
                            .get("errorCode")
                            .and_then(|code| code.as_str())
                            .map(|code| code == "UNREGISTERED")
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false);

            if is_unregistered {
                log::warn!(
                    "FCM token is unregistered (device may have uninstalled app or token expired) for user {}: token={}...",
                    user_id,
                    &token[..std::cmp::min(20, token.len())]
                );
                // Сохраняем токен для удаления из БД
                invalid_tokens.push(token.clone());
            } else {
                log::error!(
                    "FCM v1 push failed for user {}: status={}, error={}, token={}...",
                    user_id,
                    status,
                    error_text,
                    &token[..std::cmp::min(20, token.len())]
                );
                // Для других ошибок продолжаем, но не удаляем токен (может быть временная проблема)
            }
        }
    }

    // Удаляем недействительные токены из БД
    if !invalid_tokens.is_empty() {
        log::info!(
            "Removing {} invalid FCM tokens from database for user {}",
            invalid_tokens.len(),
            user_id
        );
        for token in &invalid_tokens {
            if let Err(e) = sqlx::query::<sqlx::Postgres>(
                "UPDATE user_devices
                 SET active = FALSE, updated_at = NOW()
                 WHERE user_id = $1 AND provider = 'fcm' AND push_token = $2",
            )
            .bind(user_id)
            .bind(token)
            .execute(db_pool)
            .await
            {
                log::error!("Failed to remove invalid token from database: {:?}", e);
            }
        }
    }

    Ok(sent_count)
}

/// Получение OAuth 2.0 access token из Service Account
async fn get_oauth_token(
    client_email: &str,
    private_key_pem: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    use chrono::Utc;

    // Создаём JWT claims
    let now = Utc::now();
    let exp = now + chrono::Duration::hours(1);

    #[derive(serde::Serialize)]
    struct Claims {
        iss: String,
        scope: String,
        aud: String,
        exp: i64,
        iat: i64,
    }

    let claims = Claims {
        iss: client_email.to_string(),
        scope: "https://www.googleapis.com/auth/firebase.messaging".to_string(),
        aud: "https://oauth2.googleapis.com/token".to_string(),
        exp: exp.timestamp(),
        iat: now.timestamp(),
    };

    // Используем jsonwebtoken для создания JWT с RS256
    let header = Header::new(Algorithm::RS256);

    // Создаём EncodingKey из PEM строки
    let encoding_key = EncodingKey::from_rsa_pem(private_key_pem.as_bytes())
        .map_err(|e| format!("Failed to create encoding key from PEM: {}", e))?;

    // Кодируем JWT
    let jwt = encode(&header, &claims, &encoding_key)
        .map_err(|e| format!("Failed to encode JWT: {}", e))?;

    // Отправляем JWT на OAuth endpoint для получения access token
    let client = reqwest::Client::new();
    let token_response = client
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
            ("assertion", &jwt),
        ])
        .send()
        .await?;

    let status = token_response.status();
    if !status.is_success() {
        let error_text = token_response
            .text()
            .await
            .unwrap_or_else(|_| "unknown".to_string());
        return Err(format!("Failed to get OAuth token: {} - {}", status, error_text).into());
    }

    let token_data: serde_json::Value = token_response.json().await?;
    let access_token = token_data["access_token"]
        .as_str()
        .ok_or("Missing access_token in OAuth response")?;

    Ok(access_token.to_string())
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
