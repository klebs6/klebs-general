crate::ix!();

#[derive(Debug, Getters)]
pub struct CrateActivityData {

    #[getset(get = "pub")]
    summaries: Vec<CrateUsageSummary>,
    
    #[getset(get = "pub")]
    interval_downloads_1d: HashMap<String, i64>,
    
    #[getset(get = "pub")]
    interval_downloads_3d: HashMap<String, i64>,

    #[getset(get = "pub")]
    interval_downloads_7d: HashMap<String, i64>,
}

#[tracing::instrument(level = "info", skip_all)]
pub async fn gather_crate_activity_data(
    ignore_cache:   bool,
    crate_names:    &[String],
    user_agent:     &str,
    config_dir:     &Path,
    one_day_ago:    NaiveDate,
    three_days_ago: NaiveDate,
    seven_days_ago: NaiveDate,
) -> Result<CrateActivityData, CrateActivityError> {
    use futures::{StreamExt};

    tracing::info!(
        "Gathering crate activity data for {} crates (ignore_cache={})",
        crate_names.len(),
        ignore_cache
    );

    // We'll limit concurrency to avoid overwhelming crates.io.
    let concurrency_limit = 8usize;

    // Create a stream of futures (one for each crate).
    let crate_fetches = futures::stream::iter(crate_names.iter().map(|crate_name| {
        let crate_name = crate_name.clone();
        let ua = user_agent.to_string();
        let cfg_dir = config_dir.to_path_buf();
        async move {
            tracing::debug!("Fetching usage for crate '{}'", crate_name);
            match fetch_usage(ignore_cache, &ua, &cfg_dir, &crate_name).await {
                Ok(Some(response)) => {
                    tracing::info!("Successfully fetched usage for crate '{}'", crate_name);
                    Some((crate_name, response))
                },
                Ok(None) => {
                    tracing::warn!("No data for crate '{}'", crate_name);
                    None
                },
                Err(e) => {
                    tracing::error!("Failed to fetch data for '{}': {:?}", crate_name, e);
                    None
                }
            }
        }
    }))
    .buffer_unordered(concurrency_limit);

    // Collect all results in parallel.
    let results: Vec<Option<(String, CrateResponse)>> = crate_fetches.collect().await;

    let mut summaries             = Vec::new();
    let mut interval_downloads_1d = HashMap::new();
    let mut interval_downloads_3d = HashMap::new();
    let mut interval_downloads_7d = HashMap::new();

    // Process the completed fetches.
    for item in results {
        if let Some((crate_name, response)) = item {
            let summary = analyze_usage(&crate_name, response.version_downloads().to_vec());
            summaries.push(summary);

            let downloads_last_1d: i64 = response
                .version_downloads()
                .iter()
                .filter(|d| *d.date() >= one_day_ago)
                .map(|d| d.downloads())
                .sum();

            let downloads_last_3d: i64 = response
                .version_downloads()
                .iter()
                .filter(|d| *d.date() >= three_days_ago)
                .map(|d| d.downloads())
                .sum();

            let downloads_last_7d: i64 = response
                .version_downloads()
                .iter()
                .filter(|d| *d.date() >= seven_days_ago)
                .map(|d| d.downloads())
                .sum();

            interval_downloads_1d.insert(crate_name.clone(), downloads_last_1d);
            interval_downloads_3d.insert(crate_name.clone(), downloads_last_3d);
            interval_downloads_7d.insert(crate_name.clone(), downloads_last_7d);
        }
    }

    tracing::info!(
        "Collected activity data for {} crates (out of {} requested).",
        summaries.len(),
        crate_names.len()
    );

    Ok(CrateActivityData {
        summaries,
        interval_downloads_1d,
        interval_downloads_3d,
        interval_downloads_7d,
    })
}
