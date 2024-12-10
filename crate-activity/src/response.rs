crate::ix!();

#[derive(Getters,Setters,Clone,Debug,Serialize,Deserialize)]
pub struct VersionDownload {
    #[getset(get = "pub", set = "pub")] version:   u64,
    #[getset(get = "pub", set = "pub")] downloads: u64,
    #[getset(get = "pub", set = "pub")] date:      NaiveDate,
}

#[derive(Getters,Setters,Debug, Serialize,Deserialize)]
pub struct CrateResponse {
    #[getset(get = "pub", set = "pub")] version_downloads: Vec<VersionDownload>,
}
