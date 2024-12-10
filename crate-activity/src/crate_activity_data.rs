crate::ix!();

#[derive(Debug, Getters)]
pub struct CrateActivityData {

    #[getset(get = "pub")]
    summaries: Vec<CrateUsageSummary>,
    
    #[getset(get = "pub")]
    interval_downloads_1d: HashMap<String, i64>,
    
    #[getset(get = "pub")]
    interval_downloads_3d: HashMap<String, i64>,
}

pub async fn gather_crate_activity_data(
    crate_names:    &[String],
    user_agent:     &str,
    config_dir:     &Path,
    one_day_ago:    NaiveDate,
    three_days_ago: NaiveDate,

) -> Result<CrateActivityData, CrateActivityError> {

    let mut summaries             = Vec::new();
    let mut interval_downloads_1d = HashMap::new();
    let mut interval_downloads_3d = HashMap::new();

    for crate_name in crate_names {
        match fetch_usage(user_agent, config_dir, crate_name).await {
            Ok(Some(response)) => {
                let summary = analyze_usage(crate_name, response.version_downloads().to_vec());
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

                interval_downloads_1d.insert(crate_name.clone(), downloads_last_1d);
                interval_downloads_3d.insert(crate_name.clone(), downloads_last_3d);
            }
            Ok(None) => {
                eprintln!("No data for crate: {}", crate_name);
            }
            Err(e) => {
                eprintln!("Failed to fetch data for {}: {:?}", crate_name, e);
            }
        }
    }

    Ok(CrateActivityData {
        summaries,
        interval_downloads_1d,
        interval_downloads_3d,
    })
}
