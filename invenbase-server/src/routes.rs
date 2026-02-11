use actix_web::{web, HttpResponse};
use crate::app_state::AppState;
use crate::handlers::*;
use crate::middleware::{Authenticated, AdminOnly, AdminOrResponsible};

pub fn configure_api(cfg: &mut web::ServiceConfig) {
    cfg
        // Аутентификация (публичные endpoints)
        .route("/auth/login", web::post().to(auth::login))
        
        // Пользователи
        .service(
            web::scope("/users")
                .route("/me", web::get().to(|state: web::Data<AppState>, auth: Authenticated| async move {
                    auth::get_current_user(state, auth.claims()).await
                }))
                .route("", web::get().to(|state: web::Data<AppState>, auth: AdminOnly| async move {
                    users::get_users(state, auth.claims()).await
                }))
                .route("", web::post().to(|state: web::Data<AppState>, auth: AdminOnly, req: web::Json<_>| async move {
                    users::create_user(state, auth.claims(), req).await
                }))
                .route("/{id}", web::get().to(|state: web::Data<AppState>, auth: Authenticated, path: web::Path<_>| async move {
                    users::get_user(state, auth.claims(), path).await
                }))
                .route("/{id}", web::put().to(|state: web::Data<AppState>, auth: Authenticated, path: web::Path<_>, req: web::Json<_>| async move {
                    users::update_user(state, auth.claims(), path, req).await
                }))
                .route("/{id}", web::delete().to(|state: web::Data<AppState>, auth: AdminOnly, path: web::Path<_>| async move {
                    users::delete_user(state, auth.claims(), path).await
                }))
        )
        
        // Оборудование
        .service(
            web::scope("/equipment")
                .route("", web::get().to(|state: web::Data<AppState>, auth: Authenticated, query: web::Query<equipment::EquipmentListQuery>| async move {
                    equipment::get_equipment_list(state, auth.claims(), query).await
                }))
                .route("", web::post().to(|state: web::Data<AppState>, auth: AdminOrResponsible, req: web::Json<_>| async move {
                    equipment::create_equipment(state, auth.claims(), req).await
                }))
                .route("/qr/{qr_code}", web::get().to(|state: web::Data<AppState>, auth: Authenticated, path: web::Path<_>| async move {
                    equipment::get_equipment_by_qr(state, auth.claims(), path).await
                }))
                .route("/{id}/movements", web::get().to(|state: web::Data<AppState>, auth: Authenticated, path: web::Path<_>| async move {
                    equipment::get_equipment_movements(state, auth.claims(), path).await
                }))
                .route("/{id}/move", web::post().to(|state: web::Data<AppState>, auth: AdminOrResponsible, path: web::Path<_>, req: web::Json<_>| async move {
                    equipment::move_equipment(state, auth.claims(), path, req).await
                }))
                .route("/{id}", web::get().to(|state: web::Data<AppState>, auth: Authenticated, path: web::Path<_>| async move {
                    equipment::get_equipment(state, auth.claims(), path).await
                }))
                .route("/{id}", web::put().to(|state: web::Data<AppState>, auth: AdminOrResponsible, path: web::Path<_>, req: web::Json<_>| async move {
                    equipment::update_equipment(state, auth.claims(), path, req).await
                }))
                .route("/{id}", web::delete().to(|state: web::Data<AppState>, auth: AdminOrResponsible, path: web::Path<_>| async move {
                    equipment::delete_equipment(state, auth.claims(), path).await
                }))
        )
        
        // Категории
        .service(
            web::scope("/categories")
                .route("", web::get().to(|state: web::Data<AppState>, auth: Authenticated, query: web::Query<categories::CategoriesQuery>| async move {
                    categories::get_categories(state, auth.claims(), query).await
                }))
                .route("", web::post().to(|state: web::Data<AppState>, auth: AdminOrResponsible, req: web::Json<_>| async move {
                    categories::create_category(state, auth.claims(), req).await
                }))
                .route("/{id}", web::get().to(|state: web::Data<AppState>, auth: Authenticated, path: web::Path<_>| async move {
                    categories::get_category(state, auth.claims(), path).await
                }))
                .route("/{id}", web::put().to(|state: web::Data<AppState>, auth: AdminOrResponsible, path: web::Path<_>, req: web::Json<_>| async move {
                    categories::update_category(state, auth.claims(), path, req).await
                }))
                .route("/{id}", web::delete().to(|state: web::Data<AppState>, auth: AdminOrResponsible, path: web::Path<_>| async move {
                    categories::delete_category(state, auth.claims(), path).await
                }))
        )
        
        // Группы оборудования
        .service(
            web::scope("/groups")
                .route("", web::get().to(|state: web::Data<AppState>, auth: Authenticated| async move {
                    groups::get_groups(state, auth.claims()).await
                }))
                .route("", web::post().to(|state: web::Data<AppState>, auth: AdminOrResponsible, req: web::Json<_>| async move {
                    groups::create_group(state, auth.claims(), req).await
                }))
                .route("/{id}", web::get().to(|state: web::Data<AppState>, auth: Authenticated, path: web::Path<_>| async move {
                    groups::get_group(state, auth.claims(), path).await
                }))
                .route("/{id}", web::put().to(|state: web::Data<AppState>, auth: AdminOrResponsible, path: web::Path<_>, req: web::Json<_>| async move {
                    groups::update_group(state, auth.claims(), path, req).await
                }))
                .route("/{id}", web::delete().to(|state: web::Data<AppState>, auth: AdminOrResponsible, path: web::Path<_>| async move {
                    groups::delete_group(state, auth.claims(), path).await
                }))
        )
        
        // Сквады (для разделения кабинетов и зон ответственности)
        .service(
            web::scope("/squads")
                .route("", web::get().to(|state: web::Data<AppState>, auth: Authenticated| async move {
                    squads::get_squads(state, auth.claims()).await
                }))
                .route("", web::post().to(|state: web::Data<AppState>, auth: AdminOrResponsible, req: web::Json<_>| async move {
                    squads::create_squad(state, auth.claims(), req).await
                }))
                .route("/{id}/equipment", web::get().to(|state: web::Data<AppState>, auth: Authenticated, path: web::Path<_>| async move {
                    squads::get_squad_equipment(state, auth.claims(), path).await
                }))
                .route("/{id}", web::get().to(|state: web::Data<AppState>, auth: Authenticated, path: web::Path<_>| async move {
                    squads::get_squad(state, auth.claims(), path).await
                }))
                .route("/{id}", web::put().to(|state: web::Data<AppState>, auth: AdminOrResponsible, path: web::Path<_>, req: web::Json<_>| async move {
                    squads::update_squad(state, auth.claims(), path, req).await
                }))
                .route("/{id}", web::delete().to(|state: web::Data<AppState>, auth: AdminOrResponsible, path: web::Path<_>| async move {
                    squads::delete_squad(state, auth.claims(), path).await
                }))
        )
        
        // Бронирования
        .service(
            web::scope("/bookings")
                .route("", web::get().to(|state: web::Data<AppState>, auth: Authenticated| async move {
                    bookings::get_bookings(state, auth.claims()).await
                }))
                .route("bulk", web::post().to(|state: web::Data<AppState>, auth: Authenticated, req: web::Json<_>| async move {
                    bookings::create_bulk_bookings(state, auth.claims(), req).await
                }))
                .route("", web::post().to(|state: web::Data<AppState>, auth: Authenticated, req: web::Json<_>| async move {
                    bookings::create_booking(state, auth.claims(), req).await
                }))
                .route("/{id}", web::get().to(|state: web::Data<AppState>, auth: Authenticated, path: web::Path<_>| async move {
                    bookings::get_booking(state, auth.claims(), path).await
                }))
                .route("/{id}", web::put().to(|state: web::Data<AppState>, auth: Authenticated, path: web::Path<_>, req: web::Json<_>| async move {
                    bookings::update_booking(state, auth.claims(), path, req).await
                }))
                .route("/{id}/approve", web::post().to(|state: web::Data<AppState>, auth: AdminOrResponsible, path: web::Path<_>| async move {
                    bookings::approve_booking(state, auth.claims(), path).await
                }))
                .route("/{id}/reject", web::post().to(|state: web::Data<AppState>, auth: AdminOrResponsible, path: web::Path<_>| async move {
                    bookings::reject_booking(state, auth.claims(), path).await
                }))
                .route("/{id}", web::delete().to(|state: web::Data<AppState>, auth: Authenticated, path: web::Path<_>| async move {
                    bookings::delete_booking(state, auth.claims(), path).await
                }))
        )
        
        // Разрешения
        .service(
            web::scope("/permissions")
                .route("", web::get().to(|state: web::Data<AppState>, auth: Authenticated| async move {
                    permissions::get_permissions(state, auth.claims()).await
                }))
                .route("", web::post().to(|state: web::Data<AppState>, auth: AdminOrResponsible, req: web::Json<_>| async move {
                    permissions::create_permission(state, auth.claims(), req).await
                }))
                .route("/{id}/revoke", web::post().to(|state: web::Data<AppState>, auth: AdminOrResponsible, path: web::Path<_>| async move {
                    permissions::revoke_permission(state, auth.claims(), path).await
                }))
        )
        
        // Уведомления
        .service(
            web::scope("/notifications")
                .route("", web::get().to(|state: web::Data<AppState>, auth: Authenticated| async move {
                    notifications::get_notifications(state, auth.claims()).await
                }))
                // Старый путь, который использует веб-клиент
                .route("/count", web::get().to(|state: web::Data<AppState>, auth: Authenticated| async move {
                    notifications::get_unread_count(state, auth.claims()).await
                }))
                // Новый путь для Android: /notifications/unread-count
                .route("/unread-count", web::get().to(|state: web::Data<AppState>, auth: Authenticated| async move {
                    notifications::get_unread_count(state, auth.claims()).await
                }))
                .route("/{id}/read", web::post().to(|state: web::Data<AppState>, auth: Authenticated, path: web::Path<_>| async move {
                    notifications::mark_as_read(state, auth.claims(), path).await
                }))
                .route("/read-all", web::post().to(|state: web::Data<AppState>, auth: Authenticated| async move {
                    notifications::mark_all_as_read(state, auth.claims()).await
                }))
                .route("", web::post().to(|state: web::Data<AppState>, auth: AdminOrResponsible, req: web::Json<_>| async move {
                    notifications::create_notification(state, auth.claims(), req).await
                }))
        )
        
        // Логи
        .service(
            web::scope("/logs")
                .route("", web::get().to(|state: web::Data<AppState>, auth: AdminOrResponsible, query: web::Query<_>| async move {
                    logs::get_logs(state, auth.claims(), query).await
                }))
        )
        
        // Отчёты
        .service(
            web::scope("/reports")
                .route("/equipment", web::get().to(|state: web::Data<AppState>, auth: AdminOrResponsible| async move {
                    reports::get_equipment_report(state, auth.claims()).await
                }))
                .route("/bookings", web::get().to(|state: web::Data<AppState>, auth: AdminOrResponsible| async move {
                    reports::get_booking_report(state, auth.claims()).await
                }))
                .route("/bookings/detailed", web::get().to(|state: web::Data<AppState>, auth: AdminOrResponsible, query: web::Query<_>| async move {
                    reports::get_booking_detailed_report(state, auth.claims(), query).await
                }))
        )
        
        // QR-коды
        .service(
            web::scope("/qr")
                .route("/{id}", web::get().to(|state: web::Data<AppState>, auth: AdminOrResponsible, path: web::Path<_>| async move {
                    qr::generate_qr_code(state, auth.claims(), path).await
                }))
                .route("/{id}/data", web::get().to(|state: web::Data<AppState>, auth: Authenticated, path: web::Path<_>| async move {
                    qr::get_qr_code_data(state, auth.claims(), path).await
                }))
        )
        // Push-уведомления / регистрация токена
        .service(
            web::scope("/push")
                .route("/register-token", web::post().to(|state: web::Data<AppState>, auth: Authenticated, body: web::Json<_>| async move {
                    push::register_push_token(state, auth.claims(), body).await
                }))
        );
}

pub fn configure_web(cfg: &mut web::ServiceConfig) {
    cfg
        .route("/", web::get().to(|| async {
            HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(include_str!("../static/index.html"))
        }))
        .route("/admin", web::get().to(|| async {
            HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(include_str!("../static/admin.html"))
        }));
}

