// IBD matrix computation and aggregation functions

use sprs::CsMat;
use rayon::prelude::*;
use std::collections::HashSet;
use tracing::{info, error};

use crate::api::ApiError;
use super::{VISUALIZATION_CACHE, ComputedMatrix, Group};

/// IBD matrix computation algorithms
pub struct IbdComputation;

impl IbdComputation {
    /// Compute mean pairwise IBD matrix between groups using spawn_blocking with pairwise caching
    pub async fn compute_group_ibd_matrix(groups: &[Group]) -> Result<ComputedMatrix, ApiError> {
        if groups.is_empty() {
            return Err(ApiError::ValidationError("No groups provided".to_string()));
        }

        let matrix = VISUALIZATION_CACHE.ibd_matrix.as_ref()
            .ok_or_else(|| ApiError::InternalServerError)?;

        let n_groups = groups.len();
        let mut result_matrix: Vec<Vec<f32>> = vec![vec![0.0; n_groups]; n_groups];
        let mut pairs_to_compute: Vec<(usize, usize, String, String)> = Vec::new();
        let mut cache_hits = 0;
        
        // Check cache for each pair and identify what needs to be computed
        {
            let cache = VISUALIZATION_CACHE.computed_matrices.read()
                .map_err(|_| ApiError::InternalServerError)?;
            
            for i in 0..n_groups {
                for j in i..n_groups { // Only compute upper triangle + diagonal
                    let cache_key = Self::generate_pair_cache_key(&groups[i].label, &groups[j].label);
                    
                    if let Some(cached_matrix) = cache.get(&cache_key) {
                        // Extract the single value from cached 1x1 or 2x2 matrix
                        let cached_value = if i == j {
                            cached_matrix.matrix[0][0] // Intra-group (1x1 matrix)
                        } else {
                            cached_matrix.matrix[0][1] // Inter-group (off-diagonal of 2x2 matrix)
                        };
                        
                        result_matrix[i][j] = cached_value;
                        if i != j {
                            result_matrix[j][i] = cached_value; // Symmetric
                        }
                        cache_hits += 1;
                    } else {
                        // Need to compute this pair
                        pairs_to_compute.push((i, j, groups[i].label.clone(), groups[j].label.clone()));
                    }
                }
            }
        }
        
        info!("IBD matrix cache: {} hits, {} pairs to compute", cache_hits, pairs_to_compute.len());
        
        // Compute missing pairs
        if !pairs_to_compute.is_empty() {
            // Clone data for spawn_blocking
            let groups_for_computation: Vec<_> = pairs_to_compute.iter()
                .map(|(i, j, _, _)| (groups[*i].clone(), groups[*j].clone()))
                .collect();
            let matrix_clone = matrix.clone();
            
            // Compute all missing pairs in parallel
            let computed_values = tokio::task::spawn_blocking(move || {
                Self::compute_pairs_blocking(groups_for_computation, matrix_clone)
            }).await
            .map_err(|e| {
                error!("Failed to spawn pair computation task: {}", e);
                ApiError::InternalServerError
            })??;
            
            // Store computed values in result matrix and cache
            {
                let mut cache = VISUALIZATION_CACHE.computed_matrices.write()
                    .map_err(|_| ApiError::InternalServerError)?;
                
                for ((i, j, label_i, label_j), computed_value) in pairs_to_compute.iter().zip(computed_values.iter()) {
                    result_matrix[*i][*j] = *computed_value;
                    if i != j {
                        result_matrix[*j][*i] = *computed_value; // Symmetric
                    }
                    
                    // Cache the computed pair
                    let cache_key = Self::generate_pair_cache_key(label_i, label_j);
                    let cached_matrix = if i == j {
                        // Intra-group: 1x1 matrix
                        ComputedMatrix {
                            matrix: vec![vec![*computed_value]],
                            group_labels: vec![label_i.clone()],
                            group_sizes: vec![groups[*i].size],
                        }
                    } else {
                        // Inter-group: 2x2 symmetric matrix
                        ComputedMatrix {
                            matrix: vec![
                                vec![0.0, *computed_value],
                                vec![*computed_value, 0.0]
                            ],
                            group_labels: vec![label_i.clone(), label_j.clone()],
                            group_sizes: vec![groups[*i].size, groups[*j].size],
                        }
                    };
                    
                    cache.insert(cache_key, cached_matrix);
                }
                
                info!("Cached {} new IBD pairs (cache size: {})", pairs_to_compute.len(), cache.len());
            }
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
        matrix: CsMat<f32>
    ) -> Result<Vec<f32>, ApiError> {
        info!("Computing {} IBD pairs", group_pairs.len());
        
        // Get transposed matrix for adaptive approach
        let matrix_t = VISUALIZATION_CACHE.ibd_matrix_t.as_ref()
            .ok_or_else(|| ApiError::InternalServerError)?;
        
        let results: Vec<f32> = group_pairs.into_iter()
            .map(|(group_a, group_b)| {
                Self::compute_group_mean_adaptive(&matrix, &matrix_t, &group_a.individuals, &group_b.individuals)
            })
            .collect();
        
        Ok(results)
    }

    /// Optimized blocking computation of IBD matrix using adaptive approach
    fn compute_matrix_blocking(
        groups_data: Vec<(String, usize, Vec<usize>)>,
        matrix: CsMat<f32>
    ) -> Result<ComputedMatrix, ApiError> {
        info!("Computing IBD matrix for {} groups using optimized approach", groups_data.len());
        let start_time = std::time::Instant::now();

        // Get transposed matrix from cache for adaptive approach
        let matrix_t = VISUALIZATION_CACHE.ibd_matrix_t.as_ref()
            .ok_or_else(|| ApiError::InternalServerError)?;

        // Parallel computation of upper triangle + diagonal only
        let n_groups = groups_data.len();
        let upper_triangle: Vec<f32> = (0..n_groups).into_par_iter()
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

        let group_labels: Vec<String> = groups_data.iter()
            .map(|(label, _, _)| label.clone())
            .collect();
        
        let group_sizes: Vec<usize> = groups_data.iter()
            .map(|(_, size, _)| *size)
            .collect();

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
    fn compute_group_mean_rowwise(
        matrix: &CsMat<f32>,
        rows: &[usize],
        cols: &[usize],
    ) -> f32 {
        let col_set: HashSet<usize> = cols.iter().copied().collect();
        let mut sum = 0.0f64;

        // Debug: Sample some actual matrix values
        if rows.len() > 0 && cols.len() > 0 {
            let sample_row = rows[0];
            let sample_col = cols.iter().next().copied().unwrap_or(0);
            if let Some(sample_value) = matrix.get(sample_row, sample_col) {
                tracing::info!("Sample IBD matrix lookup: matrix[{},{}] = {}", sample_row, sample_col, sample_value);
            }
        }

        for &row_idx in rows {
            if let Some(row) = matrix.outer_view(row_idx) {
                for (col_idx, &value) in row.iter() {
                    if col_set.contains(&col_idx) {
                        sum += value as f64;
                    }
                }
            }
        }

        // Divide by total possible pairs (including zeros), not just non-zero pairs
        let total_pairs = rows.len() * cols.len();
        if total_pairs > 0 {
            (sum / total_pairs as f64) as f32
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

        // Divide by total possible pairs (including zeros), not just non-zero pairs
        let total_pairs = group_a.len() * group_b.len();
        if total_pairs > 0 {
            (sum / total_pairs as f64) as f32
        } else {
            0.0
        }
    }
}