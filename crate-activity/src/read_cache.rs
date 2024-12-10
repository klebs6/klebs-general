crate::ix!();

// Cache the response to a file
pub async fn cache_response(config_dir: &Path, crate_name: &str, date: NaiveDate, response: &CrateResponse) -> io::Result<()> {
    let cache_dir = config_dir.join("cache");
    fs::create_dir_all(&cache_dir).await?;

    let cache_file = cache_dir.join(format!("{}_{}.json", crate_name, date));
    let json = serde_json::to_string(response)?; // Serialize to string
    fs::write(cache_file, json).await // Write as bytes
}

// Load a cached response if available
pub async fn load_cached_response(config_dir: &Path, crate_name: &str, date: NaiveDate) -> Option<CrateResponse> {
    let cache_file = config_dir.join("cache").join(format!("{}_{}.json", crate_name, date));

    if cache_file.exists() {
        if let Ok(json) = fs::read_to_string(&cache_file).await {
            if let Ok(response) = serde_json::from_str::<CrateResponse>(&json) {
                return Some(response);
            }
        }
    }
    None
}
