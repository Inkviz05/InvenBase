use std::time::Duration;

use kvantoriym_server::config::Config;
use kvantoriym_server::database::Database;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let config = Config::from_env()?;
    let database = Database::with_options(
        &config.database_url,
        config.database_max_connections,
        Duration::from_secs(config.database_acquire_timeout_seconds),
    )
    .await?;

    database.init(config.default_admin.as_ref()).await?;
    log::info!("Database migrations completed successfully");
    Ok(())
}
