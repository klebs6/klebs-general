// ---------------- [ File: workspacer-errors/src/errors.rs ]
crate::ix!();

error_tree!{

    #[derive(Clone)]
    #[allow(non_camel_case_types)]
    pub enum SourceFileRegistrationError {
        EncounteredAnXMacroAfterWeAlreadySawANonAttributeItem {
            file_path:      PathBuf,
            offending_item: String,
        },
        CrateError(CrateError),
        LibRsSyntaxErrors { 
            parse_errors: Vec<String> 
        },
        LibRsParseTreeError {
            file_path: PathBuf,
        },
        FoundAnUnhandlableTopLevelMacroCallWithAttributes,
        MultipleItemsInXMacroUnsupported { 
            chunk: String 
        },
        FoundARawModNameWhichWeDontHandlePleaseRemoveOrUnifyWithXMacros { 
            mod_name:  String 
        },
        EncounteredAnXMacroAfterWeAlreadySawANonAttributeItem_NotRewritingSafely,
    }

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
            cargo_toml_file: PathBuf,
        },
    }

    #[derive(Clone)]
    pub enum CargoTomlError {
        TomlSerializeError {
            message: String, 
        },
        IoWriteError {
            path: PathBuf,
            source: Arc<std::io::Error>,
        },
        TopLevelNotATable {
            path: PathBuf,
            details: String,
        },
        WorkspacerFallbackError(WorkspacerFallbackError),
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
        MissingVersionKey {
            cargo_toml_file: PathBuf,
        },
        TomlParseError {
            cargo_toml_file: PathBuf,
            toml_parse_error: toml::de::Error,
        },
        TomlEditError {
            cargo_toml_file: PathBuf,
            toml_parse_error: toml_edit::TomlError,
        },
        
        // Error indicating that a file was not found.
        FileNotFound {
            missing_file: PathBuf,
        },

        FileIsNotAFile {
            invalid_path: PathBuf,
        },
        SemverError(Arc<semver::Error>),
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
        AiReadmeWriterError,
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
        SimulatedIntegrityFailureInMockCrate,
        SimulatedInvalidVersionFormat,
        SemverError(Arc<semver::Error>),

        ReadmeWriteError(ReadmeWriteError),

        // Indicates that the crate's `is_private()` check returned `true`, so
        // the crate is not publishable.
        CrateIsPrivate {
            crate_path: PathBuf,
        },
        SortAndFormatImportsInTextError {
            message: String,
        },
        FailedToRunCargoPublish {
            crate_name:    String,
            crate_version: semver::Version,
            which_err:     Arc<WhichError>,
        },
        CargoPublishFailedForCrateWithExitCode {
            crate_name:    String,
            crate_version: semver::Version,
            exit_code:     Option<i32>,
        },
        FailedtoRunCargoPublish {
            crate_name:    String,
            crate_version: semver::Version,
            io_err:        Arc<io::Error>,
        },
        CrateAlreadyPublishedOnCratesIo {
            crate_name:    String,
            crate_version: semver::Version,
        },
        FailedCratesIoCheck {
            crate_name:    String,
            crate_version: semver::Version,
            error:         Arc<reqwest::Error>,
        },
        LockfileParseFailed {
            path:    PathBuf,
            message: String,
        },
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
        CargoTomlError(CargoTomlError),

        IoError {
            io_error: Arc<io::Error>,
            context: String,
        },
        TokioJoinError {
            join_error: Arc<tokio::task::JoinError>,
        },
        BuildError(BuildError),
        TestFailure(TestFailure),
        WatchError(WatchError),
    }
}

impl From<tokio::task::JoinError> for CrateError {
    fn from(join_error: tokio::task::JoinError) -> Self {
        CrateError::TokioJoinError {
            join_error: Arc::new(join_error),
        }
    }
}

// You may have `CrateError::IoError` or similar. We'll define a helper
// function or `From<std::io::Error>` so we can map I/O errors with `?`.
impl From<std::io::Error> for CrateError {
    fn from(e: std::io::Error) -> Self {
        // For demonstration, we wrap it in a custom variant, or if you have
        // a preexisting IoError variant, adapt accordingly:
        CrateError::IoError {
            io_error: Arc::new(e),
            context: "I/O operation failed".to_string(),
        }
    }
}


error_tree!{

    #[derive(Clone)]
    pub enum WatchError {
        NotifyError(Arc<notify::Error>),
        IoError {
            io:      Arc<io::Error>,
            context: String,
        },
        ChannelRecvError(std::sync::mpsc::RecvError),
    }

    #[derive(Clone)]
    pub enum GitError {
        FailedToRunGitStatusMakeSureGitIsInstalled,
        WorkingDirectoryIsNotCleanAborting,
        IoError {
            io:      Arc<io::Error>,
            context: String,
        }
    }

    // Enum representing possible errors in the `workspace-detail` crate.
    #[derive(Clone)]
    pub enum WorkspaceError {
        SourceFileRegistrationError(SourceFileRegistrationError),
        CycleDetectedInWorkspaceDependencyGraph {
            cycle_node_id: NodeIndex,
        },
        ActuallyInSingleCrate {
            path: PathBuf,
        },
        GitError(GitError),
        CrateError(CrateError),
        CratePinFailed {
            crate_path: PathBuf,
            source:     Box<CrateError>,
        },

        TokioJoinError {
            join_error: Arc<tokio::task::JoinError>,
        },
        IoError {
            io_error: Arc<io::Error>,
            context:  String,
        },
        InvalidLockfile {
            path:    PathBuf,
            message: String,
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
        MockBuildTestFailedWithStatus {
            status: std::process::ExitStatus,
        },
        BumpError {
            crate_path: PathBuf,
            source: Box<CrateError>,
        },
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
            context: "no explicit context".to_string()
        }
    }
}

impl CargoMetadataError {

    pub fn is_cyclic_package_dependency_error(&self) -> bool {
        self.to_string().contains("cyclic package dependency")
    }
}
