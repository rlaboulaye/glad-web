use axum::Json;
use serde_json::Value;
use std::fs;

use crate::api::{ApiError, ApiResult};

pub async fn get_pca_data() -> ApiResult<Json<Value>> {
    let data_path = "data/db_data/pca_merged.json";
    
    match fs::read_to_string(data_path) {
        Ok(content) => {
            match serde_json::from_str::<Value>(&content) {
                Ok(json_value) => Ok(Json(json_value)),
                Err(e) => {
                    tracing::error!("Failed to parse PCA data JSON: {}", e);
                    Err(ApiError::InternalServerError)
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to read PCA data file: {}", e);
            Err(ApiError::InternalServerError)
        }
    }
}