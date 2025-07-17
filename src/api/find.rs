use axum::{
    extract::Request,
    Json,
};
use serde::{Deserialize, Serialize};

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

pub async fn submit_find_controls(request: Request) -> ApiResult<Json<FindControlsResponse>> {
    let username = crate::auth::middleware::get_username_from_request(&request)
        .ok_or(ApiError::AuthenticationError("Not authenticated".to_string()))?;

    // Extract JSON payload from request
    let payload: FindControlsRequest = {
        let bytes = axum::body::to_bytes(request.into_body(), usize::MAX)
            .await
            .map_err(|_| ApiError::ValidationError("Invalid request body".to_string()))?;
        serde_json::from_slice(&bytes)
            .map_err(|_| ApiError::ValidationError("Invalid JSON payload".to_string()))?
    };

    // Validate description
    if payload.description.trim().len() < 4 {
        return Err(ApiError::ValidationError("Description must be at least 4 characters long".to_string()));
    }

    // Validate n_controls
    if payload.n_controls == 0 {
        return Err(ApiError::ValidationError("Number of controls must be greater than 0".to_string()));
    }

    // Insert query
    let query_id = Query::insert(
        username,
        payload.description.trim().to_string(),
        payload.self_described_latino,
        payload.n_controls,
        payload.excluded_cohorts,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to insert query: {}", e);
        ApiError::InternalServerError
    })?;

    Ok(Json(FindControlsResponse {
        query_id,
        message: "Query submitted successfully".to_string(),
    }))
}