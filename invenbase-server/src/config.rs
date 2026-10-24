use std::env;

#[derive(Clone)]
pub struct DefaultAdminConfig {
    pub username: String,
    pub password: String,
    pub email: String,
    pub full_name: String,
}

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub database_max_connections: u32,
    pub database_acquire_timeout_seconds: u64,
    pub host: String,
    pub port: u16,
    pub jwt_secret: String,
    pub jwt_expiration: u64,
    pub cors_allowed_origins: Vec<String>,
    /// Project ID сервиса RuStore Push.
    pub rustore_project_id: Option<String>,
    /// Сервисный токен RuStore Push из консоли разработчика.
    pub rustore_service_token: Option<String>,
    /// OAuth 2.0 client ID приложения из AppGallery Connect.
    pub huawei_client_id: Option<String>,
    /// OAuth 2.0 client secret приложения из AppGallery Connect.
    pub huawei_client_secret: Option<String>,
    pub default_admin: Option<DefaultAdminConfig>,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        // Загружаем .env файл
        if let Err(e) = dotenv::dotenv() {
            log::debug!(".env file was not loaded: {:?}", e);
        }

        // Читаем переменные окружения
        let rustore_project_id = non_empty_value(env::var("RUSTORE_PROJECT_ID").ok());
        let rustore_service_token = non_empty_value(env::var("RUSTORE_SERVICE_TOKEN").ok());
        let huawei_client_id = non_empty_value(env::var("HUAWEI_CLIENT_ID").ok());
        let huawei_client_secret = non_empty_value(env::var("HUAWEI_CLIENT_SECRET").ok());

        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| "JWT_SECRET is required and must contain at least 32 characters")?;
        if jwt_secret.len() < 32 {
            return Err("JWT_SECRET must contain at least 32 characters".into());
        }

        let default_admin = if env::var("CREATE_DEFAULT_ADMIN")
            .unwrap_or_else(|_| "false".to_string())
            .eq_ignore_ascii_case("true")
        {
            let password = env::var("DEFAULT_ADMIN_PASSWORD")
                .map_err(|_| "DEFAULT_ADMIN_PASSWORD is required when CREATE_DEFAULT_ADMIN=true")?;
            if password.len() < 12 {
                return Err("DEFAULT_ADMIN_PASSWORD must contain at least 12 characters".into());
            }

            Some(DefaultAdminConfig {
                username: env::var("DEFAULT_ADMIN_USERNAME")
                    .unwrap_or_else(|_| "admin".to_string()),
                password,
                email: env::var("DEFAULT_ADMIN_EMAIL")
                    .unwrap_or_else(|_| "admin@kvantoriym.local".to_string()),
                full_name: env::var("DEFAULT_ADMIN_FULL_NAME")
                    .unwrap_or_else(|_| "Administrator".to_string()),
            })
        } else {
            None
        };

        let database_max_connections = env::var("DATABASE_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "10".to_string())
            .parse::<u32>()
            .map_err(|_| "DATABASE_MAX_CONNECTIONS must be a positive integer")?;
        if database_max_connections == 0 {
            return Err("DATABASE_MAX_CONNECTIONS must be greater than zero".into());
        }

        let database_acquire_timeout_seconds = env::var("DATABASE_ACQUIRE_TIMEOUT_SECONDS")
            .unwrap_or_else(|_| "5".to_string())
            .parse::<u64>()
            .map_err(|_| "DATABASE_ACQUIRE_TIMEOUT_SECONDS must be a positive integer")?;
        if database_acquire_timeout_seconds == 0 {
            return Err("DATABASE_ACQUIRE_TIMEOUT_SECONDS must be greater than zero".into());
        }

        Ok(Config {
            database_url: env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgresql://postgres:postgres@localhost:5432/kvantoriym".to_string()
            }),
            database_max_connections,
            database_acquire_timeout_seconds,
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            jwt_secret,
            jwt_expiration: env::var("JWT_EXPIRATION")
                .unwrap_or_else(|_| "86400".to_string())
                .parse()
                .unwrap_or(86400), // 24 часа
            cors_allowed_origins: env::var("CORS_ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:3000,http://127.0.0.1:3000".to_string())
                .split(',')
                .map(str::trim)
                .filter(|origin| !origin.is_empty())
                .map(ToOwned::to_owned)
                .collect(),
            rustore_project_id,
            rustore_service_token,
            huawei_client_id,
            huawei_client_secret,
            default_admin,
        })
    }
}

fn non_empty_value(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

#[cfg(test)]
mod tests {
    use super::non_empty_value;

    #[test]
    fn empty_optional_configuration_is_treated_as_missing() {
        assert_eq!(non_empty_value(None), None);
        assert_eq!(non_empty_value(Some(String::new())), None);
        assert_eq!(non_empty_value(Some("   ".to_string())), None);
    }

    #[test]
    fn optional_configuration_is_trimmed() {
        assert_eq!(
            non_empty_value(Some("  configured-value  ".to_string())),
            Some("configured-value".to_string())
        );
    }
}
