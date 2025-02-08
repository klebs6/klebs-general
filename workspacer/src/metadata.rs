// ---------------- [ File: src/metadata.rs ]
crate::ix!();

#[async_trait]
impl GetCargoMetadata for Workspace {

    type Error = WorkspaceError;

    /// Helper method to get cargo metadata asynchronously
    async fn get_cargo_metadata(&self) -> Result<Metadata, Self::Error> {
        let path = self.as_ref().to_path_buf();
        let metadata = task::spawn_blocking(move || {
            MetadataCommand::new()
                .current_dir(&path)
                .exec()
                .map_err(|e| CargoMetadataError::MetadataError { error: e.into() })
        })
        .await
        .map_err(|e| TokioError::JoinError { join_error: e.into() })??;
        Ok(metadata)
    }
}
