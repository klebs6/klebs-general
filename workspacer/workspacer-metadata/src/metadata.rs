// ---------------- [ File: workspacer-metadata/src/metadata.rs ]
crate::ix!();

/// 1) Extend `GetCargoMetadata` to also handle a single crate (e.g., `CrateHandle`).
#[async_trait]
pub trait GetCargoMetadata {
    type Error;
    async fn get_cargo_metadata(&self) -> Result<Metadata, Self::Error>;
}

/// 2) Implement `GetCargoMetadata` for your `Workspace` (already done):
#[async_trait]
impl<P,H> GetCargoMetadata for Workspace<P,H>
where
    H: CrateHandleInterface<P>,
    for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait,
{
    type Error = WorkspaceError;

    /// We run `cargo metadata` from the workspace root directory
    async fn get_cargo_metadata(&self) -> Result<Metadata, Self::Error> {
        let path = self.as_ref().to_path_buf();

        // We spawn a blocking task because cargo_metadata is a blocking I/O operation
        let metadata = tokio::task::spawn_blocking(move || {
            MetadataCommand::new()
                .current_dir(&path)
                .exec()
                .map_err(|e| CargoMetadataError::MetadataError { error: e.into() })
        })
        .await
        .map_err(|e| WorkspaceError::TokioError(TokioError::JoinError { join_error: e.into() }))??;

        Ok(metadata)
    }
}

/// 3) For single crates (`CrateHandle`), we do a similar approach, but we run
///    `cargo metadata` from that crate’s directory (the parent of Cargo.toml),
///    or pass `--manifest-path` if you prefer. 
///
#[async_trait]
impl GetCargoMetadata for CrateHandle {
    type Error = CrateError;

    async fn get_cargo_metadata(&self) -> Result<Metadata, Self::Error> {
        // 1) We'll lock the cargo_toml to locate the exact path to Cargo.toml
        let cargo_toml_arc = self.cargo_toml_direct();
        let cargo_toml_guard = cargo_toml_arc.lock().await;
        let cargo_toml_path = cargo_toml_guard.as_ref().to_path_buf(); // e.g. /some/dir/Cargo.toml

        // 2) We can pass `--manifest-path` to cargo metadata, or `current_dir()`.
        //    We'll use `--manifest-path` for clarity:
        let cargo_toml_path2 = cargo_toml_path.clone();

        // 3) Spawn blocking to run cargo_metadata
        let metadata = tokio::task::spawn_blocking(move || {
            let mut cmd = MetadataCommand::new();
            cmd.manifest_path(cargo_toml_path2);
            cmd.exec().map_err(|e| CargoMetadataError::MetadataError { error: e.into() })
        })
        .await??; // Double question marks to unwrap spawn error and the map_err

        Ok(metadata)
    }
}
