crate::ix!();

/// Represents the location of a Rust crate in
/// various scenarios.
///
/// This enum can handle crates that are vendored,
/// in the current workspace, or identified by
/// a direct filesystem path.
///
#[derive(Debug)]
pub enum CrateLocation<'a> {

    /// Represents a vendored crate with a given
    /// name.
    ///
    Vendored {
        crate_name: &'a str,
    },

    /// Represents a crate within the current
    /// workspace with a given name.
    ///
    InCurrentWorkspace {
        crate_name: &'a str,
    },

    /// Represents a crate identified by the
    /// absolute path to its Cargo.toml.
    ///
    DirectPath {
        absolute_path_to_cargo_toml: String,
    }
}

impl<'a> CrateLocation<'a> {

    /// Returns the name of the crate.
    ///
    /// This function extracts the crate name
    /// based on the variant of `CrateLocation`.
    ///
    pub fn name(&self) -> String {

        match self {
            CrateLocation::Vendored           { crate_name } => crate_name.to_string(),
            CrateLocation::InCurrentWorkspace { crate_name } => crate_name.to_string(),

            CrateLocation::DirectPath         { absolute_path_to_cargo_toml } => {
                get_crate_name_from_cargo_toml(&absolute_path_to_cargo_toml).unwrap()
            },
        }
    }

    /// Returns the root directory of the crate.
    ///
    /// The returned path depends on the variant
    /// of `CrateLocation`
    ///
    pub fn root(&self) -> String {

        match self {
            CrateLocation::Vendored           { crate_name } => format!("vendor/{}", crate_name),
            CrateLocation::InCurrentWorkspace { crate_name } => format!("{}", crate_name),

            CrateLocation::DirectPath         { absolute_path_to_cargo_toml } => {
                get_parent_directory_from_cargo_toml_path(&absolute_path_to_cargo_toml).unwrap()
            },
        }
    }

    /// Retrieves the contents of all source files
    /// in the crate.
    ///
    /// This function walks the directory tree
    /// starting from the crate's root and reads
    /// the content of all `.rs` files.
    ///
    pub fn all_source_file_contents(&self) -> Vec<String> {

        let root = self.root();

        let mut contents = Vec::new();
        let mut errors   = Vec::new();

        let src_path = Path::new(&root).join("src");

        for entry in WalkDir::new(&src_path) {

            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("Error reading directory entry: {}", e);
                    continue;
                },
            };

            let path = entry.path();

            // Check for Rust source files
            if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                // Read the file contents
                match std::fs::read_to_string(path) {
                    Ok(content) => contents.push(content),
                    Err(e) => errors.push((path.to_path_buf(), e)),
                }
            }
        }

        // Handle errors
        for (path, e) in errors {
            eprintln!("Failed to read file {}: {}", path.display(), e);
        }

        contents
    }
}
