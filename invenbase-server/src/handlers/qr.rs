use actix_web::{web, HttpResponse};
use uuid::Uuid;
use qrcode::{QrCode, Color};

use crate::auth::Claims;
use crate::errors::AppError;
use crate::app_state::AppState;

pub async fn generate_qr_code(
    state: web::Data<AppState>,
    _claims: Claims,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let equipment_id = path.into_inner();

    // Получаем QR-код оборудования
    let qr_code: Option<(String,)> = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT qr_code FROM equipment WHERE id = $1"
    )
    .bind(equipment_id)
    .fetch_optional(&state.db.pool)
    .await?;

    let qr_code_str = qr_code.ok_or_else(|| AppError::NotFound("Equipment not found".to_string()))?.0;

    // Генерируем QR-код
    let code = QrCode::new(&qr_code_str)
        .map_err(|e| AppError::InternalError(format!("Failed to generate QR code: {}", e)))?;

    // Создаём SVG вручную для правильного отображения
    let size = code.width();
    let module_size = 10u32;
    let quiet_zone = 4u32;
    let total_size = (size as u32 * module_size) + (quiet_zone * 2 * module_size);
    
    let mut svg = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    svg.push_str(&format!(
        "<svg width=\"{}\" height=\"{}\" xmlns=\"http://www.w3.org/2000/svg\">\n",
        total_size, total_size
    ));
    
    // Белый фон
    svg.push_str(&format!(
        "<rect width=\"{}\" height=\"{}\" fill=\"white\"/>\n",
        total_size, total_size
    ));
    
    // Рисуем QR-код
    for y in 0..size {
        for x in 0..size {
            if code[(x, y)] == Color::Dark {
                let px = (quiet_zone * module_size) + (x as u32 * module_size);
                let py = (quiet_zone * module_size) + (y as u32 * module_size);
                svg.push_str(&format!(
                    "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"black\"/>\n",
                    px, py, module_size, module_size
                ));
            }
        }
    }
    
    svg.push_str("</svg>");

    Ok(HttpResponse::Ok()
        .content_type("image/svg+xml")
        .body(svg))
}

pub async fn get_qr_code_data(
    state: web::Data<AppState>,
    _claims: Claims,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let equipment_id = path.into_inner();

    let qr_code: Option<(String,)> = sqlx::query_as::<sqlx::Postgres, _>(
        "SELECT qr_code FROM equipment WHERE id = $1"
    )
    .bind(equipment_id)
    .fetch_optional(&state.db.pool)
    .await?;

    let qr_code = qr_code.ok_or_else(|| AppError::NotFound("Equipment not found".to_string()))?.0;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "equipment_id": equipment_id,
        "qr_code": qr_code
    })))
}
