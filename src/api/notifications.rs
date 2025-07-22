use axum::{
    response::Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    api::{ApiError, ApiResult},
    auth::get_username_from_headers,
    models::Notification,
};

#[derive(Serialize)]
pub struct NotificationsResponse {
    pub notifications: Vec<Notification>,
    pub unread_count: i64,
}

#[derive(Deserialize)]
pub struct MarkAsReadRequest {
    pub notification_ids: Vec<i64>,
}

/// Get all notifications for the authenticated user
pub async fn get_notifications(
    headers: axum::http::HeaderMap,
) -> ApiResult<Json<NotificationsResponse>> {
    let username = get_username_from_headers(&headers)
        .ok_or(ApiError::AuthenticationError("Not authenticated".to_string()))?;
    
    let user_id = sqlx::query_scalar!("SELECT user_id FROM user WHERE username = $1", username)
        .fetch_one(crate::database::get_db())
        .await
        .map_err(|_| ApiError::UserNotFound)?
        .ok_or(ApiError::UserNotFound)?;

    let notifications = Notification::for_user(user_id)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    let unread_count = Notification::unread_count_for_user(user_id)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    Ok(Json(NotificationsResponse {
        notifications,
        unread_count,
    }))
}

/// Get unread notification count for the authenticated user
pub async fn get_unread_count(
    headers: axum::http::HeaderMap,
) -> ApiResult<Json<serde_json::Value>> {
    let username = get_username_from_headers(&headers)
        .ok_or(ApiError::AuthenticationError("Not authenticated".to_string()))?;
    
    let user_id = sqlx::query_scalar!("SELECT user_id FROM user WHERE username = $1", username)
        .fetch_one(crate::database::get_db())
        .await
        .map_err(|_| ApiError::UserNotFound)?
        .ok_or(ApiError::UserNotFound)?;

    let unread_count = Notification::unread_count_for_user(user_id)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    Ok(Json(serde_json::json!({ "unread_count": unread_count })))
}

/// Mark specific notifications as read
pub async fn mark_as_read(
    headers: axum::http::HeaderMap,
    Json(request): Json<MarkAsReadRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let username = get_username_from_headers(&headers)
        .ok_or(ApiError::AuthenticationError("Not authenticated".to_string()))?;
    
    let user_id = sqlx::query_scalar!("SELECT user_id FROM user WHERE username = $1", username)
        .fetch_one(crate::database::get_db())
        .await
        .map_err(|_| ApiError::UserNotFound)?
        .ok_or(ApiError::UserNotFound)?;

    let mut marked_count = 0;
    for notification_id in request.notification_ids {
        if Notification::mark_as_read(notification_id, user_id)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?
        {
            marked_count += 1;
        }
    }

    Ok(Json(serde_json::json!({ 
        "marked_count": marked_count,
        "message": format!("Marked {} notifications as read", marked_count)
    })))
}

/// Mark all notifications as read for the authenticated user
pub async fn mark_all_as_read(
    headers: axum::http::HeaderMap,
) -> ApiResult<Json<serde_json::Value>> {
    let username = get_username_from_headers(&headers)
        .ok_or(ApiError::AuthenticationError("Not authenticated".to_string()))?;
    
    let user_id = sqlx::query_scalar!("SELECT user_id FROM user WHERE username = $1", username)
        .fetch_one(crate::database::get_db())
        .await
        .map_err(|_| ApiError::UserNotFound)?
        .ok_or(ApiError::UserNotFound)?;

    let marked_count = Notification::mark_all_as_read(user_id)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    Ok(Json(serde_json::json!({ 
        "marked_count": marked_count,
        "message": format!("Marked {} notifications as read", marked_count)
    })))
}

/// Manually trigger notification processing (for testing/admin purposes)
pub async fn process_pending_notifications(
    _headers: axum::http::HeaderMap, // Authentication not required for now, but structure is ready
) -> ApiResult<Json<serde_json::Value>> {
    let created_count = Notification::process_pending_notifications()
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "created_count": created_count,
        "message": format!("Processed {} pending notifications", created_count)
    })))
}