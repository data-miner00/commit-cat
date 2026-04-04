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
