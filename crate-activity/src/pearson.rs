crate::ix!();

pub fn pearson_correlation(x: &[i64], y: &[i64]) -> f64 {

    if x.len() != y.len() || x.is_empty() {
        return 0.0; // Handle mismatched or empty data
    }

    let n = x.len() as f64;

    // Convert to f64 early to prevent overflow or precision loss
    let sum_x: f64 = x.iter().map(|&xi| xi as f64).sum();
    let sum_y: f64 = y.iter().map(|&yi| yi as f64).sum();
    let sum_x_squared: f64 = x.iter().map(|&xi| (xi as f64).powi(2)).sum();
    let sum_y_squared: f64 = y.iter().map(|&yi| (yi as f64).powi(2)).sum();
    let sum_xy: f64 = x.iter().zip(y).map(|(&xi, &yi)| (xi as f64) * (yi as f64)).sum();

    let numerator = sum_xy - ((sum_x * sum_y) / n);
    let denominator = ((sum_x_squared - (sum_x.powi(2) / n)) * (sum_y_squared - (sum_y.powi(2) / n))).sqrt();

    if denominator == 0.0 {
        0.0 // No correlation if denominator is zero
    } else {
        numerator / denominator
    }
}

#[cfg(test)]
mod pearson_correlation_tests {
    use super::*;

    #[test]
    fn test_empty_inputs() {
        let x: Vec<i64> = vec![];
        let y: Vec<i64> = vec![];
        let result = pearson_correlation(&x, &y);
        assert_eq!(result, 0.0, "Empty inputs should return 0.0.");
    }

    #[test]
    fn test_mismatched_lengths() {
        let x = vec![1, 2, 3];
        let y = vec![1, 2];
        let result = pearson_correlation(&x, &y);
        assert_eq!(result, 0.0, "Mismatched lengths should return 0.0.");
    }

    #[test]
    fn test_all_zeros() {
        let x = vec![0, 0, 0];
        let y = vec![0, 0, 0];
        let result = pearson_correlation(&x, &y);
        assert_eq!(result, 0.0, "All zeros should return 0.0.");
    }

    #[test]
    fn test_perfect_positive_correlation() {
        let x = vec![1, 2, 3];
        let y = vec![2, 4, 6];
        let result = pearson_correlation(&x, &y);
        assert!((result - 1.0).abs() < 1e-9, "Perfect positive correlation should return 1.0.");
    }

    #[test]
    fn test_perfect_negative_correlation() {
        let x = vec![1, 2, 3];
        let y = vec![6, 4, 2];
        let result = pearson_correlation(&x, &y);
        assert!((result + 1.0).abs() < 1e-9, "Perfect negative correlation should return -1.0.");
    }

    #[test]
    fn test_no_correlation() {
        let x = vec![1, 2, 3];
        let y = vec![2, 2, 2];
        let result = pearson_correlation(&x, &y);
        assert_eq!(result, 0.0, "No correlation should return 0.0.");
    }

    #[test]
    fn test_single_element() {
        let x = vec![1];
        let y = vec![2];
        let result = pearson_correlation(&x, &y);
        assert_eq!(result, 0.0, "Single-element inputs should return 0.0.");
    }

    #[test]
    fn test_high_variance_with_noise() {
        let x = vec![1, 2, 3, 4, 5];
        let y = vec![10, 9, 8, 7, 6]; // Negative correlation with noise
        let result = pearson_correlation(&x, &y);
        assert!(result < 0.0, "Should return a negative correlation for this dataset.");
    }

    #[test]
    fn test_large_inputs() {
        let size = 1000; // A large dataset
        let x: Vec<i64> = (1..=size).collect();
        let y: Vec<i64> = (1..=size).collect();

        let result = pearson_correlation(&x, &y);
        assert!((result - 1.0).abs() < 1e-9, "Large identical ranges should have perfect correlation.");
    }

    #[test]
    fn test_mixed_positive_and_negative_values() {
        let x = vec![1, 2, 3, 4, 5];
        let y = vec![1, -2, 3, -4, 5];
        let result = pearson_correlation(&x, &y);
        assert!(result.abs() < 0.5, "Mixed positive and negative values should have low correlation.");
    }

    #[test]
    fn test_uniformly_spaced_values() {
        let x = vec![1, 3, 5, 7, 9];
        let y = vec![2, 4, 6, 8, 10];
        let result = pearson_correlation(&x, &y);
        assert!((result - 1.0).abs() < 1e-9, "Uniformly spaced values should have perfect positive correlation.");
    }

    #[test]
    fn test_precision_sensitivity() {
        let x = vec![1_000_000, 1_000_001, 1_000_002];
        let y = vec![2_000_000, 2_000_001, 2_000_002];
        let result = pearson_correlation(&x, &y);
        assert!((result - 1.0).abs() < 1e-9, "Close values should still yield perfect positive correlation.");
    }

    #[test]
    fn test_randomized_datasets() {
        use rand::Rng;

        let mut rng = rand::thread_rng();
        let x: Vec<i64> = (0..100).map(|_| rng.gen_range(0..1000)).collect();
        let y: Vec<i64> = x.iter().map(|&xi| xi + rng.gen_range(0..10)).collect();
        let result = pearson_correlation(&x, &y);
        assert!(result > 0.9, "Randomized correlated datasets should have high positive correlation.");
    }
}
