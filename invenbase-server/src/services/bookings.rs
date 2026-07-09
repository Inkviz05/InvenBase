use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{Booking, CreateBookingRequest};

pub const STATUS_PENDING: &str = "pending";
pub const STATUS_APPROVED: &str = "approved";
pub const STATUS_REJECTED: &str = "rejected";
pub const STATUS_CANCELLED: &str = "cancelled";
pub const STATUS_EXPIRED: &str = "expired";
pub const STATUS_AWAITING_RETURN: &str = "awaiting_return";
pub const STATUS_RETURNED: &str = "returned";
pub const PERMISSION_TYPE_INTERNAL: &str = "internal";
pub const PERMISSION_TYPE_EXTERNAL: &str = "external";

pub fn booking_holds_equipment(status: &str) -> bool {
    matches!(
        status,
        STATUS_PENDING | STATUS_APPROVED | STATUS_AWAITING_RETURN
    )
}

pub fn validate_booking_request(req: &CreateBookingRequest) -> Result<(), AppError> {
    if req.equipment_id.is_none() && req.group_id.is_none() {
        return Err(AppError::BadRequest(
            "Нужно выбрать оборудование или группу".to_string(),
        ));
    }

    if req.equipment_id.is_some() && req.group_id.is_some() {
        return Err(AppError::BadRequest(
            "Нельзя выбрать оборудование и группу одновременно".to_string(),
        ));
    }

    if req.quantity <= 0 {
        return Err(AppError::BadRequest(
            "Количество должно быть больше нуля".to_string(),
        ));
    }

    if req.start_date >= req.end_date {
        return Err(AppError::BadRequest(
            "Дата начала должна быть раньше даты окончания".to_string(),
        ));
    }

    if let Some(permission_type) = req.permission_type.as_deref() {
        if !matches!(
            permission_type,
            PERMISSION_TYPE_INTERNAL | PERMISSION_TYPE_EXTERNAL
        ) {
            return Err(AppError::BadRequest("Invalid permission type".to_string()));
        }
    }

    Ok(())
}

pub async fn create_reserved_booking(
    pool: &PgPool,
    user_id: Uuid,
    req: &CreateBookingRequest,
) -> Result<Booking, AppError> {
    let mut tx = pool.begin().await?;
    let booking = create_reserved_booking_in_tx(&mut tx, user_id, req).await?;
    tx.commit().await?;

    Ok(booking)
}

pub async fn create_reserved_bookings(
    pool: &PgPool,
    user_id: Uuid,
    requests: &[CreateBookingRequest],
) -> Result<Vec<Booking>, AppError> {
    if requests.is_empty() {
        return Err(AppError::BadRequest("Список бронирований пуст".to_string()));
    }

    let mut tx = pool.begin().await?;
    let mut created = Vec::with_capacity(requests.len());

    for req in requests {
        created.push(create_reserved_booking_in_tx(&mut tx, user_id, req).await?);
    }

    tx.commit().await?;
    Ok(created)
}

async fn create_reserved_booking_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    req: &CreateBookingRequest,
) -> Result<Booking, AppError> {
    validate_booking_request(req)?;
    reserve_equipment_if_needed(tx, req.equipment_id, req.quantity).await?;

    let booking_id = Uuid::new_v4();
    let permission_type = req
        .permission_type
        .as_deref()
        .unwrap_or(PERMISSION_TYPE_INTERNAL)
        .to_string();

    sqlx::query::<Postgres>(
        "INSERT INTO bookings (id, user_id, equipment_id, group_id, quantity, start_date, end_date, purpose, permission_type, status)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'pending')"
    )
    .bind(booking_id)
    .bind(user_id)
    .bind(&req.equipment_id)
    .bind(&req.group_id)
    .bind(req.quantity)
    .bind(req.start_date)
    .bind(req.end_date)
    .bind(&req.purpose)
    .bind(&permission_type)
    .execute(&mut **tx)
    .await?;

    let booking = fetch_booking_for_update(tx, booking_id).await?;
    Ok(booking)
}

pub async fn approve_booking(
    pool: &PgPool,
    booking_id: Uuid,
    issued_by: Uuid,
) -> Result<Booking, AppError> {
    let mut tx = pool.begin().await?;
    let booking = fetch_booking_for_update(&mut tx, booking_id).await?;

    if booking.status != STATUS_PENDING {
        return Err(AppError::BadRequest(
            "Можно одобрить только ожидающую заявку".to_string(),
        ));
    }

    update_booking_status(&mut tx, booking_id, STATUS_APPROVED).await?;

    if let Some(equipment_id) = booking.equipment_id {
        sqlx::query::<Postgres>(
            "INSERT INTO permissions (id, booking_id, equipment_id, permission_type, issued_by, status)
             VALUES ($1, $2, $3, $4, $5, 'active')"
        )
        .bind(Uuid::new_v4())
        .bind(booking_id)
        .bind(equipment_id)
        .bind(booking.permission_type.clone())
        .bind(issued_by)
        .execute(&mut *tx)
        .await?;
    }

    let updated = fetch_booking_for_update(&mut tx, booking_id).await?;
    tx.commit().await?;
    Ok(updated)
}

pub async fn reject_booking(pool: &PgPool, booking_id: Uuid) -> Result<Booking, AppError> {
    transition_releasing_equipment(pool, booking_id, STATUS_REJECTED, &[STATUS_PENDING]).await
}

pub async fn cancel_booking(pool: &PgPool, booking_id: Uuid) -> Result<Booking, AppError> {
    transition_releasing_equipment(
        pool,
        booking_id,
        STATUS_CANCELLED,
        &[STATUS_PENDING, STATUS_APPROVED, STATUS_AWAITING_RETURN],
    )
    .await
}

pub async fn expire_pending_booking(pool: &PgPool, booking_id: Uuid) -> Result<Booking, AppError> {
    transition_releasing_equipment(pool, booking_id, STATUS_EXPIRED, &[STATUS_PENDING]).await
}

pub async fn mark_awaiting_return(pool: &PgPool, booking_id: Uuid) -> Result<Booking, AppError> {
    let mut tx = pool.begin().await?;
    let booking = fetch_booking_for_update(&mut tx, booking_id).await?;

    if booking.status != STATUS_APPROVED {
        return Err(AppError::BadRequest(
            "Ожидать возврата может только одобренная бронь".to_string(),
        ));
    }

    update_booking_status(&mut tx, booking_id, STATUS_AWAITING_RETURN).await?;
    let updated = fetch_booking_for_update(&mut tx, booking_id).await?;
    tx.commit().await?;
    Ok(updated)
}

pub async fn confirm_return(pool: &PgPool, booking_id: Uuid) -> Result<Booking, AppError> {
    transition_releasing_equipment(
        pool,
        booking_id,
        STATUS_RETURNED,
        &[STATUS_APPROVED, STATUS_AWAITING_RETURN],
    )
    .await
}

async fn transition_releasing_equipment(
    pool: &PgPool,
    booking_id: Uuid,
    target_status: &str,
    allowed_from: &[&str],
) -> Result<Booking, AppError> {
    let mut tx = pool.begin().await?;
    let booking = fetch_booking_for_update(&mut tx, booking_id).await?;

    if !allowed_from.contains(&booking.status.as_str()) {
        return Err(AppError::BadRequest(format!(
            "Недопустимый переход статуса: {} -> {}",
            booking.status, target_status
        )));
    }

    if booking_holds_equipment(&booking.status) {
        release_equipment_if_needed(&mut tx, booking.equipment_id, booking.quantity).await?;
    }

    update_booking_status(&mut tx, booking_id, target_status).await?;
    let updated = fetch_booking_for_update(&mut tx, booking_id).await?;
    tx.commit().await?;
    Ok(updated)
}

async fn reserve_equipment_if_needed(
    tx: &mut Transaction<'_, Postgres>,
    equipment_id: Option<Uuid>,
    quantity: i32,
) -> Result<(), AppError> {
    let Some(equipment_id) = equipment_id else {
        return Err(AppError::BadRequest(
            "Бронирование групп пока не поддерживает транзакционный расчет остатков".to_string(),
        ));
    };

    let equipment: Option<(i32, String)> = sqlx::query_as::<Postgres, _>(
        "SELECT available_quantity, status FROM equipment WHERE id = $1 FOR UPDATE",
    )
    .bind(equipment_id)
    .fetch_optional(&mut **tx)
    .await?;

    let Some((available_qty, status)) = equipment else {
        return Err(AppError::NotFound("Equipment not found".to_string()));
    };

    if status != "available" {
        return Err(AppError::BadRequest(
            "Оборудование сейчас недоступно для бронирования".to_string(),
        ));
    }

    if available_qty < quantity {
        return Err(AppError::BadRequest(format!(
            "Недостаточно оборудования. Доступно: {}",
            available_qty
        )));
    }

    sqlx::query::<Postgres>(
        "UPDATE equipment SET available_quantity = available_quantity - $1 WHERE id = $2",
    )
    .bind(quantity)
    .bind(equipment_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn release_equipment_if_needed(
    tx: &mut Transaction<'_, Postgres>,
    equipment_id: Option<Uuid>,
    quantity: i32,
) -> Result<(), AppError> {
    if let Some(equipment_id) = equipment_id {
        sqlx::query::<Postgres>(
            "UPDATE equipment SET available_quantity = LEAST(quantity, available_quantity + $1) WHERE id = $2"
        )
        .bind(quantity)
        .bind(equipment_id)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

async fn fetch_booking_for_update(
    tx: &mut Transaction<'_, Postgres>,
    booking_id: Uuid,
) -> Result<Booking, AppError> {
    let booking = sqlx::query_as::<Postgres, Booking>(
        "SELECT id, user_id, equipment_id, group_id, quantity, start_date, end_date, purpose, status, permission_type, created_at, updated_at
         FROM bookings WHERE id = $1 FOR UPDATE"
    )
    .bind(booking_id)
    .fetch_optional(&mut **tx)
    .await?;

    booking.ok_or_else(|| AppError::NotFound("Booking not found".to_string()))
}

async fn update_booking_status(
    tx: &mut Transaction<'_, Postgres>,
    booking_id: Uuid,
    status: &str,
) -> Result<(), AppError> {
    sqlx::query::<Postgres>(
        "UPDATE bookings SET status = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2",
    )
    .bind(status)
    .bind(booking_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    fn valid_request() -> CreateBookingRequest {
        let start_date = Utc::now();
        CreateBookingRequest {
            equipment_id: Some(Uuid::new_v4()),
            group_id: None,
            quantity: 1,
            start_date,
            end_date: start_date + Duration::hours(1),
            purpose: Some("test".to_string()),
            permission_type: Some(PERMISSION_TYPE_INTERNAL.to_string()),
        }
    }

    fn assert_bad_request(result: Result<(), AppError>) {
        assert!(matches!(result, Err(AppError::BadRequest(_))));
    }

    #[test]
    fn booking_holds_equipment_only_for_active_stock_statuses() {
        assert!(booking_holds_equipment(STATUS_PENDING));
        assert!(booking_holds_equipment(STATUS_APPROVED));
        assert!(booking_holds_equipment(STATUS_AWAITING_RETURN));

        assert!(!booking_holds_equipment(STATUS_REJECTED));
        assert!(!booking_holds_equipment(STATUS_CANCELLED));
        assert!(!booking_holds_equipment(STATUS_EXPIRED));
        assert!(!booking_holds_equipment(STATUS_RETURNED));
    }

    #[test]
    fn validate_booking_request_accepts_valid_equipment_booking() {
        assert!(validate_booking_request(&valid_request()).is_ok());
    }

    #[test]
    fn validate_booking_request_rejects_missing_target() {
        let mut req = valid_request();
        req.equipment_id = None;

        assert_bad_request(validate_booking_request(&req));
    }

    #[test]
    fn validate_booking_request_rejects_multiple_targets() {
        let mut req = valid_request();
        req.group_id = Some(Uuid::new_v4());

        assert_bad_request(validate_booking_request(&req));
    }

    #[test]
    fn validate_booking_request_rejects_non_positive_quantity() {
        let mut req = valid_request();
        req.quantity = 0;

        assert_bad_request(validate_booking_request(&req));
    }

    #[test]
    fn validate_booking_request_rejects_invalid_date_order() {
        let mut req = valid_request();
        req.end_date = req.start_date;

        assert_bad_request(validate_booking_request(&req));
    }

    #[test]
    fn validate_booking_request_rejects_invalid_permission_type() {
        let mut req = valid_request();
        req.permission_type = Some("temporary".to_string());

        assert_bad_request(validate_booking_request(&req));
    }
}
