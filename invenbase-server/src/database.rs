use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

use crate::config::DefaultAdminConfig;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(Duration::from_secs(5))
            .connect(database_url)
            .await?;

        Ok(Database { pool })
    }

    pub async fn init(
        &self,
        default_admin: Option<&DefaultAdminConfig>,
    ) -> Result<(), sqlx::Error> {
        MIGRATOR.run(&self.pool).await?;

        if let Some(default_admin) = default_admin {
            self.create_default_admin(default_admin).await?;
        }

        Ok(())
    }

    async fn create_default_admin(
        &self,
        default_admin: &DefaultAdminConfig,
    ) -> Result<(), sqlx::Error> {
        use bcrypt::{hash, DEFAULT_COST};

        let admin_exists: Option<(bool,)> =
            sqlx::query_as("SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)")
                .bind(&default_admin.username)
                .fetch_optional(&self.pool)
                .await?;

        if let Some((true,)) = admin_exists {
            return Ok(());
        }

        let password_hash = hash(&default_admin.password, DEFAULT_COST).unwrap();

        sqlx::query(
            r#"
            INSERT INTO users (username, password_hash, email, full_name, role)
            VALUES ($1, $2, $3, $4, 'admin')
            ON CONFLICT (username) DO NOTHING
            "#,
        )
        .bind(&default_admin.username)
        .bind(password_hash)
        .bind(&default_admin.email)
        .bind(&default_admin.full_name)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
