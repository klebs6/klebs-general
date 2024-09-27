crate::ix!();

/// Configuration for creating a mock crate in the workspace.
#[derive(Debug)]
pub struct CrateConfig {
    name:           String,
    add_readme:     bool,
    add_src_files:  bool,
    add_test_files: bool,
}

impl CrateConfig {

    pub fn add_readme(&self) -> bool {
        self.add_readme
    }

    pub fn add_src_files(&self) -> bool {
        self.add_src_files
    }

    pub fn add_test_files(&self) -> bool {
        self.add_test_files
    }

    /// Create a new CrateConfig with the given crate name.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            add_readme: false,
            add_src_files: false,
            add_test_files: false,
        }
    }

    /// Set whether to add a README.md file.
    pub fn with_readme(mut self) -> Self {
        self.add_readme = true;
        self
    }

    /// Set whether to add source files in the src/ directory.
    pub fn with_src_files(mut self) -> Self {
        self.add_src_files = true;
        self
    }

    /// Set whether to add test files in the tests/ directory.
    pub fn with_test_files(mut self) -> Self {
        self.add_test_files = true;
        self
    }

    /// Accessor to get the crate name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Accessor to check if README.md is to be added.
    pub fn has_readme(&self) -> bool {
        self.add_readme
    }

    /// Accessor to check if src/ files are to be added.
    pub fn has_src_files(&self) -> bool {
        self.add_src_files
    }

    /// Accessor to check if test files are to be added.
    pub fn has_test_files(&self) -> bool {
        self.add_test_files
    }
}
