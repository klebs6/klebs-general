crate::ix!();

#[derive(Debug)]
pub struct GitHubInfo(String);

impl GitHubInfo {
    pub fn new(github: String) -> Self {
        Self(github)
    }

    pub fn url(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for GitHubInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
