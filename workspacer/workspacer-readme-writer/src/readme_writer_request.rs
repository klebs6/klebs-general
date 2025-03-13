// ---------------- [ File: workspacer-readme-writer/src/readme_writer_request.rs ]
crate::ix!();

#[derive(Builder, Getters, Clone)]
#[builder(setter(into))]
#[getset(get = "pub")]
pub struct AiReadmeWriterRequest<P> {
    crate_handle: Arc<dyn ReadmeWritingCrateHandle<P>>,
    crate_name:   String,
    version:      semver::Version,
    consolidated_crate_interface:     ConsolidatedCrateInterface,
    maybe_cargo_toml_package_authors: Option<Vec<String>>,
    maybe_cargo_toml_rust_edition:    Option<String>,
    maybe_cargo_toml_license:         Option<String>,
    maybe_cargo_toml_crate_repository_location: Option<String>,
}

impl<P> AiReadmeWriterRequest<P> {
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

        // 2) We'll store it as Arc<dyn ReadmeWritingCrateHandle<P>>.
        let crate_handle_obj: Arc<dyn ReadmeWritingCrateHandle<P>> = handle;

        // 3) Now we do a short synchronous read from CargoToml (like get_package_authors),
        //    but we must not hold any lock across await, so keep it “direct.”
        let direct_authors = {
            let cargo_toml = crate_handle_obj.cargo_toml();
            let mut guard = cargo_toml.lock().unwrap();
            guard.get_package_authors()?
        };
        let maybe_cargo_toml_package_authors = if direct_authors.is_some() {
            direct_authors
        } else {
            // do fallback or None
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

impl<P> std::fmt::Display for AiReadmeWriterRequest<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AiReadmeWriterRequest for crate: {}", self.crate_name)
    }
}

impl<P> Named for AiReadmeWriterRequest<P> {
    fn name(&self) -> std::borrow::Cow<'_, str> {
        std::borrow::Cow::Owned(format!("{}-ai-readme-request", self.crate_name))
    }
}
