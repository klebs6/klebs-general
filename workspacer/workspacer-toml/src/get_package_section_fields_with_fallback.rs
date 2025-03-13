// ---------------- [ File: workspacer-toml/src/get_package_section_fields_with_fallback.rs ]
// We use the helper function `field_with_fallback` to remove repetition
// in each fallback trait implementation.
crate::ix!();

#[async_trait]
impl GetPackageAuthorsOrFallback for CargoToml {
    type Error = CargoTomlError;

    async fn get_package_authors_or_fallback(&self) -> Result<Option<Vec<String>>, Self::Error> {
        field_with_fallback(
            self.get_package_authors()?, // direct authors
            "authors",
            |cfg: &WorkspacerConfig| cfg.authors().clone(),
        ).await
    }
}

#[async_trait]
impl GetRustEditionOrFallback for CargoToml {
    type Error = CargoTomlError;

    async fn get_rust_edition_or_fallback(&self) -> Result<Option<String>, Self::Error> {
        field_with_fallback(
            self.get_rust_edition()?, // direct edition
            "edition",
            |cfg: &WorkspacerConfig| cfg.rust_edition().clone(),
        ).await
    }
}

#[async_trait]
impl GetLicenseTypeOrFallback for CargoToml {
    type Error = CargoTomlError;

    async fn get_license_type_or_fallback(&self) -> Result<Option<String>, Self::Error> {
        field_with_fallback(
            self.get_license_type()?, // direct license
            "license",
            |cfg: &WorkspacerConfig| cfg.license().clone(),
        ).await
    }
}

#[async_trait]
impl GetCrateRepositoryLocationOrFallback for CargoToml {
    type Error = CargoTomlError;

    async fn get_crate_repository_location_or_fallback(&self) -> Result<Option<String>, Self::Error> {
        field_with_fallback(
            self.get_crate_repository_location()?, // direct repository
            "repository",
            |cfg: &WorkspacerConfig| cfg.repository().clone(),
        ).await
    }
}

/// A shared helper function that tries a direct CargoToml value first,
/// then attempts to load the .workspacer-config fallback if the direct value
/// is missing (`None`).
///
/// - `direct_val`: the direct field value from CargoToml (e.g. `authors`, `edition`, etc.).
/// - `field_name`: a string used only for logging (e.g. `"authors"`, `"repository"`).
/// - `extract_fallback`: a closure that extracts the same field from a loaded `WorkspacerConfig`.
///
/// Returns `Ok(Some(value))` if found from CargoToml or fallback,
/// `Ok(None)` if both are missing, or `Err(CargoTomlError::FallbackError(...))` if
/// fallback loading fails for I/O or parse reasons.
async fn field_with_fallback<T: Debug + Clone>(
    direct_val: Option<T>,
    field_name: &str,
    extract_fallback: impl FnOnce(&WorkspacerConfig) -> Option<T>,
) -> Result<Option<T>, CargoTomlError> {
    // 1) Check direct CargoToml
    if let Some(val) = direct_val {
        debug!("Found `{}` directly in CargoToml, returning it.", field_name);
        return Ok(Some(val));
    }

    debug!(
        "`{}` not found in CargoToml; attempting fallback .workspacer-config.",
        field_name
    );

    // 2) Attempt to load fallback
    match WorkspacerConfig::load_with_fallback().await? {
        Some(cfg) => {
            let fallback_val = extract_fallback(&cfg);
            if let Some(ref v) = fallback_val {
                info!(
                    "Using `{}` from fallback .workspacer-config: {:?}",
                    field_name, v
                );
            } else {
                warn!(
                    "Fallback .workspacer-config is present but no `{}` field found.",
                    field_name
                );
            }
            Ok(fallback_val)
        }
        None => {
            warn!(
                "No local or global .workspacer-config found; no fallback for `{}`.",
                field_name
            );
            Ok(None)
        }
    }
}
