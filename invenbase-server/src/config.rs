use std::env;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub host: String,
    pub port: u16,
    pub jwt_secret: String,
    pub jwt_expiration: u64,
    /// Ключ сервера FCM для отправки push-уведомлений (Legacy API - отключен в июле 2024)
    pub fcm_server_key: Option<String>,
    /// Project ID для FCM HTTP v1 API
    pub fcm_project_id: Option<String>,
    /// Путь к Service Account JSON файлу для FCM HTTP v1 API
    pub fcm_service_account_path: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        // Загружаем .env файл
        if let Err(e) = dotenv::dotenv() {
            log::warn!("Failed to load .env file: {:?}", e);
        }

        // Читаем переменные окружения
        let fcm_project_id = env::var("FCM_PROJECT_ID").ok();
        let fcm_service_account_path_raw = env::var("FCM_SERVICE_ACCOUNT_PATH").ok();
        let fcm_server_key = env::var("FCM_SERVER_KEY").ok();

        // Логируем загруженные значения для отладки (до нормализации)
        log::info!("FCM configuration loaded:");
        log::info!("  FCM_PROJECT_ID: {:?}", fcm_project_id);
        log::info!("  FCM_SERVICE_ACCOUNT_PATH (raw): {:?}", &fcm_service_account_path_raw);
        log::info!("  FCM_SERVER_KEY: {}", if fcm_server_key.is_some() { "***SET***" } else { "NOT SET" });

        // Нормализуем путь к Service Account (преобразуем относительный в абсолютный, если нужно)
        let fcm_service_account_path = fcm_service_account_path_raw.map(|path| {
            // Если путь относительный, преобразуем в абсолютный относительно текущей директории
            if path.starts_with("./") || (!path.contains(':') && !path.starts_with('/')) {
                // Относительный путь
                if let Ok(current_dir) = std::env::current_dir() {
                    let normalized = current_dir.join(path.trim_start_matches("./")).to_string_lossy().to_string();
                    log::info!("  FCM_SERVICE_ACCOUNT_PATH normalized: {} -> {}", path, normalized);
                    normalized
                } else {
                    path
                }
            } else {
                // Абсолютный путь - оставляем как есть
                path
            }
        });
        
        // Дополнительная диагностика: проверяем все переменные окружения, начинающиеся с FCM_
        if std::env::var("RUST_LOG").is_ok() {
            for (key, value) in std::env::vars() {
                if key.starts_with("FCM_") {
                    log::debug!("  Env var {} = {:?}", key, if key.contains("KEY") { "***HIDDEN***" } else { &value });
                }
            }
        }

        // Проверяем существование файла Service Account, если путь указан
        if let Some(ref path) = fcm_service_account_path {
            if std::path::Path::new(path).exists() {
                log::info!("  Service Account file exists: {}", path);
            } else {
                log::warn!("  Service Account file NOT FOUND: {}", path);
            }
        }

        Ok(Config {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/kvantoriym".to_string()),
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "your-secret-key-change-in-production".to_string()),
            jwt_expiration: env::var("JWT_EXPIRATION")
                .unwrap_or_else(|_| "86400".to_string())
                .parse()
                .unwrap_or(86400), // 24 часа
            fcm_server_key,
            fcm_project_id,
            fcm_service_account_path,
        })
    }
}

