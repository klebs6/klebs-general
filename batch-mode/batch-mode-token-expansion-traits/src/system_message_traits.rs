crate::ix!();

/// A trait that describes the system message goal for a given axis set.
/// This is typically implemented for the entire enum and returns a single,
/// static system message goal string for the entire type.
pub trait SystemMessageGoal {
    /// Returns the system message goal associated with the entire enum.
    fn system_message_goal(&self) -> Cow<'_,str>;
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct SystemMessageHeader {
    content: String,
}

impl SystemMessageHeader {

    pub fn new(x: &str) -> Self { Self { content: x.to_string() } }

    pub fn get(&self) -> &str {
        &self.content
    }
}

impl std::fmt::Display for SystemMessageHeader {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get())
    }
}
