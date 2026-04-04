use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub exp: usize,
}

pub fn create_token(user_id: &str, username: &str, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = Utc::now() + Duration::days(30);
    let claims = Claims {
        sub: user_id.to_string(),
        username: username.to_string(),
        exp: expiration.timestamp() as usize,
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
}

#[allow(dead_code)]
pub fn verify_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let data = decode::<Claims>(token, &DecodingKey::from_secret(secret.as_ref()), &Validation::default())?;
    Ok(data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_verify_token() {
        let secret = "test-secret";
        let token = create_token("user-123", "testuser", secret).unwrap();
        let claims = verify_token(&token, secret).unwrap();
        assert_eq!(claims.sub, "user-123");
        assert_eq!(claims.username, "testuser");
    }

    #[test]
    fn invalid_secret_fails() {
        let token = create_token("user-1", "u", "secret-a").unwrap();
        assert!(verify_token(&token, "secret-b").is_err());
    }

    #[test]
    fn expired_token_fails() {
        let secret = "test-secret";
        let exp = (chrono::Utc::now() - chrono::Duration::hours(1)).timestamp() as usize;
        let claims = Claims {
            sub: "user-1".to_string(),
            username: "u".to_string(),
            exp,
        };
        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(secret.as_ref()),
        ).unwrap();
        assert!(verify_token(&token, secret).is_err());
    }
}
