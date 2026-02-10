use actix_web::{web, HttpResponse};

use crate::models::{EquipmentReport, CategoryStatistics, BookingWithDetails, BookingReportQuery};
use crate::auth::{AuthService, Claims};
use crate::errors::AppError;
use crate::app_state::AppState;

pub async fn get_equipment_report(
    state: web::Data<AppState>,
    claims: Claims,
) -> Result<HttpResponse, AppError> {
    AuthService::require_any_role(&claims, &["admin", "responsible"])?;

    // Общая статистика
    let total: (i64,) = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT COUNT(*) FROM equipment"
    )
    .fetch_one(&state.db.pool)
    .await?;

    let available: (i64,) = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT COUNT(*) FROM equipment WHERE status = 'available' AND available_quantity > 0"
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
        "#
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
        "SELECT COUNT(*) FROM bookings WHERE status = 'pending'"
    )
    .fetch_one(&state.db.pool)
    .await?;

    let approved: (i64,) = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT COUNT(*) FROM bookings WHERE status = 'approved' AND end_date > CURRENT_TIMESTAMP"
    )
    .fetch_one(&state.db.pool)
    .await?;

    let expired: (i64,) = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT COUNT(*) FROM bookings WHERE status = 'approved' AND end_date < CURRENT_TIMESTAMP"
    )
    .fetch_one(&state.db.pool)
    .await?;

    let rejected: (i64,) = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT COUNT(*) FROM bookings WHERE status = 'rejected'"
    )
    .fetch_one(&state.db.pool)
    .await?;

    let cancelled: (i64,) = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT COUNT(*) FROM bookings WHERE status = 'cancelled'"
    )
    .fetch_one(&state.db.pool)
    .await?;

    let completed: (i64,) = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT COUNT(*) FROM bookings WHERE status = 'completed'"
    )
    .fetch_one(&state.db.pool)
    .await?;

    let total: (i64,) = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT COUNT(*) FROM bookings"
    )
    .fetch_one(&state.db.pool)
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "pending": pending.0,
        "approved": approved.0,
        "expired": expired.0,
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
        "#
    )
    .bind(from)
    .bind(to)
    .fetch_all(&state.db.pool)
    .await?;

    Ok(HttpResponse::Ok().json(bookings))
}

