use axum::{extract::Query, Json};
use serde::Deserialize;
use serde_json::Value;
use std::fs;

use crate::api::{ApiError, ApiResult};
use crate::visualization::VISUALIZATION_CACHE;

const PCA_DATA_PATH: &str = "data/visualization_data/pca_merged.json";

#[derive(Deserialize)]
pub struct CommunitiesQuery {
    pub limit: Option<usize>,
}

#[derive(Deserialize)]
pub struct IbdGroupsQuery {
    pub grouping: String,        // Comma-separated field names
    pub min_size: Option<usize>, // Minimum group size filter
}

#[derive(Deserialize)]
pub struct IbdMatrixQuery {
    pub grouping: String,                // "ibd_community,sex"  
    pub selected_groups: Vec<String>,    // ["212 | Male", "49 | Female"]
}

pub async fn get_pca_data() -> ApiResult<Json<Value>> {
    // Access the global visualization cache
    let cache = &*VISUALIZATION_CACHE;
    
    // Convert individuals to JSON format expected by frontend
    let pca_data: Vec<_> = cache.individuals
        .iter()
        .map(|individual| serde_json::json!(individual))
        .collect();
    
    Ok(Json(serde_json::json!(pca_data)))
}

pub async fn get_ibd_communities(Query(params): Query<CommunitiesQuery>) -> ApiResult<Json<Value>> {
    // Access the global visualization cache
    let cache = &*VISUALIZATION_CACHE;
    
    // Get limit parameter (default to 30)
    let limit = params.limit.unwrap_or(30);
    
    // Use pre-computed and sorted communities
    let communities: Vec<_> = cache.communities_by_size
        .iter()
        .take(limit)
        .map(|community| serde_json::json!({
            "id": community.id,
            "size": community.size
        }))
        .collect();
    
    let response = serde_json::json!({
        "communities": communities,
        "total_communities": cache.communities_by_size.len(),
        "total_individuals": cache.individual_to_index.len(),
        "returned_count": communities.len(),
        "limit": limit
    });
    
    Ok(Json(response))
}

pub async fn get_ibd_groups(Query(params): Query<IbdGroupsQuery>) -> ApiResult<Json<Value>> {
    let cache = &*VISUALIZATION_CACHE;
    let result = cache.get_ibd_groups(params.grouping, params.min_size).await?;
    Ok(Json(result))
}

pub async fn compute_ibd_matrix(Json(params): Json<IbdMatrixQuery>) -> ApiResult<Json<Value>> {
    let cache = &*VISUALIZATION_CACHE;
    let result = cache.get_ibd_matrix(params.grouping, params.selected_groups).await?;
    Ok(Json(result))
}