crate::ix!();

lazy_static! {
    /// A global registry of *any* typed MockWorkspace instances keyed by `PathBuf`.
    /// We store them as `Box<dyn Any + Send + Sync>` so they can be downcast to the appropriate
    /// generic type at runtime. This allows us to store multiple `MockWorkspace<P,H>` types
    /// while keeping a single global map.
    static ref MOCK_WORKSPACE_REGISTRY: AsyncMutex<HashMap<PathBuf, Box<dyn std::any::Any + Send + Sync>>> 
        = AsyncMutex::new(HashMap::new());
}

impl MockWorkspace<PathBuf, MockCrateHandle> {
    /// Registers this `MockWorkspace<PathBuf, MockCrateHandle>` in the global registry so that
    /// subsequent calls to `MockWorkspace::<PathBuf, MockCrateHandle>::new(&path)` will pick up
    /// these exact simulation settings for the given path.
    pub async fn register_in_global(&self) {
        let path: PathBuf = self.path().to_path_buf();
        trace!("Registering MockWorkspace<PathBuf,MockCrateHandle> in global map, path={:?}", path);
        let mut lock = MOCK_WORKSPACE_REGISTRY.lock().await;
        // We store it as a `Box<dyn Any + Send + Sync>` so we can downcast later:
        lock.insert(path.clone(), Box::new(self.clone()));
        info!("MockWorkspace<PathBuf,MockCrateHandle> with path={:?} has been registered", path);
    }
}

#[derive(Builder, MutGetters, Getters, Debug, Clone)]
#[builder(setter(into))]
#[getset(get = "pub", get_mut = "pub")]
pub struct MockWorkspace<P, H>
where
    H: Clone + CrateHandleInterface<P>,
    for<'async_trait> P: Clone + From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
{
    /// A mock notion of the workspace path.
    /// Often used as a "root" path for the workspace in tests.
    path: P,

    /// A list of crates in this mock workspace.
    /// Each crate is itself wrapped in an Arc<AsyncMutex<...>> so the calling code
    /// can lock and operate on them.
    #[builder(default = "Vec::new()")]
    crates: Vec<Arc<AsyncMutex<H>>>,

    /// If `simulate_missing_cargo_toml` is true, then we pretend that there's
    /// no top-level Cargo.toml, causing `AsyncTryFrom::new(...)` to fail
    /// with `WorkspaceError::InvalidWorkspace`.
    #[builder(default = "false")]
    simulate_missing_cargo_toml: bool,

    /// If `simulate_not_a_workspace` is true, then we pretend that the top-level
    /// Cargo.toml exists but does NOT contain a `[workspace]` table,
    /// causing `AsyncTryFrom::new(...)` to fail
    /// with `WorkspaceError::ActuallyInSingleCrate`.
    #[builder(default = "false")]
    simulate_not_a_workspace: bool,

    /// If `simulate_failed_integrity` is true, calls to `validate_integrity()`
    /// will fail directly, simulating an overall workspace integrity error.
    #[builder(default = "false")]
    simulate_failed_integrity: bool,

    /// If `simulate_no_crates` is true, calls to `AsyncFindItems::find_items(..)`
    /// return an empty Vec, simulating that no crates were discovered in the workspace.
    #[builder(default = "false")]
    simulate_no_crates: bool,
}

impl<P, H> MockWorkspace<P, H>
where
    H: Clone + CrateHandleInterface<P>,
    for<'async_trait> P: Clone + From<PathBuf> + AsRef<Path> + Clone + Send + Sync + 'async_trait,
{
    /// A convenience constructor returning a "fully valid" mock workspace:
    /// - Has a top-level Cargo.toml (simulate_missing_cargo_toml = false)
    /// - Declares itself to be a workspace (simulate_not_a_workspace = false)
    /// - Integrity checks pass (simulate_failed_integrity = false)
    /// - It has some crates in `crates` (you can decide how many).
    /// - `simulate_no_crates = false` so that `AsyncFindItems::find_items(...)` will
    ///   return those crates.
    pub fn fully_valid_config() -> Self {
        trace!("MockWorkspace::fully_valid_config constructor called");
        // For demonstration, let's add two crates into the workspace.
        // You might choose to store real `MockCrateHandle`s or
        // any `H: CrateHandleInterface<P>` to represent them.
        // We'll assume H == MockCrateHandle in typical usage,
        // but here we won't force that type.
        // We'll just "pretend" we have an empty list if we don't control H's constructor.
        // Adjust as needed.

        // If `H` has a convenience constructor, you might do:
        // let crate_a = Arc::new(AsyncMutex::new(H::new_from_mock("crateA")?));
        // let crate_b = Arc::new(AsyncMutex::new(H::new_from_mock("crateB")?));
        // For simplicity, we'll keep it empty unless you have a known H constructor.
        let crates_list = vec![];

        // Build our mock workspace
        MockWorkspaceBuilder::default()
            .path(PathBuf::from("/fake/mock/workspace/path")) // or any arbitrary "root"
            .crates(crates_list)
            .simulate_missing_cargo_toml(false)
            .simulate_not_a_workspace(false)
            .simulate_failed_integrity(false)
            .simulate_no_crates(false)
            .build()
            .unwrap()
    }
}

#[async_trait]
impl<P, H> AsyncTryFrom<P> for MockWorkspace<P, H>
where
    for<'async_trait> H: Clone + CrateHandleInterface<P> + Send + Sync + 'async_trait,
    for<'async_trait> P: Clone + From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
{
    type Error = WorkspaceError;

    async fn new(path: &P) -> Result<Self, Self::Error> {
        trace!("MockWorkspace::AsyncTryFrom::new called, path={:?}", path.as_ref());
        let path_buf = path.as_ref().to_path_buf();

        // 1) See if there's a pre-registered MockWorkspace in the global map.
        //    If found, attempt a downcast to `MockWorkspace<P,H>`.
        {
            let lock = MOCK_WORKSPACE_REGISTRY.lock().await;
            if let Some(boxed_any) = lock.get(&path_buf) {
                debug!("Found a pre-registered Box<dyn Any> for path={:?}", path_buf);
                if let Some(existing_ws) = boxed_any.downcast_ref::<MockWorkspace<P, H>>() {
                    trace!("Successfully downcast to MockWorkspace<P,H>, applying simulation checks");

                    if *existing_ws.simulate_missing_cargo_toml() {
                        error!("simulate_missing_cargo_toml=true => returning InvalidWorkspace");
                        return Err(WorkspaceError::InvalidWorkspace {
                            invalid_workspace_path: path_buf,
                        });
                    }

                    if *existing_ws.simulate_not_a_workspace() {
                        error!("simulate_not_a_workspace=true => returning ActuallyInSingleCrate");
                        return Err(WorkspaceError::ActuallyInSingleCrate {
                            path: path_buf,
                        });
                    }

                    info!("Returning a clone of the pre-registered MockWorkspace<P,H>");
                    return Ok(existing_ws.clone());
                } else {
                    warn!("Failed downcasting pre-registered item to MockWorkspace<P,H>; ignoring it");
                }
            }
        }

        // 2) Otherwise, build a new default "fully_valid_config" instance and apply checks
        let mut ws = Self::fully_valid_config();
        ws.path = path.clone();

        if *ws.simulate_missing_cargo_toml() {
            error!("simulate_missing_cargo_toml=true => returning InvalidWorkspace");
            return Err(WorkspaceError::InvalidWorkspace {
                invalid_workspace_path: path_buf,
            });
        }

        if *ws.simulate_not_a_workspace() {
            error!("simulate_not_a_workspace=true => returning ActuallyInSingleCrate");
            return Err(WorkspaceError::ActuallyInSingleCrate {
                path: path_buf,
            });
        }

        info!("Returning a new fully_valid_config-based MockWorkspace<P,H>");
        Ok(ws)
    }
}

impl<P, H> WorkspaceInterface<P, H> for MockWorkspace<P, H>
where
    for<'async_trait> H: Clone + CrateHandleInterface<P> + 'async_trait,
    for<'async_trait> P: Clone + From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
{
    // This is a marker trait aggregator. All sub-traits must be implemented.
}

impl<P, H> GetCrates<P, H> for MockWorkspace<P, H>
where
    H: Clone + CrateHandleInterface<P>,
    for<'async_trait> P: Clone + From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
{
    fn crates(&self) -> &[Arc<AsyncMutex<H>>] {
        trace!("MockWorkspace::GetCrates::crates called");
        self.crates()
    }
}

impl<P, H> GetCratesMut<P, H> for MockWorkspace<P, H>
where
    H: Clone + CrateHandleInterface<P>,
    for<'async_trait> P: Clone + From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
{
    fn crates_mut(&mut self) -> &mut Vec<Arc<AsyncMutex<H>>> {
        trace!("MockWorkspace::GetCratesMut::crates called");
        self.crates_mut()
    }
}

impl<P, H> NumCrates for MockWorkspace<P, H>
where
    H: Clone + CrateHandleInterface<P>,
    for<'async_trait> P: Clone + From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
{
    fn n_crates(&self) -> usize {
        trace!("MockWorkspace::NumCrates::n_crates called");
        self.crates().len()
    }
}

#[async_trait]
impl<P, H> ValidateIntegrity for MockWorkspace<P, H>
where
    H: Clone + CrateHandleInterface<P>,
    for<'async_trait> P: Clone + From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
{
    type Error = WorkspaceError;

    async fn validate_integrity(&self) -> Result<(), Self::Error> {
        trace!("MockWorkspace::validate_integrity called");

        // If simulate_missing_cargo_toml is true, pretend there's no top-level Cargo.toml:
        if *self.simulate_missing_cargo_toml() {
            error!("simulate_missing_cargo_toml=true => returning InvalidWorkspace");
            return Err(WorkspaceError::InvalidWorkspace {
                invalid_workspace_path: self.path().as_ref().to_path_buf(),
            });
        }

        // If simulate_not_a_workspace is true, pretend there's no [workspace] table:
        if *self.simulate_not_a_workspace() {
            error!("simulate_not_a_workspace=true => returning ActuallyInSingleCrate");
            return Err(WorkspaceError::ActuallyInSingleCrate {
                path: self.path().as_ref().to_path_buf(),
            });
        }

        // If simulate_failed_integrity is set, we bail:
        if *self.simulate_failed_integrity() {
            error!("simulate_failed_integrity=true => returning InvalidWorkspace");
            return Err(WorkspaceError::InvalidWorkspace {
                invalid_workspace_path: self.path().as_ref().to_path_buf(),
            });
        }

        // If simulate_no_crates => do nothing special here; the logic 
        // for find_items might produce an empty list, etc.

        // Otherwise, validate each crate:
        for c in self.crates().iter() {
            let guard = c.lock().await;
            guard.validate_integrity().await?;
        }

        info!("MockWorkspace: integrity validation passed");
        Ok(())
    }
}

#[async_trait]
impl<P, H> FindCrateByName<P, H> for MockWorkspace<P, H>
where
    H: Clone + CrateHandleInterface<P>,
    for<'async_trait> P: Clone + From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
{
    async fn find_crate_by_name(&self, name: &str) -> Option<Arc<AsyncMutex<H>>> {
        trace!("MockWorkspace::FindCrateByName::find_crate_by_name called, name={}", name);
        for crate_arc in self.crates().iter() {
            let guard = crate_arc.lock().await;
            if guard.name() == name {
                debug!("MockWorkspace: found crate matching name='{}'", name);
                return Some(Arc::clone(crate_arc));
            }
        }
        info!("MockWorkspace: no crate matching name='{}'", name);
        None
    }
}

#[async_trait]
impl<P, H> GetAllCrateNames for MockWorkspace<P, H>
where
    H: Clone + CrateHandleInterface<P>,
    for<'async_trait> P: Clone + From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
{
    async fn get_all_crate_names(&self) -> Vec<String> {
        trace!("MockWorkspace::GetAllCrateNames::get_all_crate_names called");
        let mut names = vec![];
        for crate_arc in self.crates().iter() {
            let guard = crate_arc.lock().await;
            names.push(guard.name().to_string());
        }
        info!("MockWorkspace: returning crate names: {:?}", names);
        names
    }
}

impl<P, H> AsRef<Path> for MockWorkspace<P, H>
where
    H: Clone + CrateHandleInterface<P>,
    for<'async_trait> P: Clone + From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
{
    fn as_ref(&self) -> &Path {
        trace!("MockWorkspace::as_ref called, returning path={:?}", self.path().as_ref());
        self.path().as_ref()
    }
}

#[async_trait]
impl<P, H> AsyncFindItems for MockWorkspace<P, H>
where
    H: Clone + CrateHandleInterface<P> + Send + Sync,
    for<'async_trait> P: Clone + From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
{
    type Item = Arc<AsyncMutex<H>>;
    type Error = WorkspaceError;

    async fn find_items(_path: &Path) -> Result<Vec<Self::Item>, Self::Error> {
        trace!("MockWorkspace::AsyncFindItems::find_items called (static), ignoring path={:?}", _path);
        // Because this is a static method, we can't reference `self`.
        // We'll just check if the mock is simulating "no crates" or not.
        // For a real mock scenario, you might store the "simulate_xxx" state in a global
        // or pass a static reference. To keep it simple, we'll always return an empty list
        // or a single crate if you prefer. We'll do an empty list for demonstration.

        info!("MockWorkspace: returning an empty crate list from find_items");
        Ok(vec![])
    }
}

#[async_trait]
impl<P, H> AsyncPathValidator for MockWorkspace<P, H>
where
    H: Clone + CrateHandleInterface<P> + Send + Sync,
    for<'async_trait> P: Clone + From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
{
    async fn is_valid(path: &Path) -> bool {
        trace!("MockWorkspace::AsyncPathValidator::is_valid called for path={:?}", path);
        // We'll just see if the path ends with "simulate_invalid" for demonstration.
        // In real usage, you'd do something else. For now, let's pretend it's always valid:
        debug!("MockWorkspace: returning true for is_valid(...) by default");
        true
    }
}

// ----------------------------------------------------------------------
// Tests for MockWorkspace
// ----------------------------------------------------------------------
#[cfg(test)]
mod test_mock_workspace {
    use super::*;

    // Because we can't combine #[traced_test] with #[tokio::test],
    // we can manually create a tokio runtime if we want to run async code.

    #[traced_test]
    fn test_fully_valid_config_works() {
        let mock_ws = MockWorkspace::<PathBuf, MockCrateHandle>::fully_valid_config();
        assert!(!mock_ws.simulate_missing_cargo_toml(), "Should default to false");
        assert!(!mock_ws.simulate_not_a_workspace(), "Should default to false");
        assert!(!mock_ws.simulate_failed_integrity(), "Should default to false");
        assert!(!mock_ws.simulate_no_crates(), "Should default to false");

        // Check that n_crates() matches the crates we inserted
        // (Here, we did an empty list in the code above, so it should be 0.)
        assert_eq!(mock_ws.n_crates(), 0, "Currently we have an empty crates list");
    }

    #[traced_test]
    async fn test_mock_workspace_new_missing_cargo_toml_fails() {
        // The key fix is to use a UNIQUE path per test so we don't overwrite or conflict
        // with another global registry entry in a different test that sets different flags.
        let path = PathBuf::from("/fake/mock/workspace/path_for_missing_cargo_toml");

        trace!("test_mock_workspace_new_missing_cargo_toml_fails starting");
        // 1) Build a "fully valid" MockWorkspace
        let mut failing_ws = MockWorkspace::<PathBuf, MockCrateHandle>::fully_valid_config();
        // 2) Provide a custom path so we don't conflict with other tests
        *failing_ws.path_mut() = path.clone();
        // 3) Toggle the simulation flag to simulate missing Cargo.toml
        *failing_ws.simulate_missing_cargo_toml_mut() = true;
        // 4) Register in the global map, making sure to `await`
        failing_ws.register_in_global().await;

        // 5) Now call `MockWorkspace::new(...)`, which should see that
        //    `simulate_missing_cargo_toml = true` for this path
        let result = MockWorkspace::<PathBuf, MockCrateHandle>::new(&path).await;

        // 6) Check that we do, in fact, get `WorkspaceError::InvalidWorkspace`
        assert!(result.is_err(), "Should fail with simulate_missing_cargo_toml=true");
        match result.err().unwrap() {
            WorkspaceError::InvalidWorkspace { .. } => {
                info!("Got expected WorkspaceError::InvalidWorkspace");
            }
            other => {
                panic!("Expected InvalidWorkspace error, got: {:?}", other);
            }
        }
    }

    #[traced_test]
    async fn test_mock_workspace_new_not_a_workspace_fails() {
        // Again, use a UNIQUE path for not-a-workspace scenario
        let path = PathBuf::from("/fake/mock/workspace/path_for_not_a_workspace");

        trace!("test_mock_workspace_new_not_a_workspace_fails starting");
        // 1) Build a "fully valid" MockWorkspace
        let mut failing_ws = MockWorkspace::<PathBuf, MockCrateHandle>::fully_valid_config();
        // 2) Provide a custom path so we don't conflict with other tests
        *failing_ws.path_mut() = path.clone();
        // 3) Toggle the simulation flag
        *failing_ws.simulate_not_a_workspace_mut() = true;
        // 4) Register in the global map
        failing_ws.register_in_global().await;

        // 5) This should find the pre-registered configuration and produce
        //    `WorkspaceError::ActuallyInSingleCrate`
        let result = MockWorkspace::<PathBuf, MockCrateHandle>::new(&path).await;
        assert!(result.is_err(), "Should fail with simulate_not_a_workspace=true");

        match result.err().unwrap() {
            WorkspaceError::ActuallyInSingleCrate { .. } => {
                info!("Got expected WorkspaceError::ActuallyInSingleCrate");
            }
            other => {
                panic!("Expected ActuallyInSingleCrate error, got: {:?}", other);
            }
        }
    }

    #[traced_test]
    async fn test_mock_workspace_validate_integrity_fails_when_simulated() {
        let mut ws = MockWorkspace::<PathBuf, MockCrateHandle>::fully_valid_config();
        *ws.simulate_failed_integrity_mut() = true;
        let result = ws.validate_integrity().await;
        assert!(result.is_err(), "Should fail if simulate_failed_integrity=true");
    }

    #[traced_test]
    async fn test_mock_workspace_find_crate_by_name() {
        // We'll put some crates in the list and see if we can find them by name
        // For demonstration, we'll create 2 MockCrateHandles with names "crateA" and "crateB"
        let crate_a = Arc::new(AsyncMutex::new(
            MockCrateHandle::fully_valid_config()
                .to_builder()
                .crate_name("crateA")
                .build()
                .unwrap()
        ));
        let crate_b = Arc::new(AsyncMutex::new(
            MockCrateHandle::fully_valid_config()
                .to_builder()
                .crate_name("crateB")
                .build()
                .unwrap()
        ));

        let ws = MockWorkspaceBuilder::<PathBuf, MockCrateHandle>::default()
            .path(PathBuf::from("/fake/mock/workspace/path"))
            .crates(vec![crate_a.clone(), crate_b.clone()])
            .build()
            .unwrap();

        assert_eq!(ws.n_crates(), 2, "We have 2 crates total");
        let found = ws.find_crate_by_name("crateB").await;
        assert!(found.is_some(), "Should find crateB by name");
        let found = found.unwrap();
        assert_eq!(found.lock().await.name(), "crateB");
    }

    #[traced_test]
    async fn test_mock_workspace_get_all_crate_names() {
        // Similar to above
        let crate_1 = Arc::new(AsyncMutex::new(
            MockCrateHandle::fully_valid_config()
                .to_builder()
                .crate_name("crateAlpha")
                .build()
                .unwrap()
        ));
        let crate_2 = Arc::new(AsyncMutex::new(
            MockCrateHandle::fully_valid_config()
                .to_builder()
                .crate_name("crateBeta")
                .build()
                .unwrap()
        ));

        let ws = MockWorkspaceBuilder::<PathBuf, MockCrateHandle>::default()
            .path(PathBuf::from("/fake/mock/workspace/path"))
            .crates(vec![crate_1.clone(), crate_2.clone()])
            .build()
            .unwrap();

        let names = ws.get_all_crate_names().await;
        assert_eq!(names.len(), 2, "Should have 2 names total");
        assert!(names.contains(&"crateAlpha".to_string()));
        assert!(names.contains(&"crateBeta".to_string()));
    }
}
