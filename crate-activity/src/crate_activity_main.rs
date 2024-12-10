crate::ix!();

pub async fn crate_activity_main() -> Result<(),CrateActivityError> {

    tracing_setup::configure_tracing();

    // Configuration directory
    let config_dir = dirs::home_dir().map(|p| p.join(".published-crates")).unwrap_or_else(|| PathBuf::from(".published-crates"));

    ensure_config_structure(&config_dir).await?;

    // Read crate list and user agent
    let crate_names = read_crate_list(&config_dir).await;
    let user_agent  = read_user_agent(&config_dir).await;

    let mut summaries = Vec::new();
    let mut interval_downloads_1d = HashMap::new();
    let mut interval_downloads_3d = HashMap::new();

    let today          = Utc::now().date_naive();
    let one_day_ago    = today - chrono::Duration::days(1);
    let three_days_ago = today - chrono::Duration::days(3);

    for crate_name in crate_names {
        match fetch_usage(&user_agent,&config_dir,&crate_name).await {
            Ok(response) => {
                if let Some(response) = response {
                    let summary = analyze_usage(&crate_name, response.version_downloads().to_vec());
                    summaries.push(summary);

                    // Calculate downloads for the last 1 and 3 days
                    let downloads_last_1d: u64 = response.version_downloads()
                        .iter()
                        .filter(|d| *d.date() >= one_day_ago)
                        .map(|d| d.downloads())
                        .sum();

                    let downloads_last_3d: u64 = response.version_downloads()
                        .iter()
                        .filter(|d| *d.date() >= three_days_ago)
                        .map(|d| d.downloads())
                        .sum();

                    interval_downloads_1d.insert(crate_name.clone(), downloads_last_1d);
                    interval_downloads_3d.insert(crate_name.clone(), downloads_last_3d);
                }
            }
            Err(e) => eprintln!("Failed to fetch data for {}: {:?}", crate_name, e),
        }
    }

    let activity_summary = CrateActivitySummary::new(
        &summaries,
        interval_downloads_1d,
        interval_downloads_3d,
        one_day_ago,
        three_days_ago,
    );

    // Debug print the summary
    println!("{}", activity_summary);

    tracing::info!("Crate usage analysis completed.");

    Ok(())
}
