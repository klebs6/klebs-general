// ---------------- [ File: workspacer-crate/src/pin_wildcard_deps.rs ]
crate::ix!();

#[async_trait]
impl PinWildcardDependencies for CrateHandle {

    type Error = CrateError;

    /// Pins wildcard dependencies in this crate's Cargo.toml to versions found in the lock file.
    ///
    ///  - If multiple distinct versions exist for a crate, picks the highest and logs a warning.
    ///  - If no version is found in lock, logs a warning and leaves it as "*".
    ///  - Writes the updated Cargo.toml in-place.
    async fn pin_wildcard_dependencies(
        &self,
        lock_versions: &BTreeMap<String, BTreeSet<cargo_lock::Version>>,
    ) -> Result<(), CrateError> {
        // Delegates to cargo_toml_handle's method
        self.cargo_toml_handle()
            .pin_wildcard_dependencies(lock_versions)
            .await
            // Convert `CargoTomlError` to `CrateError` if needed
            .map_err(|toml_err| CrateError::CargoTomlError(toml_err))
    }
}
