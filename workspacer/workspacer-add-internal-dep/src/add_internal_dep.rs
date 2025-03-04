crate::ix!();

// ---------------------- [ File: workspacer-add-internal-dep/src/lib.rs ] ----------------------
/// A trait for adding an internal dep from one crate to another.
///
#[async_trait]
pub trait AddInternalDependency {
    type Error;
    /// Attempt to add a dependency on `dep_crate` to the `target_crate`.
    /// - Edits the target crate’s Cargo.toml to add `[dependencies] dep_name = { path=... }`
    /// - Optionally updates `src/imports.rs` with `pub(crate) use ...`
    async fn add_internal_dependency(
        &self,
        target_crate: &CrateHandle,
        dep_crate:    &CrateHandle,
    ) -> Result<(), Self::Error>;
}

#[async_trait]
impl<P,H:CrateHandleInterface<P>> AddInternalDependency for Workspace<P,H> 
where for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait
{
    type Error = WorkspaceError;

    async fn add_internal_dependency(
        &self,
        target_crate: &CrateHandle,
        dep_crate:    &CrateHandle,
    ) -> Result<(), WorkspaceError> {

        info!("Adding dependency: target={} depends on {}", target_crate.name(), dep_crate.name());
        
        // 1) Edit target_crate’s Cargo.toml to add the path dep
        // For demonstration, we do naive string insertion. In real code,
        // parse the TOML, then re-serialize (via toml_edit or similar).
        let target_path = target_crate.as_ref().to_path_buf();
        let cargo_toml_path = target_path.join("Cargo.toml");
        let cargo_contents = tokio::fs::read_to_string(&cargo_toml_path).await
            .map_err(|io_err| WorkspaceError::IoError {
                context: format!("reading Cargo.toml in '{}'", target_crate.name()),
                io_error: Arc::new(io_err),
            })?;
        
        // We'll just do a hacky append:
        let new_dep = format!(
r#"
[dependencies.{}]
path = "{}"
"#, dep_crate.name(), dep_crate.as_ref().display());
        
        let updated = format!("{}\n{}", cargo_contents, new_dep);

        // Overwrite
        tokio::fs::write(&cargo_toml_path, updated).await.map_err(|io_err| WorkspaceError::IoError {
            context: format!("writing Cargo.toml in '{}'", target_crate.name()),
            io_error: Arc::new(io_err),
        })?;

        // 2) Optionally update src/imports.rs
        let imports_rs = target_path.join("src").join("imports.rs");
        if imports_rs.exists() {
            let mut existing = match tokio::fs::read_to_string(&imports_rs).await {
                Ok(txt) => txt,
                Err(e) => {
                    warn!("Could not read imports.rs from {}: {:?}", imports_rs.display(), e);
                    String::new()
                }
            };
            let reexport = format!("pub(crate) use {}::*;\n", dep_crate.name());
            existing.push_str(&reexport);
            tokio::fs::write(&imports_rs, existing).await.map_err(|io_err| WorkspaceError::IoError {
                context: format!("writing imports.rs in '{}'", target_crate.name()),
                io_error: Arc::new(io_err),
            })?;
        } else {
            info!("No src/imports.rs found in {}; skipping reexport step", target_crate.name());
        }

        info!("Successfully added internal dep from '{}' to '{}'", target_crate.name(), dep_crate.name());
        Ok(())
    }
}
