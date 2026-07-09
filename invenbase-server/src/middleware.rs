use actix_web::{web, Error as ActixError, FromRequest, HttpRequest};
use futures::future::{ready, Ready};

use crate::app_state::AppState;
use crate::auth::{AuthService, Claims};
use crate::errors::AppError;

// Middleware для проверки JWT токена
pub struct Authenticated(Claims);

impl FromRequest for Authenticated {
    type Error = ActixError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let app_state = req
            .app_data::<web::Data<AppState>>()
            .ok_or_else(|| {
                ActixError::from(AppError::InternalError("App state not found".to_string()))
            })
            .map_err(ActixError::from);

        if let Err(e) = app_state {
            return ready(Err(e));
        }

        let app_state = app_state.unwrap();
        let auth_service = AuthService::new(
            &app_state.config.jwt_secret,
            app_state.config.jwt_expiration,
        );

        let token = match AuthService::extract_token_from_request(req) {
            Some(t) => t,
            None => {
                return ready(Err(ActixError::from(AppError::Unauthorized(
                    "Missing authorization token".to_string(),
                ))));
            }
        };

        match auth_service.validate_token(&token) {
            Ok(claims) => ready(Ok(Authenticated(claims))),
            Err(e) => ready(Err(ActixError::from(e))),
        }
    }
}

// Middleware для проверки роли администратора
pub struct AdminOnly(Claims);

impl FromRequest for AdminOnly {
    type Error = ActixError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let auth_result = Authenticated::from_request(req, &mut actix_web::dev::Payload::None);

        match auth_result.into_inner() {
            Ok(Authenticated(claims)) => {
                if claims.role == "admin" {
                    ready(Ok(AdminOnly(claims)))
                } else {
                    ready(Err(ActixError::from(AppError::Unauthorized(
                        "Admin access required".to_string(),
                    ))))
                }
            }
            Err(e) => ready(Err(e)),
        }
    }
}

// Middleware для проверки роли администратора или ответственного
pub struct AdminOrResponsible(Claims);

impl FromRequest for AdminOrResponsible {
    type Error = ActixError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let auth_result = Authenticated::from_request(req, &mut actix_web::dev::Payload::None);

        match auth_result.into_inner() {
            Ok(Authenticated(claims)) => {
                if claims.role == "admin" || claims.role == "responsible" {
                    ready(Ok(AdminOrResponsible(claims)))
                } else {
                    ready(Err(ActixError::from(AppError::Unauthorized(
                        "Admin or responsible access required".to_string(),
                    ))))
                }
            }
            Err(e) => ready(Err(e)),
        }
    }
}

// Вспомогательные функции для извлечения Claims
impl Authenticated {
    pub fn claims(&self) -> Claims {
        self.0.clone()
    }
}

impl AdminOnly {
    pub fn claims(&self) -> Claims {
        self.0.clone()
    }
}

impl AdminOrResponsible {
    pub fn claims(&self) -> Claims {
        self.0.clone()
    }
}
