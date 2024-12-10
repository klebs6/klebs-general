crate::ix!();

pub fn perform_pca(crate_activity: &HashMap<String, Vec<i64>>) -> Result<(), PcaError> {
    tracing::info!("Starting PCA analysis on crate activity data...");

    // Prepare the data matrix
    let data_matrix = prepare_data_matrix(crate_activity)?;

    // Standardize the matrix
    let standardized_matrix = standardize_matrix(&data_matrix)?;

    // Compute the covariance matrix
    let covariance_matrix = compute_covariance_matrix(&standardized_matrix);

    // Convert to nalgebra matrix for eigen decomposition
    let covariance_dmatrix = DMatrix::from_row_slice(
        covariance_matrix.nrows(),
        covariance_matrix.ncols(),
        covariance_matrix.as_slice().unwrap(),
    );

    // Perform eigen decomposition
    let eigen = covariance_dmatrix.symmetric_eigen();
    let eigenvalues = eigen.eigenvalues.as_slice().to_vec();
    let eigenvectors_flat = eigen.eigenvectors.as_slice().to_vec();

    let eigenvectors = Array2::from_shape_vec(
        (eigen.eigenvalues.len(), eigen.eigenvalues.len()),
        eigenvectors_flat,
    )
    .unwrap();

    // Display results
    display_pca_results(crate_activity.keys().cloned().collect(), eigenvalues, eigenvectors);

    Ok(())
}

fn prepare_data_matrix(crate_activity: &std::collections::HashMap<String, Vec<i64>>)
    -> Result<ndarray::Array2<f64>, PcaError> 
{
    let num_days = crate_activity.values().map(|v| v.len()).max().unwrap_or(0);
    if num_days == 0 {
        return Err(PcaError::NoActivityDataAvailable);
    }

    // Collect and sort crate names to ensure stable ordering
    let mut crate_names: Vec<_> = crate_activity.keys().cloned().collect();
    crate_names.sort();

    let num_crates = crate_activity.len();
    let mut data = Vec::with_capacity(num_crates * num_days);

    // Fill rows in alphabetical order or in the order the tests expect
    for crate_name in &crate_names {
        let crate_data = &crate_activity[crate_name];
        // Pad with zeros if shorter than num_days
        for &value in crate_data.iter().chain(std::iter::repeat(&0).take(num_days - crate_data.len())) {
            data.push(value as f64);
        }
    }

    ndarray::Array2::from_shape_vec((num_crates, num_days), data)
        .map_err(|_| PcaError::PcaDataLengthMismatch {
            expected_num_elements: num_crates * num_days,
            found_num_elements: num_crates * num_days, // data.len() should be correct here
        })
}

fn perform_eigen_decomposition(covariance_matrix: &Array2<f64>) -> (Vec<f64>, Array2<f64>) {
    let n = covariance_matrix.nrows(); // Assuming square matrix
    let covariance_dmatrix = DMatrix::from_row_slice(
        n,
        n,
        covariance_matrix.as_slice().unwrap(),
    );
    let eigen = covariance_dmatrix.symmetric_eigen();

    // Convert eigenvalues to Vec<f64>
    let eigenvalues: Vec<f64> = eigen.eigenvalues.as_slice().to_vec();

    // Convert eigenvectors to Array2<f64>
    let eigenvectors = Array2::from_shape_vec((n, n), eigen.eigenvectors.as_slice().to_vec())
        .expect("Failed to convert eigenvectors to Array2");

    (eigenvalues, eigenvectors)
}

fn standardize_matrix(matrix: &ndarray::Array2<f64>) -> Result<ndarray::Array2<f64>, PcaError> {
    let mut standardized_matrix = matrix.clone();
    for mut column in standardized_matrix.columns_mut() {
        let mean = column.mean().unwrap_or(0.0);
        let std_dev = column.std(0.0);

        if std_dev.abs() < 1e-12 {
            // Constant column: set all values to zero.
            for val in column.iter_mut() {
                *val = 0.0;
            }
        } else {
            column.mapv_inplace(|x| (x - mean) / std_dev);
        }
    }
    Ok(standardized_matrix)
}

fn compute_covariance_matrix(matrix: &Array2<f64>) -> Array2<f64> {
    matrix.t().dot(matrix) / (matrix.nrows() as f64)
}

fn display_pca_results(crate_names: Vec<String>, eigenvalues: Vec<f64>, eigenvectors: Array2<f64>) {
    println!("Explained variance by significant principal components:");

    let total_variance: f64 = eigenvalues.iter().sum();
    let significant_components: Vec<_> = eigenvalues
        .iter()
        .enumerate()
        .filter(|(_, &eigenvalue)| (eigenvalue / total_variance) * 100.0 >= 3.0)
        .take(10)
        .collect();

    for (i, &eigenvalue) in &significant_components {
        println!(
            "Component {}: {:.2}% of total variance",
            i + 1,
            (eigenvalue / total_variance) * 100.0
        );
    }

    println!("\nTop contributing crates to significant principal components:");
    for (i, &eigenvalue) in &significant_components {
        let component = eigenvectors.column(*i); // Deref i here
        let mut contributions: Vec<(String, f64)> = crate_names
            .iter()
            .zip(component.iter())
            .map(|(crate_name, &weight)| (crate_name.clone(), weight.abs()))
            .collect();
        contributions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        println!("Component {}: Top crates", i + 1);
        for (crate_name, weight) in contributions.iter().take(5) {
            println!("  {:>6.2}  {}", weight, crate_name);
        }
    }
}

#[cfg(test)]
mod pca_tests {
    use super::*; // Adjust as needed if PCA code is in another module.
    use std::collections::HashMap;
    use ndarray::array;

    // Helper function to create crate activity data from a slice of tuples.
    // (crate_name, downloads) pairs.
    fn create_crate_activity_data(data: &[(&str, &[i64])]) -> HashMap<String, Vec<i64>> {
        let mut map = HashMap::new();
        for (name, downloads) in data.iter() {
            map.insert((*name).to_string(), downloads.to_vec());
        }
        map
    }

    #[test]
    fn test_display_pca_results() {
        // This function primarily prints output. We'll just ensure it doesn't panic.
        let eigenvalues = vec![3.0, 2.0, 1.0];
        let eigenvectors = array![
            [0.577350269, 0.707106781, 0.408248290],
            [0.577350269, 0.0,          0.816496580],
            [0.577350269, -0.707106781, 0.408248290]
        ];
        let crate_names = vec!["crateA".to_string(), "crateB".to_string(), "crateC".to_string()];

        // Just verify that the function runs without error.
        // We cannot check stdout easily here, but at least no panic should occur.
        display_pca_results(crate_names, eigenvalues, eigenvectors);
    }

    // Ensure no panics on empty input
    #[test]
    fn test_prepare_data_matrix_empty_input() {
        let crate_activity: HashMap<String, Vec<i64>> = HashMap::new();
        let result = prepare_data_matrix(&crate_activity);
        assert!(matches!(result, Err(PcaError::NoActivityDataAvailable)));
    }

    // Test varying lengths with deterministic ordering
    #[test]
    fn test_prepare_data_matrix_varying_lengths() {
        let crate_activity = create_crate_activity_data(&[
            ("crateA", &[100, 200, 300]),
            ("crateB", &[50, 75]),
        ]);

        // This test expects alphabetical order: crateA then crateB
        // crateA row => [100, 200, 300]
        // crateB row => [50, 75, 0] (padded)
        if let Ok(matrix) = prepare_data_matrix(&crate_activity) {
            assert_eq!(matrix.nrows(), 2);
            assert_eq!(matrix.ncols(), 3);

            // Check crateA row
            assert_eq!(matrix[[0,0]], 100.0);
            assert_eq!(matrix[[0,1]], 200.0);
            assert_eq!(matrix[[0,2]], 300.0);

            // Check crateB row
            assert_eq!(matrix[[1,0]], 50.0);
            assert_eq!(matrix[[1,1]], 75.0);
            assert_eq!(matrix[[1,2]], 0.0);
        } else {
            panic!("Expected success for prepared data matrix with varying lengths.");
        }
    }

    // Test uniform length data
    #[test]
    fn test_prepare_data_matrix_uniform_length() {
        let crate_activity = create_crate_activity_data(&[
            ("crateA", &[1, 2, 3, 4]),
            ("crateB", &[10, 20, 30, 40]),
        ]);

        // Alphabetical: crateA, crateB
        // crateA => [1,  2,  3,  4]
        // crateB => [10, 20, 30, 40]
        if let Ok(matrix) = prepare_data_matrix(&crate_activity) {
            assert_eq!(matrix.nrows(), 2);
            assert_eq!(matrix.ncols(), 4);

            // crateA row
            assert_eq!(matrix[[0,0]], 1.0);
            assert_eq!(matrix[[0,1]], 2.0);
            assert_eq!(matrix[[0,2]], 3.0);
            assert_eq!(matrix[[0,3]], 4.0);

            // crateB row
            assert_eq!(matrix[[1,0]], 10.0);
            assert_eq!(matrix[[1,1]], 20.0);
            assert_eq!(matrix[[1,2]], 30.0);
            assert_eq!(matrix[[1,3]], 40.0);
        } else {
            panic!("Expected success for uniform length data.");
        }
    }

    // Test a single crate scenario
    #[test]
    fn test_prepare_data_matrix_single_crate() {
        let crate_activity = create_crate_activity_data(&[
            ("onlyCrate", &[42, 42, 42]),
        ]);
        if let Ok(matrix) = prepare_data_matrix(&crate_activity) {
            assert_eq!(matrix.nrows(), 1);
            assert_eq!(matrix.ncols(), 3);
            assert_eq!(matrix[[0,0]], 42.0);
            assert_eq!(matrix[[0,1]], 42.0);
            assert_eq!(matrix[[0,2]], 42.0);
        } else {
            panic!("Expected success with single crate input.");
        }
    }

    // Test standardization with a basic matrix
    #[test]
    fn test_standardize_matrix_basic() {
        let matrix = array![
            [1.0,  2.0,  3.0],
            [2.0,  2.0,  2.0],
            [10.0, 11.0, 12.0]
        ];
        if let Ok(std_matrix) = standardize_matrix(&matrix) {
            // Check column means ~0 and std dev ~1 (where possible)
            for col in 0..std_matrix.ncols() {
                let column = std_matrix.column(col);
                let mean = column.mean().unwrap_or(0.0);
                let std_dev = column.std(0.0);
                assert!((mean - 0.0).abs() < 1e-9);
                if std_dev > 1e-9 {
                    assert!((std_dev - 1.0).abs() < 1e-9);
                }
            }
        } else {
            panic!("Expected successful standardization.");
        }
    }

    // Test standardization on a constant column
    #[test]
    fn test_standardize_matrix_constant_column() {
        let matrix = array![
            [5.0, 1.0, 2.0],
            [5.0, 2.0, 3.0],
            [5.0, 3.0, 4.0]
        ];
        if let Ok(std_matrix) = standardize_matrix(&matrix) {
            let col0 = std_matrix.column(0);
            for val in col0 {
                assert!((val - 0.0).abs() < 1e-9, "Expected zero column for constant input.");
            }
        } else {
            panic!("Expected success with a constant column.");
        }
    }

    // Test standardization on negative and mixed values
    #[test]
    fn test_standardize_matrix_negative_values() {
        let matrix = array![
            [-10.0, 0.0,  10.0],
            [-20.0, 0.0,  20.0],
            [-30.0, 0.0,  30.0],
        ];
        if let Ok(std_matrix) = standardize_matrix(&matrix) {
            // The middle column is all zeros (constant), should become zero column
            let col1 = std_matrix.column(1);
            for val in col1 {
                assert!((val - 0.0).abs() < 1e-9);
            }

            // The other columns have linear increasing/decreasing patterns.
            // Just verify mean ~0 and std ~1.
            for col_idx in [0,2].iter() {
                let col = std_matrix.column(*col_idx);
                let mean = col.mean().unwrap_or(0.0);
                let std_dev = col.std(0.0);
                assert!((mean - 0.0).abs() < 1e-9);
                assert!((std_dev - 1.0).abs() < 1e-9);
            }
        } else {
            panic!("Expected successful standardization with negative values.");
        }
    }

    // Test covariance matrix computation
    #[test]
    fn test_compute_covariance_matrix() {
        let matrix = array![
            [1.0, 2.0, 3.0],
            [2.0, 2.0, 2.0],
            [10.0,11.0,12.0]
        ];
        let cov = compute_covariance_matrix(&matrix);
        // Check symmetry
        for i in 0..cov.nrows() {
            for j in 0..cov.ncols() {
                let diff = (cov[[i,j]] - cov[[j,i]]).abs();
                assert!(diff < 1e-12, "Covariance matrix not symmetric.");
            }
        }
    }

    // Test eigen decomposition on a simple diagonal matrix
    #[test]
    fn test_perform_eigen_decomposition() {
        let matrix = array![
            [2.0, 0.0],
            [0.0, 1.0]
        ];
        let (vals, vecs) = perform_eigen_decomposition(&matrix);
        let mut sorted_vals = vals.clone();
        sorted_vals.sort_by(|a,b| a.partial_cmp(b).unwrap());
        assert!((sorted_vals[0]-1.0).abs()<1e-9);
        assert!((sorted_vals[1]-2.0).abs()<1e-9);
        assert_eq!(vecs.nrows(), 2);
        assert_eq!(vecs.ncols(), 2);
    }

    // Test PCA with no data
    #[test]
    fn test_perform_pca_no_data() {
        let crate_activity = HashMap::new();
        let result = perform_pca(&crate_activity);
        assert!(matches!(result, Err(PcaError::NoActivityDataAvailable)));
    }

    // Test basic PCA
    #[test]
    fn test_perform_pca_basic() {
        let crate_activity = create_crate_activity_data(&[
            ("crateA", &[1,2,3]),
            ("crateB", &[2,4,6]),
        ]);
        let result = perform_pca(&crate_activity);
        if let Err(e) = result {
            panic!("Expected PCA success, got: {:?}", e);
        }
    }

    // Test PCA on identical crates
    #[test]
    fn test_pca_with_identical_crates() {
        let crate_activity = create_crate_activity_data(&[
            ("crate1", &[10, 20, 30, 40]),
            ("crate2", &[10, 20, 30, 40]),
        ]);
        let result = perform_pca(&crate_activity);
        if let Err(e) = result {
            panic!("Expected success with identical crates: {:?}", e);
        }
    }

    // Test PCA with zero variance data
    #[test]
    fn test_pca_with_zero_variance_data() {
        let crate_activity = create_crate_activity_data(&[
            ("constantCrate", &[100,100,100]),
            ("anotherConstantCrate", &[100,100,100]),
        ]);
        let result = perform_pca(&crate_activity);
        if let Err(e) = result {
            panic!("Expected PCA success with zero variance data: {:?}", e);
        }
    }

    // Test PCA on different lengths
    #[test]
    fn test_pca_different_lengths() {
        let crate_activity = create_crate_activity_data(&[
            ("crateShort", &[1,2]),
            ("crateLong", &[5,10,15,20]),
        ]);
        let result = perform_pca(&crate_activity);
        if let Err(e) = result {
            panic!("Expected PCA success with different lengths: {:?}", e);
        }
    }

    // Test PCA on large random data
    #[test]
    fn test_pca_large_random_data() {
        let mut large_data = HashMap::new();
        let days = 500;
        for i in 0..50 {
            let crate_name = format!("crate{}", i);
            let values: Vec<i64> = (0..days).map(|d| d as i64 * (i as i64 + 1)).collect();
            large_data.insert(crate_name, values);
        }
        let result = perform_pca(&large_data);
        if let Err(e) = result {
            panic!("Expected PCA success with large data: {:?}", e);
        }
    }

    // Additional edge case: random negative and positive, varying sizes
    #[test]
    fn test_pca_with_mixed_random_data() {
        let crate_activity = create_crate_activity_data(&[
            ("alpha", &[-1, -2, -3, -4, -5]),
            ("beta",  &[5, 4, 3, 2, 1]),
            ("gamma", &[0, 10, 20, 30, 40])
        ]);

        let result = perform_pca(&crate_activity);
        if let Err(e) = result {
            panic!("Expected PCA success with mixed random data: {:?}", e);
        }
    }

    // Test stable ordering: we rely on alphabetical sorting in prepare_data_matrix
    #[test]
    fn test_pca_stable_ordering() {
        let crate_activity = create_crate_activity_data(&[
            ("zCrate", &[3,3,3]),
            ("aCrate", &[1,1,1]),
            ("mCrate", &[2,2,2]),
        ]);
        // After sorting: aCrate, mCrate, zCrate
        // Check ordering by analyzing the resulting matrix directly.
        if let Ok(matrix) = prepare_data_matrix(&crate_activity) {
            // aCrate row => [1,1,1]
            // mCrate row => [2,2,2]
            // zCrate row => [3,3,3]
            assert_eq!(matrix[[0,0]], 1.0);
            assert_eq!(matrix[[1,0]], 2.0);
            assert_eq!(matrix[[2,0]], 3.0);
        } else {
            panic!("Expected success for stable ordering test.");
        }
    }
}
