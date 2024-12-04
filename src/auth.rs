use actix_web::{post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, Header, EncodingKey, decode, DecodingKey, Validation};
use jsonwebtoken::errors::Result as JwtResult;
use chrono::{Utc, Duration};
use std::env;

// around 10 seconds (for testing purposes)
pub const TOKEN_EXPIRATION: usize = 10;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthData {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub username: String,
    pub exp: usize,
}

impl Claims {
    fn new(username: &str, expiration: usize) -> Self {
        let expiration_time = Utc::now()
            .checked_add_signed(Duration::seconds(expiration as i64))
            .expect("valid timestamp")
            .timestamp() as usize;
        Claims {
            username: username.to_owned(),
            exp: expiration_time,
        }
    }
}

#[derive(Debug, Serialize)]
struct TokenResponse {
    token: String,
}

fn create_jwt(username: &str) -> JwtResult<String> {
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "mysecret".to_string());
    let exp = TOKEN_EXPIRATION;
    let claims = Claims::new(username, exp);
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
}

pub fn verify_jwt(token: &str) -> JwtResult<Claims> {
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "mysecret".to_string());
    let token_data = decode::<Claims>(token, &DecodingKey::from_secret(secret.as_ref()), &Validation::default())?;
    Ok(token_data.claims)
}

#[post("/lime/authenticate")]
async fn authenticate(auth_data: web::Json<AuthData>) -> impl Responder {
    // Currently only these users are supported (no registration possible rn)
    let users = vec!["alice", "bob", "carol", "dave"];

    if users.contains(&auth_data.username.as_str()) && auth_data.username == auth_data.password {
        match create_jwt(&auth_data.username) {
            Ok(token) => {
                let response = TokenResponse { token };
                HttpResponse::Ok().json(response)
            }
            Err(_) => HttpResponse::InternalServerError().body("Failed to create token"),
        }
    } else {
        HttpResponse::Unauthorized().body("Invalid username or password")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_jwt() {
        let token = create_jwt("alice").unwrap();
        let claims = verify_jwt(&token).unwrap();
        assert_eq!(claims.username, "alice");
    }
}
