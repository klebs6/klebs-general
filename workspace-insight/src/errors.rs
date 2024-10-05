crate::ix!();

error_tree!{

    pub enum TestFailure {
        UnknownError {
            stdout: Option<String>,
            stderr: Option<String>,
        }
    }

    pub enum TokioError {
        JoinError {
            join_error: tokio::task::JoinError,
        },
    }

    pub enum DirectoryError {
        CreateDirAllError {
            io: io::Error,
        },
        ReadDirError {
            io: io::Error,
        },
        GetNextEntryError {
            io: io::Error,
        },
    }

    pub enum FileError {
        CreationError {
            io: io::Error,
        },
        WriteError {
            io: io::Error,
        },
        GetMetadataError {
            io: io::Error,
        },
        OpenError {
            io: io::Error,
        },
        GetNextLineError {
            io: io::Error,
        },
    }

    pub enum ReadmeWriteError {
        WriteBlankReadmeError {
            io: io::Error,
        },
    }

    pub enum CrateWriteError {
        ReadmeWriteError(ReadmeWriteError),
        WriteDummyMainError {
            io: io::Error,
        },
        WriteDummyTestError {
            io: io::Error,
        },
        WriteLibRsFileError {
            io: io::Error,
        },
        WriteMainFnError {
            io: io::Error,
        },
    }

    pub enum CargoTomlWriteError {
        WriteWorkspaceHeaderError {
            io: io::Error,
        },
        OpenWorkspaceMembersFieldError {
            io: io::Error,
        },
        WritePackageSectionError {
            io: io::Error,
        },
        WriteWorkspaceMember {
            io: io::Error,
        },
        CloseWorkspaceMembersFieldError {
            io: io::Error,
        },
        WriteError {
            io: io::Error,
        },
    }

    pub enum CargoTomlError {
        ReadError {
            io: io::Error,
        },
        CargoTomlWriteError(CargoTomlWriteError),
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
        
        // Error indicating that a file was not found.
        FileNotFound {
            missing_file: PathBuf,
        },
    }

    pub enum TestCoverageError {
        TestFailure {
            stdout: Option<String>,
            stderr: Option<String>,
        },
        UnknownError {
            stdout: Option<String>,
            stderr: Option<String>,
        },
        CoverageParseError,
        CommandError {
            io: io::Error,
        },
    }

    pub enum CargoDocError {
        CommandError {
            io: io::Error,
        },
        UnknownError {
            stdout: Option<String>,
            stderr: Option<String>,
        }
    }

    pub enum LintingError {
        CommandError {
            io: io::Error,
        },
        UnknownError {
            stdout: Option<String>,
            stderr: Option<String>,
        }
    }

    pub enum CargoMetadataError {
        MetadataError {
            error: cargo_metadata::Error,
        },
        CircularDependency,
    }

    pub enum WatchError {
        NotifyError(notify::Error),
        IoError {
            io: io::Error,
        },
        ChannelRecvError(std::sync::mpsc::RecvError),
    }

    pub enum BuildError {
        CommandError {
            io: io::Error,
        },
        BuildFailed {
            stderr: String,
        },
    }

    // Enum representing possible errors in the `workspace-detail` crate.
    pub enum WorkspaceError {

        // Error indicating that a directory was not found.
        DirectoryNotFound {
            missing_directory: PathBuf,
        },

        // Error indicating that a file was not found.
        FileNotFound {
            missing_file: PathBuf,
        },

        InvalidWorkspace {
            invalid_workspace_path: PathBuf,
        },

        FailedToGetFileNameForPath {
            path: PathBuf,
        },

        CargoDocError(CargoDocError),
        CrateWriteError(CrateWriteError),
        TokioError(TokioError),
        CargoMetadataError(CargoMetadataError),
        CircularDependency {
            // To store the detected cycles
            detected_cycles: Vec<Vec<String>>,
        },  
        CoverageParseError,
        DirectoryRemovalError,
        FileRemovalError,
        InvalidCargoToml(CargoTomlError),
        LintingError(LintingError),
        MultipleErrors(Vec<WorkspaceError>),
        TestCoverageError(TestCoverageError),
        TestFailure(TestFailure),
        WatchError(WatchError),
        BuildError(BuildError),
        FileError(FileError),
        DirectoryError(DirectoryError),
        WorkspaceNotReadyForCargoPublish,
        FileWatchError,
    }
}

impl CargoMetadataError {

    pub fn is_cyclic_package_dependency_error(&self) -> bool {
        self.to_string().contains("cyclic package dependency")
    }
}

impl fmt::Display for WorkspaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl fmt::Display for CargoMetadataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}
