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
    /// Compute mean pairwise IBD matrix between groups using spawn_blocking
    pub async fn compute_group_ibd_matrix(groups: &[Group]) -> Result<ComputedMatrix, ApiError> {
        if groups.is_empty() {
            return Err(ApiError::ValidationError("No groups provided".to_string()));
        }

        let matrix = VISUALIZATION_CACHE.ibd_matrix.as_ref()
            .ok_or_else(|| ApiError::InternalServerError)?;

        // Clone data needed for computation
        let groups_data: Vec<(String, usize, Vec<usize>)> = groups.iter()
            .map(|g| (g.label.clone(), g.size, g.individuals.clone()))
            .collect();
        
        let matrix_clone = matrix.clone();

        // Use spawn_blocking to avoid blocking the async runtime
        let result = tokio::task::spawn_blocking(move || {
            Self::compute_matrix_blocking(groups_data, matrix_clone)
        }).await
        .map_err(|e| {
            error!("Failed to spawn matrix computation task: {}", e);
            ApiError::InternalServerError
        })?;

        result
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