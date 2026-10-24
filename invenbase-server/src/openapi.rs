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
            "429": {
                "description": "Rate limit exceeded",
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

fn schema_ref(name: &str) -> Value {
    json!({ "$ref": format!("#/components/schemas/{name}") })
}

fn json_content(schema: Value) -> Value {
    json!({ "application/json": { "schema": schema } })
}

fn set_request(paths: &mut Map<String, Value>, path: &str, method: &str, schema: &str) {
    paths[path][method]["requestBody"] = json!({
        "required": true,
        "content": json_content(schema_ref(schema))
    });
}

fn set_response(
    paths: &mut Map<String, Value>,
    path: &str,
    method: &str,
    status: &str,
    description: &str,
    schema: Value,
) {
    if status == "201" || status == "204" {
        paths[path][method]["responses"]
            .as_object_mut()
            .expect("responses must be an object")
            .remove("200");
    }
    paths[path][method]["responses"][status] = json!({
        "description": description,
        "content": json_content(schema)
    });
}

fn array_of(name: &str) -> Value {
    json!({ "type": "array", "items": schema_ref(name) })
}

fn add_query_parameter(
    paths: &mut Map<String, Value>,
    path: &str,
    method: &str,
    name: &str,
    schema: Value,
) {
    let parameters = paths[path][method]
        .as_object_mut()
        .expect("operation must be an object")
        .entry("parameters")
        .or_insert_with(|| Value::Array(Vec::new()));
    parameters
        .as_array_mut()
        .expect("parameters must be an array")
        .push(json!({ "name": name, "in": "query", "required": false, "schema": schema }));
}

fn add_path_parameters(paths: &mut Map<String, Value>) {
    for (path, item) in paths {
        for parameter in ["id", "qr_code"] {
            if !path.contains(&format!("{{{parameter}}}")) {
                continue;
            }
            let schema = if parameter == "id" {
                json!({ "type": "string", "format": "uuid" })
            } else {
                json!({ "type": "string" })
            };
            for operation in item
                .as_object_mut()
                .into_iter()
                .flat_map(|item| item.values_mut())
            {
                operation["parameters"] = json!([{
                    "name": parameter,
                    "in": "path",
                    "required": true,
                    "schema": schema
                }]);
            }
        }
    }
}

fn describe_domain_contracts(paths: &mut Map<String, Value>) {
    set_response(
        paths,
        "/api/instance",
        "get",
        "200",
        "Installation metadata",
        schema_ref("InstanceResponse"),
    );
    set_request(paths, "/api/instance/pair", "post", "PairInstanceRequest");
    set_response(
        paths,
        "/api/instance/pair",
        "post",
        "200",
        "Pairing confirmed",
        schema_ref("PairInstanceResponse"),
    );
    set_request(
        paths,
        "/api/instance/setup-codes",
        "post",
        "CreateSetupCodeRequest",
    );
    set_response(
        paths,
        "/api/instance/setup-codes",
        "post",
        "201",
        "One-time setup code created",
        schema_ref("CreateSetupCodeResponse"),
    );
    paths["/api/instance/setup-codes/{id}"]["delete"]["responses"]
        .as_object_mut()
        .expect("responses must be an object")
        .remove("200");
    paths["/api/instance/setup-codes/{id}"]["delete"]["responses"]["204"] =
        json!({"description":"Setup code revoked"});

    set_response(
        paths,
        "/api/auth/login",
        "post",
        "200",
        "JWT and authenticated user",
        schema_ref("LoginResponse"),
    );

    set_response(
        paths,
        "/api/users/me",
        "get",
        "200",
        "Current user",
        schema_ref("UserResponse"),
    );
    set_response(
        paths,
        "/api/users",
        "get",
        "200",
        "Users",
        array_of("UserResponse"),
    );
    set_request(paths, "/api/users", "post", "CreateUserRequest");
    set_response(
        paths,
        "/api/users",
        "post",
        "201",
        "User created",
        schema_ref("UserResponse"),
    );
    set_response(
        paths,
        "/api/users/{id}",
        "get",
        "200",
        "User",
        schema_ref("UserResponse"),
    );
    set_request(paths, "/api/users/{id}", "put", "UpdateUserRequest");
    set_response(
        paths,
        "/api/users/{id}",
        "put",
        "200",
        "User updated",
        schema_ref("UserResponse"),
    );

    set_response(
        paths,
        "/api/equipment",
        "get",
        "200",
        "Equipment",
        array_of("EquipmentWithDetails"),
    );
    add_query_parameter(
        paths,
        "/api/equipment",
        "get",
        "squad_id",
        json!({"type":"string","format":"uuid"}),
    );
    set_request(paths, "/api/equipment", "post", "CreateEquipmentRequest");
    set_response(
        paths,
        "/api/equipment",
        "post",
        "201",
        "Equipment created",
        schema_ref("Equipment"),
    );
    set_response(
        paths,
        "/api/equipment/{id}",
        "get",
        "200",
        "Equipment",
        schema_ref("EquipmentWithDetails"),
    );
    set_request(
        paths,
        "/api/equipment/{id}",
        "put",
        "UpdateEquipmentRequest",
    );
    set_response(
        paths,
        "/api/equipment/{id}",
        "put",
        "200",
        "Equipment updated",
        schema_ref("Equipment"),
    );
    set_response(
        paths,
        "/api/equipment/qr/{qr_code}",
        "get",
        "200",
        "Equipment",
        schema_ref("EquipmentWithDetails"),
    );
    set_response(
        paths,
        "/api/equipment/{id}/movements",
        "get",
        "200",
        "Movements",
        array_of("EquipmentMovement"),
    );
    set_request(
        paths,
        "/api/equipment/{id}/move",
        "post",
        "CreateEquipmentMovementRequest",
    );
    set_response(
        paths,
        "/api/equipment/{id}/move",
        "post",
        "201",
        "Movement created",
        schema_ref("EquipmentMovement"),
    );

    set_response(
        paths,
        "/api/bookings",
        "get",
        "200",
        "Bookings",
        array_of("BookingWithDetails"),
    );
    set_request(paths, "/api/bookings", "post", "CreateBookingRequest");
    set_response(
        paths,
        "/api/bookings",
        "post",
        "201",
        "Booking created",
        schema_ref("Booking"),
    );
    set_request(
        paths,
        "/api/bookings/bulk",
        "post",
        "CreateBulkBookingsRequest",
    );
    set_response(
        paths,
        "/api/bookings/bulk",
        "post",
        "201",
        "Bookings created",
        array_of("Booking"),
    );
    set_response(
        paths,
        "/api/bookings/{id}",
        "get",
        "200",
        "Booking",
        schema_ref("BookingWithDetails"),
    );
    set_request(paths, "/api/bookings/{id}", "put", "UpdateBookingRequest");
    for action in ["approve", "reject", "cancel", "return"] {
        set_response(
            paths,
            &format!("/api/bookings/{{id}}/{action}"),
            "post",
            "200",
            "Booking updated",
            schema_ref("Booking"),
        );
    }

    set_response(
        paths,
        "/api/notifications",
        "get",
        "200",
        "Notifications",
        array_of("Notification"),
    );
    set_request(
        paths,
        "/api/notifications",
        "post",
        "CreateNotificationRequest",
    );
    set_response(
        paths,
        "/api/notifications",
        "post",
        "201",
        "Notification created",
        schema_ref("Notification"),
    );
    for path in [
        "/api/notifications/count",
        "/api/notifications/unread-count",
    ] {
        set_response(
            paths,
            path,
            "get",
            "200",
            "Unread count",
            schema_ref("UnreadCountResponse"),
        );
    }

    set_response(
        paths,
        "/api/support/requests",
        "get",
        "200",
        "Support requests",
        array_of("SupportRequestWithMessages"),
    );
    set_request(
        paths,
        "/api/support/requests",
        "post",
        "CreateSupportRequestRequest",
    );
    set_response(
        paths,
        "/api/support/requests",
        "post",
        "201",
        "Support request created",
        schema_ref("SupportRequest"),
    );
    set_request(
        paths,
        "/api/support/requests/{id}",
        "put",
        "UpdateSupportRequestRequest",
    );
    set_response(
        paths,
        "/api/support/requests/{id}",
        "put",
        "200",
        "Support request updated",
        schema_ref("SupportRequestWithUser"),
    );
    set_request(
        paths,
        "/api/support/requests/{id}/messages",
        "post",
        "AddSupportMessageRequest",
    );
    set_response(
        paths,
        "/api/support/requests/{id}/messages",
        "post",
        "201",
        "Message created",
        schema_ref("SupportRequestMessage"),
    );

    set_response(
        paths,
        "/api/reports/equipment",
        "get",
        "200",
        "Equipment report",
        schema_ref("EquipmentReport"),
    );
    set_response(
        paths,
        "/api/reports/bookings",
        "get",
        "200",
        "Booking totals",
        schema_ref("BookingReport"),
    );
    set_response(
        paths,
        "/api/reports/bookings/detailed",
        "get",
        "200",
        "Detailed bookings",
        array_of("BookingWithDetails"),
    );
    for path in ["/api/reports/bookings/detailed", "/api/reports/audit"] {
        add_query_parameter(
            paths,
            path,
            "get",
            "from",
            json!({"type":"string","format":"date"}),
        );
        add_query_parameter(
            paths,
            path,
            "get",
            "to",
            json!({"type":"string","format":"date"}),
        );
    }
    for (name, schema) in [
        ("action", json!({"type":"string"})),
        ("entity_type", json!({"type":"string"})),
        ("user_id", json!({"type":"string","format":"uuid"})),
        (
            "limit",
            json!({"type":"integer","minimum":0,"maximum":5000,"default":500}),
        ),
        ("offset", json!({"type":"integer","minimum":0,"default":0})),
    ] {
        add_query_parameter(paths, "/api/reports/audit", "get", name, schema);
    }
    set_response(
        paths,
        "/api/reports/audit",
        "get",
        "200",
        "Audit entries",
        array_of("AuditReportEntry"),
    );

    set_response(
        paths,
        "/api/categories",
        "get",
        "200",
        "Categories",
        array_of("EquipmentCategoryWithSquad"),
    );
    add_query_parameter(
        paths,
        "/api/categories",
        "get",
        "squad_id",
        json!({"type":"string","format":"uuid"}),
    );
    set_request(paths, "/api/categories", "post", "CreateCategoryRequest");
    set_response(
        paths,
        "/api/categories",
        "post",
        "201",
        "Category created",
        schema_ref("EquipmentCategory"),
    );
    set_response(
        paths,
        "/api/categories/{id}",
        "get",
        "200",
        "Category",
        schema_ref("EquipmentCategory"),
    );
    set_request(
        paths,
        "/api/categories/{id}",
        "put",
        "CreateCategoryRequest",
    );
    set_response(
        paths,
        "/api/categories/{id}",
        "put",
        "200",
        "Category updated",
        schema_ref("EquipmentCategory"),
    );

    set_response(
        paths,
        "/api/groups",
        "get",
        "200",
        "Equipment groups",
        array_of("EquipmentGroup"),
    );
    set_request(paths, "/api/groups", "post", "CreateEquipmentGroupRequest");
    set_response(
        paths,
        "/api/groups",
        "post",
        "201",
        "Equipment group created",
        schema_ref("EquipmentGroup"),
    );
    set_response(
        paths,
        "/api/groups/{id}",
        "get",
        "200",
        "Equipment group",
        schema_ref("EquipmentGroup"),
    );
    set_request(
        paths,
        "/api/groups/{id}",
        "put",
        "CreateEquipmentGroupRequest",
    );
    set_response(
        paths,
        "/api/groups/{id}",
        "put",
        "200",
        "Equipment group updated",
        schema_ref("EquipmentGroup"),
    );

    set_response(
        paths,
        "/api/squads",
        "get",
        "200",
        "Squads",
        array_of("SquadWithDetails"),
    );
    set_request(paths, "/api/squads", "post", "CreateSquadRequest");
    set_response(
        paths,
        "/api/squads",
        "post",
        "201",
        "Squad created",
        schema_ref("Squad"),
    );
    set_response(
        paths,
        "/api/squads/{id}",
        "get",
        "200",
        "Squad",
        schema_ref("SquadWithDetails"),
    );
    set_request(paths, "/api/squads/{id}", "put", "UpdateSquadRequest");
    set_response(
        paths,
        "/api/squads/{id}",
        "put",
        "200",
        "Squad updated",
        schema_ref("SquadWithDetails"),
    );
    set_response(
        paths,
        "/api/squads/{id}/equipment",
        "get",
        "200",
        "Squad equipment",
        array_of("EquipmentWithDetails"),
    );

    set_response(
        paths,
        "/api/permissions",
        "get",
        "200",
        "Permissions",
        array_of("Permission"),
    );
    set_request(paths, "/api/permissions", "post", "CreatePermissionRequest");
    set_response(
        paths,
        "/api/permissions",
        "post",
        "201",
        "Permission created",
        schema_ref("Permission"),
    );

    set_response(
        paths,
        "/api/logs",
        "get",
        "200",
        "Activity logs",
        array_of("ActivityLog"),
    );
    add_query_parameter(
        paths,
        "/api/logs",
        "get",
        "user_id",
        json!({"type":"string","format":"uuid"}),
    );
    add_query_parameter(
        paths,
        "/api/logs",
        "get",
        "entity_type",
        json!({"type":"string"}),
    );
    add_query_parameter(
        paths,
        "/api/logs",
        "get",
        "limit",
        json!({"type":"integer","minimum":0,"maximum":1000,"default":100}),
    );
    add_query_parameter(
        paths,
        "/api/logs",
        "get",
        "offset",
        json!({"type":"integer","minimum":0,"default":0}),
    );

    paths["/api/qr/{id}"]["get"]["responses"]["200"] = json!({
        "description":"QR code PNG",
        "content":{"image/png":{"schema":{"type":"string","contentEncoding":"binary"}}}
    });
    set_response(
        paths,
        "/api/qr/{id}/data",
        "get",
        "200",
        "QR code data",
        schema_ref("QrCodeData"),
    );

    for (path, method) in [
        ("/api/users/{id}", "delete"),
        ("/api/equipment/{id}", "delete"),
        ("/api/categories/{id}", "delete"),
        ("/api/groups/{id}", "delete"),
        ("/api/squads/{id}", "delete"),
        ("/api/bookings/{id}", "delete"),
        ("/api/permissions/{id}/revoke", "post"),
        ("/api/support/requests/{id}", "delete"),
        ("/api/notifications/{id}/read", "post"),
        ("/api/notifications/read-all", "post"),
        ("/api/push/register-token", "post"),
        ("/api/push/unregister-device", "post"),
    ] {
        paths[path][method]["responses"]
            .as_object_mut()
            .expect("responses must be an object")
            .remove("200");
        paths[path][method]["responses"]["204"] = json!({"description":"No content"});
    }

    set_request(
        paths,
        "/api/push/register-token",
        "post",
        "RegisterPushTokenRequest",
    );
    set_request(
        paths,
        "/api/push/unregister-device",
        "post",
        "UnregisterPushDeviceRequest",
    );
    add_path_parameters(paths);
}

fn domain_schemas() -> Map<String, Value> {
    let mut schemas = Map::new();
    let mut add = |name: &str, required: &[&str], properties: Value| {
        schemas.insert(
            name.to_string(),
            json!({ "type": "object", "required": required, "properties": properties }),
        );
    };
    let uuid = || json!({ "type": "string", "format": "uuid" });
    let nullable_uuid = || json!({ "type": ["string", "null"], "format": "uuid" });
    let nullable_string = || json!({ "type": ["string", "null"] });
    let date_time = || json!({ "type": "string", "format": "date-time" });

    add(
        "InstanceResponse",
        &[
            "product",
            "instance_id",
            "instance_name",
            "api_protocol_version",
            "api_base_path",
        ],
        json!({
            "product":{"type":"string","const":"InvenBase"},
            "instance_id":uuid(),
            "instance_name":{"type":"string"},
            "api_protocol_version":{"type":"integer","minimum":1},
            "api_base_path":{"type":"string","const":"/api"}
        }),
    );
    add(
        "CreateSetupCodeRequest",
        &[],
        json!({"expires_in_minutes":{"type":["integer","null"],"minimum":1,"maximum":60,"default":10}}),
    );
    add(
        "CreateSetupCodeResponse",
        &["id", "setup_code", "expires_at"],
        json!({"id":uuid(),"setup_code":{"type":"string","pattern":"^[0-9A-F]{4}(-[0-9A-F]{4}){5}$"},"expires_at":date_time()}),
    );
    add(
        "PairInstanceRequest",
        &["setup_code"],
        json!({"setup_code":{"type":"string"}}),
    );
    add(
        "PairInstanceResponse",
        &["instance", "paired_at"],
        json!({"instance":schema_ref("InstanceResponse"),"paired_at":date_time()}),
    );

    add(
        "UserResponse",
        &["id", "username", "email", "full_name", "role"],
        json!({
            "id": uuid(), "username": {"type":"string"}, "email": nullable_string(),
            "full_name": nullable_string(), "role": {"type":"string", "enum":["admin","responsible","user"]}
        }),
    );
    add(
        "CreateUserRequest",
        &["username", "password"],
        json!({
            "username":{"type":"string"}, "password":{"type":"string","format":"password","minLength":6},
            "email":nullable_string(), "full_name":nullable_string(), "role":{"type":["string","null"],"enum":["admin","responsible","user",null]}
        }),
    );
    add(
        "UpdateUserRequest",
        &[],
        json!({
            "username":nullable_string(), "password":{"type":["string","null"],"format":"password"},
            "email":nullable_string(), "full_name":nullable_string(), "role":{"type":["string","null"],"enum":["admin","responsible","user",null]}
        }),
    );

    let equipment_properties = json!({
        "id":uuid(), "name":{"type":"string"}, "description":nullable_string(), "category_id":nullable_uuid(),
        "squad_id":nullable_uuid(), "quantity":{"type":"integer"}, "available_quantity":{"type":"integer"},
        "is_unique":{"type":"boolean"}, "location":nullable_string(), "qr_code":nullable_string(),
        "responsible_user_id":nullable_uuid(), "status":{"type":"string"}, "created_at":date_time(), "updated_at":date_time()
    });
    add(
        "Equipment",
        &[
            "id",
            "name",
            "description",
            "category_id",
            "squad_id",
            "quantity",
            "available_quantity",
            "is_unique",
            "location",
            "qr_code",
            "responsible_user_id",
            "status",
            "created_at",
            "updated_at",
        ],
        equipment_properties.clone(),
    );
    let mut detailed_equipment = equipment_properties.as_object().unwrap().clone();
    detailed_equipment.insert("category_name".into(), nullable_string());
    detailed_equipment.insert("squad_name".into(), nullable_string());
    detailed_equipment.insert("responsible_name".into(), nullable_string());
    add(
        "EquipmentWithDetails",
        &[
            "id",
            "name",
            "description",
            "category_id",
            "category_name",
            "squad_id",
            "squad_name",
            "quantity",
            "available_quantity",
            "is_unique",
            "location",
            "qr_code",
            "responsible_user_id",
            "responsible_name",
            "status",
            "created_at",
            "updated_at",
        ],
        Value::Object(detailed_equipment),
    );
    add(
        "CreateEquipmentRequest",
        &["name"],
        json!({
            "name":{"type":"string"}, "description":nullable_string(), "category_id":nullable_uuid(), "squad_id":nullable_uuid(),
            "quantity":{"type":["integer","null"],"minimum":1}, "location":nullable_string(), "responsible_user_id":nullable_uuid(), "is_unique":{"type":"boolean","default":false}
        }),
    );
    add(
        "UpdateEquipmentRequest",
        &[],
        json!({
            "name":nullable_string(), "description":nullable_string(), "category_id":nullable_uuid(), "squad_id":nullable_uuid(),
            "quantity":{"type":["integer","null"]}, "available_quantity":{"type":["integer","null"]}, "location":nullable_string(),
            "responsible_user_id":nullable_uuid(), "status":nullable_string(), "is_unique":{"type":["boolean","null"]}
        }),
    );
    add(
        "CreateEquipmentMovementRequest",
        &[],
        json!({"to_squad_id":nullable_uuid(),"to_location":nullable_string(),"comment":nullable_string()}),
    );
    add(
        "EquipmentMovement",
        &[
            "id",
            "equipment_id",
            "from_squad_id",
            "to_squad_id",
            "from_location",
            "to_location",
            "moved_by",
            "moved_by_name",
            "comment",
            "moved_at",
        ],
        json!({
            "id":uuid(),"equipment_id":uuid(),"from_squad_id":nullable_uuid(),"to_squad_id":nullable_uuid(),"from_location":nullable_string(),"to_location":nullable_string(),"moved_by":nullable_uuid(),"moved_by_name":nullable_string(),"comment":nullable_string(),"moved_at":date_time()
        }),
    );

    let booking_properties = json!({
        "id":uuid(),"user_id":uuid(),"equipment_id":nullable_uuid(),"group_id":nullable_uuid(),"quantity":{"type":"integer"},
        "start_date":date_time(),"end_date":date_time(),"purpose":nullable_string(),"status":{"type":"string"},
        "permission_type":{"type":"string"},"created_at":date_time(),"updated_at":date_time()
    });
    add(
        "Booking",
        &[
            "id",
            "user_id",
            "equipment_id",
            "group_id",
            "quantity",
            "start_date",
            "end_date",
            "purpose",
            "status",
            "permission_type",
            "created_at",
            "updated_at",
        ],
        booking_properties.clone(),
    );
    let mut detailed_booking = booking_properties.as_object().unwrap().clone();
    for field in ["username", "full_name", "equipment_name", "group_name"] {
        detailed_booking.insert(field.into(), nullable_string());
    }
    add(
        "BookingWithDetails",
        &[
            "id",
            "user_id",
            "username",
            "full_name",
            "equipment_id",
            "equipment_name",
            "group_id",
            "group_name",
            "quantity",
            "start_date",
            "end_date",
            "purpose",
            "status",
            "permission_type",
            "created_at",
            "updated_at",
        ],
        Value::Object(detailed_booking),
    );
    add(
        "CreateBookingRequest",
        &["quantity", "start_date", "end_date"],
        json!({
            "equipment_id":nullable_uuid(),"group_id":nullable_uuid(),"quantity":{"type":"integer","minimum":1},"start_date":date_time(),"end_date":date_time(),"purpose":nullable_string(),"permission_type":nullable_string()
        }),
    );
    add(
        "CreateBulkBookingsRequest",
        &["bookings"],
        json!({"bookings":{"type":"array","minItems":1,"items":schema_ref("CreateBookingRequest")}}),
    );
    add(
        "UpdateBookingRequest",
        &[],
        json!({"status":nullable_string(),"start_date":{"type":["string","null"],"format":"date-time"},"end_date":{"type":["string","null"],"format":"date-time"}}),
    );

    add(
        "Notification",
        &[
            "id",
            "user_id",
            "title",
            "message",
            "notification_type",
            "is_read",
            "created_at",
        ],
        json!({
            "id":uuid(),"user_id":uuid(),"title":{"type":"string"},"message":{"type":"string"},"notification_type":{"type":"string"},"is_read":{"type":"boolean"},"created_at":date_time()
        }),
    );
    add(
        "CreateNotificationRequest",
        &["user_id", "title", "message", "notification_type"],
        json!({"user_id":uuid(),"title":{"type":"string"},"message":{"type":"string"},"notification_type":{"type":"string"}}),
    );
    add(
        "UnreadCountResponse",
        &["count"],
        json!({"count":{"type":"integer","format":"int64","minimum":0}}),
    );

    add(
        "SupportRequest",
        &[
            "id",
            "user_id",
            "subject",
            "message",
            "status",
            "created_at",
            "updated_at",
            "admin_comment",
        ],
        json!({
            "id":uuid(),"user_id":uuid(),"subject":{"type":"string"},"message":{"type":"string"},"status":{"type":"string"},"created_at":date_time(),"updated_at":date_time(),"admin_comment":nullable_string()
        }),
    );
    add(
        "SupportRequestWithMessages",
        &[
            "id",
            "user_id",
            "user_name",
            "subject",
            "message",
            "status",
            "created_at",
            "updated_at",
            "admin_comment",
            "messages",
        ],
        json!({
            "id":uuid(),"user_id":uuid(),"user_name":nullable_string(),"subject":{"type":"string"},"message":{"type":"string"},"status":{"type":"string"},"created_at":date_time(),"updated_at":date_time(),"admin_comment":nullable_string(),"messages":{"type":"array","items":schema_ref("SupportRequestMessage")}
        }),
    );
    add(
        "SupportRequestWithUser",
        &[
            "id",
            "user_id",
            "user_name",
            "subject",
            "message",
            "status",
            "created_at",
            "updated_at",
            "admin_comment",
        ],
        json!({
            "id":uuid(),"user_id":uuid(),"user_name":nullable_string(),"subject":{"type":"string"},"message":{"type":"string"},"status":{"type":"string"},"created_at":date_time(),"updated_at":date_time(),"admin_comment":nullable_string()
        }),
    );
    add(
        "CreateSupportRequestRequest",
        &["subject", "message"],
        json!({"subject":{"type":"string"},"message":{"type":"string"}}),
    );
    add(
        "UpdateSupportRequestRequest",
        &[],
        json!({"status":nullable_string(),"admin_comment":nullable_string()}),
    );
    add(
        "AddSupportMessageRequest",
        &["message"],
        json!({"message":{"type":"string"}}),
    );
    add(
        "SupportRequestMessage",
        &[
            "id",
            "support_request_id",
            "author_user_id",
            "is_staff",
            "message",
            "created_at",
        ],
        json!({"id":uuid(),"support_request_id":uuid(),"author_user_id":nullable_uuid(),"is_staff":{"type":"boolean"},"message":{"type":"string"},"created_at":date_time()}),
    );

    add(
        "CategoryStatistics",
        &[
            "category_id",
            "category_name",
            "total",
            "available",
            "booked",
        ],
        json!({"category_id":uuid(),"category_name":{"type":"string"},"total":{"type":"integer","format":"int64"},"available":{"type":"integer","format":"int64"},"booked":{"type":"integer","format":"int64"}}),
    );
    add(
        "EquipmentReport",
        &[
            "total_equipment",
            "available_equipment",
            "booked_equipment",
            "by_category",
        ],
        json!({"total_equipment":{"type":"integer","format":"int64"},"available_equipment":{"type":"integer","format":"int64"},"booked_equipment":{"type":"integer","format":"int64"},"by_category":{"type":"array","items":schema_ref("CategoryStatistics")}}),
    );
    add(
        "BookingReport",
        &[
            "pending",
            "approved",
            "expired",
            "awaiting_return",
            "returned",
            "rejected",
            "cancelled",
            "completed",
            "total",
        ],
        json!({"pending":{"type":"integer"},"approved":{"type":"integer"},"expired":{"type":"integer"},"awaiting_return":{"type":"integer"},"returned":{"type":"integer"},"rejected":{"type":"integer"},"cancelled":{"type":"integer"},"completed":{"type":"integer"},"total":{"type":"integer"}}),
    );
    add(
        "AuditReportEntry",
        &[
            "id",
            "user_id",
            "username",
            "full_name",
            "action",
            "entity_type",
            "entity_id",
            "entity_name",
            "details",
            "created_at",
        ],
        json!({"id":uuid(),"user_id":nullable_uuid(),"username":nullable_string(),"full_name":nullable_string(),"action":{"type":"string"},"entity_type":{"type":"string"},"entity_id":nullable_uuid(),"entity_name":nullable_string(),"details":{"type":["object","null"],"additionalProperties":true},"created_at":date_time()}),
    );

    add(
        "EquipmentCategory",
        &["id", "name", "description", "squad_id", "created_at"],
        json!({"id":uuid(),"name":{"type":"string"},"description":nullable_string(),"squad_id":nullable_uuid(),"created_at":date_time()}),
    );
    add(
        "EquipmentCategoryWithSquad",
        &[
            "id",
            "name",
            "description",
            "squad_id",
            "squad_name",
            "created_at",
        ],
        json!({"id":uuid(),"name":{"type":"string"},"description":nullable_string(),"squad_id":nullable_uuid(),"squad_name":nullable_string(),"created_at":date_time()}),
    );
    add(
        "CreateCategoryRequest",
        &["name"],
        json!({"name":{"type":"string"},"description":nullable_string(),"squad_id":nullable_uuid()}),
    );

    add(
        "EquipmentGroup",
        &["id", "name", "description", "created_at"],
        json!({"id":uuid(),"name":{"type":"string"},"description":nullable_string(),"created_at":date_time()}),
    );
    add(
        "EquipmentGroupItemRequest",
        &["equipment_id", "quantity"],
        json!({"equipment_id":uuid(),"quantity":{"type":"integer","minimum":1}}),
    );
    add(
        "CreateEquipmentGroupRequest",
        &["name", "equipment_items"],
        json!({"name":{"type":"string"},"description":nullable_string(),"equipment_items":{"type":"array","items":schema_ref("EquipmentGroupItemRequest")}}),
    );

    let squad_properties = json!({
        "id":uuid(),"name":{"type":"string"},"description":nullable_string(),"location":nullable_string(),
        "responsible_user_id":nullable_uuid(),"created_at":date_time(),"updated_at":date_time()
    });
    add(
        "Squad",
        &[
            "id",
            "name",
            "description",
            "location",
            "responsible_user_id",
            "created_at",
            "updated_at",
        ],
        squad_properties.clone(),
    );
    let mut squad_details = squad_properties.as_object().unwrap().clone();
    squad_details.insert("responsible_name".into(), nullable_string());
    add(
        "SquadWithDetails",
        &[
            "id",
            "name",
            "description",
            "location",
            "responsible_user_id",
            "responsible_name",
            "created_at",
            "updated_at",
        ],
        Value::Object(squad_details),
    );
    add(
        "CreateSquadRequest",
        &["name"],
        json!({"name":{"type":"string"},"description":nullable_string(),"location":nullable_string(),"responsible_user_id":nullable_uuid()}),
    );
    add(
        "UpdateSquadRequest",
        &[],
        json!({"name":nullable_string(),"description":nullable_string(),"location":nullable_string(),"responsible_user_id":nullable_uuid()}),
    );

    add(
        "Permission",
        &[
            "id",
            "booking_id",
            "equipment_id",
            "permission_type",
            "issued_by",
            "issued_at",
            "expires_at",
            "status",
        ],
        json!({"id":uuid(),"booking_id":uuid(),"equipment_id":nullable_uuid(),"permission_type":{"type":"string"},"issued_by":nullable_uuid(),"issued_at":date_time(),"expires_at":{"type":["string","null"],"format":"date-time"},"status":{"type":"string"}}),
    );
    add(
        "CreatePermissionRequest",
        &["booking_id", "permission_type"],
        json!({"booking_id":uuid(),"equipment_id":nullable_uuid(),"permission_type":{"type":"string"},"expires_at":{"type":["string","null"],"format":"date-time"}}),
    );

    add(
        "ActivityLog",
        &[
            "id",
            "user_id",
            "action",
            "entity_type",
            "entity_id",
            "details",
            "created_at",
        ],
        json!({"id":uuid(),"user_id":nullable_uuid(),"action":{"type":"string"},"entity_type":{"type":"string"},"entity_id":nullable_uuid(),"details":{"type":["object","null"],"additionalProperties":true},"created_at":date_time()}),
    );
    add(
        "QrCodeData",
        &["equipment_id", "qr_code", "name", "description"],
        json!({"equipment_id":uuid(),"qr_code":{"type":"string"},"name":nullable_string(),"description":nullable_string()}),
    );

    add(
        "RegisterPushTokenRequest",
        &["token", "provider"],
        json!({"token":{"type":"string"},"platform":nullable_string(),"device_id":nullable_string(),"provider":{"type":"string","enum":["rustore","huawei"]}}),
    );
    add(
        "UnregisterPushDeviceRequest",
        &["device_id", "provider"],
        json!({"device_id":{"type":"string"},"provider":{"type":"string","enum":["rustore","huawei"]}}),
    );
    schemas
}

pub fn document() -> Value {
    let mut paths = Map::new();
    for (path, method, tag, summary, protected) in [
        ("/health", "get", "System", "Liveness check", false),
        ("/ready", "get", "System", "Database readiness check", false),
        (
            "/api/instance",
            "get",
            "Instance",
            "Get installation metadata",
            false,
        ),
        (
            "/api/instance/pair",
            "post",
            "Instance",
            "Consume a one-time setup code",
            false,
        ),
        (
            "/api/instance/setup-codes",
            "post",
            "Instance",
            "Create a one-time setup code",
            true,
        ),
        (
            "/api/instance/setup-codes/{id}",
            "delete",
            "Instance",
            "Revoke an unused setup code",
            true,
        ),
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

    describe_domain_contracts(&mut paths);

    let mut schemas = domain_schemas();
    schemas.insert(
        "ApiErrorResponse".to_string(),
        json!({
            "type": "object",
            "required": ["code", "message", "details", "request_id"],
            "properties": {
                "code": { "type": "string", "example": "VALIDATION_ERROR" },
                "message": { "type": "string" },
                "details": { "type": ["object", "null"], "additionalProperties": true },
                "request_id": { "type": ["string", "null"], "format": "uuid" }
            }
        }),
    );
    schemas.insert(
        "LoginRequest".to_string(),
        json!({
            "type": "object",
            "required": ["username", "password"],
            "properties": {
                "username": { "type": "string" },
                "password": { "type": "string", "format": "password" }
            }
        }),
    );
    schemas.insert(
        "LoginResponse".to_string(),
        json!({
            "type": "object",
            "required": ["token", "user"],
            "properties": {
                "token": { "type": "string" },
                "user": schema_ref("UserResponse")
            }
        }),
    );

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
            "schemas": schemas
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
    use serde_json::Value;

    fn assert_schema_refs_resolve(value: &Value, schemas: &Value) {
        match value {
            Value::Object(object) => {
                if let Some(reference) = object.get("$ref").and_then(Value::as_str) {
                    let name = reference
                        .strip_prefix("#/components/schemas/")
                        .expect("only local schema references are supported");
                    assert!(
                        schemas.get(name).is_some(),
                        "unresolved schema ref {reference}"
                    );
                }
                for nested in object.values() {
                    assert_schema_refs_resolve(nested, schemas);
                }
            }
            Value::Array(values) => {
                for nested in values {
                    assert_schema_refs_resolve(nested, schemas);
                }
            }
            _ => {}
        }
    }

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
        assert_schema_refs_resolve(&document, &document["components"]["schemas"]);
    }
}
