// ---------------- [ File: workspacer/src/lib.rs ]
pub use workspacer_add_new_crate_to_workspace::*;
pub use workspacer_analysis::*;
pub use workspacer_cleanup::*;
pub use workspacer_consolidate::*;
pub use workspacer_crate::*;
pub use workspacer_detect_circular_deps::*;
pub use workspacer_docs::*;
pub use workspacer_git::*;
pub use workspacer_interface::*;
pub use workspacer_linting::*;
pub use workspacer_metadata::*;
pub use workspacer_name_all_files::*;
pub use workspacer_pin::*;
pub use workspacer_publish::*;
pub use workspacer_rebuild_or_test::*;
pub use workspacer_syntax::*;
pub use workspacer_test_coverage::*;
pub use workspacer_toml::*;
pub use workspacer_watch_and_reload::*;
pub use workspacer_workspace::*;
pub use workspacer_errors::*;
pub use workspacer_workspace_interface::*;
pub use workspacer_crate_interface::*;

use std::path::{Path,PathBuf};

pub trait ExtendedWorkspaceInterface<P,T>
: WorkspaceInterface<P,T> 
+ CleanupWorkspace
+ Analyze
+ DetectCircularDependencies
+ GenerateDependencyTree
+ ReadyForCargoPublish<Error=WorkspaceError>
+ RunLinting
+ RebuildOrTest
+ GenerateDocs
+ GetCargoMetadata
+ WatchAndReload
+ RunTestsWithCoverage
+ PinAllWildcardDependencies<Error=WorkspaceError>
+ EnsureGitClean<Error=GitError>
+ NameAllFiles<Error=WorkspaceError>
+ TryPublish<Error=WorkspaceError>
where 
for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
T: CrateHandleInterface<P>
{}

pub trait ExtendedCrateInterface<P>
: CrateHandleInterface<P>
+ TryPublish<Error=CrateError>
+ EnsureGitClean<Error=GitError>
+ NameAllFiles<Error=CrateError>
+ PinWildcardDependencies<Error=CrateError>
+ PinAllWildcardDependencies<Error=CrateError>
+ ReadyForCargoPublish<Error=CrateError>
where 
    for<'async_trait> 
    P
    : HasCargoTomlPathBuf 
    + AsRef<Path> 
    + Send 
    + Sync
    + 'async_trait,

    CrateError: From<<P as HasCargoTomlPathBuf>::Error>,
{}
