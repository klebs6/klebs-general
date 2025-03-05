// ---------------- [ File: workspacer-crate/src/mock_crate_config.rs ]
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

#[cfg(test)]
mod test_crate_config {
    use super::*;

    /// Demonstrates that a new `CrateConfig` initialized with a given name
    /// has default values for readme, src files, and test files set to false.
    #[test]
    fn test_new_crate_config_defaults() {
        let config = CrateConfig::new("example_crate");

        // Verify name
        assert_eq!(config.name(), "example_crate", "Crate name should match constructor input");

        // Verify defaults
        assert!(!config.has_readme(), "Expected no README by default");
        assert!(!config.has_src_files(), "Expected no src files by default");
        assert!(!config.has_test_files(), "Expected no test files by default");
    }

    /// Demonstrates that calling `with_readme`, `with_src_files`, or `with_test_files`
    /// modifies the config accordingly.
    #[test]
    fn test_crate_config_with_readme_src_and_tests() {
        let config = CrateConfig::new("my_crate")
            .with_readme()
            .with_src_files()
            .with_test_files();

        // All flags are now true
        assert!(config.has_readme(), "Expected README to be set");
        assert!(config.has_src_files(), "Expected src files to be set");
        assert!(config.has_test_files(), "Expected test files to be set");
    }

    /// Demonstrates partial usage: e.g., setting only README and src files but not test files.
    #[test]
    fn test_crate_config_partial_flags() {
        let config = CrateConfig::new("partial_crate")
            .with_readme()
            .with_src_files();
        // We did not call `.with_test_files()`

        assert!(config.has_readme(), "Expected README to be set");
        assert!(config.has_src_files(), "Expected src files to be set");
        assert!(!config.has_test_files(), "Did not set test files, should remain false");
    }

    /// Demonstrates that the name is unaffected by enabling readme/src/tests.
    #[test]
    fn test_name_is_unaffected_by_flags() {
        let original_name = "unchanged_crate_name";
        let config = CrateConfig::new(original_name)
            .with_readme()
            .with_test_files();

        assert_eq!(config.name(), original_name, "Name should remain unchanged");
    }

    /// Demonstrates that the user can check the underlying booleans
    /// via both the builder methods and the getters `add_readme()`, `add_src_files()`, etc.
    #[test]
    fn test_getter_vs_builder_fields() {
        let config = CrateConfig::new("getter_vs_builder")
            .with_readme()
            .with_src_files();

        assert!(config.has_readme(), "has_readme() should match the builder");
        assert!(config.add_readme(), "add_readme() raw field should also be true");
        assert!(config.add_src_files(), "add_src_files() raw field should also be true");
        assert!(!config.add_test_files(), "didn't enable test files, should be false");
    }
}
