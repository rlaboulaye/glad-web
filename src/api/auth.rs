use axum::{
    extract::Request,
    http::{header, HeaderValue},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::api::{ApiError, ApiResult};
use crate::auth::jwt::{encode_token, TokenClaims};
use crate::models::{User, verify_password};

// Token duration in seconds (1 hour)
const TOKEN_DURATION: usize = 3600;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub message: String,
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct SignupRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub bio: String,
}

#[derive(Debug, Serialize)]
pub struct SignupResponse {
    pub message: String,
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub username: String,
    pub email: String,
    pub bio: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct ResetPasswordResponse {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordConfirmRequest {
    pub token: String,
    pub password: String,
    pub confirm: String,
}

#[derive(Debug, Serialize)]
pub struct ResetPasswordConfirmResponse {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSettingsRequest {
    pub bio: String,
    pub email: String,
    pub password: String,
    pub confirm_password: String,
}

#[derive(Debug, Serialize)]
pub struct UpdateSettingsResponse {
    pub message: String,
}

pub async fn login(Json(payload): Json<LoginRequest>) -> ApiResult<impl IntoResponse> {
    // Validate input
    if payload.username.is_empty() || payload.password.is_empty() {
        return Err(ApiError::ValidationError("Username and password are required".to_string()));
    }

    // Get user password hash from database
    let hash = sqlx::query_scalar!(
        "SELECT password FROM user WHERE username = $1",
        payload.username
    )
    .fetch_one(crate::database::get_db())
    .await
    .map_err(|_| ApiError::InvalidCredentials)?;

    // Verify password
    verify_password(payload.password, hash)
        .await
        .map_err(|_| ApiError::InvalidCredentials)?;

    // Generate JWT token
    let token = encode_token(TokenClaims {
        sub: payload.username.clone(),
        exp: (sqlx::types::chrono::Utc::now().timestamp() as usize) + TOKEN_DURATION,
    })
    .map_err(|_| ApiError::InternalServerError)?;

    // Create response with cookie
    let cookie = format!("token={}; Path=/; HttpOnly; SameSite=Strict", token);
    let mut response = Json(LoginResponse {
        message: "Login successful".to_string(),
        username: payload.username,
    }).into_response();

    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie).unwrap(),
    );

    Ok(response)
}

pub async fn logout() -> impl IntoResponse {
    let remove_cookie = "token=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0";
    let mut response = Json(serde_json::json!({
        "message": "Logout successful"
    })).into_response();

    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(remove_cookie).unwrap(),
    );

    response
}

pub async fn signup(Json(payload): Json<SignupRequest>) -> ApiResult<impl IntoResponse> {
    // Validate and create user
    let user = User::default()
        .set_username(payload.username.clone())
        .map_err(|e| ApiError::ValidationError(e.to_string()))?
        .set_email(payload.email)
        .map_err(|e| ApiError::ValidationError(e.to_string()))?
        .set_password(payload.password)
        .map_err(|e| ApiError::ValidationError(e.to_string()))?
        .set_bio(payload.bio)
        .map_err(|e| ApiError::ValidationError(e.to_string()))?;

    // Insert user into database
    user.insert().await?;

    // Generate JWT token
    let token = encode_token(TokenClaims {
        sub: payload.username.clone(),
        exp: (sqlx::types::chrono::Utc::now().timestamp() as usize) + TOKEN_DURATION,
    })
    .map_err(|_| ApiError::InternalServerError)?;

    // Create response with cookie
    let cookie = format!("token={}; Path=/; HttpOnly; SameSite=Strict", token);
    let mut response = Json(SignupResponse {
        message: "Signup successful".to_string(),
        username: payload.username,
    }).into_response();

    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie).unwrap(),
    );

    Ok(response)
}

pub async fn me(request: Request) -> ApiResult<Json<UserResponse>> {
    let username = crate::auth::middleware::get_username_from_request(&request)
        .ok_or(ApiError::AuthenticationError("Not authenticated".to_string()))?;

    let user = User::get(username)
        .await
        .map_err(|_| ApiError::UserNotFound)?;

    Ok(Json(UserResponse {
        username: user.username(),
        email: user.email(),
        bio: user.bio().unwrap_or_default(),
    }))
}

pub async fn reset_password(Json(payload): Json<ResetPasswordRequest>) -> ApiResult<Json<ResetPasswordResponse>> {
    use std::env;
    
    // Check if user exists
    match crate::models::User::get_email(payload.email.clone()).await {
        Ok(_) => {
            // User exists, proceed with email sending
            let mailer_email = env::var("MAILER_EMAIL")
                .map_err(|_| ApiError::InternalServerError)?;
            let mailer_passwd = env::var("MAILER_PASSWD")
                .map_err(|_| ApiError::InternalServerError)?;
            let smtp_server = env::var("MAILER_SMTP_SERVER")
                .map_err(|_| ApiError::InternalServerError)?;

            // Generate JWT token for password reset (valid for 1 hour)
            let token = encode_token(TokenClaims {
                sub: payload.email.clone(),
                exp: (sqlx::types::chrono::Utc::now().timestamp() as usize) + 3600,
            })
            .map_err(|_| ApiError::InternalServerError)?;

            // Build reset URL - in production should use HTTPS
            let schema = if cfg!(debug_assertions) { "http" } else { "https" };
            let host = "localhost:5173"; // For development
            let reset_url = format!("{}://{}/reset-password?token={}", schema, host, token);

            // Build email message
            let message = mail_send::mail_builder::MessageBuilder::new()
                .from(("GLAD", mailer_email.as_str()))
                .to(vec![("User", payload.email.as_str())])
                .subject("Password Reset - GLAD")
                .text_body(format!(
                    "You can reset your password by clicking the following link: {}\n\nThis link will expire in 1 hour.",
                    reset_url
                ));

            // Send email
            match mail_send::SmtpClientBuilder::new(smtp_server.as_str(), 587)
                .implicit_tls(false)
                .credentials((mailer_email.as_str(), mailer_passwd.as_str()))
                .connect()
                .await
            {
                Ok(mut client) => {
                    if let Err(e) = client.send(message).await {
                        tracing::error!("Failed to send email: {}", e);
                        return Err(ApiError::InternalServerError);
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to connect to SMTP server: {}", e);
                    return Err(ApiError::InternalServerError);
                }
            }
        }
        Err(_) => {
            // User doesn't exist, but don't reveal this for security
            tracing::debug!("Password reset requested for non-existent email: {}", payload.email);
        }
    }

    // Always return success to prevent email enumeration
    Ok(Json(ResetPasswordResponse {
        message: "If an account with that email exists, a password reset link has been sent".to_string(),
    }))
}

pub async fn reset_password_confirm(Json(payload): Json<ResetPasswordConfirmRequest>) -> ApiResult<Json<ResetPasswordConfirmResponse>> {
    // Validate passwords match
    if payload.password != payload.confirm {
        return Err(ApiError::ValidationError("Passwords do not match".to_string()));
    }

    // Validate password strength using shared validation
    crate::models::User::validate_password(&payload.password)
        .map_err(|e| ApiError::ValidationError(e.to_string()))?;

    // Decode and validate token
    let claims = crate::auth::jwt::decode_token(&payload.token)
        .map_err(|_| ApiError::ValidationError("Invalid or expired reset token".to_string()))?;

    let email = claims.claims.sub;

    // Get user by email
    let mut user = crate::models::User::get_email(email.clone())
        .await
        .map_err(|_| ApiError::UserNotFound)?;

    // Update password
    user = user.set_password(payload.password)
        .map_err(|e| ApiError::ValidationError(e.to_string()))?;

    // Save updated user
    user.update()
        .await
        .map_err(|e| {
            tracing::error!("Failed to update user password: {}", e);
            ApiError::InternalServerError
        })?;

    Ok(Json(ResetPasswordConfirmResponse {
        message: "Password successfully reset. You can now log in with your new password.".to_string(),
    }))
}

pub async fn update_settings(request: Request) -> ApiResult<Json<UpdateSettingsResponse>> {
    let username = crate::auth::middleware::get_username_from_request(&request)
        .ok_or(ApiError::AuthenticationError("Not authenticated".to_string()))?;

    // Extract JSON payload from request
    let payload: UpdateSettingsRequest = {
        let bytes = axum::body::to_bytes(request.into_body(), usize::MAX)
            .await
            .map_err(|_| ApiError::ValidationError("Invalid request body".to_string()))?;
        serde_json::from_slice(&bytes)
            .map_err(|_| ApiError::ValidationError("Invalid JSON payload".to_string()))?
    };

    // Get current user
    let mut user = User::get(username)
        .await
        .map_err(|_| ApiError::UserNotFound)?;

    // Update bio and email
    user = user.set_bio(payload.bio)
        .map_err(|e| ApiError::ValidationError(e.to_string()))?
        .set_email(payload.email)
        .map_err(|e| ApiError::ValidationError(e.to_string()))?;

    // If password is provided, validate and update it
    if !payload.password.is_empty() {
        if payload.password != payload.confirm_password {
            return Err(ApiError::ValidationError("Passwords do not match".to_string()));
        }
        
        // Validate password strength using shared validation
        crate::models::User::validate_password(&payload.password)
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;
        
        user = user.set_password(payload.password)
            .map_err(|e| ApiError::ValidationError(e.to_string()))?;
    }

    // Save updated user
    user.update()
        .await
        .map_err(|e| {
            tracing::error!("Failed to update user settings: {}", e);
            ApiError::from(crate::models::DatabaseError::from(e))
        })?;

    Ok(Json(UpdateSettingsResponse {
        message: "Settings updated successfully".to_string(),
    }))
}