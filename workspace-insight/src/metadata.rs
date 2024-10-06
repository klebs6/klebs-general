crate::ix!();

impl Workspace {

    /// Helper method to get cargo metadata asynchronously
    pub async fn get_cargo_metadata(&self) -> Result<Metadata, WorkspaceError> {
        let path = self.path().to_path_buf();
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
