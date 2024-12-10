crate::ix!();

pub fn compute_pairwise_correlations(
    summaries: &[CrateUsageSummary],
) -> Vec<(String, String, f64)> {
    let mut correlations = Vec::new();

    for i in 0..summaries.len() {
        for j in (i + 1)..summaries.len() {
            let crate_a = summaries[i].crate_name();
            let crate_b = summaries[j].crate_name();

            let downloads_a = summaries[i].version_downloads();
            let downloads_b = summaries[j].version_downloads();

            // Extract and intersect date ranges
            let dates_a: Vec<_> = downloads_a.iter().map(|d| *d.date()).collect();
            let dates_b: Vec<_> = downloads_b.iter().map(|d| *d.date()).collect();
            let common_dates = intersect_date_ranges(&dates_a, &dates_b);

            // Align data to the common date range
            let aligned_a = align_and_normalize_data(downloads_a, &common_dates);
            let aligned_b = align_and_normalize_data(downloads_b, &common_dates);

            // Skip if either dataset lacks variance
            if !has_significant_variance(&aligned_a) || !has_significant_variance(&aligned_b) {
                continue;
            }

            // Compute correlation
            let correlation = pearson_correlation(&aligned_a, &aligned_b);
            correlations.push((crate_a.clone(), crate_b.clone(), correlation));
        }
    }

    correlations
}

pub fn debug_correlation(crate_a: &str, crate_b: &str, correlation: f64) {
    println!("Correlation for {} and {}: {:.4}", crate_a, crate_b, correlation);
}
