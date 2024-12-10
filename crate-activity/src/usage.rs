crate::ix!();

#[derive(Clone,Debug,Serialize,Deserialize)]
pub enum DownloadTrend {
    Increasing,
    Decreasing,
    Stable,
}

#[derive(Clone,Builder,Getters,Setters,Debug,Serialize,Deserialize)]
#[builder(setter(into))]
pub struct CrateUsageSummary {
    #[getset(get = "pub", set = "pub")] crate_name:              String,
    #[getset(get = "pub", set = "pub")] total_downloads:         i64,
    #[getset(get = "pub", set = "pub")] average_daily_downloads: f64,
    #[getset(get = "pub", set = "pub")] peak_daily_downloads:    i64,
    #[getset(get = "pub", set = "pub")] download_trend:          Option<DownloadTrend>,
    #[getset(get = "pub", set = "pub")] version_downloads:       Vec<VersionDownload>, // Add this
}
