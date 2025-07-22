use once_cell::sync::Lazy;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use sprs::{CsMat, TriMatI};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, RwLock};
use tracing::{error, info};

pub mod cache;
pub mod ibd;
pub mod pca;

use crate::api::ApiError;

// File paths
const PCA_DATA_PATH: &str = "data/visualization_data/pca_merged.json";
const IBD_COMMUNITIES_PATH: &str = "data/visualization_data/info_all_membership_df.tsv";
const IBD_MATRIX_PATH: &str = "data/visualization_data/clust_all_mat.mtx";

/// Canonical order for grouping fields (ensures consistent cache keys)
/// Matches frontend field order for consistency
const CANONICAL_FIELD_ORDER: &[&str] = &[
    "phs", 
    "country", 
    "region", 
    "sex", 
    "ethnicity", 
    "self_described", 
    "ibd_community"
];

/// Minimum group size for privacy protection
const MIN_GROUP_SIZE: usize = 30;


/// Global visualization cache instance
pub static VISUALIZATION_CACHE: Lazy<VisualizationCache> = Lazy::new(|| {
    match VisualizationCache::new() {
        Ok(cache) => {
            info!("Visualization cache initialized successfully");
            cache
        }
        Err(e) => {
            error!("Failed to initialize visualization cache: {:?}", e);
            panic!("Could not initialize visualization cache: {:?}", e);
        }
    }
});

/// Main visualization cache structure
pub struct VisualizationCache {
    /// Mapping from individual ID to matrix index
    pub individual_to_index: HashMap<String, usize>,
    /// Mapping from IBD community ID to list of matrix indices
    pub community_to_individuals: HashMap<u32, Vec<usize>>,
    /// Individual data indexed by matrix position
    pub individuals: Vec<Individual>,
    /// Pre-sorted communities by size (largest first)
    pub communities_by_size: Vec<CommunityInfo>,
    /// Sparse IBD matrix
    pub ibd_matrix: Option<CsMat<f32>>,
    /// Transposed IBD matrix for optimization
    pub ibd_matrix_t: Option<CsMat<f32>>,
    /// Cached computed matrices for different groupings
    pub computed_matrices: Arc<RwLock<HashMap<String, ComputedMatrix>>>,
    /// Available groups for each grouping combination
    pub available_groups: Arc<RwLock<HashMap<String, Vec<Group>>>>,
}

/// A computed IBD matrix for a specific grouping
#[derive(Debug, Clone, Serialize)]
pub struct ComputedMatrix {
    /// The aggregated matrix values
    pub matrix: Vec<Vec<f32>>,
    /// Labels for each group (row/column)
    pub group_labels: Vec<String>,
    /// Size of each group
    pub group_sizes: Vec<usize>,
}

/// Represents a group of individuals for aggregation
#[derive(Debug, Clone, Serialize)]
pub struct Group {
    /// Human-readable label for the group
    pub label: String,
    /// Number of individuals in the group
    pub size: usize,
    /// Matrix indices of individuals in this group
    pub individuals: Vec<usize>,
}

/// Information about an IBD community (for caching)
#[derive(Debug, Clone)]
pub struct CommunityInfo {
    /// Community ID
    pub id: u32,
    /// Number of individuals in the community
    pub size: usize,
}

/// Individual sample data (merged from PCA + IBD data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Individual {
    pub id: String,
    pub pc: Vec<f64>,
    pub country: Option<String>,
    pub ethnicity: Option<String>,
    pub region: Option<String>,
    pub sex: Option<String>,
    pub phs: Option<String>,
    pub self_described: Option<String>,
    pub project: Option<String>,
    pub ibd_community: Option<u32>,
    pub glad_status: Option<String>,
    /// Index in the IBD sparse matrix (row/column index), None if not in IBD data
    #[serde(skip)]
    pub ibd_matrix_index: Option<usize>,
}

impl VisualizationCache {
    /// Initialize the visualization cache
    pub fn new() -> Result<Self, ApiError> {
        info!("Initializing visualization cache...");
        
        let mut cache = VisualizationCache {
            individual_to_index: HashMap::new(),
            community_to_individuals: HashMap::new(),
            individuals: Vec::new(),
            communities_by_size: Vec::new(),
            ibd_matrix: None,
            ibd_matrix_t: None,
            computed_matrices: Arc::new(RwLock::new(HashMap::new())),
            available_groups: Arc::new(RwLock::new(HashMap::new())),
        };

        // Load and merge PCA + IBD data
        cache.load_and_merge_individual_data()?;
        
        // Load sparse matrix
        cache.load_ibd_matrix()?;
        
        // Pre-compute and sort communities by size
        cache.compute_communities_by_size();
        
        info!("Visualization cache initialized with {} individuals and {} communities", 
              cache.individual_to_index.len(), 
              cache.communities_by_size.len());
        
        Ok(cache)
    }

    /// Load and merge PCA and IBD data
    fn load_and_merge_individual_data(&mut self) -> Result<(), ApiError> {
        info!("Loading and merging PCA and IBD data...");
        
        // Step 1: Load PCA data
        let pca_content = std::fs::read_to_string(PCA_DATA_PATH)
            .map_err(|e| {
                error!("Failed to read PCA file {}: {}", PCA_DATA_PATH, e);
                ApiError::InternalServerError
            })?;

        let pca_data: Vec<serde_json::Value> = serde_json::from_str(&pca_content)
            .map_err(|e| {
                error!("Failed to parse PCA JSON: {}", e);
                ApiError::InternalServerError
            })?;

        // Create individuals from PCA data
        let mut individuals_map: HashMap<String, Individual> = HashMap::new();
        
        for item in pca_data {
            if let (Some(id), Some(pc_array)) = (
                item.get("id").and_then(|v| v.as_str()),
                item.get("pc").and_then(|v| v.as_array())
            ) {
                let pc: Vec<f64> = pc_array.iter()
                    .filter_map(|v| v.as_f64())
                    .collect();
                
                let individual = Individual {
                    id: id.to_string(),
                    pc,
                    country: item.get("country").and_then(|v| v.as_str()).map(String::from),
                    ethnicity: item.get("ethnicity").and_then(|v| v.as_str()).map(String::from),
                    region: item.get("region").and_then(|v| v.as_str()).map(String::from),
                    sex: item.get("sex").and_then(|v| v.as_str()).map(String::from),
                    phs: item.get("phs").and_then(|v| v.as_str()).map(String::from),
                    self_described: item.get("self_described").and_then(|v| v.as_str()).map(String::from),
                    project: None,
                    ibd_community: None,
                    glad_status: None,
                    ibd_matrix_index: None,
                };
                
                individuals_map.insert(id.to_string(), individual);
            }
        }

        info!("Loaded {} individuals from PCA data", individuals_map.len());

        // Step 2: Load IBD data and merge additional fields
        let ibd_content = std::fs::read_to_string(IBD_COMMUNITIES_PATH)
            .map_err(|e| {
                error!("Failed to read IBD TSV file {}: {}", IBD_COMMUNITIES_PATH, e);
                ApiError::InternalServerError
            })?;

        let mut csv_reader = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .from_reader(ibd_content.as_bytes());

        // Define a struct for parsing IBD community TSV rows
        #[derive(Deserialize)]
        struct IbdCommunityRow {
            #[serde(rename = "Vcf_ID")]
            matrix_index: usize,
            #[serde(rename = "Sample")]
            sample: String,
            #[serde(rename = "Membership")]
            membership: Option<u32>,
            #[serde(rename = "Project")]
            project: Option<String>,
            #[serde(rename = "GLAD_Status")]
            glad_status: Option<String>,
        }

        // Parse TSV and merge data, using Vcf_ID as the matrix index
        for result in csv_reader.deserialize::<IbdCommunityRow>() {
            let row = result.map_err(|e| {
                error!("Failed to parse IBD community TSV row: {}", e);
                ApiError::InternalServerError
            })?;

            if let Some(individual) = individuals_map.get_mut(&row.sample) {
                // Merge additional fields from IBD data
                individual.ibd_community = row.membership;
                individual.project = row.project;
                individual.glad_status = row.glad_status;
                // Use the Vcf_ID as the IBD matrix index
                individual.ibd_matrix_index = Some(row.matrix_index);
            }
        }

        // Step 3: Build final data structures
        self.individuals = individuals_map.into_values().collect();
        
        // Build index mappings
        for (array_index, individual) in self.individuals.iter().enumerate() {
            self.individual_to_index.insert(individual.id.clone(), array_index);

            // Build community mappings using IBD matrix indices (not array indices)
            if let (Some(community_id), Some(ibd_matrix_index)) = (individual.ibd_community, individual.ibd_matrix_index) {
                self.community_to_individuals
                    .entry(community_id)
                    .or_insert_with(Vec::new)
                    .push(ibd_matrix_index); // Use the correct IBD matrix index
            }
        }

        info!("Merged data for {} individuals with {} IBD communities", 
              self.individuals.len(), 
              self.community_to_individuals.len());

        // Debug: Check community 212 indices
        if let Some(community_212_indices) = self.community_to_individuals.get(&212) {
            info!("Community 212 has {} individuals with IBD matrix indices: {:?}", 
                  community_212_indices.len(), 
                  &community_212_indices[0..std::cmp::min(10, community_212_indices.len())]);
        }
        
        Ok(())
    }

    /// Load the sparse IBD matrix
    fn load_ibd_matrix(&mut self) -> Result<(), ApiError> {
        info!("Loading sparse IBD matrix...");
        
        let file = File::open(IBD_MATRIX_PATH)
            .map_err(|e| {
                error!("Failed to open matrix file {}: {}", IBD_MATRIX_PATH, e);
                ApiError::InternalServerError
            })?;

        let mut buf_reader = BufReader::new(file);
        
        let matrix: TriMatI<f32, usize> = sprs::io::read_matrix_market_from_bufread::<f32, usize, BufReader<File>>(&mut buf_reader)
            .map_err(|e| {
                error!("Failed to load matrix from {}: {}", IBD_MATRIX_PATH, e);
                ApiError::InternalServerError
            })?;

        // Convert to CsMat for efficient operations
        let csr_matrix = matrix.to_csr();
        
        // Pre-compute transposed matrix for optimization
        info!("Computing transposed matrix for optimization...");
        let transposed_matrix = csr_matrix.clone().transpose_into();
        
        self.ibd_matrix = Some(csr_matrix);
        self.ibd_matrix_t = Some(transposed_matrix);
        
        info!("Loaded {}x{} sparse matrix with {} non-zero entries", 
              self.ibd_matrix.as_ref().unwrap().rows(),
              self.ibd_matrix.as_ref().unwrap().cols(),
              self.ibd_matrix.as_ref().unwrap().nnz());
        
        Ok(())
    }

    /// Get matrix value at (i, j)
    pub fn get_ibd_value(&self, i: usize, j: usize) -> Option<f32> {
        self.ibd_matrix.as_ref()?.get(i, j).copied()
    }

    /// Get individuals in a specific community
    pub fn get_community_individuals(&self, community_id: u32) -> Option<&Vec<usize>> {
        self.community_to_individuals.get(&community_id)
    }

    /// Get matrix index for an individual
    pub fn get_individual_index(&self, individual_id: &str) -> Option<usize> {
        self.individual_to_index.get(individual_id).copied()
    }

    /// Pre-compute and sort communities by size (largest first)
    fn compute_communities_by_size(&mut self) {
        info!("Pre-computing community size rankings...");
        
        self.communities_by_size = self.community_to_individuals
            .iter()
            .map(|(&id, individuals)| CommunityInfo {
                id,
                size: individuals.len(),
            })
            .collect();

        // Sort by size in descending order (largest first)
        self.communities_by_size.sort_by(|a, b| b.size.cmp(&a.size));
        
        info!("Sorted {} communities by size", self.communities_by_size.len());
    }

    /// Generate groups based on field combinations with early pruning
    pub fn generate_groups(&self, fields: &[String]) -> Vec<Group> {
        self.generate_groups_with_min_size(fields, 1) // Default minimum of 1
    }

    /// Generate groups with minimum size constraint and early pruning
    pub fn generate_groups_with_min_size(&self, fields: &[String], min_size: usize) -> Vec<Group> {
        if fields.is_empty() {
            return Vec::new();
        }

        // Step 1: Pre-filter field values that can meet minimum size requirement
        let valid_field_values = self.pre_filter_field_values(fields, min_size);

        // Step 2: Only process individuals with all valid field values
        let mut group_map: HashMap<String, Vec<usize>> = HashMap::new();

        for (index, individual) in self.individuals.iter().enumerate() {
            // Check if individual has valid values for all fields
            if self.individual_has_valid_values(individual, fields, &valid_field_values) {
                if let Some(group_key) = self.create_group_key(individual, fields) {
                    // Only include individuals that have IBD matrix indices
                    if let Some(ibd_matrix_index) = individual.ibd_matrix_index {
                        group_map.entry(group_key).or_insert_with(Vec::new).push(ibd_matrix_index);
                    }
                }
                // If create_group_key returns None, individual is automatically excluded
            }
        }

        // Step 3: Convert to Group structs and filter by minimum size
        let mut groups: Vec<Group> = group_map
            .into_iter()
            .filter(|(_, individuals)| individuals.len() >= min_size)
            .map(|(label, individuals)| Group {
                label,
                size: individuals.len(),
                individuals,
            })
            .collect();

        // Sort by size (largest first)
        groups.sort_by(|a, b| b.size.cmp(&a.size));
        
        groups
    }

    /// Pre-filter field values that have enough individuals to potentially meet min_size
    fn pre_filter_field_values(&self, fields: &[String], min_size: usize) -> HashMap<String, HashSet<String>> {
        let mut field_value_counts: HashMap<String, HashMap<String, usize>> = HashMap::new();

        // Count occurrences of each field value
        for individual in &self.individuals {
            for field in fields {
                if let Some(value) = self.get_field_value(individual, field) {
                    *field_value_counts
                        .entry(field.clone())
                        .or_insert_with(HashMap::new)
                        .entry(value)
                        .or_insert(0) += 1;
                }
                // Skip individuals with None values - they won't be counted
            }
        }

        // Filter values that meet minimum size requirement
        field_value_counts
            .into_iter()
            .map(|(field, value_counts)| {
                let valid_values: HashSet<String> = value_counts
                    .into_iter()
                    .filter(|(_, count)| *count >= min_size)
                    .map(|(value, _)| value)
                    .collect();
                (field, valid_values)
            })
            .collect()
    }

    /// Check if individual has valid field values for all specified fields
    fn individual_has_valid_values(
        &self,
        individual: &Individual,
        fields: &[String],
        valid_field_values: &HashMap<String, HashSet<String>>
    ) -> bool {
        fields.iter().all(|field| {
            if let Some(value) = self.get_field_value(individual, field) {
                valid_field_values
                    .get(field)
                    .map(|valid_values| valid_values.contains(&value))
                    .unwrap_or(false)
            } else {
                // Individual has None for this field, exclude them
                false
            }
        })
    }

    /// Create a group key from individual's field values (using canonical field order)
    /// Returns None if any required field is missing
    fn create_group_key(&self, individual: &Individual, fields: &[String]) -> Option<String> {
        // Use canonical order to ensure consistent cache keys
        let ordered_fields: Option<Vec<String>> = CANONICAL_FIELD_ORDER.iter()
            .filter(|field| fields.contains(&field.to_string()))
            .map(|field| self.get_field_value(individual, field))
            .collect();
        
        ordered_fields.map(|fields| fields.join(" | "))
    }

    /// Get field value from individual (only canonical fields)
    fn get_field_value(&self, individual: &Individual, field: &str) -> Option<String> {
        match field {
            "phs" => individual.phs.clone(),
            "sex" => individual.sex.clone(),
            "country" => individual.country.clone(),
            "region" => individual.region.clone(),
            "ethnicity" => individual.ethnicity.clone(),
            "self_described" => individual.self_described.clone(),
            "ibd_community" => individual.ibd_community.map(|c| c.to_string()),
            _ => None,  // Reject non-canonical fields
        }
    }

    /// Compute mean pairwise IBD matrix between groups using spawn_blocking
    pub async fn compute_group_ibd_matrix(&self, groups: &[Group]) -> Result<ComputedMatrix, ApiError> {
        ibd::IbdComputation::compute_group_ibd_matrix(groups).await
    }

    pub async fn compute_asymmetric_group_ibd_matrix(&self, row_groups: &[Group], column_groups: &[Group]) -> Result<ComputedMatrix, ApiError> {
        ibd::IbdComputation::compute_asymmetric_group_ibd_matrix(row_groups, column_groups).await
    }

    /// Validate and parse grouping fields
    fn validate_and_parse_fields(&self, grouping: &str) -> Result<Vec<String>, ApiError> {
        let fields: Vec<String> = grouping
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        
        // Validate all fields are canonical
        for field in &fields {
            if !CANONICAL_FIELD_ORDER.contains(&field.as_str()) {
                return Err(ApiError::ValidationError(
                    format!("Invalid grouping field: '{}'. Valid fields are: {}", 
                           field, CANONICAL_FIELD_ORDER.join(", "))
                ));
            }
        }
        
        Ok(fields)
    }

    /// Get IBD groups with validation and processing
    pub async fn get_ibd_groups(&self, grouping: String, min_size: Option<usize>) -> Result<serde_json::Value, ApiError> {
        
        let fields = self.validate_and_parse_fields(&grouping)?;
        let min_size = min_size.unwrap_or(MIN_GROUP_SIZE);
        let groups = self.generate_groups_with_min_size(&fields, min_size);
        
        let filtered_groups: Vec<_> = groups.into_iter()
            .map(|group| serde_json::json!({
                "label": group.label,
                "size": group.size
            }))
            .collect();
        
        Ok(serde_json::json!({
            "groups": filtered_groups,
            "grouping": grouping,
            "min_size": min_size,
            "total_groups": filtered_groups.len()
        }))
    }

    /// Compute IBD matrix with validation and processing
    pub async fn get_ibd_matrix(&self, grouping: String, selected_groups: Vec<String>) -> Result<serde_json::Value, ApiError> {
        
        let fields = self.validate_and_parse_fields(&grouping)?;
        
        // Generate all groups to find the selected ones
        let all_groups = self.generate_groups_with_min_size(&fields, MIN_GROUP_SIZE);
        let selected_groups_data: Vec<_> = all_groups.into_iter()
            .filter(|group| selected_groups.contains(&group.label))
            .collect();
        
        // Validate all selected groups exist
        if selected_groups_data.len() != selected_groups.len() {
            return Err(ApiError::ValidationError("One or more selected groups do not exist".to_string()));
        }
        
        // Double-check minimum group size for privacy protection
        for group in &selected_groups_data {
            if group.size < MIN_GROUP_SIZE {
                return Err(ApiError::ValidationError(
                    format!("Group '{}' has {} individuals, minimum is {}", 
                           group.label, group.size, MIN_GROUP_SIZE)
                ));
            }
        }
        
        // Compute IBD matrix
        let computed_matrix = self.compute_group_ibd_matrix(&selected_groups_data).await?;
        
        Ok(serde_json::json!({
            "matrix": computed_matrix.matrix,
            "group_labels": computed_matrix.group_labels,
            "group_sizes": computed_matrix.group_sizes,
            "grouping": grouping
        }))
    }

    /// Compute asymmetric IBD matrix with different row and column groupings
    pub async fn get_asymmetric_ibd_matrix(
        &self, 
        row_grouping: String, 
        column_grouping: String, 
        selected_row_groups: Vec<String>, 
        selected_column_groups: Vec<String>
    ) -> Result<serde_json::Value, ApiError> {
        
        // Validate and parse both grouping fields
        let row_fields = self.validate_and_parse_fields(&row_grouping)?;
        let column_fields = self.validate_and_parse_fields(&column_grouping)?;
        
        // Generate all groups for both axes
        let all_row_groups = self.generate_groups_with_min_size(&row_fields, MIN_GROUP_SIZE);
        let all_column_groups = self.generate_groups_with_min_size(&column_fields, MIN_GROUP_SIZE);
        
        // Find selected groups
        let selected_row_groups_data: Vec<_> = all_row_groups.into_iter()
            .filter(|group| selected_row_groups.contains(&group.label))
            .collect();
        let selected_column_groups_data: Vec<_> = all_column_groups.into_iter()
            .filter(|group| selected_column_groups.contains(&group.label))
            .collect();
        
        // Validate all selected groups exist
        if selected_row_groups_data.len() != selected_row_groups.len() {
            return Err(ApiError::ValidationError("One or more selected row groups do not exist".to_string()));
        }
        if selected_column_groups_data.len() != selected_column_groups.len() {
            return Err(ApiError::ValidationError("One or more selected column groups do not exist".to_string()));
        }
        
        // Double-check minimum group size for privacy protection
        for group in &selected_row_groups_data {
            if group.size < MIN_GROUP_SIZE {
                return Err(ApiError::ValidationError(
                    format!("Row group '{}' has {} individuals, minimum is {}", 
                           group.label, group.size, MIN_GROUP_SIZE)
                ));
            }
        }
        for group in &selected_column_groups_data {
            if group.size < MIN_GROUP_SIZE {
                return Err(ApiError::ValidationError(
                    format!("Column group '{}' has {} individuals, minimum is {}", 
                           group.label, group.size, MIN_GROUP_SIZE)
                ));
            }
        }
        
        // Compute asymmetric IBD matrix
        let computed_matrix = self.compute_asymmetric_group_ibd_matrix(
            &selected_row_groups_data, 
            &selected_column_groups_data
        ).await?;
        
        Ok(serde_json::json!({
            "matrix": computed_matrix.matrix,
            "group_labels": computed_matrix.group_labels,  // Column group labels
            "group_sizes": computed_matrix.group_sizes,    // Column group sizes
            "row_group_labels": selected_row_groups_data.iter().map(|g| g.label.clone()).collect::<Vec<String>>(),
            "row_group_sizes": selected_row_groups_data.iter().map(|g| g.size).collect::<Vec<usize>>(),
            "row_grouping": row_grouping,
            "column_grouping": column_grouping
        }))
    }
}