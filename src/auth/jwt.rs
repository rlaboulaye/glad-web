use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String, // Subject (username)
    pub exp: usize,  // Expiration time
}

pub fn encode_token(claims: TokenClaims) -> jsonwebtoken::errors::Result<String> {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn decode_token(token: &str) -> jsonwebtoken::errors::Result<jsonwebtoken::TokenData<TokenClaims>> {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    decode::<TokenClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
}