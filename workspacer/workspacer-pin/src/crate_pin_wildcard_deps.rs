// ---------------- [ File: workspacer-pin/src/crate_pin_wildcard_deps.rs ]
crate::ix!();

#[async_trait]
impl PinWildcardDependencies for CrateHandle {
    type Error = CrateError;

    async fn pin_wildcard_dependencies(
        &mut self,
        lock_versions: &LockVersionMap,
    ) -> Result<(), CrateError> {
        trace!(
            "pin_wildcard_dependencies: starting for CrateHandle at path={:?}",
            self.as_ref()
        );

        // Store the Arc so it isn't dropped before we're done using guard
        let cargo_toml_arc = self.cargo_toml_direct();
        let cargo_toml_path = {
            let guard = cargo_toml_arc.lock().unwrap();
            let path = guard.as_ref().to_path_buf();
            trace!("pin_wildcard_dependencies: got Cargo.toml path={:?}", path);
            path
        };

        // Load CargoToml from that path (no guard held across .await)
        let mut ephemeral = match CargoToml::new(&cargo_toml_path).await {
            Ok(ctoml) => {
                trace!(
                    "pin_wildcard_dependencies: successfully opened CargoToml at {:?}",
                    cargo_toml_path
                );
                ctoml
            }
            Err(e) => {
                error!(
                    "pin_wildcard_dependencies: cannot open CargoToml at {:?}: {:?}",
                    cargo_toml_path, e
                );
                return Err(e.into());
            }
        };

        // Perform async pinning on the ephemeral object
        ephemeral.pin_wildcard_dependencies(lock_versions).await?;
        trace!(
            "pin_wildcard_dependencies: ephemeral CargoToml pinned successfully at path={:?}",
            cargo_toml_path
        );

        // Now overwrite the in-memory CargoToml under lock
        {
            let mut guard = cargo_toml_arc.lock().unwrap();
            *guard = ephemeral;
            debug!(
                "pin_wildcard_dependencies: replaced in-memory CargoToml at path={:?}",
                cargo_toml_path
            );
        }

        info!(
            "pin_wildcard_dependencies: finished for CrateHandle at path={:?}",
            self.as_ref()
        );
        Ok(())
    }
}

#[async_trait]
impl PinAllWildcardDependencies for CrateHandle {
    type Error = CrateError;

    async fn pin_all_wildcard_dependencies(&mut self) -> Result<(), Self::Error> {
        // 1) We need to build a LockVersionMap somehow.
        //    For single-crate, you can do the same build_lock_versions step
        //    that you do for a workspace, or a simpler "only read local lockfile"
        //    approach. Let's assume you have a function that does it:
        let lock_versions = build_lock_versions(self).await?;

        // 2) Then call the lower-level .pin_wildcard_dependencies():
        self.pin_wildcard_dependencies(&lock_versions).await
    }
}


#[cfg(test)]
mod test_pin_all_wildcard_dependencies_for_cratehandle {
    use super::*;
    use tracing::{info, debug, trace};
    use tempfile::tempdir;
    use std::fs;

    #[traced_test]
    async fn pins_star_in_single_crate() {
        info!("Starting test_pin_all_wildcard_dependencies_for_cratehandle::pins_star_in_single_crate");

        // We'll create an ephemeral directory so we don't rely on any real on-disk cargo project.
        let temp = tempdir().expect("failed to create tempdir");
        let temp_path = temp.path().to_path_buf();

        // 1) Create a minimal Cargo.toml with a wildcard dependency for testing
        let cargo_toml_contents = r#"
            [package]
            name = "fakecrate"
            version = "0.1.0"
            
            [dependencies]
            anyhow = "*"
        "#;

        let cargo_toml_path = temp_path.join("Cargo.toml");
        fs::write(&cargo_toml_path, cargo_toml_contents).expect("failed to write Cargo.toml");

        // 2) (Optional) Create a minimal Cargo.lock with a pinned version for "anyhow"
        //    This helps test that wildcard gets pinned from the lock.
        let cargo_lock_contents = r#"
            [[package]]
            name = "anyhow"
            version = "1.0.68"
            source = "registry+https://github.com/rust-lang/crates.io-index"
        "#;

        let cargo_lock_path = temp_path.join("Cargo.lock");
        fs::write(&cargo_lock_path, cargo_lock_contents).expect("failed to write Cargo.lock");

        // 3) Create a CrateHandle pointing to this ephemeral "crate"
        let mut handle = CrateHandle::new(&temp_path).await.unwrap();

        // 4) Now call pin_all_wildcard_dependencies
        //    This should read the Cargo.lock, find "anyhow" = 1.0.68, and pin it.
        let result = handle.pin_all_wildcard_dependencies().await;
        assert!(result.is_ok(), "pin_all_wildcard_dependencies failed: {:?}", result);

        // 5) Verify that Cargo.toml was updated (re-read it)
        //    Depending on your exact design, the pinned version should have replaced "*"
        let pinned_cargo_toml = CargoToml::new(&cargo_toml_path)
            .await
            .expect("failed to re-open pinned Cargo.toml");
        let pinned_doc = pinned_cargo_toml.document_clone().await
            .expect("failed to clone pinned Cargo.toml doc");

        // Double-check that `[dependencies].anyhow` is now "1.0.68" in the pinned doc.
        let deps_table = pinned_doc.as_table().get("dependencies").expect("no [dependencies]");
        let pinned_anyhow = deps_table.get("anyhow").expect("missing anyhow entry");
        let pinned_version = pinned_anyhow.as_str().unwrap();
        assert_eq!(pinned_version, "1.0.68", "Expected the wildcard to be pinned to 1.0.68");

        debug!("test_pin_all_wildcard_dependencies_for_cratehandle::pins_star_in_single_crate passed");
    }
}
