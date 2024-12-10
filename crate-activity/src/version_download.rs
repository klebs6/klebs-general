crate::ix!();

#[derive(Builder,Getters,Setters,Clone,Debug,Serialize,Deserialize)]
#[builder(setter(into))]
pub struct VersionDownload {
    #[getset(get = "pub", set = "pub")] version:   i64,
    #[getset(get = "pub", set = "pub")] downloads: i64,
    #[getset(get = "pub", set = "pub")] date:      NaiveDate,
}
