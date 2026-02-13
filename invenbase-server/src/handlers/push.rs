use actix_web::{web, HttpResponse};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;
use std::fs;
use std::collections::{HashMap, HashSet};
use jsonwebtoken::{encode, Header, EncodingKey, Algorithm};

use crate::app_state::AppState;
use crate::auth::Claims;
use crate::errors::AppError;

#[derive(Debug, Deserialize)]
pub struct RegisterPushTokenRequest {
    pub token: String,
    pub platform: Option<String>,
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

    // Сохраняем токен, если настроен хотя бы один способ отправки: Legacy (FCM_SERVER_KEY) или HTTP v1 (FCM_PROJECT_ID + FCM_SERVICE_ACCOUNT_PATH)
    let fcm_configured = state.config.fcm_server_key.is_some()
        || (state.config.fcm_project_id.is_some() && state.config.fcm_service_account_path.is_some());
    if !fcm_configured {
        log::warn!("FCM not configured: token not saved. Set FCM_SERVER_KEY or FCM_PROJECT_ID+FCM_SERVICE_ACCOUNT_PATH.");
        return Ok(HttpResponse::NoContent().finish());
    }

    let platform = body
        .platform
        .as_deref()
        .unwrap_or("android")
        .to_string();

    // Таблица user_devices должна существовать в БД:
    //
    // CREATE TABLE IF NOT EXISTS user_devices (
    //   id UUID PRIMARY KEY,
    //   user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    //   fcm_token TEXT NOT NULL,
    //   platform TEXT,
    //   created_at TIMESTAMPTZ DEFAULT NOW(),
    //   updated_at TIMESTAMPTZ DEFAULT NOW()
    // );
    // CREATE UNIQUE INDEX IF NOT EXISTS user_devices_user_token_idx
    //   ON user_devices(user_id, fcm_token);

    sqlx::query::<sqlx::Postgres>(
        r#"
        INSERT INTO user_devices (id, user_id, fcm_token, platform)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (user_id, fcm_token)
        DO UPDATE SET
            platform = EXCLUDED.platform,
            updated_at = NOW()
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(&body.token)
    .bind(&platform)
    .execute(&state.db.pool)
    .await?;

    log::info!("Push token registered for user {} (platform: {})", user_id, platform);
    Ok(HttpResponse::NoContent().finish())
}

/// Вспомогательная функция для отправки push-уведомления всем устройствам пользователя.
///
/// Сначала пытается использовать HTTP v1 API (если настроен), затем fallback на Legacy API.
pub async fn send_push_to_user(
    state: &AppState,
    user_id: Uuid,
    title: &str,
    body: &str,
    mut data: serde_json::Map<String, serde_json::Value>,
) {
    // Загружаем все устройства пользователя
    let devices: Vec<(String,)> = match sqlx::query_as::<sqlx::Postgres, (String,)>(
        "SELECT fcm_token FROM user_devices WHERE user_id = $1",
    )
    .bind(user_id)
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
        log::warn!("No devices found for user {}, push notification skipped", user_id);
        return;
    }

    log::info!("Sending push notification to user {} ({} devices): title='{}', body='{}'", 
               user_id, devices.len(), title, body);

    // Общие поля data
    data.insert(
        "title".to_string(),
        serde_json::Value::String(title.to_string()),
    );
    data.insert(
        "body".to_string(),
        serde_json::Value::String(body.to_string()),
    );

    // Пытаемся использовать HTTP v1 API, если настроен (приоритет)
    if let (Some(project_id), Some(service_account_path)) = (
        state.config.fcm_project_id.clone(),
        state.config.fcm_service_account_path.clone(),
    ) {
        match send_push_v1(&state.db.pool, &project_id, &service_account_path, title, body, &data, &devices, user_id).await {
            Ok(sent_count) => {
                if sent_count > 0 {
                    log::info!("Push notifications sent successfully via HTTP v1 API to user {} ({} devices)", user_id, sent_count);
                    return;
                } else {
                    log::warn!("No valid devices found for user {}, falling back to Legacy API", user_id);
                }
            }
            Err(e) => {
                log::error!("Failed to send push via HTTP v1 API: {:?}, falling back to Legacy API", e);
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
        let mut fcm_data: std::collections::HashMap<String, String> = std::collections::HashMap::new();
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

        log::debug!("FCM Legacy API payload: {}", serde_json::to_string(&payload).unwrap_or_default());

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
                    log::info!("FCM push sent successfully to user {} (token: {}...)", 
                               user_id, &token[..std::cmp::min(20, token.len())]);
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
pub async fn send_push_to_user_excluding_tokens(
    state: &AppState,
    user_id: Uuid,
    title: &str,
    body: &str,
    mut data: serde_json::Map<String, serde_json::Value>,
    exclude_tokens: &[String],
) {
    let devices: Vec<(String,)> = match sqlx::query_as::<sqlx::Postgres, (String,)>(
        "SELECT fcm_token FROM user_devices WHERE user_id = $1",
    )
    .bind(user_id)
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
                log::info!("Push sent via HTTP v1 to user {} ({} devices)", user_id, sent_count);
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
        let mut fcm_data: std::collections::HashMap<String, String> = std::collections::HashMap::new();
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

/// Отправка push-уведомления через FCM HTTP v1 API
/// Возвращает количество успешно отправленных уведомлений
async fn send_push_v1(
    db_pool: &sqlx::PgPool,
    project_id: &str,
    service_account_path: &str,
    title: &str,
    body: &str,
    data: &serde_json::Map<String, serde_json::Value>,
    devices: &[(String,)],
    user_id: Uuid,
) -> Result<usize, Box<dyn std::error::Error>> {
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
    let url = format!("https://fcm.googleapis.com/v1/projects/{}/messages:send", project_id);

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
                "notification": {
                    "title": title,
                    "body": body
                },
                "data": fcm_data,
                "android": {
                    "priority": "high",
                    "notification": {
                        "channel_id": "invenbase_notifications",
                        "sound": "default",
                        "default_sound": true,
                        "default_vibrate_timings": true,
                        "default_light_settings": true
                    }
                },
                "apns": {
                    "payload": {
                        "aps": {
                            "sound": "default",
                            "badge": 1
                        }
                    }
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
            log::info!("FCM v1 push sent successfully to user {} (token: {}...)", 
                       user_id, &token[..std::cmp::min(20, token.len())]);
            sent_count += 1;
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "unknown".to_string());
            
            // Парсим ошибку для проверки на UNREGISTERED
            let error_json: Result<serde_json::Value, _> = serde_json::from_str(&error_text);
            let is_unregistered = error_json.as_ref().ok()
                .and_then(|json| json.get("error"))
                .and_then(|err| err.get("details"))
                .and_then(|details| details.as_array())
                .map(|details| {
                    details.iter().any(|detail| {
                        detail.get("errorCode")
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
        log::info!("Removing {} invalid FCM tokens from database for user {}", invalid_tokens.len(), user_id);
        for token in &invalid_tokens {
            if let Err(e) = sqlx::query::<sqlx::Postgres>(
                "DELETE FROM user_devices WHERE user_id = $1 AND fcm_token = $2"
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
async fn get_oauth_token(client_email: &str, private_key_pem: &str) -> Result<String, Box<dyn std::error::Error>> {
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
        let error_text = token_response.text().await.unwrap_or_else(|_| "unknown".to_string());
        return Err(format!("Failed to get OAuth token: {} - {}", status, error_text).into());
    }

    let token_data: serde_json::Value = token_response.json().await?;
    let access_token = token_data["access_token"]
        .as_str()
        .ok_or("Missing access_token in OAuth response")?;

    Ok(access_token.to_string())
}



