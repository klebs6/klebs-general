// ---------------- [ File: workspacer-export/src/lib.rs ]
#[macro_use] mod imports; use imports::*;

pub use lightweight_command_runner::*;
pub use workspacer_add_new_crate_to_workspace::*;
pub use workspacer_register::*;
pub use workspacer_register_internal_crate_in_prefix_group::*;
pub use workspacer_add_internal_dep::*;
pub use workspacer_analysis::*;
pub use workspacer_bump::*;
pub use workspacer_tree::*;
pub use workspacer_format_imports::*;
pub use workspacer_lock::*;
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
pub use workspacer_check_publish_ready::*;
pub use workspacer_readme_writer::*;
pub use workspacer_topo::*;

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
    + HasCargoTomlPathBufSync 
    + AsRef<Path> 
    + Send 
    + Sync
    + 'async_trait,

    CrateError
    : From<<P as HasCargoTomlPathBuf>::Error> 
    + From<<P as HasCargoTomlPathBufSync>::Error>,
{}
