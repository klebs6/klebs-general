crate::ix!();

//------------------------------------------------------------------
// Example errors for the "workspace-bump" crate
//------------------------------------------------------------------
error_tree!{
    pub enum BumpVersionsError {
        // I/O error reading or writing Cargo.toml
        IoError {
            context: String,
            io_error: Arc<std::io::Error>,
        },
        // Could not parse the Cargo.toml
        CargoTomlParseError {
            path: PathBuf,
            details: String,
        },
        // Missing [package] or version field
        MissingPackageSection {
            path: PathBuf,
        },
        // The version string was invalid semver
        InvalidVersion {
            path:        PathBuf,
            version_str: String,
            details:     String,
        },
        // We tried to bump a crate that doesn't exist
        NoSuchCrate {
            crate_name: String,
        },
    }
}
