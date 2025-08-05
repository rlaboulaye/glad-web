// IBD matrix computation and aggregation functions

use rayon::prelude::*;
use sprs::CsMat;
use std::collections::HashSet;
use tracing::{error, info};

use super::{ComputedMatrix, Group, VISUALIZATION_CACHE};
use crate::api::ApiError;

/// IBD matrix computation algorithms
pub struct IbdComputation;

impl IbdComputation {
    /// Compute mean pairwise IBD matrix between groups using spawn_blocking with pairwise caching
    pub async fn compute_group_ibd_matrix(groups: &[Group]) -> Result<ComputedMatrix, ApiError> {
        if groups.is_empty() {
            return Err(ApiError::ValidationError("No groups provided".to_string()));
        }

        let matrix = VISUALIZATION_CACHE
            .ibd_matrix
            .as_ref()
            .ok_or_else(|| ApiError::InternalServerError)?;

        let n_groups = groups.len();
        let mut result_matrix: Vec<Vec<f32>> = vec![vec![0.0; n_groups]; n_groups];
        let mut pairs_to_compute: Vec<(usize, usize, String, String)> = Vec::new();
        let mut cache_hits = 0;

        // Check cache for each pair and identify what needs to be computed
        for i in 0..n_groups {
            for j in i..n_groups {
                // Only compute upper triangle + diagonal
                let cache_key =
                    Self::generate_pair_cache_key(&groups[i].label, &groups[j].label);

                if let Some(cached_value) = VISUALIZATION_CACHE.pairwise_ibd_cache.get(&cache_key) {
                    result_matrix[i][j] = cached_value;
                    if i != j {
                        result_matrix[j][i] = cached_value; // Symmetric
                    }
                    cache_hits += 1;
                } else {
                    // Need to compute this pair
                    pairs_to_compute.push((
                        i,
                        j,
                        groups[i].label.clone(),
                        groups[j].label.clone(),
                    ));
                }
            }
        }

        info!(
            "IBD matrix cache: {} hits, {} pairs to compute",
            cache_hits,
            pairs_to_compute.len()
        );

        // Compute missing pairs
        if !pairs_to_compute.is_empty() {
            // Clone data for spawn_blocking
            let groups_for_computation: Vec<_> = pairs_to_compute
                .iter()
                .map(|(i, j, _, _)| (groups[*i].clone(), groups[*j].clone()))
                .collect();
            let matrix_clone = matrix.clone();

            // Compute all missing pairs in parallel
            let computed_values = tokio::task::spawn_blocking(move || {
                Self::compute_pairs_blocking(groups_for_computation, matrix_clone)
            })
            .await
            .map_err(|e| {
                error!("Failed to spawn pair computation task: {}", e);
                ApiError::InternalServerError
            })??;

            // Store computed values in result matrix and cache
            for ((i, j, label_i, label_j), computed_value) in
                pairs_to_compute.iter().zip(computed_values.iter())
            {
                result_matrix[*i][*j] = *computed_value;
                if i != j {
                    result_matrix[*j][*i] = *computed_value; // Symmetric
                }

                // Cache the computed pair value
                let cache_key = Self::generate_pair_cache_key(label_i, label_j);
                VISUALIZATION_CACHE.pairwise_ibd_cache.insert(cache_key, *computed_value);
            }

            info!(
                "Cached {} new IBD pairs (cache size: {})",
                pairs_to_compute.len(),
                VISUALIZATION_CACHE.pairwise_ibd_cache.len()
            );
        }

        // Build final result
        let group_labels: Vec<String> = groups.iter().map(|g| g.label.clone()).collect();
        let group_sizes: Vec<usize> = groups.iter().map(|g| g.size).collect();

        Ok(ComputedMatrix {
            matrix: result_matrix,
            group_labels,
            group_sizes,
        })
    }

    /// Generate cache key for a pair of groups (order-independent)
    fn generate_pair_cache_key(label1: &str, label2: &str) -> String {
        if label1 <= label2 {
            format!("{}|{}", label1, label2)
        } else {
            format!("{}|{}", label2, label1)
        }
    }

    /// Compute IBD values for specific pairs of groups
    fn compute_pairs_blocking(
        group_pairs: Vec<(Group, Group)>,
        matrix: CsMat<f32>,
    ) -> Result<Vec<f32>, ApiError> {
        info!("Computing {} IBD pairs", group_pairs.len());

        // Get transposed matrix for adaptive approach
        let matrix_t = VISUALIZATION_CACHE
            .ibd_matrix_t
            .as_ref()
            .ok_or_else(|| ApiError::InternalServerError)?;

        let results: Vec<f32> = group_pairs
            .into_iter()
            .map(|(group_a, group_b)| {
                Self::compute_group_mean_adaptive(
                    &matrix,
                    &matrix_t,
                    &group_a.individuals,
                    &group_b.individuals,
                )
            })
            .collect();

        Ok(results)
    }

    /// Optimized blocking computation of IBD matrix using adaptive approach
    fn compute_matrix_blocking(
        groups_data: Vec<(String, usize, Vec<usize>)>,
        matrix: CsMat<f32>,
    ) -> Result<ComputedMatrix, ApiError> {
        info!(
            "Computing IBD matrix for {} groups using optimized approach",
            groups_data.len()
        );
        let start_time = std::time::Instant::now();

        // Get transposed matrix from cache for adaptive approach
        let matrix_t = VISUALIZATION_CACHE
            .ibd_matrix_t
            .as_ref()
            .ok_or_else(|| ApiError::InternalServerError)?;

        // Parallel computation of upper triangle + diagonal only
        let n_groups = groups_data.len();
        let upper_triangle: Vec<f32> = (0..n_groups)
            .into_par_iter()
            .flat_map(|i| (i..n_groups).into_par_iter().map(move |j| (i, j)))
            .map(|(i, j)| {
                let (_, _, individuals_a) = &groups_data[i];
                let (_, _, individuals_b) = &groups_data[j];

                // Use adaptive approach: choose matrix orientation based on group sizes
                Self::compute_group_mean_adaptive(&matrix, &matrix_t, individuals_a, individuals_b)
            })
            .collect();

        // Reconstruct full symmetric matrix from upper triangle
        let mut computed_matrix: Vec<Vec<f32>> = vec![vec![0.0; n_groups]; n_groups];
        let mut idx = 0;
        for i in 0..n_groups {
            for j in i..n_groups {
                let value = upper_triangle[idx];
                computed_matrix[i][j] = value;
                if i != j {
                    computed_matrix[j][i] = value;
                }
                idx += 1;
            }
        }

        let group_labels: Vec<String> = groups_data
            .iter()
            .map(|(label, _, _)| label.clone())
            .collect();

        let group_sizes: Vec<usize> = groups_data.iter().map(|(_, size, _)| *size).collect();

        let elapsed = start_time.elapsed();
        info!("Matrix computation completed in {:?}", elapsed);

        Ok(ComputedMatrix {
            matrix: computed_matrix,
            group_labels,
            group_sizes,
        })
    }

    /// Optimized computation of mean IBD between two groups using adaptive approach
    fn compute_group_mean_adaptive(
        matrix: &CsMat<f32>,
        matrix_t: &CsMat<f32>,
        group_a: &[usize],
        group_b: &[usize],
    ) -> f32 {
        // Choose matrix orientation based on which dimension has fewer elements
        if group_a.len() <= group_b.len() {
            // Fewer elements in group_a: use original matrix, iterate over group_a rows
            Self::compute_group_mean_rowwise(matrix, group_a, group_b)
        } else {
            // Fewer elements in group_b: use transposed matrix, iterate over group_b columns (now rows)
            Self::compute_group_mean_transposed(matrix_t, group_a, group_b)
        }
    }

    /// Row-wise computation using original matrix
    fn compute_group_mean_rowwise(matrix: &CsMat<f32>, rows: &[usize], cols: &[usize]) -> f32 {
        let col_set: HashSet<usize> = cols.iter().copied().collect();
        let mut sum = 0.0f64;

        for &row_idx in rows {
            if let Some(row) = matrix.outer_view(row_idx) {
                for (col_idx, &value) in row.iter() {
                    if col_set.contains(&col_idx) {
                        sum += value as f64;
                    }
                }
            }
        }

        // Count overlapping individuals (same-individual pairs that should be excluded from denominator)
        let row_set: HashSet<usize> = rows.iter().copied().collect();
        let overlapping_individuals = col_set.intersection(&row_set).count();
        
        // Divide by total possible pairs minus same-individual pairs (diagonal entries)
        let total_pairs = rows.len() * cols.len();
        let valid_pairs = total_pairs - overlapping_individuals;
        
        if valid_pairs > 0 {
            (sum / valid_pairs as f64) as f32
        } else {
            0.0
        }
    }

    /// Transposed matrix computation (group_b becomes rows, group_a becomes columns)
    fn compute_group_mean_transposed(
        matrix_t: &CsMat<f32>,
        group_a: &[usize],
        group_b: &[usize],
    ) -> f32 {
        let row_set: HashSet<usize> = group_a.iter().copied().collect();
        let mut sum = 0.0f64;

        for &col_idx in group_b {
            if let Some(col_as_row) = matrix_t.outer_view(col_idx) {
                for (row_idx, &value) in col_as_row.iter() {
                    if row_set.contains(&row_idx) {
                        sum += value as f64;
                    }
                }
            }
        }

        // Count overlapping individuals (same-individual pairs that should be excluded from denominator)
        let col_set: HashSet<usize> = group_b.iter().copied().collect();
        let overlapping_individuals = row_set.intersection(&col_set).count();
        
        // Divide by total possible pairs minus same-individual pairs (diagonal entries)
        let total_pairs = group_a.len() * group_b.len();
        let valid_pairs = total_pairs - overlapping_individuals;
        
        if valid_pairs > 0 {
            (sum / valid_pairs as f64) as f32
        } else {
            0.0
        }
    }

    /// Compute asymmetric IBD matrix between row groups and column groups
    pub async fn compute_asymmetric_group_ibd_matrix(
        row_groups: &[Group],
        column_groups: &[Group],
    ) -> Result<ComputedMatrix, ApiError> {
        if row_groups.is_empty() || column_groups.is_empty() {
            return Err(ApiError::ValidationError(
                "Both row and column groups must be provided".to_string(),
            ));
        }

        let matrix = VISUALIZATION_CACHE
            .ibd_matrix
            .as_ref()
            .ok_or_else(|| ApiError::InternalServerError)?;

        let n_row_groups = row_groups.len();
        let n_column_groups = column_groups.len();
        // Matrix structure: [row_groups][column_groups]
        let mut result_matrix: Vec<Vec<f32>> = vec![vec![0.0; n_column_groups]; n_row_groups];
        let mut pairs_to_compute: Vec<(usize, usize, String, String)> = Vec::new();
        let mut cache_hits = 0;

        // Check cache for each row-column pair
        for i in 0..n_row_groups {
            for j in 0..n_column_groups {
                let cache_key = Self::generate_pair_cache_key(
                    &row_groups[i].label,
                    &column_groups[j].label,
                );

                if let Some(cached_value) = VISUALIZATION_CACHE.pairwise_ibd_cache.get(&cache_key) {
                    result_matrix[i][j] = cached_value;
                    cache_hits += 1;
                } else {
                    // Need to compute this pair
                    pairs_to_compute.push((
                        i,
                        j,
                        row_groups[i].label.clone(),
                        column_groups[j].label.clone(),
                    ));
                }
            }
        }

        info!(
            "Asymmetric IBD matrix cache: {} hits, {} pairs to compute",
            cache_hits,
            pairs_to_compute.len()
        );

        // Compute missing pairs
        if !pairs_to_compute.is_empty() {
            // Clone data for spawn_blocking
            let groups_for_computation: Vec<_> = pairs_to_compute
                .iter()
                .map(|(i, j, _, _)| (row_groups[*i].clone(), column_groups[*j].clone()))
                .collect();
            let matrix_clone = matrix.clone();

            // Compute all missing pairs in parallel
            let computed_values = tokio::task::spawn_blocking(move || {
                Self::compute_pairs_blocking(groups_for_computation, matrix_clone)
            })
            .await
            .map_err(|e| {
                error!("Failed to spawn asymmetric pair computation task: {}", e);
                ApiError::InternalServerError
            })??;

            // Store computed values in result matrix and cache
            for ((i, j, label_row, label_column), computed_value) in
                pairs_to_compute.iter().zip(computed_values.iter())
            {
                result_matrix[*i][*j] = *computed_value;

                // Cache the computed pair value
                let cache_key = Self::generate_pair_cache_key(label_row, label_column);
                VISUALIZATION_CACHE.pairwise_ibd_cache.insert(cache_key, *computed_value);
            }

            info!(
                "Cached {} new asymmetric IBD pairs (cache size: {})",
                pairs_to_compute.len(),
                VISUALIZATION_CACHE.pairwise_ibd_cache.len()
            );
        }

        // Build final result - return column labels and sizes for consistency with frontend expectations
        let group_labels: Vec<String> = column_groups.iter().map(|g| g.label.clone()).collect();
        let group_sizes: Vec<usize> = column_groups.iter().map(|g| g.size).collect();

        Ok(ComputedMatrix {
            matrix: result_matrix,
            group_labels, // Column labels
            group_sizes,  // Column sizes
        })
    }
}

