use axum::{
    extract::Request,
    http::{header, HeaderValue, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::auth::jwt::decode_token;

pub async fn auth_middleware(request: Request, next: Next) -> Response {
    let path = request.uri().path();
    
    // Skip authentication for public routes
    if path.starts_with("/api/auth/login") 
        || path.starts_with("/api/auth/signup") 
        || path.starts_with("/api/auth/reset-password")
        || path.starts_with("/")  // Static files
    {
        return next.run(request).await;
    }

    // Check for authentication on protected routes
    match get_username_from_request(&request) {
        Some(username) => {
            // Verify user exists in database
            match crate::models::User::get(username).await {
                Ok(_) => next.run(request).await,
                Err(crate::models::DatabaseError::UserNotFound) => {
                    tracing::warn!("User not found in database for valid token");
                    unauthorized_response()
                }
                Err(_) => {
                    tracing::error!("Database error while verifying user");
                    unauthorized_response()
                }
            }
        }
        None => {
            tracing::debug!("No valid authentication token found");
            unauthorized_response()
        }
    }
}

pub fn get_username_from_request(request: &Request) -> Option<String> {
    request
        .headers()
        .get(header::COOKIE)
        .and_then(|cookie_header| cookie_header.to_str().ok())
        .and_then(|cookie_str| {
            cookie_str
                .split(';')
                .find(|cookie| cookie.trim().starts_with("token="))
                .and_then(|token_cookie| token_cookie.split('=').nth(1))
                .and_then(|token| decode_token(token).ok())
                .map(|jwt_data| jwt_data.claims.sub)
        })
}

fn unauthorized_response() -> Response {
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header(header::CONTENT_TYPE, "application/json")
        .body(axum::body::Body::from(r#"{"error": "Unauthorized", "status": 401}"#))
        .unwrap()
}