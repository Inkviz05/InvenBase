use std::sync::Arc;

use actix_web::{http::StatusCode, test, web, App};
use anyhow::Result;

use kvantoriym_server::app_state::AppState;
use kvantoriym_server::config::Config;
use kvantoriym_server::database::Database;
use kvantoriym_server::routes;

fn test_config(database_url: String) -> Config {
    Config {
        database_url,
        database_max_connections: 2,
        database_acquire_timeout_seconds: 1,
        host: "127.0.0.1".to_string(),
        port: 8080,
        jwt_secret: "readiness-test-secret-at-least-32-characters".to_string(),
        jwt_expiration: 3600,
        cors_allowed_origins: vec![],
        rustore_project_id: None,
        rustore_service_token: None,
        huawei_client_id: None,
        huawei_client_secret: None,
        default_admin: None,
    }
}

#[actix_web::test]
async fn health_is_live_while_ready_reflects_database_state() -> Result<()> {
    let Ok(database_url) = std::env::var("TEST_DATABASE_URL") else {
        eprintln!("Skipping readiness integration test: TEST_DATABASE_URL is not set");
        return Ok(());
    };

    let db = Arc::new(Database::new(&database_url).await?);
    db.init(None).await?;
    let state = AppState::new(db.clone(), test_config(database_url));
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .service(web::scope("").configure(routes::configure_web)),
    )
    .await;

    let health = test::TestRequest::get().uri("/health").to_request();
    assert_eq!(
        test::call_service(&app, health).await.status(),
        StatusCode::OK
    );

    let ready = test::TestRequest::get().uri("/ready").to_request();
    assert_eq!(
        test::call_service(&app, ready).await.status(),
        StatusCode::OK
    );

    db.pool.close().await;

    let health_after_database_close = test::TestRequest::get().uri("/health").to_request();
    assert_eq!(
        test::call_service(&app, health_after_database_close)
            .await
            .status(),
        StatusCode::OK
    );

    let ready_after_database_close = test::TestRequest::get().uri("/ready").to_request();
    assert_eq!(
        test::call_service(&app, ready_after_database_close)
            .await
            .status(),
        StatusCode::SERVICE_UNAVAILABLE
    );

    Ok(())
}
