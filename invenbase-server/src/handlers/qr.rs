use actix_web::{web, HttpResponse};
use image::{codecs::png::PngEncoder, GrayImage, ImageEncoder, Luma};
use qrcode::{Color, QrCode};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::auth::Claims;
use crate::errors::AppError;

const MODULE_SIZE: u32 = 8;
const QUIET_ZONE: u32 = 4;

pub async fn generate_qr_code(
    state: web::Data<AppState>,
    _claims: Claims,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let equipment_id = path.into_inner();

    let qr_code: Option<(String,)> =
        sqlx::query_as::<sqlx::Postgres, _>("SELECT qr_code FROM equipment WHERE id = $1")
            .bind(equipment_id)
            .fetch_optional(&state.db.pool)
            .await?;

    let qr_code_str = qr_code
        .ok_or_else(|| AppError::NotFound("Equipment not found".to_string()))?
        .0;

    let code = QrCode::new(qr_code_str.as_bytes())
        .map_err(|e| AppError::InternalError(format!("Failed to generate QR code: {}", e)))?;

    let size = code.width();
    let total = (size as u32) * MODULE_SIZE + 2 * QUIET_ZONE * MODULE_SIZE;
    let mut img = GrayImage::from_pixel(total, total, Luma([255u8]));

    for y in 0..size {
        for x in 0..size {
            if code[(x, y)] == Color::Dark {
                let px = (QUIET_ZONE * MODULE_SIZE) + (x as u32 * MODULE_SIZE);
                let py = (QUIET_ZONE * MODULE_SIZE) + (y as u32 * MODULE_SIZE);
                for dy in 0..MODULE_SIZE {
                    for dx in 0..MODULE_SIZE {
                        img.put_pixel(px + dx, py + dy, Luma([0u8]));
                    }
                }
            }
        }
    }

    let (width, height) = img.dimensions();
    let mut png_bytes: Vec<u8> = Vec::new();
    PngEncoder::new(&mut png_bytes)
        .write_image(img.as_raw(), width, height, image::ColorType::L8)
        .map_err(|e| AppError::InternalError(format!("Failed to encode PNG: {}", e)))?;

    Ok(HttpResponse::Ok().content_type("image/png").body(png_bytes))
}

pub async fn get_qr_code_data(
    state: web::Data<AppState>,
    _claims: Claims,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let equipment_id = path.into_inner();

    let row: Option<(String, Option<String>, Option<String>)> =
        sqlx::query_as::<sqlx::Postgres, _>(
            "SELECT qr_code, name, description FROM equipment WHERE id = $1",
        )
        .bind(equipment_id)
        .fetch_optional(&state.db.pool)
        .await?;

    let (qr_code, name, description) =
        row.ok_or_else(|| AppError::NotFound("Equipment not found".to_string()))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "equipment_id": equipment_id,
        "qr_code": qr_code,
        "name": name,
        "description": description
    })))
}
