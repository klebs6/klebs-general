crate::ix!();

#[derive(Getters,Debug,Clone)]
#[getset(get="pub")]
pub struct AiReadmeWriterRequest {
    crate_name:                                 String,
    version:                                    semver::Version,
    consolidated_crate_interface:               ConsolidatedCrateInterface,
    maybe_cargo_toml_package_authors:           Option<Vec<String>>,
    maybe_cargo_toml_rust_edition:              Option<String>,
    maybe_cargo_toml_license:                   Option<String>,
    maybe_cargo_toml_crate_repository_location: Option<String>,
}

impl AiReadmeWriterRequest 
{
    pub async fn async_try_from<P,H>(x: &H) -> Result<Self,ReadmeWriterExecutionError> 
    where     
        for<'x> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'x,
        H: CrateHandleInterface<P> 
            + ConsolidateCrateInterface
            + Sync 
            + Send 
            + 'static,
    {

        let consolidation_opts = ConsolidationOptions::new()
            .with_docs()
            .with_fn_bodies();

        let toml = x.cargo_toml();

        Ok(Self {
            crate_name:                                 x.name().to_string(),
            version:                                    x.version().expect("we expect our toml to have a version for this to work"),
            consolidated_crate_interface:               x.consolidate_crate_interface(&consolidation_opts).await?,
            maybe_cargo_toml_package_authors:           toml.get_package_authors_or_fallback().await?,
            maybe_cargo_toml_rust_edition:              toml.get_rust_edition_or_fallback().await?,
            maybe_cargo_toml_license:                   toml.get_license_type_or_fallback().await?,
            maybe_cargo_toml_crate_repository_location: toml.get_crate_repository_location_or_fallback().await?,
        })
    }
}

impl std::fmt::Display for AiReadmeWriterRequest {

    fn fmt(&self, x: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(x,"ai-toml-writer-request-for-{}",&self.crate_name())
    }
}

impl Named for AiReadmeWriterRequest {

    fn name(&self) -> Cow<'_,str> {
        Cow::Owned(format!("{}-ai-toml-writer-request",&self.crate_name()))
    }
}
