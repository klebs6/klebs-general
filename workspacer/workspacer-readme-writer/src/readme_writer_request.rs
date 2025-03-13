// ---------------- [ File: workspacer-readme-writer/src/readme_writer_request.rs ]
crate::ix!();

#[derive(Serialize, Deserialize, Builder, Getters, Clone)]
#[builder(setter(into))]
#[getset(get = "pub")]
pub struct AiReadmeWriterRequest<P>
where
    P: AsRef<Path> + Send + Sync + 'static,
{
    /// The crate handle is skipped in serialization, because a trait object
    /// cannot be trivially serialized. We'll try to reconstruct it after
    /// deserialization using `crate_name` or other info.
    #[serde(skip)]
    crate_handle: Option<Arc<dyn ReadmeWritingCrateHandle<P>>>,

    crate_name: String,

    version: Version,

    consolidated_crate_interface: ConsolidatedCrateInterface,

    maybe_cargo_toml_package_authors: Option<Vec<String>>,

    maybe_cargo_toml_rust_edition: Option<String>,

    maybe_cargo_toml_license: Option<String>,

    maybe_cargo_toml_crate_repository_location: Option<String>,
}

impl<P> AiReadmeWriterRequest<P>
where
    P: AsRef<Path> + Send + Sync + 'static,
{
    /// Construct from an existing handle. This is used
    /// in normal usage (not from deserialization).
    pub async fn async_try_from<H>(
        handle: Arc<H>,
    ) -> Result<Self, ReadmeWriterExecutionError>
    where
        H: ReadmeWritingCrateHandle<P>, // the super-trait
    {
        use std::ops::Deref;
        let consolidation_opts = ConsolidationOptions::new().with_docs().with_fn_bodies();

        // 1) We can call name(), version(), etc. because H: CrateHandleInterface<P>.
        let crate_name = handle.name().to_string();
        let version = handle
            .version()
            .expect("expected a valid version in the crate");

        let consolidated_crate_interface = handle
            .consolidate_crate_interface(&consolidation_opts)
            .await?;

        // 2) We'll store it in an Option for later usage 
        //    (likely `Some(...)` in normal usage).
        let crate_handle_obj: Arc<dyn ReadmeWritingCrateHandle<P>> = handle;

        // 3) We do a short synchronous read from CargoToml
        let direct_authors = {
            let cargo_toml = crate_handle_obj.cargo_toml();
            let mut guard = cargo_toml.lock().unwrap();
            guard.get_package_authors()?
        };
        let maybe_cargo_toml_package_authors = if direct_authors.is_some() {
            direct_authors
        } else {
            None
        };

        let direct_edition = {
            let cargo_toml = crate_handle_obj.cargo_toml();
            let mut guard = cargo_toml.lock().unwrap();
            guard.get_rust_edition()?
        };
        let maybe_cargo_toml_rust_edition = if direct_edition.is_some() {
            direct_edition
        } else {
            None
        };

        let direct_license = {
            let cargo_toml = crate_handle_obj.cargo_toml();
            let mut guard = cargo_toml.lock().unwrap();
            guard.get_license_type()?
        };
        let maybe_cargo_toml_license = if direct_license.is_some() {
            direct_license
        } else {
            None
        };

        let direct_repo = {
            let cargo_toml = crate_handle_obj.cargo_toml();
            let mut guard = cargo_toml.lock().unwrap();
            guard.get_crate_repository_location()?
        };
        let maybe_cargo_toml_crate_repository_location = if direct_repo.is_some() {
            direct_repo
        } else {
            None
        };

        Ok(Self {
            crate_handle: Some(crate_handle_obj),
            crate_name,
            version,
            consolidated_crate_interface,
            maybe_cargo_toml_package_authors,
            maybe_cargo_toml_rust_edition,
            maybe_cargo_toml_license,
            maybe_cargo_toml_crate_repository_location,
        })
    }

    /// Attempt to re-initialize the crate handle after deserialization,
    /// if it is currently `None`. For example, we might look up the local
    /// filesystem to see if there's a crate at `./{crate_name}` or so.
    ///
    /// If already `Some(...)`, we do nothing. If we can't find it, we
    /// leave it as `None`.
    pub fn try_init_crate_handle(&mut self) -> Result<(), ReadmeWriterExecutionError> {
        // If we've already got a handle, do nothing
        if self.crate_handle.is_some() {
            return Ok(());
        }

        // Otherwise, try to find or build a CrateHandle somehow.
        let path = std::path::PathBuf::from(self.crate_name.clone());
        // Or do more advanced logic like a registry or workspace search

        // We'll show an example that tries to do something like:
        // let handle = CrateHandle::new(&path).await?; // This is an async call,
        // so you'd need to rework this function to be async if that's required.

        // For demonstration, let's do a "stub" attempt:
        let maybe_handle = Self::attempt_local_crate_lookup(&path)?;

        self.crate_handle = maybe_handle;
        Ok(())
    }

    /// A local, synchronous example that attempts to see if a directory
    /// named `crate_name` is present, etc. For real usage, you might do
    /// an async approach or a registry approach.
    fn attempt_local_crate_lookup(
        path: &PathBuf
    ) -> Result<Option<Arc<dyn ReadmeWritingCrateHandle<P>>>, ReadmeWriterExecutionError> {
        if path.is_dir() {
            // we pretend that we can create a real handle:
            let handle = CrateHandle::new_sync(&path)  // you'd define a sync version 
                .map_err(|e| ReadmeWriterExecutionError::CrateError(e))?;

            // cast to trait object:
            let handle_obj: Arc<dyn ReadmeWritingCrateHandle<P>> = Arc::new(handle);
            Ok(Some(handle_obj))
        } else {
            // no handle found
            Ok(None)
        }
    }

    /// If you want a direct setter, you can define it for usage post-deser:
    pub fn set_crate_handle(&mut self, handle: Arc<dyn ReadmeWritingCrateHandle<P>>) {
        self.crate_handle = Some(handle);
    }
}

impl<P> std::fmt::Display for AiReadmeWriterRequest<P>
where
    P: AsRef<Path> + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AiReadmeWriterRequest for crate: {}", self.crate_name)
    }
}

impl<P> Named for AiReadmeWriterRequest<P>
where
    P: AsRef<Path> + Send + Sync + 'static,
{
    fn name(&self) -> std::borrow::Cow<'_, str> {
        // Example name logic
        std::borrow::Cow::Owned(format!("{}-ai-readme-request", self.crate_name))
    }
}

impl<P> AiReadmeWriterRequest<P>
where
    P: AsRef<Path> + Send + Sync + 'static
{
    /// If you still need to use the handle in normal usage, 
    /// you can wrap it in a getter that returns `Option<&Arc<...>>`.
    pub fn crate_handle(&self) -> Option<&Arc<dyn ReadmeWritingCrateHandle<P>>> {
        self.crate_handle.as_ref()
    }
}
