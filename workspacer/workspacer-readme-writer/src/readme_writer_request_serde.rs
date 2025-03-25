crate::ix!();

pub mod crate_handle_serde {
    use super::*;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    // Bring in `Serialize` and `Deserialize` from the official crate rather than relying on
    // the overshadowed references.
    use ::serde::{Serialize, Deserialize};
    use ::serde::ser::{Serializer, SerializeStruct};
    use ::serde::de::{Deserializer, Visitor, Error as DeError, MapAccess};

    // Adjust these imports to your real crate paths:
    use super::{
        CrateHandle,
        CrateError,
        supertrait::ReadmeWritingCrateHandle,
    };

    /// A simple struct that we will serialize as `{"path": "..."}`
    #[derive(Serialize, Deserialize)]
    struct CrateHandleJsonRepr {
        path: PathBuf,
    }

    /// The function Serde calls when serializing `Arc<dyn ReadmeWritingCrateHandle<P>>`.
    pub fn serialize<S, P>(
        field: &Arc<AsyncMutex<dyn ReadmeWritingCrateHandle<P>>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        P: AsRef<Path> + Send + Sync + 'static,
    {
        let field_clone = field.clone();
        
        let helper = run_async_without_nested_runtime(async move {

            let guard = field_clone.lock().await;

            // Convert the handle to a path
            let path_buf = guard.root_dir_path_buf();

            let helper = CrateHandleJsonRepr { path: path_buf };

            helper
        });

        helper.serialize(serializer)
    }

    /// The function Serde calls when deserializing `Arc<dyn ReadmeWritingCrateHandle<P>>`.
    /// We'll do a "blocking" creation from the path, by calling `CrateHandle::new_sync(...)`.
    /// If your code must truly do async I/O, consider storing the path and deferring creation instead.
    pub fn deserialize<'de, D, P>(
        deserializer: D
    ) -> Result<Arc<AsyncMutex<dyn ReadmeWritingCrateHandle<P>>>, D::Error>
    where
        D: Deserializer<'de>,
        P: AsRef<Path> + Send + Sync + 'static,
    {
        let helper = CrateHandleJsonRepr::deserialize(deserializer)?;

        // We'll call a synchronous constructor. Adjust to your real code:
        match CrateHandle::new_sync(&helper.path) {
            Ok(real_handle) => Ok(Arc::new(AsyncMutex::new(real_handle))),
            Err(e) => Err(DeError::custom(format!("Cannot build CrateHandle: {:?}", e))),
        }
    }
}

#[cfg(test)]
mod crate_handle_serde_tests {
    use super::*;
    use tracing::{trace, info, error};
    use std::sync::Arc;
    use semver::Version;
    use std::path::PathBuf;

    #[traced_test]
    async fn test_serde_with_trait_object_field() -> Result<(), WorkspaceError> {
        trace!("test_serde_with_trait_object_field: starting");

        // 1) Use the mock workspace to create a valid Cargo.toml at a real path
        //    so that `CrateHandle::new(...)` won't fail with "FileNotFound".
        let crate_name = "my_mock_crate";
        let workspace_root = create_mock_workspace(vec![
            CrateConfig::new(crate_name)
                .with_readme()
                .with_src_files()
        ])
        .await?;

        // 2) Build the path to the newly created crate
        let crate_path = workspace_root.join(crate_name);
        trace!("Mock workspace created at: {:?}", workspace_root);
        trace!("Crate path: {:?}", crate_path);

        // 3) Initialize the handle from that real path
        //    so that reading Cargo.toml will succeed.
        let handle = CrateHandle::new(&crate_path)
            .await
            .map_err(|e| {
                error!("CrateHandle::new failed: {:?}", e);
                e
            })?;

        // 4) Wrap it in an Arc<dyn ReadmeWritingCrateHandle<PathBuf>>
        let handle_arc: Arc<AsyncMutex<dyn ReadmeWritingCrateHandle<PathBuf>>> = Arc::new(AsyncMutex::new(handle));

        // 5) Build the AiReadmeWriterRequest
        let original = AiReadmeWriterRequestBuilder::<PathBuf>::default()
            .crate_handle(handle_arc)
            .crate_name(crate_name.to_string())
            .version(Version::parse("1.2.3").unwrap())
            .consolidated_crate_interface(ConsolidatedCrateInterface::new())
            .maybe_cargo_toml_package_authors(Some(vec!["Alice".into(), "Bob".into()]))
            .maybe_cargo_toml_rust_edition(Some("2021".into()))
            .maybe_cargo_toml_license(Some("MIT".into()))
            .maybe_cargo_toml_crate_repository_location(Some("https://github.com/example/repo".into()))
            .build()
            .unwrap();

        trace!("Original request built: {:?}", original);

        // 6) Serialize to JSON
        let json = serde_json::to_string(&original)
            .expect("failed to serialize AiReadmeWriterRequest");
        info!("Serialized JSON: {}", json);

        // 7) Deserialize
        let roundtrip: AiReadmeWriterRequest<PathBuf> =
            serde_json::from_str(&json).expect("failed to deserialize AiReadmeWriterRequest");
        trace!("Deserialized request: {:?}", roundtrip);

        let crate_handle = roundtrip.crate_handle();
        let guard = crate_handle.lock().await;

        // 8) Check that the path is what we expect
        let reconstructed_path = guard.root_dir_path_buf();
        info!("Reconstructed path = {:?}", reconstructed_path);

        // Should match the mock crate path we used
        assert_eq!(reconstructed_path, crate_path);

        trace!("test_serde_with_trait_object_field: finishing successfully");
        Ok(())
    }
}
