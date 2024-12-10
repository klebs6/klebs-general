crate::ix!();

#[derive(Getters,Setters,Debug, Serialize,Deserialize)]
pub struct CrateResponse {
    #[getset(get = "pub", set = "pub")] version_downloads: Vec<VersionDownload>,
}
