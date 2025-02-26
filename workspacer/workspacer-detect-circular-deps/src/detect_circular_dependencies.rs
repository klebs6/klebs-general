// ---------------- [ File: src/detect_circular_dependencies.rs ]
crate::ix!();

#[async_trait]
pub trait DetectCircularDependencies {

    type Error;
    async fn detect_circular_dependencies(&self) -> Result<(), Self::Error>;
}

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

#[cfg(test)]
mod test_detect_circular_dependencies_real {
    use super::*;

    /// 1) No cycle => returns Ok(()).
    #[tokio::test]
    async fn test_no_cycle() {
        // a normal multi-crate workspace with no circular reference
        let crate_a = CrateConfig::new("crate_a").with_src_files();
        let crate_b = CrateConfig::new("crate_b").with_src_files();

        // build the workspace
        let root = create_mock_workspace(vec![crate_a, crate_b])
            .await
            .expect("mock workspace creation should succeed");

        // We didn't create a cycle, so detect_circular_dependencies => Ok
        let ws = Workspace::<PathBuf,CrateHandle>::new(&root).await.expect("create workspace");
        let result = ws.detect_circular_dependencies().await;
        assert!(result.is_ok(), "No cycle => Ok(())");
    }

    /// 2) A local path-based cycle => should yield `CyclicPackageDependency` error.
    ///
    /// We'll do crate_a depends on crate_b, and crate_b depends on crate_a.
    /// Cargo sees that as a cycle.
    #[tokio::test]
    async fn test_cycle_results_in_cyclic_package_dependency_error() {
        let crate_a = CrateConfig::new("cyc_a").with_src_files();
        let crate_b = CrateConfig::new("cyc_b").with_src_files();

        let root = create_mock_workspace(vec![crate_a, crate_b])
            .await
            .expect("mock workspace creation");

        // create cyc: cyc_a depends on cyc_b, cyc_b depends on cyc_a
        let cyc_a_toml = root.join("cyc_a").join("Cargo.toml");
        let cyc_b_toml = root.join("cyc_b").join("Cargo.toml");

        let a_orig = tokio::fs::read_to_string(&cyc_a_toml).await.unwrap();
        let a_new = format!(
            r#"{}
[dependencies]
cyc_b = {{ path = "../cyc_b" }}
"#,
            a_orig
        );
        tokio::fs::write(&cyc_a_toml, a_new).await.unwrap();

        let b_orig = tokio::fs::read_to_string(&cyc_b_toml).await.unwrap();
        let b_new = format!(
            r#"{}
[dependencies]
cyc_a = {{ path = "../cyc_a" }}
"#,
            b_orig
        );
        tokio::fs::write(&cyc_b_toml, b_new).await.unwrap();

        let ws = Workspace::<PathBuf,CrateHandle>::new(&root).await.expect("create workspace ok");

        let result = ws.detect_circular_dependencies().await;
        match result {
            Err(WorkspaceError::CargoMetadataError(CargoMetadataError::CyclicPackageDependency)) => {
                // This is the expected outcome for a cycle
            }
            Ok(_) => {
                panic!("Expected a cycle error, got Ok(())");
            }
            Err(e) => {
                panic!("Expected `CyclicPackageDependency`, got a different error: {:?}", e);
            }
        }
    }

    /// 3) If some other cargo metadata error occurs (like a broken manifest),
    ///    we pass that error through as is. We'll simulate that by messing up a Cargo.toml.
    #[tokio::test]
    async fn test_other_cargo_metadata_error_passes_through() {
        let single = CrateConfig::new("broken_crate").with_src_files();
        let root = create_mock_workspace(vec![single])
            .await
            .expect("mock ws creation should succeed");

        // We'll break the manifest by adding nonsense
        let broken_toml = root.join("broken_crate").join("Cargo.toml");
        let content = tokio::fs::read_to_string(&broken_toml)
            .await
            .expect("Failed to read broken crate Cargo.toml");
        let appended = format!("{}\nnot_valid_toml??=??", content);
        tokio::fs::write(&broken_toml, appended)
            .await
            .expect("Failed to write broken lines to Cargo.toml");

        // Don't unwrap here; we want to see if creation itself fails or if
        // creation succeeds but detect_circular_dependencies fails.
        let ws_result = Workspace::<PathBuf,CrateHandle>::new(&root).await;

        match ws_result {
            Ok(workspace) => {
                // If workspace creation succeeded, next we call detect_circular_dependencies().
                let result = workspace.detect_circular_dependencies().await;
                match result {
                    Err(WorkspaceError::CargoMetadataError(
                            CargoMetadataError::CyclicPackageDependency,
                    )) => {
                        panic!("Should not be a cycle error for a broken toml!");
                    }
                    Err(e) => {
                        // This is the “other cargo metadata error” path you wanted to test.
                        println!("Got a non-cycle cargo-metadata-related error: {e:?}");
                    }
                    Ok(_) => {
                        panic!("We expected an error from detect_circular_dependencies() because the toml is broken!");
                    }
                }
            }
            Err(err) => {
                // If workspace creation itself fails due to the parse error, that’s also an
                // acceptable “non-cycle” cargo-type error. So we check it's not the cycle variant.
                match err {
                    WorkspaceError::CargoMetadataError(
                        CargoMetadataError::CyclicPackageDependency,
                    ) => {
                        panic!("Should not be a cycle error for a broken toml!");
                    }
                    _ => {
                        // Great, we got some other error (like TomlParseError).
                        // That still counts as “an error that isn't a cycle.”
                        println!("Got a non-cycle error at workspace creation time: {err:?}");
                    }
                }
            }
        }
    }

}
