crate::ix!();

error_tree!{

    pub enum CargoTomlError {
        MissingRequiredFieldForPublishing {
            cargo_toml_file: PathBuf,
            field: String,
        },
        MissingRequiredFieldForIntegrity {
            cargo_toml_file: PathBuf,
            field: String,
        },
        InvalidVersionFormat {
            cargo_toml_file: PathBuf,
            version: String,
        },
        MissingPackageSection {
            cargo_toml_file: PathBuf,
        },
        TomlParseError {
            cargo_toml_file: PathBuf,
            toml_parse_error: toml::de::Error,
        },
        IoError(io::Error),
        
        // Error indicating that a file was not found.
        FileNotFound {
            missing_file: PathBuf,
        },
    }

    // Enum representing possible errors in the `workspace-detail` crate.
    pub enum WorkspaceError {
        MultipleErrors(Vec<WorkspaceError>),

        // Error indicating that a directory was not found.
        DirectoryNotFound {
            missing_directory: PathBuf,
        },

        InvalidCargoToml(CargoTomlError),

        // Error indicating that a file was not found.
        FileNotFound {
            missing_file: PathBuf,
        },

        InvalidWorkspace {
            invalid_workspace_path: PathBuf,
        },

        WorkspaceNotReadyForCargoPublish,

        DirectoryRemovalError,
        FileRemovalError,
        CargoMetadataError(String),

        FailedToGetFileNameForPath {
            path: PathBuf,
        }
    }
}
