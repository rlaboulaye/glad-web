use axum::{
    extract::{Path, Request, Multipart},
    http::HeaderMap,
    Json,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

use crate::api::{ApiError, ApiResult};
use crate::models::{Cohort, Query};

#[derive(Debug, Deserialize)]
pub struct FindControlsRequest {
    pub description: String,
    pub self_described_latino: bool,
    pub n_controls: usize,
    pub excluded_cohorts: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct FindControlsResponse {
    pub query_id: i64,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct CohortsResponse {
    pub cohorts: Vec<Cohort>,
}

pub async fn get_cohorts() -> ApiResult<Json<CohortsResponse>> {
    let cohorts = Cohort::retrieve_all()
        .await
        .map_err(|e| {
            tracing::error!("Failed to retrieve cohorts: {}", e);
            ApiError::InternalServerError
        })?;

    Ok(Json(CohortsResponse { cohorts }))
}

pub async fn submit_find_controls(headers: HeaderMap, mut multipart: Multipart) -> ApiResult<Json<FindControlsResponse>> {
    // Use existing function to get username from headers
    let username = crate::auth::get_username_from_headers(&headers)
        .ok_or(ApiError::AuthenticationError("Not authenticated".to_string()))?;

    // Get user_id for directory structure
    let user_id = sqlx::query_scalar!("SELECT user_id FROM user WHERE username = $1", username)
        .fetch_one(crate::database::get_db())
        .await
        .map_err(|e| {
            tracing::error!("Failed to get user_id: {}", e);
            ApiError::UserNotFound
        })?
        .ok_or(ApiError::UserNotFound)?;

    // Parse form fields
    let mut description = String::new();
    let mut self_described_latino = false;
    let mut n_controls = 100usize;
    let mut excluded_cohorts = Vec::new();
    let mut file_data: Option<Vec<u8>> = None;

    while let Some(field) = multipart.next_field().await
        .map_err(|e| {
            tracing::error!("Failed to read multipart field: {}", e);
            ApiError::ValidationError("Invalid multipart field".to_string())
        })? {
        
        let name = field.name().unwrap_or("").to_string();
        
        match name.as_str() {
            "description" => {
                let data = field.bytes().await.map_err(|_| 
                    ApiError::ValidationError("Failed to read description".to_string()))?;
                description = String::from_utf8(data.to_vec()).map_err(|_|
                    ApiError::ValidationError("Invalid description encoding".to_string()))?;
            },
            "self_described_latino" => {
                let data = field.bytes().await.map_err(|_|
                    ApiError::ValidationError("Failed to read self_described_latino".to_string()))?;
                let value = String::from_utf8(data.to_vec()).map_err(|_|
                    ApiError::ValidationError("Invalid self_described_latino encoding".to_string()))?;
                self_described_latino = value.trim().to_lowercase() == "true";
            },
            "n_controls" => {
                let data = field.bytes().await.map_err(|_|
                    ApiError::ValidationError("Failed to read n_controls".to_string()))?;
                let value = String::from_utf8(data.to_vec()).map_err(|_|
                    ApiError::ValidationError("Invalid n_controls encoding".to_string()))?;
                n_controls = value.trim().parse().map_err(|_|
                    ApiError::ValidationError("Invalid n_controls format".to_string()))?;
            },
            "excluded_cohorts" => {
                let data = field.bytes().await.map_err(|_|
                    ApiError::ValidationError("Failed to read excluded_cohorts".to_string()))?;
                let value = String::from_utf8(data.to_vec()).map_err(|_|
                    ApiError::ValidationError("Invalid excluded_cohorts encoding".to_string()))?;
                excluded_cohorts = serde_json::from_str(&value).map_err(|_|
                    ApiError::ValidationError("Invalid excluded_cohorts JSON".to_string()))?;
            },
            "query_file" => {
                file_data = Some(field.bytes().await.map_err(|_|
                    ApiError::ValidationError("Failed to read file data".to_string()))?.to_vec());
            },
            _ => {
                // Skip unknown fields
                let _ = field.bytes().await;
            }
        }
    }

    // Validate form data
    if description.trim().len() < 4 {
        return Err(ApiError::ValidationError("Description must be at least 4 characters long".to_string()));
    }

    if n_controls == 0 {
        return Err(ApiError::ValidationError("Number of controls must be greater than 0".to_string()));
    }

    let file_data = file_data.ok_or(ApiError::ValidationError("File upload is required".to_string()))?;
    
    // Validate file size (10MB limit)
    const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;
    if file_data.len() > MAX_FILE_SIZE {
        return Err(ApiError::ValidationError("File size exceeds 10MB limit".to_string()));
    }

    // Get query root path from environment
    let query_root = std::env::var("QUERY_PATH_ROOT")
        .map_err(|_| ApiError::InternalServerError)?;

    // Generate a temporary query ID for directory structure
    // We'll use a UUID to avoid conflicts, then map it in the database
    let temp_query_uuid = uuid::Uuid::new_v4().to_string();
    let temp_relative_path = format!("{}/{}/query_samples.txt", user_id, temp_query_uuid);
    let temp_full_path = PathBuf::from(&query_root).join(&temp_relative_path);

    // Create directory structure
    let temp_dir = temp_full_path.parent().unwrap();
    fs::create_dir_all(temp_dir).await
        .map_err(|e| {
            tracing::error!("Failed to create directory {}: {}", temp_dir.display(), e);
            ApiError::InternalServerError
        })?;

    // Save file to temporary location first
    fs::write(&temp_full_path, &file_data).await
        .map_err(|e| {
            tracing::error!("Failed to write file {}: {}", temp_full_path.display(), e);
            ApiError::InternalServerError
        })?;

    // Insert query with the temporary file path (will be updated later)
    let query_id = Query::insert(
        username.clone(),
        description.trim().to_string(),
        self_described_latino,
        n_controls,
        excluded_cohorts,
        temp_relative_path.clone(),
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to insert query: {}", e);
        ApiError::InternalServerError
    })?;

    // Now move the file to the correct location with the actual query_id
    let final_relative_path = format!("{}/{}/query_samples.txt", user_id, query_id);
    let final_full_path = PathBuf::from(&query_root).join(&final_relative_path);
    let final_dir = final_full_path.parent().unwrap();
    
    // Create final directory and move file
    fs::create_dir_all(final_dir).await
        .map_err(|e| {
            tracing::error!("Failed to create final directory {}: {}", final_dir.display(), e);
            ApiError::InternalServerError
        })?;

    fs::rename(&temp_full_path, &final_full_path).await
        .map_err(|e| {
            tracing::error!("Failed to move file from {} to {}: {}", 
                temp_full_path.display(), final_full_path.display(), e);
            ApiError::InternalServerError
        })?;

    // Clean up temporary directory (remove the entire UUID directory)
    let temp_uuid_dir = PathBuf::from(&query_root).join(&user_id.to_string()).join(&temp_query_uuid);
    let _ = fs::remove_dir_all(&temp_uuid_dir).await;

    // Update the query with the correct file path
    sqlx::query!("UPDATE query SET file_path = $1 WHERE query_id = $2", 
        final_relative_path, query_id)
        .execute(crate::database::get_db())
        .await
        .map_err(|e| {
            tracing::error!("Failed to update query file path: {}", e);
            ApiError::InternalServerError
        })?;

    Ok(Json(FindControlsResponse {
        query_id,
        message: "Query submitted successfully".to_string(),
    }))
}

#[derive(Debug, Serialize)]
pub struct UserQueriesResponse {
    pub queries: Vec<Query>,
}

pub async fn get_user_queries(request: Request) -> ApiResult<Json<UserQueriesResponse>> {
    let username = crate::auth::middleware::get_username_from_request(&request)
        .ok_or(ApiError::AuthenticationError("Not authenticated".to_string()))?;

    let queries = Query::for_user_profile(username)
        .await
        .map_err(|e| {
            tracing::error!("Failed to retrieve user queries: {}", e);
            ApiError::InternalServerError
        })?;

    Ok(Json(UserQueriesResponse { queries }))
}

pub async fn get_query_details(Path(query_id): Path<i64>, request: Request) -> ApiResult<Json<Query>> {
    let username = crate::auth::middleware::get_username_from_request(&request)
        .ok_or(ApiError::AuthenticationError("Not authenticated".to_string()))?;

    let query = Query::for_query(query_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to retrieve query {}: {}", query_id, e);
            ApiError::UserNotFound
        })?;

    // Verify the query belongs to the authenticated user
    let user = crate::models::User::get(username.clone())
        .await
        .map_err(|_| ApiError::UserNotFound)?;
    
    // Convert user_id to i64 for comparison (assuming user table has integer IDs)
    let user_id = sqlx::query_scalar!("SELECT user_id FROM user WHERE username = $1", username)
        .fetch_one(crate::database::get_db())
        .await
        .map_err(|_| ApiError::UserNotFound)?;

    let user_id = user_id.ok_or(ApiError::UserNotFound)?;

    if query.user_id != user_id {
        return Err(ApiError::AuthenticationError("Access denied".to_string()));
    }

    Ok(Json(query))
}