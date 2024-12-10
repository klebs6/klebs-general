crate::ix!();

error_tree!{
    pub enum CrateActivityError {
        Reqwest(reqwest::Error),
        Serde(serde_json::Error),
        Io(std::io::Error),
    }
}
