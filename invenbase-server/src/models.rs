use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc, NaiveDate};
use uuid::Uuid;

// ========== User Models ==========

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub email: Option<String>,
    pub full_name: Option<String>,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub full_name: Option<String>,
    pub role: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            id: user.id,
            username: user.username,
            email: user.email,
            full_name: user.full_name,
            role: user.role,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
    pub full_name: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub full_name: Option<String>,
    pub role: Option<String>,
    pub password: Option<String>,
}

// ========== Equipment Models ==========

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Equipment {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
    pub squad_id: Option<Uuid>,
    pub quantity: i32,
    pub available_quantity: i32,
    #[serde(default)]
    pub is_unique: bool,
    pub location: Option<String>,
    pub qr_code: Option<String>,
    pub responsible_user_id: Option<Uuid>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct EquipmentWithDetails {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
    pub category_name: Option<String>,
    pub squad_id: Option<Uuid>,
    pub squad_name: Option<String>,
    pub quantity: i32,
    pub available_quantity: i32,
    #[serde(default)]
    pub is_unique: bool,
    pub location: Option<String>,
    pub qr_code: Option<String>,
    pub responsible_user_id: Option<Uuid>,
    pub responsible_name: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEquipmentRequest {
    pub name: String,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
    pub squad_id: Option<Uuid>,
    pub quantity: Option<i32>,
    pub location: Option<String>,
    pub responsible_user_id: Option<Uuid>,
    #[serde(default)]
    pub is_unique: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateEquipmentRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
    pub squad_id: Option<Uuid>,
    pub quantity: Option<i32>,
    pub available_quantity: Option<i32>,
    pub location: Option<String>,
    pub responsible_user_id: Option<Uuid>,
    pub status: Option<String>,
    pub is_unique: Option<bool>,
}

// ========== Category Models ==========

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct EquipmentCategory {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub squad_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct EquipmentCategoryWithSquad {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub squad_id: Option<Uuid>,
    pub squad_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub description: Option<String>,
    pub squad_id: Option<Uuid>,
}

// ========== Equipment Group Models ==========

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct EquipmentGroup {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEquipmentGroupRequest {
    pub name: String,
    pub description: Option<String>,
    pub equipment_items: Vec<EquipmentGroupItemRequest>,
}

#[derive(Debug, Deserialize)]
pub struct EquipmentGroupItemRequest {
    pub equipment_id: Uuid,
    pub quantity: i32,
}

// ========== Squad Models ==========

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Squad {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub responsible_user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct SquadWithDetails {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub responsible_user_id: Option<Uuid>,
    pub responsible_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSquadRequest {
    pub name: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub responsible_user_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSquadRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub responsible_user_id: Option<Uuid>,
}

// ========== Booking Models ==========

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Booking {
    pub id: Uuid,
    pub user_id: Uuid,
    pub equipment_id: Option<Uuid>,
    pub group_id: Option<Uuid>,
    pub quantity: i32,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub purpose: Option<String>,
    pub status: String,
    pub permission_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct BookingWithDetails {
    pub id: Uuid,
    pub user_id: Uuid,
    pub username: Option<String>,
    pub full_name: Option<String>,
    pub equipment_id: Option<Uuid>,
    pub equipment_name: Option<String>,
    pub group_id: Option<Uuid>,
    pub group_name: Option<String>,
    pub quantity: i32,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub purpose: Option<String>,
    pub status: String,
    pub permission_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBookingRequest {
    pub equipment_id: Option<Uuid>,
    pub group_id: Option<Uuid>,
    pub quantity: i32,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub purpose: Option<String>,
    pub permission_type: Option<String>,
}

/// Тело запроса массового создания бронирований из корзины (одно уведомление вместо N).
#[derive(Debug, Deserialize)]
pub struct CreateBulkBookingsRequest {
    pub bookings: Vec<CreateBookingRequest>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateBookingRequest {
    pub status: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

// ========== Permission Models ==========

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Permission {
    pub id: Uuid,
    pub booking_id: Uuid,
    pub equipment_id: Option<Uuid>,
    pub permission_type: String,
    pub issued_by: Option<Uuid>,
    pub issued_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct CreatePermissionRequest {
    pub booking_id: Uuid,
    pub equipment_id: Option<Uuid>,
    pub permission_type: String,
    pub expires_at: Option<DateTime<Utc>>,
}

// ========== Activity Log Models ==========

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ActivityLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Option<Uuid>,
    pub details: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

// ========== Equipment Movement Models ==========

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct EquipmentMovement {
    pub id: Uuid,
    pub equipment_id: Uuid,
    pub from_squad_id: Option<Uuid>,
    pub to_squad_id: Option<Uuid>,
    pub from_location: Option<String>,
    pub to_location: Option<String>,
    pub moved_by: Option<Uuid>,
    pub moved_by_name: Option<String>,
    pub comment: Option<String>,
    pub moved_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEquipmentMovementRequest {
    pub to_squad_id: Option<Uuid>,
    pub to_location: Option<String>,
    pub comment: Option<String>,
}

// ========== Notification Models ==========

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub message: String,
    pub notification_type: String,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateNotificationRequest {
    pub user_id: Uuid,
    pub title: String,
    pub message: String,
    pub notification_type: String,
}

// ========== Report Models ==========

#[derive(Debug, Serialize)]
pub struct EquipmentReport {
    pub total_equipment: i64,
    pub available_equipment: i64,
    pub booked_equipment: i64,
    pub by_category: Vec<CategoryStatistics>,
}

#[derive(Debug, Deserialize)]
pub struct BookingReportQuery {
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
}

/// Запись единого журнала учёта (аудит): действие пользователя с подстановкой ФИО/имени и наименования объекта.
#[derive(Debug, Serialize, FromRow)]
pub struct AuditReportEntry {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub username: Option<String>,
    pub full_name: Option<String>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Option<Uuid>,
    /// Наименование объекта (оборудование, пользователь, бронирование) вместо UUID.
    pub entity_name: Option<String>,
    pub details: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AuditReportQuery {
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub action: Option<String>,
    pub entity_type: Option<String>,
    pub user_id: Option<Uuid>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct CategoryStatistics {
    pub category_id: Uuid,
    pub category_name: String,
    pub total: i64,
    pub available: i64,
    pub booked: i64,
}

// ========== Support (техподдержка) ==========

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct SupportRequest {
    pub id: Uuid,
    pub user_id: Uuid,
    pub subject: String,
    pub message: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub admin_comment: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct SupportRequestWithUser {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_name: Option<String>,
    pub subject: String,
    pub message: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub admin_comment: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSupportRequestRequest {
    pub subject: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSupportRequestRequest {
    pub status: Option<String>,
    pub admin_comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct SupportRequestMessage {
    pub id: Uuid,
    pub support_request_id: Uuid,
    pub author_user_id: Option<Uuid>,
    pub is_staff: bool,
    pub message: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct SupportRequestWithMessages {
    #[serde(flatten)]
    pub request: SupportRequestWithUser,
    pub messages: Vec<SupportRequestMessage>,
}

#[derive(Debug, Deserialize)]
pub struct AddSupportMessageRequest {
    pub message: String,
}

