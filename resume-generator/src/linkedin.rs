crate::ix!();

#[derive(Debug)]
pub struct LinkedInInfo(String);

impl LinkedInInfo {
    pub fn new(linkedin: String) -> Self {
        Self(linkedin)
    }

    pub fn url(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for LinkedInInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
