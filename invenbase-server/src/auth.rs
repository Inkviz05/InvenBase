use crate::models::User;
use crate::errors::AppError;
use actix_web::HttpRequest;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user id
    pub username: String,
    pub role: String,
    pub exp: usize,
}

impl Claims {
    pub fn new(user_id: Uuid, username: String, role: String, expiration: u64) -> Self {
        let exp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize
            + expiration as usize;

        Claims {
            sub: user_id.to_string(),
            username,
            role,
            exp,
        }
    }
}

pub struct AuthService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    expiration: u64,
}

impl AuthService {
    pub fn new(secret: &str, expiration: u64) -> Self {
        AuthService {
            encoding_key: EncodingKey::from_secret(secret.as_ref()),
            decoding_key: DecodingKey::from_secret(secret.as_ref()),
            expiration,
        }
    }

    pub fn generate_token(&self, user: &User) -> Result<String, AppError> {
        let claims = Claims::new(
            user.id,
            user.username.clone(),
            user.role.clone(),
            self.expiration,
        );

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AppError::InternalError(format!("Token generation failed: {}", e)))
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, AppError> {
        let token_data = decode::<Claims>(token, &self.decoding_key, &Validation::default())
            .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;

        Ok(token_data.claims)
    }

    pub fn extract_token_from_request(req: &HttpRequest) -> Option<String> {
        req.headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| {
                if s.starts_with("Bearer ") {
                    Some(s[7..].to_string())
                } else {
                    None
                }
            })
    }

    pub fn hash_password(password: &str) -> Result<String, AppError> {
        bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .map_err(|e| AppError::InternalError(format!("Password hashing failed: {}", e)))
    }

    pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
        bcrypt::verify(password, hash)
            .map_err(|e| AppError::InternalError(format!("Password verification failed: {}", e)))
    }

    pub fn require_role(claims: &Claims, required_role: &str) -> Result<(), AppError> {
        if claims.role == required_role || claims.role == "admin" {
            Ok(())
        } else {
            Err(AppError::Unauthorized(format!(
                "Required role: {}",
                required_role
            )))
        }
    }

    pub fn require_any_role(claims: &Claims, roles: &[&str]) -> Result<(), AppError> {
        if roles.contains(&claims.role.as_str()) || claims.role == "admin" {
            Ok(())
        } else {
            Err(AppError::Unauthorized(format!(
                "Required one of roles: {:?}",
                roles
            )))
        }
    }
}

