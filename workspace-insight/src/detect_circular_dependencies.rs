crate::ix!();

impl Workspace {

    /// Detects circular dependencies in the workspace by leveraging `cargo metadata`.
    pub async fn detect_circular_dependencies(&self) -> Result<(), WorkspaceError> {
        match self.get_cargo_metadata().await {

            // No circular dependencies detected if metadata is fetched successfully.
            Ok(_) => Ok(()),

            Err(WorkspaceError::CargoMetadataError(CargoMetadataError::MetadataError { error: ref e }))
                if e.to_string().contains("cyclic package dependency") =>
                {
                    // If `cargo metadata` reported a cyclic dependency, return a user-friendly error.
                    Err(WorkspaceError::CargoMetadataError(CargoMetadataError::CircularDependency))
                }

            // Propagate other errors.
            Err(e) => Err(e),
        }
    }
}
