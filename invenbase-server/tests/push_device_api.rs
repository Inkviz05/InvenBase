use std::sync::Arc;

use actix_web::{http::StatusCode, middleware::from_fn, test, web, App};
use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;

use kvantoriym_server::app_state::AppState;
use kvantoriym_server::auth::AuthService;
use kvantoriym_server::config::Config;
use kvantoriym_server::database::Database;
use kvantoriym_server::middleware::request_id_middleware;
use kvantoriym_server::models::User;
use kvantoriym_server::routes;

const JWT_SECRET: &str = "integration-test-secret-at-least-32-characters";

async fn connect_test_db() -> Result<Option<(Database, String)>> {
    let Ok(database_url) = std::env::var("TEST_DATABASE_URL") else {
        eprintln!("Skipping push device API test: TEST_DATABASE_URL is not set");
        return Ok(None);
    };

    let db = Database::new(&database_url).await?;
    db.init(None).await?;
    Ok(Some((db, database_url)))
}

fn test_config(database_url: String) -> Config {
    Config {
        database_url,
        database_max_connections: 10,
        database_acquire_timeout_seconds: 5,
        host: "127.0.0.1".to_string(),
        port: 8080,
        jwt_secret: JWT_SECRET.to_string(),
        jwt_expiration: 3600,
        cors_allowed_origins: vec![],
        instance_name: "Test InvenBase".to_string(),
        rustore_project_id: None,
        rustore_service_token: None,
        huawei_client_id: None,
        huawei_client_secret: None,
        default_admin: None,
    }
}

async fn create_user(db: &Database, label: &str) -> Result<User> {
    let user = User {
        id: Uuid::new_v4(),
        username: format!("push_api_{label}_{}", Uuid::new_v4().simple()),
        password_hash: "not-used-by-this-test".to_string(),
        email: None,
        full_name: Some(format!("Push API {label}")),
        role: "user".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    sqlx::query(
        "INSERT INTO users (id, username, password_hash, full_name, role)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user.id)
    .bind(&user.username)
    .bind(&user.password_hash)
    .bind(&user.full_name)
    .bind(&user.role)
    .execute(&db.pool)
    .await?;

    Ok(user)
}

async fn create_login_user(db: &Database, label: &str, role: &str, password: &str) -> Result<User> {
    let user = User {
        id: Uuid::new_v4(),
        username: format!("login_{label}_{}", Uuid::new_v4().simple()),
        password_hash: AuthService::hash_password(password)?,
        email: None,
        full_name: Some(format!("Login API {label}")),
        role: role.to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    sqlx::query(
        "INSERT INTO users (id, username, password_hash, full_name, role)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user.id)
    .bind(&user.username)
    .bind(&user.password_hash)
    .bind(&user.full_name)
    .bind(&user.role)
    .execute(&db.pool)
    .await?;

    Ok(user)
}

fn bearer_token(user: &User) -> Result<String> {
    Ok(AuthService::new(JWT_SECRET, 3600).generate_token(user)?)
}

#[actix_web::test]
async fn push_device_http_lifecycle_is_user_scoped() -> Result<()> {
    let Some((db, database_url)) = connect_test_db().await? else {
        return Ok(());
    };

    let first_user = create_user(&db, "first").await?;
    let second_user = create_user(&db, "second").await?;
    let first_token = bearer_token(&first_user)?;
    let second_token = bearer_token(&second_user)?;
    let phone_id = format!("phone-{}", Uuid::new_v4());
    let tablet_id = format!("tablet-{}", Uuid::new_v4());
    let original_push_token = format!("original-{}", Uuid::new_v4());
    let current_push_token = format!("current-{}", Uuid::new_v4());
    let tablet_push_token = format!("tablet-{}", Uuid::new_v4());

    let state = AppState::new(Arc::new(db), test_config(database_url));
    let pool = state.db.pool.clone();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .wrap(from_fn(request_id_middleware))
            .service(web::scope("/api").configure(routes::configure_api)),
    )
    .await;

    let unauthorized = test::TestRequest::post()
        .uri("/api/push/register-token")
        .set_json(serde_json::json!({
            "token": original_push_token,
            "device_id": phone_id,
            "provider": "huawei"
        }))
        .to_request();
    assert_eq!(
        test::call_service(&app, unauthorized).await.status(),
        StatusCode::UNAUTHORIZED
    );

    for push_token in [&original_push_token, &current_push_token] {
        let request = test::TestRequest::post()
            .uri("/api/push/register-token")
            .insert_header(("Authorization", format!("Bearer {first_token}")))
            .set_json(serde_json::json!({
                "token": push_token,
                "platform": "android",
                "device_id": phone_id,
                "provider": "huawei"
            }))
            .to_request();
        assert_eq!(
            test::call_service(&app, request).await.status(),
            StatusCode::NO_CONTENT
        );
    }

    let phone: (i64, String, bool) = sqlx::query_as(
        "SELECT COUNT(*), MAX(push_token), BOOL_AND(active)
         FROM user_devices
         WHERE user_id = $1 AND provider = 'huawei' AND device_id = $2",
    )
    .bind(first_user.id)
    .bind(&phone_id)
    .fetch_one(&pool)
    .await?;
    assert_eq!(phone, (1, current_push_token.clone(), true));

    let tablet_request = test::TestRequest::post()
        .uri("/api/push/register-token")
        .insert_header(("Authorization", format!("Bearer {first_token}")))
        .set_json(serde_json::json!({
            "token": tablet_push_token,
            "platform": "android",
            "device_id": tablet_id,
            "provider": "rustore"
        }))
        .to_request();
    assert_eq!(
        test::call_service(&app, tablet_request).await.status(),
        StatusCode::NO_CONTENT
    );

    let active_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM user_devices WHERE user_id = $1 AND active = TRUE",
    )
    .bind(first_user.id)
    .fetch_one(&pool)
    .await?;
    assert_eq!(active_count, 2);

    let transfer_request = test::TestRequest::post()
        .uri("/api/push/register-token")
        .insert_header(("Authorization", format!("Bearer {second_token}")))
        .set_json(serde_json::json!({
            "token": current_push_token,
            "platform": "android",
            "device_id": phone_id,
            "provider": "huawei"
        }))
        .to_request();
    assert_eq!(
        test::call_service(&app, transfer_request).await.status(),
        StatusCode::NO_CONTENT
    );

    let token_owner: Uuid = sqlx::query_scalar(
        "SELECT user_id FROM user_devices WHERE provider = 'huawei' AND push_token = $1",
    )
    .bind(&current_push_token)
    .fetch_one(&pool)
    .await?;
    assert_eq!(token_owner, second_user.id);

    let unregister_request = test::TestRequest::post()
        .uri("/api/push/unregister-device")
        .insert_header(("Authorization", format!("Bearer {second_token}")))
        .set_json(serde_json::json!({
            "device_id": phone_id,
            "provider": "huawei"
        }))
        .to_request();
    assert_eq!(
        test::call_service(&app, unregister_request).await.status(),
        StatusCode::NO_CONTENT
    );

    let second_device_active: bool = sqlx::query_scalar(
        "SELECT active FROM user_devices
         WHERE user_id = $1 AND provider = 'huawei' AND device_id = $2",
    )
    .bind(second_user.id)
    .bind(&phone_id)
    .fetch_one(&pool)
    .await?;
    assert!(!second_device_active);

    let first_user_active_devices: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM user_devices WHERE user_id = $1 AND active = TRUE",
    )
    .bind(first_user.id)
    .fetch_one(&pool)
    .await?;
    assert_eq!(first_user_active_devices, 1);

    sqlx::query("DELETE FROM users WHERE id = ANY($1)")
        .bind(vec![first_user.id, second_user.id])
        .execute(&pool)
        .await?;

    Ok(())
}

#[actix_web::test]
async fn login_and_notification_flow_targets_all_account_devices() -> Result<()> {
    let Some((db, database_url)) = connect_test_db().await? else {
        return Ok(());
    };

    let password = "integration-password";
    let user = create_login_user(&db, "recipient", "user", password).await?;
    let admin = create_login_user(&db, "admin", "admin", password).await?;
    let state = AppState::new(Arc::new(db), test_config(database_url));
    let pool = state.db.pool.clone();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .wrap(from_fn(request_id_middleware))
            .service(web::scope("/api").configure(routes::configure_api)),
    )
    .await;

    let invalid_login = test::TestRequest::post()
        .uri("/api/auth/login")
        .insert_header(("x-request-id", "018f8f9e-7b2d-7b22-9f75-7b40f02e44d1"))
        .set_json(serde_json::json!({
            "username": user.username,
            "password": "wrong-password"
        }))
        .to_request();
    let invalid_login_response = test::call_service(&app, invalid_login).await;
    assert_eq!(invalid_login_response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        invalid_login_response
            .headers()
            .get("x-request-id")
            .and_then(|value| value.to_str().ok()),
        Some("018f8f9e-7b2d-7b22-9f75-7b40f02e44d1")
    );
    let invalid_login_body: serde_json::Value = test::read_body_json(invalid_login_response).await;
    assert_eq!(invalid_login_body["code"], "INVALID_CREDENTIALS");
    assert_eq!(invalid_login_body["message"], "Invalid credentials");
    assert!(invalid_login_body["details"].is_null());
    assert!(invalid_login_body["request_id"].is_null());

    let user_login = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(serde_json::json!({
            "username": user.username,
            "password": password
        }))
        .to_request();
    let user_login_response = test::call_service(&app, user_login).await;
    assert_eq!(user_login_response.status(), StatusCode::OK);
    let user_login_body: serde_json::Value = test::read_body_json(user_login_response).await;
    let user_token = user_login_body["token"]
        .as_str()
        .expect("user login response must contain a token")
        .to_string();

    let admin_login = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(serde_json::json!({
            "username": admin.username,
            "password": password
        }))
        .to_request();
    let admin_login_response = test::call_service(&app, admin_login).await;
    assert_eq!(admin_login_response.status(), StatusCode::OK);
    let admin_login_body: serde_json::Value = test::read_body_json(admin_login_response).await;
    let admin_token = admin_login_body["token"]
        .as_str()
        .expect("admin login response must contain a token")
        .to_string();

    let forbidden_notification = test::TestRequest::post()
        .uri("/api/notifications")
        .insert_header(("Authorization", format!("Bearer {user_token}")))
        .set_json(serde_json::json!({
            "user_id": user.id,
            "title": "Forbidden notification",
            "message": "A regular user cannot create this",
            "notification_type": "integration"
        }))
        .to_request();
    let forbidden_response = test::call_service(&app, forbidden_notification).await;
    assert_eq!(forbidden_response.status(), StatusCode::FORBIDDEN);
    let forbidden_body: serde_json::Value = test::read_body_json(forbidden_response).await;
    assert_eq!(forbidden_body["code"], "FORBIDDEN");

    for device in ["phone", "tablet"] {
        let request = test::TestRequest::post()
            .uri("/api/push/register-token")
            .insert_header(("Authorization", format!("Bearer {user_token}")))
            .set_json(serde_json::json!({
                "token": format!("{device}-token-{}", user.id),
                "platform": "android",
                "device_id": format!("{device}-{}", user.id),
                "provider": "rustore"
            }))
            .to_request();
        assert_eq!(
            test::call_service(&app, request).await.status(),
            StatusCode::NO_CONTENT
        );
    }

    let create_notification = test::TestRequest::post()
        .uri("/api/notifications")
        .insert_header(("Authorization", format!("Bearer {admin_token}")))
        .set_json(serde_json::json!({
            "user_id": user.id,
            "title": "Integration notification",
            "message": "Delivered through the account outbox",
            "notification_type": "integration"
        }))
        .to_request();
    assert_eq!(
        test::call_service(&app, create_notification).await.status(),
        StatusCode::CREATED
    );

    let get_notifications = test::TestRequest::get()
        .uri("/api/notifications")
        .insert_header(("Authorization", format!("Bearer {user_token}")))
        .to_request();
    let response = test::call_service(&app, get_notifications).await;
    assert_eq!(response.status(), StatusCode::OK);
    let notifications: serde_json::Value = test::read_body_json(response).await;
    assert_eq!(notifications.as_array().map(Vec::len), Some(1));
    assert_eq!(notifications[0]["title"], "Integration notification");

    let queued_recipients: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM notification_outbox outbox
         JOIN user_devices device
           ON device.user_id = outbox.user_id AND device.active = TRUE
         WHERE outbox.user_id = $1 AND outbox.status = 'pending'",
    )
    .bind(user.id)
    .fetch_one(&pool)
    .await?;
    assert_eq!(queued_recipients, 2);

    sqlx::query("DELETE FROM activity_logs WHERE user_id = ANY($1)")
        .bind(vec![user.id, admin.id])
        .execute(&pool)
        .await?;
    sqlx::query("DELETE FROM notifications WHERE user_id = $1")
        .bind(user.id)
        .execute(&pool)
        .await?;
    sqlx::query("DELETE FROM users WHERE id = ANY($1)")
        .bind(vec![user.id, admin.id])
        .execute(&pool)
        .await?;

    Ok(())
}
