// ---------------- [ File: workspacer/src/detect_circular_dependencies.rs ]
crate::ix!();

#[async_trait]
impl<P,H:CrateHandleInterface<P>> DetectCircularDependencies for Workspace<P,H> 
where for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait
{

    type Error = WorkspaceError;

    /// Detects circular dependencies in the workspace by leveraging `cargo metadata`.
    async fn detect_circular_dependencies(&self) -> Result<(), WorkspaceError> {
        match self.get_cargo_metadata().await {

            // No circular dependencies detected if metadata is fetched successfully.
            Ok(_) => Ok(()),

            // Check if the error contains specific cyclic dependency information.
            Err(WorkspaceError::CargoMetadataError(CargoMetadataError::MetadataError { error: ref e }))
                if e.to_string().contains("cyclic package dependency") =>
                {
                    // If `cargo metadata` reported a cyclic dependency, return the expected error.
                    Err(WorkspaceError::CargoMetadataError(CargoMetadataError::CyclicPackageDependency))
                }

            // Propagate other errors.
            Err(e) => Err(e),
        }
    }
}

