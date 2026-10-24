use actix_web::HttpResponse;
use serde_json::{json, Map, Value};

fn operation(tag: &str, summary: &str, protected: bool) -> Value {
    let mut operation = json!({
        "tags": [tag],
        "summary": summary,
        "responses": {
            "200": { "description": "Success" },
            "400": {
                "description": "Invalid request",
                "content": { "application/json": { "schema": { "$ref": "#/components/schemas/ApiErrorResponse" } } }
            },
            "401": {
                "description": "Authentication required",
                "content": { "application/json": { "schema": { "$ref": "#/components/schemas/ApiErrorResponse" } } }
            },
            "403": {
                "description": "Insufficient permissions",
                "content": { "application/json": { "schema": { "$ref": "#/components/schemas/ApiErrorResponse" } } }
            },
            "404": {
                "description": "Resource not found",
                "content": { "application/json": { "schema": { "$ref": "#/components/schemas/ApiErrorResponse" } } }
            },
            "409": {
                "description": "State conflict",
                "content": { "application/json": { "schema": { "$ref": "#/components/schemas/ApiErrorResponse" } } }
            },
            "500": {
                "description": "Internal server error",
                "content": { "application/json": { "schema": { "$ref": "#/components/schemas/ApiErrorResponse" } } }
            }
        }
    });
    if protected {
        operation["security"] = json!([{ "bearerAuth": [] }]);
    }
    operation
}

fn add_operation(
    paths: &mut Map<String, Value>,
    path: &str,
    method: &str,
    tag: &str,
    summary: &str,
    protected: bool,
) {
    let path_item = paths
        .entry(path.to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    path_item
        .as_object_mut()
        .expect("path item must be an object")
        .insert(method.to_string(), operation(tag, summary, protected));
}

pub fn document() -> Value {
    let mut paths = Map::new();
    for (path, method, tag, summary, protected) in [
        ("/health", "get", "System", "Liveness check", false),
        ("/ready", "get", "System", "Database readiness check", false),
        (
            "/api/auth/login",
            "post",
            "Auth",
            "Authenticate user",
            false,
        ),
        ("/api/users/me", "get", "Users", "Get current user", true),
        ("/api/users", "get", "Users", "List users", true),
        ("/api/users", "post", "Users", "Create user", true),
        ("/api/users/{id}", "get", "Users", "Get user", true),
        ("/api/users/{id}", "put", "Users", "Update user", true),
        ("/api/users/{id}", "delete", "Users", "Delete user", true),
        ("/api/equipment", "get", "Equipment", "List equipment", true),
        (
            "/api/equipment",
            "post",
            "Equipment",
            "Create equipment",
            true,
        ),
        (
            "/api/equipment/{id}",
            "get",
            "Equipment",
            "Get equipment",
            true,
        ),
        (
            "/api/equipment/{id}",
            "put",
            "Equipment",
            "Update equipment",
            true,
        ),
        (
            "/api/equipment/{id}",
            "delete",
            "Equipment",
            "Delete equipment",
            true,
        ),
        (
            "/api/equipment/qr/{qr_code}",
            "get",
            "Equipment",
            "Find equipment by QR",
            true,
        ),
        (
            "/api/equipment/{id}/movements",
            "get",
            "Equipment",
            "List equipment movements",
            true,
        ),
        (
            "/api/equipment/{id}/move",
            "post",
            "Equipment",
            "Move equipment",
            true,
        ),
        (
            "/api/categories",
            "get",
            "Categories",
            "List categories",
            true,
        ),
        (
            "/api/categories",
            "post",
            "Categories",
            "Create category",
            true,
        ),
        (
            "/api/categories/{id}",
            "get",
            "Categories",
            "Get category",
            true,
        ),
        (
            "/api/categories/{id}",
            "put",
            "Categories",
            "Update category",
            true,
        ),
        (
            "/api/categories/{id}",
            "delete",
            "Categories",
            "Delete category",
            true,
        ),
        (
            "/api/groups",
            "get",
            "Groups",
            "List equipment groups",
            true,
        ),
        (
            "/api/groups",
            "post",
            "Groups",
            "Create equipment group",
            true,
        ),
        (
            "/api/groups/{id}",
            "get",
            "Groups",
            "Get equipment group",
            true,
        ),
        (
            "/api/groups/{id}",
            "put",
            "Groups",
            "Update equipment group",
            true,
        ),
        (
            "/api/groups/{id}",
            "delete",
            "Groups",
            "Delete equipment group",
            true,
        ),
        ("/api/squads", "get", "Squads", "List squads", true),
        ("/api/squads", "post", "Squads", "Create squad", true),
        ("/api/squads/{id}", "get", "Squads", "Get squad", true),
        ("/api/squads/{id}", "put", "Squads", "Update squad", true),
        ("/api/squads/{id}", "delete", "Squads", "Delete squad", true),
        (
            "/api/squads/{id}/equipment",
            "get",
            "Squads",
            "List squad equipment",
            true,
        ),
        ("/api/bookings", "get", "Bookings", "List bookings", true),
        ("/api/bookings", "post", "Bookings", "Create booking", true),
        (
            "/api/bookings/bulk",
            "post",
            "Bookings",
            "Create bookings in bulk",
            true,
        ),
        ("/api/bookings/{id}", "get", "Bookings", "Get booking", true),
        (
            "/api/bookings/{id}",
            "put",
            "Bookings",
            "Update booking",
            true,
        ),
        (
            "/api/bookings/{id}",
            "delete",
            "Bookings",
            "Delete booking",
            true,
        ),
        (
            "/api/bookings/{id}/approve",
            "post",
            "Bookings",
            "Approve booking",
            true,
        ),
        (
            "/api/bookings/{id}/reject",
            "post",
            "Bookings",
            "Reject booking",
            true,
        ),
        (
            "/api/bookings/{id}/cancel",
            "post",
            "Bookings",
            "Cancel booking",
            true,
        ),
        (
            "/api/bookings/{id}/return",
            "post",
            "Bookings",
            "Confirm equipment return",
            true,
        ),
        (
            "/api/permissions",
            "get",
            "Permissions",
            "List permissions",
            true,
        ),
        (
            "/api/permissions",
            "post",
            "Permissions",
            "Create permission",
            true,
        ),
        (
            "/api/permissions/{id}/revoke",
            "post",
            "Permissions",
            "Revoke permission",
            true,
        ),
        (
            "/api/notifications",
            "get",
            "Notifications",
            "List notifications",
            true,
        ),
        (
            "/api/notifications",
            "post",
            "Notifications",
            "Create notification",
            true,
        ),
        (
            "/api/notifications/count",
            "get",
            "Notifications",
            "Get unread count",
            true,
        ),
        (
            "/api/notifications/unread-count",
            "get",
            "Notifications",
            "Get unread count",
            true,
        ),
        (
            "/api/notifications/{id}/read",
            "post",
            "Notifications",
            "Mark notification read",
            true,
        ),
        (
            "/api/notifications/read-all",
            "post",
            "Notifications",
            "Mark all notifications read",
            true,
        ),
        (
            "/api/push/register-token",
            "post",
            "Push",
            "Register device push token",
            true,
        ),
        (
            "/api/push/unregister-device",
            "post",
            "Push",
            "Deactivate push device",
            true,
        ),
        ("/api/logs", "get", "Audit", "List activity logs", true),
        (
            "/api/reports/equipment",
            "get",
            "Reports",
            "Equipment report",
            true,
        ),
        (
            "/api/reports/bookings",
            "get",
            "Reports",
            "Bookings report",
            true,
        ),
        (
            "/api/reports/bookings/detailed",
            "get",
            "Reports",
            "Detailed bookings report",
            true,
        ),
        ("/api/reports/audit", "get", "Reports", "Audit report", true),
        (
            "/api/qr/{id}",
            "get",
            "QR",
            "Generate equipment QR code",
            true,
        ),
        (
            "/api/qr/{id}/data",
            "get",
            "QR",
            "Get equipment QR data",
            true,
        ),
        (
            "/api/support/requests",
            "get",
            "Support",
            "List support requests",
            true,
        ),
        (
            "/api/support/requests",
            "post",
            "Support",
            "Create support request",
            true,
        ),
        (
            "/api/support/requests/{id}",
            "put",
            "Support",
            "Update support request",
            true,
        ),
        (
            "/api/support/requests/{id}",
            "delete",
            "Support",
            "Delete support request",
            true,
        ),
        (
            "/api/support/requests/{id}/messages",
            "post",
            "Support",
            "Add support message",
            true,
        ),
    ] {
        add_operation(&mut paths, path, method, tag, summary, protected);
    }

    paths["/api/auth/login"]["post"]["requestBody"] = json!({
        "required": true,
        "content": { "application/json": { "schema": { "$ref": "#/components/schemas/LoginRequest" } } }
    });
    paths["/api/auth/login"]["post"]["responses"]["200"] = json!({
        "description": "JWT and authenticated user",
        "content": { "application/json": { "schema": { "$ref": "#/components/schemas/LoginResponse" } } }
    });

    json!({
        "openapi": "3.1.0",
        "info": {
            "title": "InvenBase API",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "Inventory, booking and multi-device notification API"
        },
        "paths": paths,
        "components": {
            "securitySchemes": {
                "bearerAuth": { "type": "http", "scheme": "bearer", "bearerFormat": "JWT" }
            },
            "schemas": {
                "ApiErrorResponse": {
                    "type": "object",
                    "required": ["code", "message", "details", "request_id"],
                    "properties": {
                        "code": { "type": "string", "example": "VALIDATION_ERROR" },
                        "message": { "type": "string" },
                        "details": { "type": ["object", "null"], "additionalProperties": true },
                        "request_id": { "type": ["string", "null"], "format": "uuid" }
                    }
                },
                "LoginRequest": {
                    "type": "object",
                    "required": ["username", "password"],
                    "properties": {
                        "username": { "type": "string" },
                        "password": { "type": "string", "format": "password" }
                    }
                },
                "LoginResponse": {
                    "type": "object",
                    "required": ["token", "user"],
                    "properties": {
                        "token": { "type": "string" },
                        "user": { "type": "object", "additionalProperties": true }
                    }
                }
            }
        }
    })
}

pub async fn openapi_json() -> HttpResponse {
    HttpResponse::Ok().json(document())
}

pub async fn swagger_ui() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(
            r#"<!doctype html>
<html><head><meta charset="utf-8"><title>InvenBase API docs</title>
<link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css"></head>
<body><div id="swagger-ui"></div>
<script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
<script>SwaggerUIBundle({url:'/openapi.json',dom_id:'#swagger-ui',persistAuthorization:true});</script>
</body></html>"#,
        )
}

#[cfg(test)]
mod tests {
    use super::document;

    #[test]
    fn contract_contains_security_errors_and_critical_paths() {
        let document = document();
        assert_eq!(document["openapi"], "3.1.0");
        assert_eq!(
            document["components"]["securitySchemes"]["bearerAuth"]["scheme"],
            "bearer"
        );
        assert!(document["components"]["schemas"]["ApiErrorResponse"].is_object());
        for path in [
            "/api/auth/login",
            "/api/equipment",
            "/api/bookings",
            "/api/notifications",
            "/api/push/register-token",
            "/health",
            "/ready",
        ] {
            assert!(document["paths"][path].is_object(), "missing path {path}");
        }
    }
}
