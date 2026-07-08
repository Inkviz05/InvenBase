use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use kvantoriym_server::database::Database;

async fn connect_test_db() -> Result<Option<Database>> {
    let Ok(database_url) = std::env::var("TEST_DATABASE_URL") else {
        eprintln!("Skipping PostgreSQL schema test: TEST_DATABASE_URL is not set");
        return Ok(None);
    };

    let db = Database::new(&database_url).await?;
    db.init(None).await?;

    Ok(Some(db))
}

async fn relation_exists(pool: &PgPool, relation_name: &str) -> Result<bool> {
    let relation: Option<String> = sqlx::query_scalar("SELECT to_regclass($1)::text")
        .bind(relation_name)
        .fetch_one(pool)
        .await?;

    Ok(relation.is_some())
}

async fn constraint_exists(pool: &PgPool, constraint_name: &str) -> Result<bool> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1
            FROM pg_constraint
            WHERE conname = $1
        )",
    )
    .bind(constraint_name)
    .fetch_one(pool)
    .await?;

    Ok(exists)
}

async fn cleanup_user(pool: &PgPool, user_id: Uuid) -> Result<()> {
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(())
}

#[tokio::test]
async fn migrations_create_core_tables() -> Result<()> {
    let Some(db) = connect_test_db().await? else {
        return Ok(());
    };

    for table in [
        "users",
        "equipment",
        "equipment_categories",
        "equipment_groups",
        "bookings",
        "permissions",
        "notifications",
        "activity_logs",
        "support_requests",
        "support_request_messages",
    ] {
        assert!(
            relation_exists(&db.pool, table).await?,
            "missing table {table}"
        );
    }

    Ok(())
}

#[tokio::test]
async fn migrations_create_critical_indexes() -> Result<()> {
    let Some(db) = connect_test_db().await? else {
        return Ok(());
    };

    for index in [
        "idx_bookings_user",
        "idx_bookings_equipment",
        "idx_bookings_status",
        "idx_notifications_user",
        "idx_support_requests_status",
        "user_devices_user_token_idx",
    ] {
        assert!(
            relation_exists(&db.pool, index).await?,
            "missing index {index}"
        );
    }

    Ok(())
}

#[tokio::test]
async fn migrations_create_domain_constraints() -> Result<()> {
    let Some(db) = connect_test_db().await? else {
        return Ok(());
    };

    for constraint in [
        "users_role_check",
        "equipment_quantity_positive_check",
        "equipment_available_quantity_lte_quantity_check",
        "equipment_status_check",
        "bookings_quantity_positive_check",
        "bookings_date_order_check",
        "bookings_status_check",
        "bookings_permission_type_check",
        "bookings_target_check",
        "support_requests_status_check",
    ] {
        assert!(
            constraint_exists(&db.pool, constraint).await?,
            "missing constraint {constraint}"
        );
    }

    Ok(())
}

#[tokio::test]
async fn migrations_create_foreign_keys_for_core_relations() -> Result<()> {
    let Some(db) = connect_test_db().await? else {
        return Ok(());
    };

    for constraint in [
        "bookings_user_id_fkey",
        "bookings_equipment_id_fkey",
        "permissions_booking_id_fkey",
        "notifications_user_id_fkey",
        "user_devices_user_id_fkey",
        "support_requests_user_id_fkey",
        "support_request_messages_support_request_id_fkey",
    ] {
        assert!(
            constraint_exists(&db.pool, constraint).await?,
            "missing foreign key {constraint}"
        );
    }

    Ok(())
}

#[tokio::test]
async fn database_rejects_invalid_domain_values() -> Result<()> {
    let Some(db) = connect_test_db().await? else {
        return Ok(());
    };

    let invalid_user_role = sqlx::query(
        "INSERT INTO users (id, username, password_hash, role)
         VALUES ($1, $2, 'hash', 'manager')",
    )
    .bind(Uuid::new_v4())
    .bind(format!("invalid_role_{}", Uuid::new_v4().simple()))
    .execute(&db.pool)
    .await;

    assert!(
        invalid_user_role.is_err(),
        "users.role check should reject unknown roles"
    );

    let invalid_equipment_quantity = sqlx::query(
        "INSERT INTO equipment (id, name, quantity, available_quantity)
         VALUES ($1, $2, 0, 0)",
    )
    .bind(Uuid::new_v4())
    .bind(format!("invalid_quantity_{}", Uuid::new_v4().simple()))
    .execute(&db.pool)
    .await;

    assert!(
        invalid_equipment_quantity.is_err(),
        "equipment quantity check should reject non-positive quantity"
    );

    let invalid_notification_user = sqlx::query(
        "INSERT INTO notifications (id, user_id, title, message, notification_type)
         VALUES ($1, $2, 'title', 'message', 'test')",
    )
    .bind(Uuid::new_v4())
    .bind(Uuid::new_v4())
    .execute(&db.pool)
    .await;

    assert!(
        invalid_notification_user.is_err(),
        "notifications.user_id foreign key should reject missing users"
    );

    Ok(())
}

#[tokio::test]
async fn database_rejects_booking_without_target() -> Result<()> {
    let Some(db) = connect_test_db().await? else {
        return Ok(());
    };

    let user_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, username, password_hash, role)
         VALUES ($1, $2, 'hash', 'user')",
    )
    .bind(user_id)
    .bind(format!("booking_target_{}", user_id.simple()))
    .execute(&db.pool)
    .await?;

    let invalid_booking = sqlx::query(
        "INSERT INTO bookings (id, user_id, quantity, start_date, end_date)
         VALUES ($1, $2, 1, NOW(), NOW() + INTERVAL '1 hour')",
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .execute(&db.pool)
    .await;

    cleanup_user(&db.pool, user_id).await?;

    assert!(
        invalid_booking.is_err(),
        "bookings target check should reject rows without equipment_id or group_id"
    );

    Ok(())
}
