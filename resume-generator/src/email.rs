crate::ix!();

#[derive(Debug)]
pub struct Email(String);

impl Email {
    pub fn new(email: String) -> Self {
        Self(email)
    }

    pub fn address(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
