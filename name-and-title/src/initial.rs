crate::ix!();

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct MiddleInitial(char);

impl From<char> for MiddleInitial {
    fn from(x: char) -> MiddleInitial {
        MiddleInitial(x)
    }
}

impl fmt::Display for MiddleInitial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char(self.0)
    }
}
