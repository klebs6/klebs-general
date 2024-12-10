crate::ix!();

pub fn detect_outliers_zscore(values: &[i64], z_threshold: f64) -> Vec<bool> {
    if values.is_empty() {
        return Vec::new();
    }

    // Compute median
    let mut sorted = values.to_vec();
    sorted.sort();
    let median = if sorted.len() % 2 == 1 {
        sorted[sorted.len() / 2] as f64
    } else {
        let mid = sorted.len() / 2;
        (sorted[mid - 1] as f64 + sorted[mid] as f64) / 2.0
    };

    // Compute absolute deviations from median
    let abs_dev: Vec<f64> = values.iter().map(|&x| ((x as f64) - median).abs()).collect();

    // Compute MAD (median of absolute deviations)
    let mut abs_dev_sorted = abs_dev.clone();
    abs_dev_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mad = if abs_dev_sorted.len() % 2 == 1 {
        abs_dev_sorted[abs_dev_sorted.len() / 2]
    } else {
        let mid = abs_dev_sorted.len() / 2;
        (abs_dev_sorted[mid - 1] + abs_dev_sorted[mid]) / 2.0
    };

    // If MAD is tiny (all values identical), no outliers
    if mad < 1e-12 {
        return vec![false; values.len()];
    }

    // Compute MAD-based z-score and detect outliers
    let c = 0.6745; // scaling constant
    values.iter().map(|&x| {
        let z = c * ((x as f64) - median) / mad;
        z.abs() > z_threshold
    }).collect()
}

pub fn remove_outliers(values: &[i64], outliers: &[bool]) -> Vec<i64> {
    values.iter()
        .zip(outliers.iter())
        .filter_map(|(&val, &is_outlier)| if !is_outlier { Some(val) } else { None })
        .collect()
}

pub fn downweight_outliers(values: &[i64], outliers: &[bool], weight: f64) -> Vec<f64> {
    values.iter()
        .zip(outliers.iter())
        .map(|(&val, &is_outlier)| {
            if is_outlier {
                (val as f64) * weight
            } else {
                val as f64
            }
        }).collect()
}

#[cfg(test)]
mod outlier_detection_tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        let values: Vec<i64> = Vec::new();
        let outliers = detect_outliers_zscore(&values, 3.0);
        assert!(outliers.is_empty(), "No data means no outliers.");
    }

    #[test]
    fn test_no_outliers_uniform_data() {
        let values = vec![100,100,100,100,100];
        let outliers = detect_outliers_zscore(&values, 3.0);
        assert_eq!(outliers, vec![false;5], "All identical data means no outliers.");
    }

    #[test]
    fn test_simple_outlier_detection() {
        // Data mostly around 100, but one huge spike at 10000
        let values = vec![100,101,99,102,98,10000];
        let outliers = detect_outliers_zscore(&values, 3.0);
        assert_eq!(outliers.len(), 6);
        assert!(outliers[5], "The spike at 10000 should be detected as outlier.");
        for i in 0..5 {
            assert!(!outliers[i], "Normal values near median not outliers.");
        }
    }

    #[test]
    fn test_remove_outliers() {
        let values = vec![10,20,30,10000,40,50];
        let outliers = detect_outliers_zscore(&values, 3.0);
        let cleaned = remove_outliers(&values, &outliers);
        // Expect to remove the huge spike at 10000
        assert!(!cleaned.contains(&10000), "Should have removed the outlier.");
        assert_eq!(cleaned, vec![10,20,30,40,50]);
    }

    #[test]
    fn test_downweight_outliers() {
        let values = vec![10,20,30,5000,40,50];
        let outliers = detect_outliers_zscore(&values, 3.0);
        let adjusted = downweight_outliers(&values, &outliers, 0.1);

        let outlier_indices: Vec<_> = outliers.iter().enumerate().filter_map(|(i,&o)| if o {Some(i)} else {None}).collect();
        assert_eq!(outlier_indices.len(), 1, "Should detect exactly one outlier");
        let idx = outlier_indices[0];
        assert!((adjusted[idx] - 500.0).abs()<1e-9, "Outlier should be down-weighted by factor 0.1");

        for (i, &val) in values.iter().enumerate() {
            if i != idx {
                assert_eq!(adjusted[i], val as f64, "Non-outliers should remain unchanged.");
            }
        }
    }

    #[test]
    fn test_threshold_sensitivity() {
        let values = vec![100,102,98,95,105];
        let outliers_strict = detect_outliers_zscore(&values, 1.0);
        let outliers_loose = detect_outliers_zscore(&values, 3.0);

        let strict_count = outliers_strict.iter().filter(|&&o| o).count();
        let loose_count = outliers_loose.iter().filter(|&&o| o).count();
        assert!(strict_count >= loose_count, "Stricter threshold should produce more or equal outliers.");
    }
}
