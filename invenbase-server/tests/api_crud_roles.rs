use std::sync::Arc;

use actix_web::{http::StatusCode, middleware::from_fn, test, web, App};
use anyhow::Result;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use kvantoriym_server::app_state::AppState;
use kvantoriym_server::auth::AuthService;
use kvantoriym_server::config::Config;
use kvantoriym_server::database::Database;
use kvantoriym_server::middleware::request_id_middleware;
use kvantoriym_server::models::User;
use kvantoriym_server::routes;

const JWT_SECRET: &str = "crud-role-test-secret-at-least-32-characters";
static DB_TEST_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

async fn connect_test_db() -> Result<Option<(Database, String)>> {
    let Ok(database_url) = std::env::var("TEST_DATABASE_URL") else {
        eprintln!("Skipping CRUD role API test: TEST_DATABASE_URL is not set");
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
        fcm_server_key: None,
        fcm_project_id: None,
        fcm_service_account_path: None,
        rustore_project_id: None,
        rustore_service_token: None,
        huawei_client_id: None,
        huawei_client_secret: None,
        default_admin: None,
    }
}

async fn insert_user(pool: &PgPool, label: &str, role: &str) -> Result<User> {
    let user = User {
        id: Uuid::new_v4(),
        username: format!("crud_{label}_{}", Uuid::new_v4().simple()),
        password_hash: AuthService::hash_password("integration-password")?,
        email: Some(format!("{label}-{}@example.test", Uuid::new_v4().simple())),
        full_name: Some(format!("CRUD {label}")),
        role: role.to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    sqlx::query(
        "INSERT INTO users (id, username, password_hash, email, full_name, role)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(user.id)
    .bind(&user.username)
    .bind(&user.password_hash)
    .bind(&user.email)
    .bind(&user.full_name)
    .bind(&user.role)
    .execute(pool)
    .await?;
    Ok(user)
}

fn token(user: &User) -> Result<String> {
    Ok(AuthService::new(JWT_SECRET, 3600).generate_token(user)?)
}

async fn cleanup_users(pool: &PgPool, ids: &[Uuid]) -> Result<()> {
    sqlx::query("DELETE FROM notification_outbox WHERE user_id = ANY($1)")
        .bind(ids)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM notifications WHERE user_id = ANY($1)")
        .bind(ids)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM activity_logs WHERE user_id = ANY($1)")
        .bind(ids)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM users WHERE id = ANY($1)")
        .bind(ids)
        .execute(pool)
        .await?;
    Ok(())
}

#[actix_web::test]
async fn users_crud_enforces_admin_and_owner_roles() -> Result<()> {
    let _guard = DB_TEST_LOCK.lock().await;
    let Some((db, database_url)) = connect_test_db().await? else {
        return Ok(());
    };
    let admin = insert_user(&db.pool, "users_admin", "admin").await?;
    let regular = insert_user(&db.pool, "users_regular", "user").await?;
    let other = insert_user(&db.pool, "users_other", "user").await?;
    let admin_token = token(&admin)?;
    let regular_token = token(&regular)?;
    let pool = db.pool.clone();
    let state = AppState::new(Arc::new(db), test_config(database_url));
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .wrap(from_fn(request_id_middleware))
            .service(web::scope("/api").configure(routes::configure_api)),
    )
    .await;

    let forbidden_create = test::TestRequest::post()
        .uri("/api/users")
        .insert_header(("Authorization", format!("Bearer {regular_token}")))
        .set_json(serde_json::json!({"username":"forbidden","password":"password"}))
        .to_request();
    let response = test::call_service(&app, forbidden_create).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let username = format!("created_{}", Uuid::new_v4().simple());
    let create = test::TestRequest::post()
        .uri("/api/users")
        .insert_header(("Authorization", format!("Bearer {admin_token}")))
        .set_json(serde_json::json!({
            "username": username,
            "password": "created-password",
            "full_name": "Created User",
            "role": "user"
        }))
        .to_request();
    let response = test::call_service(&app, create).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let created: serde_json::Value = test::read_body_json(response).await;
    let created_id = Uuid::parse_str(created["id"].as_str().expect("created user id"))?;
    assert!(created.get("password_hash").is_none());

    let forbidden_read = test::TestRequest::get()
        .uri(&format!("/api/users/{}", other.id))
        .insert_header(("Authorization", format!("Bearer {regular_token}")))
        .to_request();
    let response = test::call_service(&app, forbidden_read).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let body: serde_json::Value = test::read_body_json(response).await;
    assert_eq!(body["code"], "FORBIDDEN");

    let update_self = test::TestRequest::put()
        .uri(&format!("/api/users/{}", regular.id))
        .insert_header(("Authorization", format!("Bearer {regular_token}")))
        .set_json(serde_json::json!({"full_name":"Updated Owner","role":"admin"}))
        .to_request();
    let response = test::call_service(&app, update_self).await;
    assert_eq!(response.status(), StatusCode::OK);
    let updated: serde_json::Value = test::read_body_json(response).await;
    assert_eq!(updated["full_name"], "Updated Owner");
    assert_eq!(updated["role"], "user");

    let forbidden_delete = test::TestRequest::delete()
        .uri(&format!("/api/users/{created_id}"))
        .insert_header(("Authorization", format!("Bearer {regular_token}")))
        .to_request();
    assert_eq!(
        test::call_service(&app, forbidden_delete).await.status(),
        StatusCode::FORBIDDEN
    );

    let delete = test::TestRequest::delete()
        .uri(&format!("/api/users/{created_id}"))
        .insert_header(("Authorization", format!("Bearer {admin_token}")))
        .to_request();
    assert_eq!(
        test::call_service(&app, delete).await.status(),
        StatusCode::NO_CONTENT
    );

    cleanup_users(&pool, &[admin.id, regular.id, other.id]).await?;
    Ok(())
}

#[actix_web::test]
async fn equipment_crud_allows_responsible_but_not_regular_user() -> Result<()> {
    let _guard = DB_TEST_LOCK.lock().await;
    let Some((db, database_url)) = connect_test_db().await? else {
        return Ok(());
    };
    let responsible = insert_user(&db.pool, "equipment_responsible", "responsible").await?;
    let regular = insert_user(&db.pool, "equipment_regular", "user").await?;
    let responsible_token = token(&responsible)?;
    let regular_token = token(&regular)?;
    let pool = db.pool.clone();
    let state = AppState::new(Arc::new(db), test_config(database_url));
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .wrap(from_fn(request_id_middleware))
            .service(web::scope("/api").configure(routes::configure_api)),
    )
    .await;

    let forbidden = test::TestRequest::post()
        .uri("/api/equipment")
        .insert_header(("Authorization", format!("Bearer {regular_token}")))
        .set_json(serde_json::json!({"name":"Forbidden equipment","quantity":2}))
        .to_request();
    assert_eq!(
        test::call_service(&app, forbidden).await.status(),
        StatusCode::FORBIDDEN
    );

    let create = test::TestRequest::post()
        .uri("/api/equipment")
        .insert_header(("Authorization", format!("Bearer {responsible_token}")))
        .set_json(serde_json::json!({"name":"HTTP test equipment","quantity":3,"location":"A-1"}))
        .to_request();
    let response = test::call_service(&app, create).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let created: serde_json::Value = test::read_body_json(response).await;
    let equipment_id = Uuid::parse_str(created["id"].as_str().expect("equipment id"))?;
    assert_eq!(created["available_quantity"], 3);

    let get = test::TestRequest::get()
        .uri(&format!("/api/equipment/{equipment_id}"))
        .insert_header(("Authorization", format!("Bearer {regular_token}")))
        .to_request();
    assert_eq!(test::call_service(&app, get).await.status(), StatusCode::OK);

    let forbidden_update = test::TestRequest::put()
        .uri(&format!("/api/equipment/{equipment_id}"))
        .insert_header(("Authorization", format!("Bearer {regular_token}")))
        .set_json(serde_json::json!({"name":"Forbidden rename"}))
        .to_request();
    assert_eq!(
        test::call_service(&app, forbidden_update).await.status(),
        StatusCode::FORBIDDEN
    );

    let update = test::TestRequest::put()
        .uri(&format!("/api/equipment/{equipment_id}"))
        .insert_header(("Authorization", format!("Bearer {responsible_token}")))
        .set_json(serde_json::json!({"name":"Updated equipment","location":"B-2"}))
        .to_request();
    let response = test::call_service(&app, update).await;
    assert_eq!(response.status(), StatusCode::OK);
    let updated: serde_json::Value = test::read_body_json(response).await;
    assert_eq!(updated["name"], "Updated equipment");
    assert_eq!(updated["location"], "B-2");

    let delete = test::TestRequest::delete()
        .uri(&format!("/api/equipment/{equipment_id}"))
        .insert_header(("Authorization", format!("Bearer {responsible_token}")))
        .to_request();
    assert_eq!(
        test::call_service(&app, delete).await.status(),
        StatusCode::NO_CONTENT
    );

    let missing = test::TestRequest::get()
        .uri(&format!("/api/equipment/{equipment_id}"))
        .insert_header(("Authorization", format!("Bearer {regular_token}")))
        .to_request();
    assert_eq!(
        test::call_service(&app, missing).await.status(),
        StatusCode::NOT_FOUND
    );

    cleanup_users(&pool, &[responsible.id, regular.id]).await?;
    Ok(())
}

#[actix_web::test]
async fn support_lifecycle_is_scoped_to_owner_and_admin() -> Result<()> {
    let _guard = DB_TEST_LOCK.lock().await;
    let Some((db, database_url)) = connect_test_db().await? else {
        return Ok(());
    };
    let admin = insert_user(&db.pool, "support_admin", "admin").await?;
    let responsible = insert_user(&db.pool, "support_responsible", "responsible").await?;
    let owner = insert_user(&db.pool, "support_owner", "user").await?;
    let other = insert_user(&db.pool, "support_other", "user").await?;
    let admin_token = token(&admin)?;
    let responsible_token = token(&responsible)?;
    let owner_token = token(&owner)?;
    let other_token = token(&other)?;
    let pool = db.pool.clone();
    let state = AppState::new(Arc::new(db), test_config(database_url));
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .wrap(from_fn(request_id_middleware))
            .service(web::scope("/api").configure(routes::configure_api)),
    )
    .await;

    let admin_create = test::TestRequest::post()
        .uri("/api/support/requests")
        .insert_header(("Authorization", format!("Bearer {admin_token}")))
        .set_json(serde_json::json!({"subject":"Invalid","message":"Admin cannot create"}))
        .to_request();
    assert_eq!(
        test::call_service(&app, admin_create).await.status(),
        StatusCode::BAD_REQUEST
    );

    let create = test::TestRequest::post()
        .uri("/api/support/requests")
        .insert_header(("Authorization", format!("Bearer {owner_token}")))
        .set_json(serde_json::json!({"subject":"Integration support","message":"Initial message"}))
        .to_request();
    let response = test::call_service(&app, create).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let created: serde_json::Value = test::read_body_json(response).await;
    let request_id = Uuid::parse_str(created["id"].as_str().expect("support request id"))?;

    let other_list = test::TestRequest::get()
        .uri("/api/support/requests")
        .insert_header(("Authorization", format!("Bearer {other_token}")))
        .to_request();
    let response = test::call_service(&app, other_list).await;
    let requests: serde_json::Value = test::read_body_json(response).await;
    assert!(!requests
        .as_array()
        .expect("support array")
        .iter()
        .any(|item| item["id"] == request_id.to_string()));

    let forbidden_message = test::TestRequest::post()
        .uri(&format!("/api/support/requests/{request_id}/messages"))
        .insert_header(("Authorization", format!("Bearer {other_token}")))
        .set_json(serde_json::json!({"message":"Not mine"}))
        .to_request();
    assert_eq!(
        test::call_service(&app, forbidden_message).await.status(),
        StatusCode::FORBIDDEN
    );

    let owner_message = test::TestRequest::post()
        .uri(&format!("/api/support/requests/{request_id}/messages"))
        .insert_header(("Authorization", format!("Bearer {owner_token}")))
        .set_json(serde_json::json!({"message":"More details"}))
        .to_request();
    assert_eq!(
        test::call_service(&app, owner_message).await.status(),
        StatusCode::CREATED
    );

    let responsible_update = test::TestRequest::put()
        .uri(&format!("/api/support/requests/{request_id}"))
        .insert_header(("Authorization", format!("Bearer {responsible_token}")))
        .set_json(serde_json::json!({"status":"answered"}))
        .to_request();
    assert_eq!(
        test::call_service(&app, responsible_update).await.status(),
        StatusCode::FORBIDDEN
    );

    let answer = test::TestRequest::put()
        .uri(&format!("/api/support/requests/{request_id}"))
        .insert_header(("Authorization", format!("Bearer {admin_token}")))
        .set_json(serde_json::json!({"status":"answered","admin_comment":"Admin response"}))
        .to_request();
    let response = test::call_service(&app, answer).await;
    assert_eq!(response.status(), StatusCode::OK);
    let answered: serde_json::Value = test::read_body_json(response).await;
    assert_eq!(answered["status"], "answered");

    let close = test::TestRequest::put()
        .uri(&format!("/api/support/requests/{request_id}"))
        .insert_header(("Authorization", format!("Bearer {admin_token}")))
        .set_json(serde_json::json!({"status":"closed"}))
        .to_request();
    assert_eq!(
        test::call_service(&app, close).await.status(),
        StatusCode::OK
    );

    let closed_message = test::TestRequest::post()
        .uri(&format!("/api/support/requests/{request_id}/messages"))
        .insert_header(("Authorization", format!("Bearer {owner_token}")))
        .set_json(serde_json::json!({"message":"Too late"}))
        .to_request();
    assert_eq!(
        test::call_service(&app, closed_message).await.status(),
        StatusCode::BAD_REQUEST
    );

    let delete = test::TestRequest::delete()
        .uri(&format!("/api/support/requests/{request_id}"))
        .insert_header(("Authorization", format!("Bearer {admin_token}")))
        .to_request();
    assert_eq!(
        test::call_service(&app, delete).await.status(),
        StatusCode::NO_CONTENT
    );

    cleanup_users(&pool, &[admin.id, responsible.id, owner.id, other.id]).await?;
    Ok(())
}
