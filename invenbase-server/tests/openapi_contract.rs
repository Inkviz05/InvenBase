use actix_web::{http::StatusCode, test, web, App};

use kvantoriym_server::routes;

#[actix_web::test]
async fn openapi_document_and_swagger_ui_are_public() {
    let app =
        test::init_service(App::new().service(web::scope("").configure(routes::configure_web)))
            .await;

    let openapi_request = test::TestRequest::get().uri("/openapi.json").to_request();
    let openapi_response = test::call_service(&app, openapi_request).await;
    assert_eq!(openapi_response.status(), StatusCode::OK);
    let document: serde_json::Value = test::read_body_json(openapi_response).await;
    assert_eq!(document["openapi"], "3.1.0");
    assert!(document["paths"]["/api/auth/login"]["post"].is_object());
    assert_eq!(
        document["components"]["schemas"]["ApiErrorResponse"]["required"],
        serde_json::json!(["code", "message", "details", "request_id"])
    );
    for schema in [
        "UserResponse",
        "CreateUserRequest",
        "EquipmentWithDetails",
        "CreateEquipmentRequest",
        "BookingWithDetails",
        "CreateBookingRequest",
        "Notification",
        "CreateNotificationRequest",
        "SupportRequestWithMessages",
        "EquipmentReport",
        "BookingReport",
        "AuditReportEntry",
    ] {
        assert!(
            document["components"]["schemas"][schema].is_object(),
            "missing domain schema {schema}"
        );
    }
    assert_eq!(
        document["paths"]["/api/equipment"]["post"]["requestBody"]["content"]["application/json"]
            ["schema"]["$ref"],
        "#/components/schemas/CreateEquipmentRequest"
    );
    assert_eq!(
        document["paths"]["/api/bookings"]["get"]["responses"]["200"]["content"]
            ["application/json"]["schema"]["items"]["$ref"],
        "#/components/schemas/BookingWithDetails"
    );
    assert_eq!(
        document["paths"]["/api/support/requests/{id}/messages"]["post"]["responses"]["201"]
            ["content"]["application/json"]["schema"]["$ref"],
        "#/components/schemas/SupportRequestMessage"
    );
    assert_eq!(
        document["paths"]["/api/users/{id}"]["get"]["parameters"][0]["schema"]["format"],
        "uuid"
    );

    let docs_request = test::TestRequest::get().uri("/docs").to_request();
    let docs_response = test::call_service(&app, docs_request).await;
    assert_eq!(docs_response.status(), StatusCode::OK);
    let docs = test::read_body(docs_response).await;
    assert!(String::from_utf8_lossy(&docs).contains("SwaggerUIBundle"));
}
