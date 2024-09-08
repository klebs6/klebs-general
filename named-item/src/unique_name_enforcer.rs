crate::ix!();

/// Trait for ensuring unique names across a collection or context.
pub struct UniqueNameEnforcer {
    names: HashSet<String>,
}

impl UniqueNameEnforcer {
    pub fn new() -> Self {
        Self {
            names: HashSet::new(),
        }
    }

    /// Adds a name to the unique set. Returns an error if the name already exists.
    pub fn add_unique_name(&mut self, name: &str) -> Result<(), NameError> {
        if self.names.contains(name) {
            Err(NameError::DuplicateName(name.to_string()))
        } else {
            self.names.insert(name.to_string());
            Ok(())
        }
    }
}
