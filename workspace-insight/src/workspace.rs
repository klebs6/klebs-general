crate::ix!();

#[derive(Debug)]
pub struct Workspace {
    path:   PathBuf,
    crates: Vec<CrateHandle>,
}

impl Workspace {

    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }

    pub fn n_crates(&self) -> usize {
        self.crates.len()
    }

    pub fn crates(&self) -> &[CrateHandle] {
        &self.crates
    }
}

impl ValidateIntegrity for Workspace {

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
impl ReadyForCargoPublish for Workspace {

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
            Err(WorkspaceError::MultipleErrors(errors))
        }
    }
}

impl<'a> IntoIterator for &'a Workspace {
    type Item     = &'a CrateHandle;
    type IntoIter = Iter<'a, CrateHandle>;

    fn into_iter(self) -> Self::IntoIter {
        self.crates.iter()
    }
}

#[async_trait]
impl<P> AsyncCreateWith<P> for Workspace

where 
for<'async_trait> 
P:
AsRef<Path>
+ Send
+ Sync
+ 'async_trait

{
    type Error = WorkspaceError;

    /// Asynchronously initializes a new workspace at the provided path
    async fn new(path: &P) -> Result<Self, Self::Error> {

        let path_buf = path.as_ref().to_path_buf();

        if !Workspace::is_valid(&path_buf).await {
            return Err(WorkspaceError::InvalidWorkspace {
                invalid_workspace_path: path_buf,
            });
        }

        let crates = Workspace::find_items(&path_buf).await?;

        Ok(Self { path: path_buf, crates })
    }
}

#[async_trait]
impl AsyncIsValid for Workspace {

    /// Asynchronously checks if the path is a valid Rust workspace
    async fn is_valid(path: &Path) -> bool {
        fs::metadata(path.join("Cargo.toml")).await.is_ok()
    }
}

#[async_trait]
impl AsyncFindItemsFromPath for Workspace {
    type Item = CrateHandle;
    type Error = WorkspaceError;

    /// Asynchronously finds all the crates in the workspace
    async fn find_items(path: &Path) -> Result<Vec<Self::Item>, Self::Error> {
        let mut crates = vec![];

        let mut entries = fs::read_dir(path)
            .await
            .map_err(|e| DirectoryError::ReadDirError { io: e })?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| DirectoryError::GetNextEntryError { io: e })? {
            let crate_path = entry.path();

            if fs::metadata(crate_path.join("Cargo.toml")).await.is_ok() {
                crates.push(CrateHandle::new(&crate_path).await?);
            }
        }

        Ok(crates)
    }
}

impl AsRef<Path> for Workspace {
    /// Allows `Workspace` to be treated as a path
    fn as_ref(&self) -> &Path {
        &self.path
    }
}
