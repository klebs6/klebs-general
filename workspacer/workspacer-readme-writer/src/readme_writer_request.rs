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
    #[tracing::instrument(level = "trace", skip(handle, config))]
    pub async fn async_try_from<H>(
        handle: Arc<AsyncMutex<H>>,
        config: &ReadmeWriterConfig
    ) -> Result<Self, AiReadmeWriterError>
    where
        H: ReadmeWritingCrateHandle<P>,
    {
        trace!("Beginning AiReadmeWriterRequest::async_try_from with user-supplied config.");

        let crate_handle_obj: Arc<AsyncMutex<dyn ReadmeWritingCrateHandle<P>>> = handle.clone();
        let guard = handle.lock().await;

        // 1) Build initial consolidation options from config
        let mut consolidation_opts = ConsolidationOptions::new();

        if !config.skip_docs() {
            consolidation_opts = consolidation_opts.with_docs();
        }
        if !config.skip_fn_bodies() {
            consolidation_opts = consolidation_opts.with_fn_bodies();
        }
        if *config.include_test_items() {
            consolidation_opts = consolidation_opts.with_test_items();
        }
        if *config.include_private_items() {
            consolidation_opts = consolidation_opts.with_private_items();
        }

        trace!("Initial consolidation_opts = {:?}", consolidation_opts);

        // 2) Get crate name/version
        let crate_name = guard.name().to_string();
        let version = guard
            .version()
            .map_err(|e| AiReadmeWriterError::CrateError(e))?;

        // 3) Consolidate
        let mut cci = guard
            .consolidate_crate_interface(&consolidation_opts)
            .await
            .map_err(AiReadmeWriterError::CrateError)?;

        let mut cci_str = cci.to_string();
        trace!(
            "Consolidated crate interface for {} => length = {}",
            crate_name,
            cci_str.len()
        );

        // 4) If there's a max length, check and fallback if needed
        if let Some(max_len) = config.max_interface_length() {
            if cci_str.len() > *max_len {
                warn!(
                    "Crate interface length {} exceeds max {}; applying fallback.",
                    cci_str.len(),
                    max_len
                );
                // fallback => skip docs + skip fn bodies, but preserve test/private toggles
                let mut fallback_opts = ConsolidationOptions::new();
                if *config.include_test_items() {
                    fallback_opts = fallback_opts.with_test_items();
                }
                if *config.include_private_items() {
                    fallback_opts = fallback_opts.with_private_items();
                }

                let fallback_cci = guard
                    .consolidate_crate_interface(&fallback_opts)
                    .await
                    .map_err(AiReadmeWriterError::CrateError)?;
                let fallback_str = fallback_cci.to_string();
                trace!("Fallback interface length = {}", fallback_str.len());

                cci = fallback_cci;
                cci_str = fallback_str;
            }
        }
        info!("Final interface length for {} is {}", crate_name, cci_str.len());

        // 5) Gather cargo-toml data
        let authors_opt = {
            let cargo_toml = guard.cargo_toml();
            let guard2 = cargo_toml.lock().await;
            guard2.get_package_authors().map_err(|e| AiReadmeWriterError::CargoTomlError(e))?
        };

        let edition_opt = {
            let cargo_toml = guard.cargo_toml();
            let guard2 = cargo_toml.lock().await;
            guard2.get_rust_edition().map_err(|e| AiReadmeWriterError::CargoTomlError(e))?
        };

        let license_opt = {
            let cargo_toml = guard.cargo_toml();
            let guard2 = cargo_toml.lock().await;
            guard2.get_license_type().map_err(|e| AiReadmeWriterError::CargoTomlError(e))?
        };

        let repo_opt = {
            let cargo_toml = guard.cargo_toml();
            let guard2 = cargo_toml.lock().await;
            guard2.get_crate_repository_location().map_err(|e| AiReadmeWriterError::CargoTomlError(e))?
        };

        Ok(Self {
            crate_handle: crate_handle_obj,
            crate_name,
            version,
            consolidated_crate_interface: cci,
            maybe_cargo_toml_package_authors: authors_opt,
            maybe_cargo_toml_rust_edition: edition_opt,
            maybe_cargo_toml_license: license_opt,
            maybe_cargo_toml_crate_repository_location: repo_opt,
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
