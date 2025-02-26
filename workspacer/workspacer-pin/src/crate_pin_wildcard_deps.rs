// ---------------- [ File: src/crate_pin_wildcard_deps.rs ]
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
        lock_versions: &LockVersionMap,
    ) -> Result<(), CrateError> {

        //info!("Delegating to cargo_toml_handle for wildcard pinning");

        // Delegates to cargo_toml_handle's method
        self.cargo_toml_handle()
            .pin_wildcard_dependencies(lock_versions)
            .await
            // Convert `CargoTomlError` to `CrateError` if needed
            .map_err(|toml_err| CrateError::CargoTomlError(toml_err))
    }
}

#[async_trait]
impl PinAllWildcardDependencies for CrateHandle {
    type Error = CrateError;

    /// If a user calls `pin_all_wildcard_dependencies` on a single crate,
    /// we just read the local `Cargo.lock` and pin ourselves.
    async fn pin_all_wildcard_dependencies(&self) -> Result<(), Self::Error> {
        info!("pin_all_wildcard_dependencies called for single crate at {:?}", self.as_ref());
        // 1) Build the lock_versions from local Cargo.lock
        let lock_versions = build_lock_versions(self).await?;
        // 2) Then pin our crate
        self.pin_wildcard_dependencies(&lock_versions).await
    }
}
