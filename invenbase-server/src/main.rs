mod config;
mod database;
mod models;
mod handlers;
mod routes;
mod auth;
mod errors;
mod app_state;
mod middleware;
mod background_tasks;
mod services;

use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use std::sync::Arc;

use crate::config::Config;
use crate::database::Database;
use crate::app_state::AppState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Инициализация логирования
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    log::info!("Запуск сервера Кванториум...");

    // Загрузка конфигурации
    let config = Config::from_env().expect("Не удалось загрузить конфигурацию");
    
    // Подключение к базе данных
    let db = Arc::new(
        Database::new(&config.database_url)
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
            .unwrap()
    );

    // Инициализация базы данных (создание таблиц)
    db.init(config.default_admin.as_ref()).await.expect("Не удалось инициализировать базу данных");

    log::info!("База данных инициализирована");

    let app_state = AppState::new(db.clone(), config.clone());
    let app_state_for_background = Arc::new(app_state.clone());
    
    // Запускаем фоновую задачу проверки истечения времени бронирований
    tokio::spawn(async move {
        background_tasks::start_booking_expiration_checker(app_state_for_background).await;
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
            .service(actix_files::Files::new("/static", "./static").show_files_listing())
            .service(
                web::scope("/api")
                    .configure(routes::configure_api)
            )
            .service(
                web::scope("")
                    .configure(routes::configure_web)
            )
    })
    .bind((config.host.clone(), config.port))?
    .run()
    .await
}
