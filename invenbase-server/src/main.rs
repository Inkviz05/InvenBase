use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use std::sync::Arc;
use std::time::Duration;

use kvantoriym_server::app_state::AppState;
use kvantoriym_server::background_tasks;
use kvantoriym_server::config::Config;
use kvantoriym_server::database::Database;
use kvantoriym_server::routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Инициализация логирования
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("Запуск сервера Кванториум...");

    // Загрузка конфигурации
    let config = Config::from_env().expect("Не удалось загрузить конфигурацию");

    // Подключение к базе данных
    let db = Arc::new(
        Database::with_options(
            &config.database_url,
            config.database_max_connections,
            Duration::from_secs(config.database_acquire_timeout_seconds),
        )
        .await
        .map_err(|e| {
            eprintln!("Ошибка подключения к базе данных: {:?}", e);
            eprintln!("Проверьте:");
            eprintln!("  1. PostgreSQL запущен");
            eprintln!("  2. База данных 'kvantoriym' создана");
            eprintln!("  3. DATABASE_URL в .env файле правильный");
            eprintln!("  4. Учетные данные верные");
            std::process::exit(1);
        })
        .unwrap(),
    );

    let run_migrations_on_startup = std::env::var("RUN_MIGRATIONS_ON_STARTUP")
        .map(|value| value.eq_ignore_ascii_case("true"))
        .unwrap_or(true);
    if run_migrations_on_startup {
        db.init(config.default_admin.as_ref())
            .await
            .expect("Не удалось инициализировать базу данных");
        log::info!("Миграции базы данных применены при запуске");
    } else {
        log::info!("Автоматические миграции отключены; ожидается отдельный migrate job");
    }

    let app_state = AppState::new(db.clone(), config.clone());
    let shared_app_state = Arc::new(app_state.clone());

    // Запускаем фоновую задачу проверки истечения времени бронирований
    tokio::spawn(async move {
        background_tasks::start_booking_expiration_checker(shared_app_state.clone()).await;
    });

    let outbox_app_state = Arc::new(app_state.clone());
    tokio::spawn(async move {
        background_tasks::start_notification_outbox_worker(outbox_app_state).await;
    });

    log::info!("Сервер запущен на http://{}:{}", config.host, config.port);

    // Запуск HTTP сервера
    let app_state_for_server = app_state.clone();
    HttpServer::new(move || {
        let mut cors = Cors::default()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        for origin in &app_state_for_server.config.cors_allowed_origins {
            cors = cors.allowed_origin(origin);
        }

        App::new()
            .app_data(web::Data::new(app_state_for_server.clone()))
            .app_data(web::JsonConfig::default().error_handler(|err, _req| {
                log::error!("JSON deserialization error: {:?}", err);
                actix_web::error::ErrorBadRequest(format!("Invalid JSON: {}", err))
            }))
            .wrap(cors)
            .wrap(actix_web::middleware::Logger::default())
            .service(actix_files::Files::new("/static", "./static"))
            .service(web::scope("/api").configure(routes::configure_api))
            .service(web::scope("").configure(routes::configure_web))
    })
    .bind((config.host.clone(), config.port))?
    .run()
    .await
}
