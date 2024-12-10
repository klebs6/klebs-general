crate::ix!();

pub fn analyze_usage(crate_name: &str, version_downloads: Vec<VersionDownload>) -> CrateUsageSummary {

    // Aggregate downloads by date
    let mut daily_downloads: HashMap<NaiveDate, i64> = HashMap::new();

    for download in version_downloads.iter() {
        *daily_downloads.entry(*download.date()).or_insert(0) += download.downloads();
    }

    let total_downloads: i64 = daily_downloads.values().sum();
    let average_daily_downloads = total_downloads as f64 / daily_downloads.len() as f64;
    let peak_daily_downloads = *daily_downloads.values().max().unwrap_or(&0);

    // Calculate trend (simple: increasing, decreasing, or stable)
    let mut trend = None;
    let mut diffs = Vec::new();
    let mut sorted_days: Vec<_> = daily_downloads.into_iter().collect();
    sorted_days.sort_by_key(|&(date, _)| date);

    for i in 1..sorted_days.len() {
        diffs.push(sorted_days[i].1 as i64 - sorted_days[i - 1].1 as i64);
    }

    if !diffs.is_empty() {
        let positive_diffs = diffs.iter().filter(|&&d| d > 0).count();
        let negative_diffs = diffs.iter().filter(|&&d| d < 0).count();

        trend = if positive_diffs > negative_diffs {
            Some(DownloadTrend::Increasing)
        } else if negative_diffs > positive_diffs {
            Some(DownloadTrend::Decreasing)
        } else {
            Some(DownloadTrend::Stable)
        };
    }

    // Use the builder to construct the summary
    CrateUsageSummaryBuilder::default()
        .crate_name(crate_name.to_string())
        .total_downloads(total_downloads)
        .average_daily_downloads(average_daily_downloads)
        .peak_daily_downloads(peak_daily_downloads)
        .download_trend(trend)
        .version_downloads(version_downloads)
        .build()
        .expect("Failed to build CrateUsageSummary") // Handle errors from builder
}
