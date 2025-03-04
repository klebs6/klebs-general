// ---------------- [ File: src/add_new_crate_to_workspace.rs ]
crate::ix!();

// ---------------------- [ File: workspacer-add-new-crate-to-workspace/src/lib.rs ] ----------------------

/// A trait that says: “Add a brand new crate to the workspace,” hooking it
/// into any prefix group, 3p dependencies, facade crates, etc.
///
#[async_trait]
pub trait AddNewCrateToWorkspace {
    async fn add_new_crate(
        &mut self,
        new_crate_name: &str,
        runner: &dyn CommandRunner,
    ) -> Result<(), AddNewCrateError>;
}

/// Implementation on `Workspace<PathBuf,CrateHandle>`:
pub struct CrateAdder {
    workspace: Workspace<PathBuf,CrateHandle>,
    prefix_scanner: PrefixGroupScanner,
    prefix_registrar: PrefixGroupRegistrar,
    dep_adder: InternalDepAdder,
}

impl CrateAdder {
    pub fn new(workspace: Workspace<PathBuf,CrateHandle>) -> Self {
        let prefix_scanner = PrefixGroupScanner::new(workspace.clone());
        let prefix_registrar = PrefixGroupRegistrar::new();
        let dep_adder = InternalDepAdder::new();
        Self {
            workspace,
            prefix_scanner,
            prefix_registrar,
            dep_adder,
        }
    }
}

#[async_trait]
impl AddNewCrateToWorkspace for CrateAdder {
    async fn add_new_crate(
        &mut self,
        new_crate_name: &str,
        runner: &dyn CommandRunner,
    ) -> Result<(), AddNewCrateError> {
        info!("Creating new crate '{}' in the workspace...", new_crate_name);

        // 1) Check if crate already exists
        for c in self.workspace.crates().iter() {
            if c.name() == new_crate_name {
                return Err(AddNewCrateError::AlreadyExists {
                    crate_name: new_crate_name.to_string(),
                });
            }
        }

        // 2) Create the folder + Cargo.toml
        let new_crate_path = self.workspace.as_ref().join(new_crate_name);
        info!("Creating directory at {}", new_crate_path.display());
        fs::create_dir_all(&new_crate_path).await.map_err(|io_err| AddNewCrateError::IoError {
            context: format!("creating folder for '{}'", new_crate_name),
            io_error: Arc::new(io_err),
        })?;

        let cargo_toml = new_crate_path.join("Cargo.toml");
        let initial = format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2018"
authors = ["You <you@example.com>"]
license = "MIT"
"#,
            new_crate_name
        );
        fs::write(&cargo_toml, initial).await.map_err(|io_err| AddNewCrateError::IoError {
            context: format!("writing Cargo.toml for '{}'", new_crate_name),
            io_error: Arc::new(io_err),
        })?;

        // Possibly update the root workspace Cargo.toml to add it to [workspace] members
        // (not shown, but you'd parse & rewrite).

        // 3) Now we must refresh our workspace object, or do a partial approach:
        // we can call `Workspace::new(...)` or `Workspace::find_items(...)`
        // For brevity, let's do a partial approach: just create a new CrateHandle for the new crate.
        let new_crate_handle = CrateHandle::new(&new_crate_path).await.map_err(|crate_err| {
            AddNewCrateError::IoError {
                context: format!("constructing CrateHandle for new crate '{}'", new_crate_name),
                io_error: Arc::new(std::io::Error::new(std::io::ErrorKind::Other, crate_err.to_string())),
            }
        })?;

        // 4) Identify prefix group by scanning
        let groups = self.prefix_scanner.scan().await.map_err(|scan_err| {
            AddNewCrateError::AnalysisFailed {
                crate_name: new_crate_name.to_string(),
                source: Box::new(WorkspaceError::CustomAddCrateError { details: format!("scan error: {scan_err}") }),
            }
        })?;

        let mut matched_group = None;
        let mut best_len = 0;
        for g in &groups {
            if new_crate_name.starts_with(&g.prefix) && g.prefix.len() > best_len {
                best_len = g.prefix.len();
                matched_group = Some(g.clone());
            }
        }

        // 5) If matched a group, possibly add new crate to that group’s facade
        if let Some(grp) = matched_group {
            info!("New crate '{}' belongs to prefix group '{}'", new_crate_name, grp.prefix);

            // If the group has a *-3p crate, we add an internal dep for the new crate -> that -3p crate
            if let Some(three_p) = grp.three_p_crate.clone() {
                info!("Linking new crate '{}' with the *-3p crate '{}'", new_crate_name, three_p.name());
                self.dep_adder.add_internal_dependency(&new_crate_handle, &three_p)
                    .await
                    .map_err(|e| AddNewCrateError::AddDepFailed { source: Box::new(e) })?;
            }

            // If the group has a prefix_crate, we register the new crate in it
            if let Some(prefix_facade) = grp.prefix_crate.clone() {
                info!("Registering new crate '{}' in prefix facade '{}'", new_crate_name, prefix_facade.name());
                self.prefix_registrar.register_in_prefix_crate(&prefix_facade, &new_crate_handle)
                    .await
                    .map_err(|e| AddNewCrateError::PrefixRegisterFailed { source: Box::new(e) })?;
            }
        } else {
            warn!("No existing prefix group matched '{}'; leaving it standalone", new_crate_name);
        }

        info!("Crate '{}' successfully added to workspace", new_crate_name);
        Ok(())
    }
}

//----- Example local errors (similar to before) -----
#[derive(Debug)]
pub enum AddNewCrateError {
    AlreadyExists { crate_name: String },
    IoError {
        context: String,
        io_error: Arc<std::io::Error>,
    },
    AnalysisFailed {
        crate_name: String,
        source: Box<WorkspaceError>,
    },
    AddDepFailed { source: Box<dyn std::error::Error + Send + Sync> },
    PrefixRegisterFailed { source: Box<dyn std::error::Error + Send + Sync> },
}

impl std::fmt::Display for AddNewCrateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddNewCrateError::AlreadyExists { crate_name } => {
                write!(f, "Crate '{}' already exists in workspace", crate_name)
            }
            AddNewCrateError::IoError { context, io_error } => {
                write!(f, "I/O error in {}: {}", context, io_error)
            }
            AddNewCrateError::AnalysisFailed { crate_name, source } => {
                write!(f, "Analysis for crate '{}' failed: {}", crate_name, source)
            }
            AddNewCrateError::AddDepFailed { source } => {
                write!(f, "Adding internal dep failed: {}", source)
            }
            AddNewCrateError::PrefixRegisterFailed { source } => {
                write!(f, "Registering in prefix crate failed: {}", source)
            }
        }
    }
}

impl std::error::Error for AddNewCrateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AddNewCrateError::AlreadyExists { .. } => None,
            AddNewCrateError::IoError { io_error, .. } => Some(io_error.as_ref()),
            AddNewCrateError::AnalysisFailed { source, .. } => Some(source.as_ref()),
            AddNewCrateError::AddDepFailed { source } => Some(source.as_ref()),
            AddNewCrateError::PrefixRegisterFailed { source } => Some(source.as_ref()),
        }
    }
}

// Optional test
#[cfg(test)]
mod tests {
    use super::*;
    use crate::AddNewCrateToWorkspace;
    use workspacer_3p::tokio;
    use tracing::info;
    use traced_test::traced_test;
    use lightweight_command_runner::CommandRunner;
    use std::process::Output;
    use std::os::unix::process::ExitStatusExt;

    #[derive(Default)]
    struct MockRunner;
    impl CommandRunner for MockRunner {
        fn run_command(&self, mut cmd: tokio::process::Command)
            -> tokio::task::JoinHandle<Result<Output, std::io::Error>>
        {
            tokio::spawn(async move {
                let _ = cmd;
                Ok(Output {
                    status: std::process::ExitStatus::from_raw(0),
                    stdout: b"mock success".to_vec(),
                    stderr: vec![],
                })
            })
        }
    }

    #[traced_test]
    #[tokio::test]
    async fn test_add_new_crate() {
        info!("Would test adding a new crate to a real or mock workspace...");
        // ...
    }
}
