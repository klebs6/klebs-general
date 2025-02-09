// ---------------- [ File: workspacer/src/workspace.rs ]
crate::ix!();

#[derive(Debug)]
pub struct Workspace<P,H:CrateHandleInterface<P>> 
where for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait 
{
    path:   P,
    crates: Vec<H>,
}

impl<P,H:CrateHandleInterface<P>> WorkspaceInterface<P,H> for Workspace<P,H> 
where for<'async_trait> P: From<PathBuf> + AsRef<Path> + Clone + Send + Sync + 'async_trait 
{ }

#[disable]
impl<P,H:CrateHandleInterface<P>> Into<P> for Workspace<P,H> 
where for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait
{
    fn into(self) -> P {
        self.path
    }
}

impl<P,H:CrateHandleInterface<P>> NumCrates for Workspace<P,H> 
where for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait
{

    fn n_crates(&self) -> usize {
        self.crates.len()
    }
}

impl<P,H:CrateHandleInterface<P>> GetCrates<P,H> for Workspace<P,H> 
where for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait
{

    fn crates(&self) -> &[H] {
        &self.crates
    }
}

impl<P,H:CrateHandleInterface<P>> ValidateIntegrity for Workspace<P,H> 
where for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait
{

    type Error = WorkspaceError;

    /// Validates the integrity of the entire workspace by checking each crate
    ///
    fn validate_integrity(&self) -> Result<(), Self::Error> {
        for crate_handle in self {
            crate_handle.validate_integrity()?;
        }
        Ok(())
    }
}

#[async_trait]
impl<P,H:CrateHandleInterface<P>> ReadyForCargoPublish for Workspace<P,H> 
where for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait
{
    type Error = WorkspaceError;

    /// Ensures all crates in the workspace are ready for Cargo publishing
    async fn ready_for_cargo_publish(&self) -> Result<(), WorkspaceError> {
        let mut errors = vec![];

        for crate_handle in self {
            if let Err(e) = crate_handle.ready_for_cargo_publish().await {
                errors.push(e);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            let errors: Vec<WorkspaceError> = errors.into_iter().map(|x| x.into()).collect();
            Err(WorkspaceError::MultipleErrors(errors))
        }
    }
}

impl<'a,P,H:CrateHandleInterface<P>> IntoIterator for &'a Workspace<P,H> 
where for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait
{
    type Item     = &'a H;
    type IntoIter = Iter<'a, H>;

    fn into_iter(self) -> Self::IntoIter {
        self.crates.iter()
    }
}

#[async_trait]
impl<P,H:CrateHandleInterface<P>> AsyncTryFrom<P> for Workspace<P,H>
where for<'async_trait> P: From<PathBuf> + AsRef<Path> + Clone + Send + Sync + 'async_trait
{
    type Error = WorkspaceError;

    /// Asynchronously initializes a new workspace at the provided path
    async fn new(path: &P) -> Result<Self, Self::Error> {

        let path_buf = path.as_ref().to_path_buf();

        if !Workspace::<P,H>::is_valid(&path_buf).await {
            return Err(WorkspaceError::InvalidWorkspace {
                invalid_workspace_path: path_buf,
            });
        }

        let crates = Workspace::<P,H>::find_items(&path_buf).await?;

        Ok(Self { path: path.clone(), crates })
    }
}

#[async_trait]
impl<P,H:CrateHandleInterface<P>> AsyncPathValidator for Workspace<P,H> 
where for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait
{

    /// Asynchronously checks if the path is a valid Rust workspace
    async fn is_valid(path: &Path) -> bool {
        fs::metadata(path.join("Cargo.toml")).await.is_ok()
    }
}

#[async_trait]
impl<P, H> AsyncFindItems for Workspace<P, H>
where
    // Make sure P can be constructed from a PathBuf
    for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
    H: CrateHandleInterface<P> + Send + Sync,
{
    type Item = H;
    type Error = WorkspaceError;

    /// Asynchronously finds all the crates in the workspace
    async fn find_items(path: &Path) -> Result<Vec<Self::Item>, Self::Error> {
        let mut crates = vec![];

        let mut entries = fs::read_dir(path)
            .await
            .map_err(|e| DirectoryError::ReadDirError { io: e.into() })?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| DirectoryError::GetNextEntryError { io: e.into() })?
        {
            let crate_path = entry.path();

            // If there's a Cargo.toml here, we consider it a crate:
            if fs::metadata(crate_path.join("Cargo.toml"))
                .await
                .is_ok()
            {
                // Convert crate_path (PathBuf) into your generic P:
                let converted: P = crate_path.into();
                crates.push(H::new(&converted).await?);
            }
        }

        Ok(crates)
    }
}

impl<P,H:CrateHandleInterface<P>> AsRef<Path> for Workspace<P,H> 
where for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait
{
    /// Allows `Workspace` to be treated as a path
    fn as_ref(&self) -> &Path {
        self.path.as_ref()
    }
}
