// ---------------- [ File: workspacer-pin/src/toml_pin_wildcard_deps.rs ]
crate::ix!();

pub type LockVersionMap = BTreeMap<String, BTreeSet<SemverVersion>>;

#[async_trait]
pub trait PinWildcardDependencies {
    type Error;
    async fn pin_wildcard_dependencies(
        &mut self,
        lock_versions: &LockVersionMap,
    ) -> Result<(), Self::Error>;
}

/* =========================== 5) ADDITIONAL CHANGE TO FIX E0599 ===========================
 *
 * We must ensure that `H` (the type in the Workspace) actually
 * implements `PinWildcardDependencies`, or that `MutexGuard<'_, H>`
 * can call `pin_wildcard_dependencies(...)`. Similarly, our
 * `dyn CargoTomlInterface` must implement `PinWildcardDependencies`.
 *
 * Below are two blanket implementations:
 *
 *  (A) For `dyn CargoTomlInterface`: If your `CargoTomlInterface` trait
 *      has the same methods as `CargoToml` (e.g. `document_clone()`,
 *      `write_document_back()`, etc.), we can do a direct blanket
 *      impl. 
 *
 *  (B) For `H: PinWildcardDependencies<Error = CrateError>` in the 
 *      Workspace, so that `MutexGuard<'_, H>` has `pin_wildcard_dependencies(...)`.
 */

// (A) Implement `PinWildcardDependencies` for any `dyn CargoTomlInterface` that can do I/O 
//     similarly to `CargoToml`. Adjust if your trait methods differ in name/signature.
#[async_trait]
impl PinWildcardDependencies for CargoToml {
    type Error = CargoTomlError;

    async fn pin_wildcard_dependencies(
        &mut self,
        lock_versions: &LockVersionMap,
    ) -> Result<(), Self::Error> {
        trace!("Reading original Cargo.toml from {:?}", self.as_ref());
        let mut doc = self.document_clone().await?;
        pin_wildcards_in_doc(&mut doc, lock_versions, self).await?;
        self.write_document_back(&doc).await?;
        info!("pin_wildcard_dependencies: updated {:?}", self.as_ref());
        Ok(())
    }
}
