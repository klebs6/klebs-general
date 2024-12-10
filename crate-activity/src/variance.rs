crate::ix!();

pub fn has_significant_variance(data: &[i64]) -> bool {
    let mean = data.iter().sum::<i64>() as f64 / data.len() as f64;
    let variance = data.iter().map(|&x| (x as f64 - mean).powi(2)).sum::<f64>() / data.len() as f64;

    variance > 1e-5 // Use a small threshold to filter out near-constant datasets
}

#[cfg(test)]
mod check_for_significant_variance_tests {
    use super::*;

    #[test]
    fn test_empty_data() {
        let data = [];
        assert!(!has_significant_variance(&data), "Empty data should have no significant variance.");
    }

    #[test]
    fn test_single_element() {
        let data = [10];
        assert!(!has_significant_variance(&data), "Single element should have no significant variance.");
    }

    #[test]
    fn test_identical_elements() {
        let data = [5, 5, 5, 5];
        assert!(!has_significant_variance(&data), "Identical elements should have no significant variance.");
    }

    #[test]
    fn test_high_variance() {
        let data = [1, 10, 100, 1000];
        assert!(has_significant_variance(&data), "Data with high variance should be detected.");
    }

    #[test]
    fn test_low_variance() {
        let data = [10, 10, 10, 11];
        assert!(has_significant_variance(&data), "Data with low variance should be detected.");
    }

    #[test]
    fn test_boundary_case() {
        let data = [10, 10, 10, 10_000];
        assert!(has_significant_variance(&data), "Data at the boundary of the threshold should be detected.");
    }
}
