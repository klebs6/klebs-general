// ---------------- [ File: workspacer-workspace/src/workspace.rs ]
crate::ix!();

#[derive(Builder,MutGetters,Getters,Debug)]
#[getset(get="pub",get_mut="pub")]
#[builder(setter(into))]
pub struct Workspace<P,H:CrateHandleInterface<P>> 
where for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait 
{
    path:   P,
    crates: Vec<Arc<H>>,
}

impl<P,H:CrateHandleInterface<P>> WorkspaceInterface<P,H> for Workspace<P,H> 
where for<'async_trait> P: From<PathBuf> + AsRef<Path> + Clone + Send + Sync + 'async_trait 
{ }

impl<P,H:CrateHandleInterface<P>> AsRef<Path> for Workspace<P,H> 
where for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait
{
    /// Allows `Workspace` to be treated as a path
    fn as_ref(&self) -> &Path {
        self.path.as_ref()
    }
}

impl<P,H:CrateHandleInterface<P>> GetCrates<P,H> for Workspace<P,H> 
where for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait
{
    fn crates(&self) -> &[Arc<H>] {
        &self.crates
    }

    fn crates_mut(&mut self) -> &mut Vec<Arc<H>> {
        &mut self.crates
    }
}

#[async_trait]
impl<P,H:CrateHandleInterface<P>> AsyncTryFrom<P> for Workspace<P,H>
where
    // your existing constraints
    for<'async_trait> P: From<PathBuf> + AsRef<Path> + Clone + Send + Sync + 'async_trait,
{
    type Error = WorkspaceError;

    /// Asynchronously initializes a new workspace at the provided path,
    /// ensuring there's a `[workspace]` table in Cargo.toml. If not found,
    /// returns `WorkspaceError::ActuallyInSingleCrate`.
    async fn new(path: &P) -> Result<Self, Self::Error> {
        // Step 1) Basic check: is there a Cargo.toml at all?
        let path_buf = path.as_ref().to_path_buf();
        let cargo_toml = path_buf.join("Cargo.toml");
        if !cargo_toml.exists() {
            return Err(WorkspaceError::InvalidWorkspace {
                invalid_workspace_path: path_buf,
            });
        }

        // Step 2) Read and parse the top-level Cargo.toml
        let cargo_toml_str = tokio::fs::read_to_string(&cargo_toml).await.map_err(|e| {
            WorkspaceError::IoError {
                io_error: e.into(),
                context: format!("Failed to read {:?}", cargo_toml),
            }
        })?;

        // Step 3) Check for a `[workspace]` table
        let parsed: toml::Value = toml::from_str(&cargo_toml_str).map_err(|_e| {
            WorkspaceError::InvalidWorkspace {
                invalid_workspace_path: path_buf.clone(),
            }
        })?;

        let is_workspace = parsed
            .as_table()
            .map_or(false, |tbl| tbl.contains_key("workspace"));
        if !is_workspace {
            // We found a Cargo.toml but no [workspace] => single crate
            return Err(WorkspaceError::ActuallyInSingleCrate {
                path: path_buf,
            });
        }

        // Step 4) If it’s truly a workspace, find crates as normal
        // (Your original logic)
        if !Self::is_valid(&path_buf).await {
            return Err(WorkspaceError::InvalidWorkspace {
                invalid_workspace_path: path_buf,
            });
        }

        let crates = Self::find_items(&path_buf).await?;
        Ok(Self { path: path.clone(), crates })
    }
}
