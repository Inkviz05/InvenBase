use anyhow::Result;
use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use kvantoriym_server::database::Database;
use kvantoriym_server::errors::AppError;
use kvantoriym_server::models::CreateBookingRequest;
use kvantoriym_server::services::bookings;

async fn connect_test_db() -> Result<Option<Database>> {
    let Ok(database_url) = std::env::var("TEST_DATABASE_URL") else {
        eprintln!("Skipping PostgreSQL integration test: TEST_DATABASE_URL is not set");
        return Ok(None);
    };

    let db = Database::new(&database_url).await?;
    db.init(None).await?;

    Ok(Some(db))
}

async fn insert_user(pool: &PgPool, role: &str) -> Result<Uuid> {
    let id = Uuid::new_v4();
    let username = format!("test_{}_{}", role, id.simple());

    sqlx::query(
        r#"
        INSERT INTO users (id, username, password_hash, email, full_name, role)
        VALUES ($1, $2, 'test-hash', $3, $4, $5)
        "#,
    )
    .bind(id)
    .bind(&username)
    .bind(format!("{username}@example.test"))
    .bind(format!("Test {role}"))
    .bind(role)
    .execute(pool)
    .await?;

    Ok(id)
}

async fn insert_equipment(pool: &PgPool, quantity: i32) -> Result<Uuid> {
    insert_equipment_with_status(pool, quantity, "available").await
}

async fn insert_equipment_with_status(pool: &PgPool, quantity: i32, status: &str) -> Result<Uuid> {
    let id = Uuid::new_v4();
    let name = format!("Integration equipment {}", id.simple());

    sqlx::query(
        r#"
        INSERT INTO equipment (id, name, quantity, available_quantity, status)
        VALUES ($1, $2, $3, $3, $4)
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(quantity)
    .bind(status)
    .execute(pool)
    .await?;

    Ok(id)
}

fn booking_request(equipment_id: Uuid, quantity: i32) -> CreateBookingRequest {
    let start_date = Utc::now() + Duration::minutes(5);
    CreateBookingRequest {
        equipment_id: Some(equipment_id),
        group_id: None,
        quantity,
        start_date,
        end_date: start_date + Duration::hours(2),
        purpose: Some("integration test".to_string()),
        permission_type: Some(bookings::PERMISSION_TYPE_INTERNAL.to_string()),
    }
}

async fn available_quantity(pool: &PgPool, equipment_id: Uuid) -> Result<i32> {
    let quantity =
        sqlx::query_scalar::<_, i32>("SELECT available_quantity FROM equipment WHERE id = $1")
            .bind(equipment_id)
            .fetch_one(pool)
            .await?;

    Ok(quantity)
}

async fn cleanup(
    pool: &PgPool,
    booking_id: Uuid,
    equipment_id: Uuid,
    user_ids: &[Uuid],
) -> Result<()> {
    sqlx::query("DELETE FROM permissions WHERE booking_id = $1")
        .bind(booking_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM bookings WHERE id = $1")
        .bind(booking_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM equipment WHERE id = $1")
        .bind(equipment_id)
        .execute(pool)
        .await?;

    for user_id in user_ids {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(pool)
            .await?;
    }

    Ok(())
}

async fn cleanup_equipment_and_users(
    pool: &PgPool,
    equipment_ids: &[Uuid],
    user_ids: &[Uuid],
) -> Result<()> {
    for equipment_id in equipment_ids {
        sqlx::query("DELETE FROM equipment WHERE id = $1")
            .bind(equipment_id)
            .execute(pool)
            .await?;
    }

    for user_id in user_ids {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(pool)
            .await?;
    }

    Ok(())
}

#[tokio::test]
async fn approved_booking_waits_for_return_before_releasing_stock() -> Result<()> {
    let Some(db) = connect_test_db().await? else {
        return Ok(());
    };

    let user_id = insert_user(&db.pool, "user").await?;
    let admin_id = insert_user(&db.pool, "admin").await?;
    let equipment_id = insert_equipment(&db.pool, 5).await?;

    let booking =
        bookings::create_reserved_booking(&db.pool, user_id, &booking_request(equipment_id, 2))
            .await?;
    assert_eq!(booking.status, bookings::STATUS_PENDING);
    assert_eq!(available_quantity(&db.pool, equipment_id).await?, 3);

    let booking = bookings::approve_booking(&db.pool, booking.id, admin_id).await?;
    assert_eq!(booking.status, bookings::STATUS_APPROVED);
    assert_eq!(available_quantity(&db.pool, equipment_id).await?, 3);

    let booking = bookings::mark_awaiting_return(&db.pool, booking.id).await?;
    assert_eq!(booking.status, bookings::STATUS_AWAITING_RETURN);
    assert_eq!(available_quantity(&db.pool, equipment_id).await?, 3);

    let booking = bookings::confirm_return(&db.pool, booking.id).await?;
    assert_eq!(booking.status, bookings::STATUS_RETURNED);
    assert_eq!(available_quantity(&db.pool, equipment_id).await?, 5);

    cleanup(&db.pool, booking.id, equipment_id, &[user_id, admin_id]).await?;
    Ok(())
}

#[tokio::test]
async fn invalid_return_transition_keeps_reserved_stock_until_cancelled() -> Result<()> {
    let Some(db) = connect_test_db().await? else {
        return Ok(());
    };

    let user_id = insert_user(&db.pool, "user").await?;
    let equipment_id = insert_equipment(&db.pool, 4).await?;

    let booking =
        bookings::create_reserved_booking(&db.pool, user_id, &booking_request(equipment_id, 1))
            .await?;
    assert_eq!(available_quantity(&db.pool, equipment_id).await?, 3);

    let result = bookings::confirm_return(&db.pool, booking.id).await;
    assert!(matches!(result, Err(AppError::BadRequest(_))));
    assert_eq!(available_quantity(&db.pool, equipment_id).await?, 3);

    let booking = bookings::cancel_booking(&db.pool, booking.id).await?;
    assert_eq!(booking.status, bookings::STATUS_CANCELLED);
    assert_eq!(available_quantity(&db.pool, equipment_id).await?, 4);

    cleanup(&db.pool, booking.id, equipment_id, &[user_id]).await?;
    Ok(())
}

#[tokio::test]
async fn rejecting_pending_booking_releases_reserved_stock() -> Result<()> {
    let Some(db) = connect_test_db().await? else {
        return Ok(());
    };

    let user_id = insert_user(&db.pool, "user").await?;
    let equipment_id = insert_equipment(&db.pool, 3).await?;

    let booking =
        bookings::create_reserved_booking(&db.pool, user_id, &booking_request(equipment_id, 2))
            .await?;
    assert_eq!(available_quantity(&db.pool, equipment_id).await?, 1);

    let booking = bookings::reject_booking(&db.pool, booking.id).await?;
    assert_eq!(booking.status, bookings::STATUS_REJECTED);
    assert_eq!(available_quantity(&db.pool, equipment_id).await?, 3);

    cleanup(&db.pool, booking.id, equipment_id, &[user_id]).await?;
    Ok(())
}

#[tokio::test]
async fn cancelling_pending_booking_releases_reserved_stock() -> Result<()> {
    let Some(db) = connect_test_db().await? else {
        return Ok(());
    };

    let user_id = insert_user(&db.pool, "user").await?;
    let equipment_id = insert_equipment(&db.pool, 3).await?;

    let booking =
        bookings::create_reserved_booking(&db.pool, user_id, &booking_request(equipment_id, 1))
            .await?;
    assert_eq!(available_quantity(&db.pool, equipment_id).await?, 2);

    let booking = bookings::cancel_booking(&db.pool, booking.id).await?;
    assert_eq!(booking.status, bookings::STATUS_CANCELLED);
    assert_eq!(available_quantity(&db.pool, equipment_id).await?, 3);

    cleanup(&db.pool, booking.id, equipment_id, &[user_id]).await?;
    Ok(())
}

#[tokio::test]
async fn cancelling_approved_booking_releases_reserved_stock() -> Result<()> {
    let Some(db) = connect_test_db().await? else {
        return Ok(());
    };

    let user_id = insert_user(&db.pool, "user").await?;
    let admin_id = insert_user(&db.pool, "admin").await?;
    let equipment_id = insert_equipment(&db.pool, 2).await?;

    let booking =
        bookings::create_reserved_booking(&db.pool, user_id, &booking_request(equipment_id, 1))
            .await?;
    let booking = bookings::approve_booking(&db.pool, booking.id, admin_id).await?;
    assert_eq!(booking.status, bookings::STATUS_APPROVED);
    assert_eq!(available_quantity(&db.pool, equipment_id).await?, 1);

    let booking = bookings::cancel_booking(&db.pool, booking.id).await?;
    assert_eq!(booking.status, bookings::STATUS_CANCELLED);
    assert_eq!(available_quantity(&db.pool, equipment_id).await?, 2);

    cleanup(&db.pool, booking.id, equipment_id, &[user_id, admin_id]).await?;
    Ok(())
}

#[tokio::test]
async fn overbooking_fails_without_changing_available_stock() -> Result<()> {
    let Some(db) = connect_test_db().await? else {
        return Ok(());
    };

    let user_id = insert_user(&db.pool, "user").await?;
    let equipment_id = insert_equipment(&db.pool, 1).await?;

    let result =
        bookings::create_reserved_booking(&db.pool, user_id, &booking_request(equipment_id, 2))
            .await;

    assert!(matches!(result, Err(AppError::BadRequest(_))));
    assert_eq!(available_quantity(&db.pool, equipment_id).await?, 1);

    cleanup_equipment_and_users(&db.pool, &[equipment_id], &[user_id]).await?;
    Ok(())
}

#[tokio::test]
async fn unavailable_equipment_statuses_cannot_be_reserved() -> Result<()> {
    let Some(db) = connect_test_db().await? else {
        return Ok(());
    };

    let user_id = insert_user(&db.pool, "user").await?;
    let maintenance_id = insert_equipment_with_status(&db.pool, 1, "maintenance").await?;
    let unavailable_id = insert_equipment_with_status(&db.pool, 1, "unavailable").await?;

    for equipment_id in [maintenance_id, unavailable_id] {
        let result =
            bookings::create_reserved_booking(&db.pool, user_id, &booking_request(equipment_id, 1))
                .await;

        assert!(matches!(result, Err(AppError::BadRequest(_))));
        assert_eq!(available_quantity(&db.pool, equipment_id).await?, 1);
    }

    cleanup_equipment_and_users(&db.pool, &[maintenance_id, unavailable_id], &[user_id]).await?;
    Ok(())
}
