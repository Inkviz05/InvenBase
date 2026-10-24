use std::sync::Arc;

use actix_web::{http::StatusCode, middleware::from_fn, test, web, App};
use anyhow::Result;
use chrono::Utc;
use serde_json::{json, Value};
use uuid::Uuid;

use kvantoriym_server::app_state::AppState;
use kvantoriym_server::auth::AuthService;
use kvantoriym_server::config::Config;
use kvantoriym_server::database::Database;
use kvantoriym_server::middleware::request_id_middleware;
use kvantoriym_server::models::User;
use kvantoriym_server::routes;

const JWT_SECRET: &str = "instance-pairing-test-secret-at-least-32-chars";

fn test_config(database_url: String) -> Config {
    Config {
        database_url,
        database_max_connections: 5,
        database_acquire_timeout_seconds: 5,
        host: "127.0.0.1".to_string(),
        port: 8080,
        jwt_secret: JWT_SECRET.to_string(),
        jwt_expiration: 3600,
        cors_allowed_origins: vec![],
        instance_name: "Pairing Test School".to_string(),
        rustore_project_id: None,
        rustore_service_token: None,
        huawei_client_id: None,
        huawei_client_secret: None,
        default_admin: None,
    }
}

#[actix_web::test]
async fn setup_code_is_admin_only_and_can_be_consumed_once() -> Result<()> {
    let Ok(database_url) = std::env::var("TEST_DATABASE_URL") else {
        eprintln!("Skipping instance pairing test: TEST_DATABASE_URL is not set");
        return Ok(());
    };
    let db = Database::new(&database_url).await?;
    db.init(None).await?;

    let admin = User {
        id: Uuid::new_v4(),
        username: format!("pairing_admin_{}", Uuid::new_v4().simple()),
        password_hash: "not-used".to_string(),
        email: None,
        full_name: Some("Pairing Admin".to_string()),
        role: "admin".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    sqlx::query(
        "INSERT INTO users (id, username, password_hash, full_name, role)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(admin.id)
    .bind(&admin.username)
    .bind(&admin.password_hash)
    .bind(&admin.full_name)
    .bind(&admin.role)
    .execute(&db.pool)
    .await?;
    let token = AuthService::new(JWT_SECRET, 3600).generate_token(&admin)?;

    let state = AppState::new(Arc::new(db), test_config(database_url));
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .wrap(from_fn(request_id_middleware))
            .service(web::scope("/api").configure(routes::configure_api)),
    )
    .await;

    let metadata = test::call_service(
        &app,
        test::TestRequest::get().uri("/api/instance").to_request(),
    )
    .await;
    assert_eq!(metadata.status(), StatusCode::OK);
    let metadata_body: Value = test::read_body_json(metadata).await;
    assert_eq!(metadata_body["product"], "InvenBase");
    assert_eq!(metadata_body["instance_name"], "Pairing Test School");
    assert_eq!(metadata_body["api_protocol_version"], 1);

    let unauthenticated_create = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/instance/setup-codes")
            .set_json(json!({}))
            .to_request(),
    )
    .await;
    assert_eq!(unauthenticated_create.status(), StatusCode::UNAUTHORIZED);

    let create = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/instance/setup-codes")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(json!({"expires_in_minutes": 5}))
            .to_request(),
    )
    .await;
    assert_eq!(create.status(), StatusCode::CREATED);
    let create_body: Value = test::read_body_json(create).await;
    let setup_code = create_body["setup_code"].as_str().unwrap();
    assert_eq!(setup_code.len(), 29);

    let pair_request = || {
        test::TestRequest::post()
            .uri("/api/instance/pair")
            .set_json(json!({"setup_code": setup_code}))
            .to_request()
    };
    let paired = test::call_service(&app, pair_request()).await;
    assert_eq!(paired.status(), StatusCode::OK);
    let paired_body: Value = test::read_body_json(paired).await;
    assert_eq!(
        paired_body["instance"]["instance_id"],
        metadata_body["instance_id"]
    );

    let reused = test::call_service(&app, pair_request()).await;
    assert_eq!(reused.status(), StatusCode::UNAUTHORIZED);
    let reused_body: Value = test::read_body_json(reused).await;
    assert_eq!(reused_body["code"], "UNAUTHORIZED");

    let create_revocable = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/instance/setup-codes")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(json!({}))
            .to_request(),
    )
    .await;
    let revocable_body: Value = test::read_body_json(create_revocable).await;
    let revocable_id = revocable_body["id"].as_str().unwrap();
    let revocable_code = revocable_body["setup_code"].as_str().unwrap();
    let revoked = test::call_service(
        &app,
        test::TestRequest::delete()
            .uri(&format!("/api/instance/setup-codes/{revocable_id}"))
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request(),
    )
    .await;
    assert_eq!(revoked.status(), StatusCode::NO_CONTENT);

    let revoked_pair = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/instance/pair")
            .set_json(json!({"setup_code": revocable_code}))
            .to_request(),
    )
    .await;
    assert_eq!(revoked_pair.status(), StatusCode::UNAUTHORIZED);

    Ok(())
}
