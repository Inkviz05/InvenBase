use crate::errors::AppError;
use crate::models::User;
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

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::TestRequest;
    use chrono::Utc;

    fn test_user(role: &str) -> User {
        User {
            id: Uuid::new_v4(),
            username: format!("test_{role}"),
            password_hash: "hash-is-not-used".to_string(),
            email: Some(format!("{role}@example.test")),
            full_name: Some(format!("Test {role}")),
            role: role.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn generated_token_validates_to_original_claims() {
        let user = test_user("responsible");
        let auth = AuthService::new("test-secret-minimum-32-characters", 3600);

        let token = auth
            .generate_token(&user)
            .expect("token should be generated");
        let claims = auth.validate_token(&token).expect("token should validate");

        assert_eq!(claims.sub, user.id.to_string());
        assert_eq!(claims.username, user.username);
        assert_eq!(claims.role, "responsible");
    }

    #[test]
    fn token_signed_with_different_secret_is_rejected() {
        let user = test_user("user");
        let auth = AuthService::new("test-secret-minimum-32-characters", 3600);
        let other_auth = AuthService::new("other-secret-minimum-32-characters", 3600);

        let token = auth
            .generate_token(&user)
            .expect("token should be generated");
        let result = other_auth.validate_token(&token);

        assert!(matches!(result, Err(AppError::Unauthorized(_))));
    }

    #[test]
    fn bearer_token_is_extracted_from_authorization_header() {
        let request = TestRequest::default()
            .insert_header(("Authorization", "Bearer abc.def.ghi"))
            .to_http_request();

        assert_eq!(
            AuthService::extract_token_from_request(&request),
            Some("abc.def.ghi".to_string())
        );
    }

    #[test]
    fn non_bearer_authorization_header_is_ignored() {
        let request = TestRequest::default()
            .insert_header(("Authorization", "Basic abc"))
            .to_http_request();

        assert_eq!(AuthService::extract_token_from_request(&request), None);
    }

    #[test]
    fn admin_satisfies_role_requirements() {
        let claims = Claims::new(
            Uuid::new_v4(),
            "admin".to_string(),
            "admin".to_string(),
            3600,
        );

        assert!(AuthService::require_role(&claims, "responsible").is_ok());
        assert!(AuthService::require_any_role(&claims, &["responsible", "user"]).is_ok());
    }

    #[test]
    fn missing_role_is_rejected() {
        let claims = Claims::new(Uuid::new_v4(), "user".to_string(), "user".to_string(), 3600);

        assert!(matches!(
            AuthService::require_role(&claims, "responsible"),
            Err(AppError::Unauthorized(_))
        ));
        assert!(matches!(
            AuthService::require_any_role(&claims, &["admin", "responsible"]),
            Err(AppError::Unauthorized(_))
        ));
    }

    #[test]
    fn password_hash_verifies_only_original_password() {
        let hash =
            AuthService::hash_password("correct-password").expect("password should be hashed");

        assert!(
            AuthService::verify_password("correct-password", &hash).expect("hash should verify")
        );
        assert!(!AuthService::verify_password("wrong-password", &hash).expect("hash should verify"));
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
