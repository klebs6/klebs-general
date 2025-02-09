// ---------------- [ File: workspacer-interface/src/errors.rs ]
crate::ix!();

error_tree!{

    #[derive(Clone)]
    pub enum CargoTomlWriteError {
        WriteWorkspaceHeaderError {
            io: Arc<io::Error>,
        },
        OpenWorkspaceMembersFieldError {
            io: Arc<io::Error>,
        },
        WritePackageSectionError {
            io: Arc<io::Error>,
        },
        WriteWorkspaceMember {
            io: Arc<io::Error>,
        },
        CloseWorkspaceMembersFieldError {
            io: Arc<io::Error>,
        },
        WriteError {
            io: Arc<io::Error>,
        },
    }

    #[derive(Clone)]
    pub enum CargoTomlError {
        ReadError {
            io: Arc<io::Error>,
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

    #[derive(Clone)]
    pub enum TestFailure {
        UnknownError {
            stdout: Option<String>,
            stderr: Option<String>,
        }
    }

    #[derive(Clone)]
    pub enum TokioError {
        JoinError {
            join_error: Arc<tokio::task::JoinError>,
        },
    }

    #[derive(Clone)]
    pub enum DirectoryError {
        CreateDirAllError {
            io: Arc<io::Error>,
        },
        ReadDirError {
            io: Arc<io::Error>,
        },
        GetNextEntryError {
            io: Arc<io::Error>,
        },
    }

    #[derive(Clone)]
    pub enum ReadmeWriteError {
        WriteBlankReadmeError {
            io: Arc<io::Error>,
        },
    }

    #[derive(Clone)]
    pub enum CrateWriteError {
        ReadmeWriteError(ReadmeWriteError),
        WriteDummyMainError {
            io: Arc<io::Error>,
        },
        WriteDummyTestError {
            io: Arc<io::Error>,
        },
        WriteLibRsFileError {
            io: Arc<io::Error>,
        },
        WriteMainFnError {
            io: Arc<io::Error>,
        },
    }

    #[derive(Clone)]
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
            io: Arc<io::Error>,
        },
    }

    #[derive(Clone)]
    pub enum CargoDocError {
        CommandError {
            io: Arc<io::Error>,
        },
        UnknownError {
            stdout: Option<String>,
            stderr: Option<String>,
        }
    }

    #[derive(Clone)]
    pub enum LintingError {
        CommandError {
            io: Arc<io::Error>,
        },
        UnknownError {
            stdout: Option<String>,
            stderr: Option<String>,
        }
    }

    #[derive(Clone)]
    pub enum CargoMetadataError {
        MetadataError {
            error: Arc<cargo_metadata::Error>,
        },
        CircularDependency,
        CyclicPackageDependency,
    }

    #[derive(Clone)]
    pub enum BuildError {
        CommandError {
            io: Arc<io::Error>,
        },
        BuildFailed {
            stderr: String,
        },
    }
}

error_tree!{

    #[derive(Clone)]
    pub enum CrateError {
        // Error indicating that a file was not found.
        FileNotFound {
            missing_file: PathBuf,
        },

        // Error indicating that a directory was not found.
        DirectoryNotFound {
            missing_directory: PathBuf,
        },

        FailedToGetFileNameForPath {
            path: PathBuf,
        },

        DirectoryError(DirectoryError),
        InvalidCargoToml(CargoTomlError),

        IoError {
            io_error: Arc<io::Error>,
        },
    }
}

error_tree!{

    #[derive(Clone)]
    pub enum WatchError {
        NotifyError(Arc<notify::Error>),
        IoError {
            io: Arc<io::Error>,
        },
        ChannelRecvError(std::sync::mpsc::RecvError),
    }

    // Enum representing possible errors in the `workspace-detail` crate.
    #[derive(Clone)]
    pub enum WorkspaceError {
        CrateError(CrateError),

        TokioJoinError {
            join_error: Arc<tokio::task::JoinError>,
        },
        IoError {
            io_error: Arc<io::Error>,
        },

        // Error indicating that a file was not found.
        FileNotFound {
            missing_file: PathBuf,
        },

        InvalidWorkspace {
            invalid_workspace_path: PathBuf,
        },

        CargoDocError(CargoDocError),
        CrateWriteError(CrateWriteError),
        ReadmeWriteError(ReadmeWriteError),
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
        CargoTomlWriteError(CargoTomlWriteError),
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
        TestTimeout,
    }
}

impl From<tokio::task::JoinError> for WorkspaceError {
    fn from(join_error: tokio::task::JoinError) -> Self {
        WorkspaceError::TokioJoinError {
            join_error: Arc::new(join_error),
        }
    }
}

impl From<io::Error> for WorkspaceError {
    fn from(io_error: io::Error) -> Self {
        WorkspaceError::IoError {
            io_error: Arc::new(io_error),
        }
    }
}

impl fmt::Display for WorkspaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl CargoMetadataError {

    pub fn is_cyclic_package_dependency_error(&self) -> bool {
        self.to_string().contains("cyclic package dependency")
    }
}

impl fmt::Display for CargoMetadataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}
