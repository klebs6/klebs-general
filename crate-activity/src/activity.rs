crate::ix!();

pub async fn crate_activity_main(cli: &CrateActivityCli) -> Result<(), CrateActivityError> {

    tracing_setup::configure_tracing();

    let config_dir   = configure_directory().await?;
    let crate_names  = read_crate_list(&config_dir).await;
    let user_agent   = read_user_agent(&config_dir).await;
    let ignore_cache = *cli.ignore_cache();

    let today = Utc::now().date_naive();
    let one_day_ago    = today - chrono::Duration::days(1);
    let three_days_ago = today - chrono::Duration::days(3);
    let seven_days_ago = today - chrono::Duration::days(7);

    let activity_data = gather_crate_activity_data(
        ignore_cache,
        &crate_names,
        &user_agent,
        &config_dir,
        one_day_ago,
        three_days_ago,
        seven_days_ago,
    )
    .await?;

    let activity_summary = CrateActivitySummary::new(
        activity_data.summaries(),
        activity_data.interval_downloads_1d().clone(),
        activity_data.interval_downloads_3d().clone(),
        activity_data.interval_downloads_7d().clone(),
        one_day_ago,
        three_days_ago,
        seven_days_ago,
    );

    tracing::info!("{}", activity_summary);

    let cleaned_summaries = if cli.disable_outlier_handling() {
        tracing::info!("Outlier detection disabled. Using raw data.");
        activity_data.summaries().to_vec()
    } else {
        let z_threshold = *cli.outlier_z_threshold();
        let downweight = *cli.downweight_outliers();
        let weight = *cli.outlier_weight();

        activity_data.summaries().iter().map(|s| {
            let downloads: Vec<i64> = s.version_downloads().iter().map(|d| *d.downloads()).collect();

            let outliers = detect_outliers_zscore(&downloads, z_threshold);
            let outlier_count = outliers.iter().filter(|&&o| o).count();

            if outlier_count > 0 {
                if downweight {
                    tracing::info!(
                        "Crate '{}' had {} outliers (z-threshold={:.2}); downweighting by {:.2}",
                        s.crate_name(),
                        outlier_count,
                        z_threshold,
                        weight
                    );
                } else {
                    tracing::info!(
                        "Crate '{}' had {} outliers (z-threshold={:.2}); removing them.",
                        s.crate_name(),
                        outlier_count,
                        z_threshold
                    );
                }
            } else {
                tracing::info!(
                    "Crate '{}' had no outliers detected (z-threshold={:.2}).",
                    s.crate_name(),
                    z_threshold
                );
            }

            let cleaned_version_downloads: Vec<VersionDownload> = if downweight {
                let adjusted = downweight_outliers(&downloads, &outliers, weight);
                s.version_downloads()
                 .iter()
                 .zip(adjusted.iter())
                 .map(|(vd, &val)| {
                     let adjusted_val = val.round() as i64;
                     VersionDownloadBuilder::default()
                         .version(*vd.version())
                         .downloads(adjusted_val)
                         .date(*vd.date())
                         .build()
                         .unwrap()
                 })
                 .collect()
            } else {
                s.version_downloads()
                 .iter()
                 .zip(outliers.iter())
                 .filter_map(|(vd, &is_outlier)| if !is_outlier { Some(vd.clone()) } else { None })
                 .collect()
            };

            let total: i64 = cleaned_version_downloads.iter().map(|d| d.downloads()).sum();
            let count = cleaned_version_downloads.len().max(1) as f64;
            let avg = total as f64 / count;
            let peak = cleaned_version_downloads.iter().map(|d| d.downloads()).max().cloned().unwrap_or(0);

            CrateUsageSummaryBuilder::default()
                .crate_name(s.crate_name().to_string())
                .total_downloads(total)
                .average_daily_downloads(avg)
                .peak_daily_downloads(peak)
                .download_trend(s.download_trend().clone())
                .version_downloads(cleaned_version_downloads)
                .build()
                .expect("Failed to build cleaned CrateUsageSummary")
        }).collect()
    };

    // Compute correlations on (possibly cleaned) summaries
    let correlations = compute_pairwise_correlations(&cleaned_summaries);
    if *cli.show_correlations() {
        display_correlations(&correlations);
    }

    if *cli.perform_pca() {
        let crate_activity: HashMap<_, _> = cleaned_summaries.iter().map(|summary| {
            (
                summary.crate_name().clone(),
                summary.version_downloads().iter().map(|d| *d.downloads()).collect(),
            )
        }).collect();

        perform_pca(&crate_activity)?;
    }

    if *cli.perform_hierarchical_clustering() {
        let dendrogram = perform_hierarchical_clustering(&correlations)?;
        display_dendrogram(&dendrogram);
    }

    if *cli.correlation_network() {
        let threshold = *cli.network_threshold();
        let graph = build_correlation_graph(&correlations, threshold);

        if *cli.print_summary() {
            display_graph_summary(&graph);
        }

        if let Some(target) = cli.girvan_newman() {
            let communities = girvan_newman_communities(graph.clone(), *target);
            display_network_communities(&communities);
        } else {
            let communities = find_communities(&graph);
            display_network_communities(&communities);
        }

        if *cli.compute_betweenness() {
            let (node_bet, _edge_bet) = compute_betweenness_centrality(&graph);
            display_top_betweenness_nodes(&node_bet, 10);
        }
    }

    if *cli.time_lag_correlations() {
        let max_lag = *cli.max_lag();
        let lag_results = compute_time_lag_correlations(&cleaned_summaries, max_lag);
        display_time_lag_correlations(&lag_results);
    }

    tracing::info!("Crate usage analysis completed.");

    Ok(())
}
