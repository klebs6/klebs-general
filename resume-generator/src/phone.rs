crate::ix!();

#[derive(Debug)]
pub struct PhoneNumber(String);

impl PhoneNumber {
    pub fn new(phone: String) -> Self {
        Self(phone)
    }

    pub fn number(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PhoneNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
