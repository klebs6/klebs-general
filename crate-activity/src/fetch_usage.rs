crate::ix!();

// Fetch usage from the API or cache
pub async fn fetch_usage(ignore_cache: bool, user_agent: &str, config_dir: &Path, crate_name: &str) 
    -> Result<Option<CrateResponse>, reqwest::Error> 
{
    let today = Utc::now().date_naive();

    if !ignore_cache {
        if let Some(cached) = load_cached_response(config_dir,crate_name, today).await {
            println!("Loaded cached data for {}", crate_name);
            return Ok(Some(cached));
        }
    }

    let url = format!("https://crates.io/api/v1/crates/{}/downloads", crate_name);
    let client = Client::new();

    let response = client
        .get(&url)
        .header("User-Agent", user_agent)
        .send()
        .await?;

    if response.status().is_success() {
        let body = response.text().await?;
        if let Ok(usage) = serde_json::from_str::<CrateResponse>(&body) {
            cache_response(&config_dir,crate_name, today, &usage).await.ok();
            return Ok(Some(usage));
        }
    }
    Ok(None)
}
