// ---------------- [ File: workspacer-readme-writer/src/readme_writer_request.rs ]
crate::ix!();

#[derive(Serialize,Deserialize,Debug,Builder,Getters,Clone)]
#[builder(setter(into))]
#[getset(get = "pub")]
pub struct AiReadmeWriterRequest<P> 
where
    P: AsRef<Path> + Send + Sync + 'static,
{
    #[serde(with = "crate_handle_serde")]
    crate_handle:                               Arc<AsyncMutex<dyn ReadmeWritingCrateHandle<P>>>,
    crate_name:                                 String,
    version:                                    semver::Version,
    consolidated_crate_interface:               ConsolidatedCrateInterface,
    maybe_cargo_toml_package_authors:           Option<Vec<String>>,
    maybe_cargo_toml_rust_edition:              Option<String>,
    maybe_cargo_toml_license:                   Option<String>,
    maybe_cargo_toml_crate_repository_location: Option<String>,
}

impl<P> AiReadmeWriterRequest<P> 
where
    P: AsRef<Path> + Send + Sync + 'static,
{
    pub async fn async_try_from<H>(
        handle: Arc<AsyncMutex<H>>,
    ) -> Result<Self, AiReadmeWriterError>
    where
        H: ReadmeWritingCrateHandle<P>, // the super-trait
    {

        // 2) We'll store it as Arc<dyn ReadmeWritingCrateHandle<P>>.
        let crate_handle_obj: Arc<AsyncMutex<dyn ReadmeWritingCrateHandle<P>>> = handle.clone();

        let guard = handle.lock().await;

        let consolidation_opts = ConsolidationOptions::new().with_docs().with_fn_bodies();

        // 1) We can call name(), version(), etc. because H: CrateHandleInterface<P>.
        let crate_name = guard.name().to_string();
        let version = guard
            .version()
            .expect("expected a valid version in the crate");

        let consolidated_crate_interface = guard
            .consolidate_crate_interface(&consolidation_opts)
            .await?;

        // 3) Now we do a short synchronous read from CargoToml (like get_package_authors),
        //    but we must not hold any lock across await, so keep it “direct.”
        let direct_authors = {
            let cargo_toml = guard.cargo_toml();
            let guard = cargo_toml.lock().await;
            guard.get_package_authors()?
        };
        let maybe_cargo_toml_package_authors = if direct_authors.is_some() {
            direct_authors
        } else {
            // do fallback or None
            None
        };

        let direct_edition = {
            let cargo_toml = guard.cargo_toml();
            let guard = cargo_toml.lock().await;
            guard.get_rust_edition()?
        };
        let maybe_cargo_toml_rust_edition = if direct_edition.is_some() {
            direct_edition
        } else {
            None
        };

        let direct_license = {
            let cargo_toml = guard.cargo_toml();
            let guard = cargo_toml.lock().await;
            guard.get_license_type()?
        };
        let maybe_cargo_toml_license = if direct_license.is_some() {
            direct_license
        } else {
            None
        };

        let direct_repo = {
            let cargo_toml = guard.cargo_toml();
            let guard = cargo_toml.lock().await;
            guard.get_crate_repository_location()?
        };
        let maybe_cargo_toml_crate_repository_location = if direct_repo.is_some() {
            direct_repo
        } else {
            None
        };

        Ok(Self {
            crate_handle: crate_handle_obj,
            crate_name,
            version,
            consolidated_crate_interface,
            maybe_cargo_toml_package_authors,
            maybe_cargo_toml_rust_edition,
            maybe_cargo_toml_license,
            maybe_cargo_toml_crate_repository_location,
        })
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
        std::borrow::Cow::Owned(format!("{}-ai-readme-request", self.crate_name))
    }
}

impl<P> HasAssociatedOutputName for AiReadmeWriterRequest<P> 
    where
        P: AsRef<Path> + Send + Sync + 'static,
{
    fn associated_output_name(&self) -> std::borrow::Cow<'_, str> {
        std::borrow::Cow::Owned(format!("{}-ai-generated-readme", self.crate_name()))
    }
}
