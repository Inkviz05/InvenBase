use actix_web::{web, HttpResponse};

use crate::app_state::AppState;
use crate::auth::{AuthService, Claims};
use crate::errors::AppError;
use crate::models::{
    AuditReportEntry, AuditReportQuery, BookingReportQuery, BookingWithDetails, CategoryStatistics,
    EquipmentReport,
};

pub async fn get_equipment_report(
    state: web::Data<AppState>,
    claims: Claims,
) -> Result<HttpResponse, AppError> {
    AuthService::require_any_role(&claims, &["admin", "responsible"])?;

    // Общая статистика
    let total: (i64,) = sqlx::query_as::<sqlx::Postgres, _>("SELECT COUNT(*) FROM equipment")
        .fetch_one(&state.db.pool)
        .await?;

    let available: (i64,) = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT COUNT(*) FROM equipment WHERE status = 'available' AND available_quantity > 0",
    )
    .fetch_one(&state.db.pool)
    .await?;

    let booked: (i64,) = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT COUNT(DISTINCT equipment_id) FROM bookings WHERE status = 'approved' AND end_date > CURRENT_TIMESTAMP"
    )
    .fetch_one(&state.db.pool)
    .await?;

    // Статистика по категориям
    let category_stats: Vec<CategoryStatistics> = sqlx::query_as::<sqlx::Postgres, _>(
        r#"
        SELECT
            c.id as category_id,
            c.name as category_name,
            COALESCE(SUM(e.quantity), 0) as total,
            COALESCE(SUM(e.available_quantity), 0) as available,
            COALESCE(SUM(e.quantity - e.available_quantity), 0) as booked
        FROM equipment_categories c
        LEFT JOIN equipment e ON c.id = e.category_id
        GROUP BY c.id, c.name
        ORDER BY c.name
        "#,
    )
    .fetch_all(&state.db.pool)
    .await?;

    let report = EquipmentReport {
        total_equipment: total.0,
        available_equipment: available.0,
        booked_equipment: booked.0,
        by_category: category_stats,
    };

    Ok(HttpResponse::Ok().json(report))
}

pub async fn get_booking_report(
    state: web::Data<AppState>,
    claims: Claims,
) -> Result<HttpResponse, AppError> {
    AuthService::require_any_role(&claims, &["admin", "responsible"])?;

    let pending: (i64,) = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT COUNT(*) FROM bookings WHERE status = 'pending'",
    )
    .fetch_one(&state.db.pool)
    .await?;

    let approved: (i64,) = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT COUNT(*) FROM bookings WHERE status = 'approved' AND end_date > CURRENT_TIMESTAMP",
    )
    .fetch_one(&state.db.pool)
    .await?;

    let expired: (i64,) = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT COUNT(*) FROM bookings WHERE status = 'expired'",
    )
    .fetch_one(&state.db.pool)
    .await?;

    let awaiting_return: (i64,) = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT COUNT(*) FROM bookings WHERE status = 'awaiting_return'",
    )
    .fetch_one(&state.db.pool)
    .await?;

    let returned: (i64,) = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT COUNT(*) FROM bookings WHERE status = 'returned'",
    )
    .fetch_one(&state.db.pool)
    .await?;

    let rejected: (i64,) = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT COUNT(*) FROM bookings WHERE status = 'rejected'",
    )
    .fetch_one(&state.db.pool)
    .await?;

    let cancelled: (i64,) = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT COUNT(*) FROM bookings WHERE status = 'cancelled'",
    )
    .fetch_one(&state.db.pool)
    .await?;

    let completed: (i64,) = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT COUNT(*) FROM bookings WHERE status = 'completed'",
    )
    .fetch_one(&state.db.pool)
    .await?;

    let total: (i64,) = sqlx::query_as::<sqlx::Postgres, _>("SELECT COUNT(*) FROM bookings")
        .fetch_one(&state.db.pool)
        .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "pending": pending.0,
        "approved": approved.0,
        "expired": expired.0,
        "awaiting_return": awaiting_return.0,
        "returned": returned.0,
        "rejected": rejected.0,
        "cancelled": cancelled.0,
        "completed": completed.0,
        "total": total.0
    })))
}

/// Детализированный отчёт по бронированиям за период
pub async fn get_booking_detailed_report(
    state: web::Data<AppState>,
    claims: Claims,
    query: web::Query<BookingReportQuery>,
) -> Result<HttpResponse, AppError> {
    AuthService::require_any_role(&claims, &["admin", "responsible"])?;

    let from = query.from;
    let to = query.to;

    let bookings: Vec<BookingWithDetails> = sqlx::query_as::<sqlx::Postgres, _>(
        r#"
        SELECT
            b.id, b.user_id, u.username, b.equipment_id, e.name as equipment_name,
            b.group_id, g.name as group_name, b.quantity, b.start_date, b.end_date,
            b.purpose, b.status, b.permission_type, b.created_at, b.updated_at
        FROM bookings b
        LEFT JOIN users u ON b.user_id = u.id
        LEFT JOIN equipment e ON b.equipment_id = e.id
        LEFT JOIN equipment_groups g ON b.group_id = g.id
        WHERE ($1::date IS NULL OR b.start_date::date >= $1::date)
          AND ($2::date IS NULL OR b.start_date::date <= $2::date)
        ORDER BY b.start_date ASC
        "#,
    )
    .bind(from)
    .bind(to)
    .fetch_all(&state.db.pool)
    .await?;

    Ok(HttpResponse::Ok().json(bookings))
}

/// Единый журнал учёта: все действия (оборудование, бронирования, пользователи, вход, перемещения) с фильтрами и выгрузкой.
pub async fn get_audit_report(
    state: web::Data<AppState>,
    claims: Claims,
    query: web::Query<AuditReportQuery>,
) -> Result<HttpResponse, AppError> {
    AuthService::require_any_role(&claims, &["admin", "responsible"])?;

    let limit = query.limit.unwrap_or(500).min(5000);
    let offset = query.offset.unwrap_or(0);

    let logs: Vec<AuditReportEntry> = sqlx::query_as::<sqlx::Postgres, _>(
        r#"
        SELECT
            l.id,
            l.user_id,
            u.username,
            u.full_name,
            l.action,
            l.entity_type,
            l.entity_id,
            CASE
                WHEN l.entity_type = 'equipment' THEN (SELECT name FROM equipment WHERE id = l.entity_id LIMIT 1)
                WHEN l.entity_type = 'user' THEN (SELECT COALESCE(TRIM(NULLIF(full_name, '')), username) FROM users WHERE id = l.entity_id LIMIT 1)
                WHEN l.entity_type = 'booking' THEN (SELECT COALESCE(e.name, 'Бронирование') FROM bookings b LEFT JOIN equipment e ON e.id = b.equipment_id WHERE b.id = l.entity_id LIMIT 1)
                ELSE NULL
            END AS entity_name,
            l.details,
            l.created_at
        FROM activity_logs l
        LEFT JOIN users u ON l.user_id = u.id
        WHERE ($1::date IS NULL OR (l.created_at AT TIME ZONE 'UTC')::date >= $1)
          AND ($2::date IS NULL OR (l.created_at AT TIME ZONE 'UTC')::date <= $2)
          AND ($3::text IS NULL OR l.action = $3)
          AND ($4::text IS NULL OR l.entity_type = $4)
          AND ($5::uuid IS NULL OR l.user_id = $5)
        ORDER BY l.created_at DESC
        LIMIT $6 OFFSET $7
        "#
    )
    .bind(query.from)
    .bind(query.to)
    .bind(&query.action)
    .bind(&query.entity_type)
    .bind(query.user_id)
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(&state.db.pool)
    .await?;

    Ok(HttpResponse::Ok().json(logs))
}
