// ---------------- [ File: workspacer-mock/src/test_mock.rs ]
crate::ix!();

// -----------------------------------------------------
// Setup: Minimal Mocks for P, H, and the Workspace
// -----------------------------------------------------

// 1) We'll define a minimal mock for P that can be constructed from a PathBuf,
//    and implements AsRef<Path>. 
#[derive(Debug, Clone)]
pub struct MockPath(pub PathBuf);

impl AsRef<Path> for MockPath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl From<PathBuf> for MockPath {
    fn from(pb: PathBuf) -> Self {
        Self(pb)
    }
}

// 2) We'll define a minimal mock for H that implements CrateHandleInterface<P>.
#[derive(Debug, Clone)]
pub struct MockCrateHandle {
    // e.g. store the path or some data
    crate_path: PathBuf,
    publish_ready: bool, // We'll track if it's "ready" for publishing
}

#[async_trait]
impl<P> CrateHandleInterface<P> for MockCrateHandle
where
    // This trait typically requires these bounds:
    for<'async_trait> P: AsRef<Path> + Send + Sync + 'async_trait,
{
    // Stub out or skip other methods if not used in tests
}

// For ReadyForCargoPublish, we define a minimal:
#[async_trait]
impl ReadyForCargoPublish for MockCrateHandle {
    type Error = CrateError; // or any mapped error

    async fn ready_for_cargo_publish(&self) -> Result<(), Self::Error> {
        if self.publish_ready {
            Ok(())
        } else {
            // Simulate an error
            Err(CrateError::CargoTomlError(CargoTomlError::MissingPackageSection {
                cargo_toml_file: self.crate_path.clone()
            }))
        }
    }
}

// 3) We'll define a minimal "Workspace<P,H>" that has a `crates: Vec<H>` field
//    so that the IntoIterator trait, etc. can operate.
#[derive(Debug)]
pub struct MockWorkspace<P, H> {
    path: P,
    crates: Vec<H>,
}

impl<P,H> MockWorkspace<P,H> {
    fn new(path: P, crates: Vec<H>) -> Self {
        Self { path, crates }
    }
}

// Implement the real traits in question:

// a) IntoIterator
impl<'a, P, H: CrateHandleInterface<P>> IntoIterator for &'a MockWorkspace<P, H>
where
    for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait
{
    type Item = &'a H;
    type IntoIter = Iter<'a, H>;

    fn into_iter(self) -> Self::IntoIter {
        self.crates.iter()
    }
}

// b) AsyncPathValidator
#[async_trait]
impl<P,H> AsyncPathValidator for MockWorkspace<P,H>
where
    for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
    H: CrateHandleInterface<P> + Send + Sync,
{
    async fn is_valid(path: &Path) -> bool {
        fs::metadata(path.join("Cargo.toml")).await.is_ok()
    }
}

// c) ReadyForCargoPublish
#[async_trait]
impl<P,H> ReadyForCargoPublish for MockWorkspace<P,H>
where
    for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
    H: CrateHandleInterface<P> + ReadyForCargoPublish<Error=CrateError> + Send + Sync,
{
    type Error = WorkspaceError;

    async fn ready_for_cargo_publish(&self) -> Result<(), Self::Error> {
        let mut errors = vec![];
        for crate_handle in self {
            if let Err(e) = crate_handle.ready_for_cargo_publish().await {
                errors.push(e.into());
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(WorkspaceError::MultipleErrors(errors))
        }
    }
}
