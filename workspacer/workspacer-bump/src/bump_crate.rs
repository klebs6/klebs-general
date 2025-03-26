// ---------------- [ File: workspacer-bump/src/bump_crate.rs ]
crate::ix!();

#[async_trait]
impl<T> Bump for T
where
    T: ValidateIntegrity<Error = CrateError>
        + HasCargoToml
        + HasCargoTomlPathBuf<Error = CrateError>
        + AsRef<Path>
        + Send
        + Sync,
{
    type Error = CrateError;

    async fn bump(&mut self, release: ReleaseType) -> Result<(), Self::Error> {
        trace!("Entered Bump::bump with release={:?}", release);

        // A) Lock briefly just to *read* the old version
        let old_version_str = {
            let cargo_toml_arc = self.cargo_toml();
            let guard = cargo_toml_arc.lock().await;

            // read old version as a string, BUT unify missing file => IoError
            let ver = match guard.version() {
                Err(CargoTomlError::FileNotFound { missing_file }) => {
                    error!("bump(): mapping 'FileNotFound' to CrateError::IoError");
                    return Err(CrateError::IoError {
                        io_error: Arc::new(std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            format!("Cargo.toml not found at {}", missing_file.display())
                        )),
                        context: format!("Cannot read Cargo.toml at {:?}", missing_file),
                    });
                }
                Err(e) => {
                    error!("bump(): got CargoTomlError => returning CrateError::CargoTomlError({:?})", e);
                    return Err(CrateError::CargoTomlError(e));
                }
                Ok(ver_ok) => ver_ok,
            };
            let ver_str = ver.to_string();
            debug!("Current version (before bump) is '{}', about to do integrity check", ver_str);
            ver_str
        };

        // B) Now do sabotage check and forced re-parse from disk
        trace!("Validating crate integrity before proceeding with bump");
        self.validate_integrity().await?;

        // C) Re-lock to do the actual bump patch
        let cargo_toml_path;
        let new_version_str;
        {
            let cargo_toml_arc = self.cargo_toml();
            let mut guard = cargo_toml_arc.lock().await;

            // parse the old version again (fresh)
            let mut old_ver = match guard.version() {
                Err(CargoTomlError::FileNotFound { missing_file }) => {
                    error!("bump(): second read => mapping 'FileNotFound' to IoError");
                    return Err(CrateError::IoError {
                        io_error: Arc::new(std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            format!("Cargo.toml not found at {}", missing_file.display())
                        )),
                        context: format!("Cannot re-read Cargo.toml at {:?}", missing_file),
                    });
                }
                Err(e) => {
                    error!("bump(): second read => CargoTomlError => returning CrateError::CargoTomlError({:?})", e);
                    return Err(CrateError::CargoTomlError(e));
                }
                Ok(ver_ok) => ver_ok,
            };
            let old_clone = old_ver.clone();

            // apply the release
            release.apply_to(&mut old_ver);

            // preserve the old build metadata
            old_ver.build = old_clone.build.clone();

            // Overwrite in-memory
            let bumped = old_ver.to_string();
            {
                let pkg = guard.get_package_section_mut()?;
                // same code as before for setting the version in the table
                if let Some(tbl) = pkg.as_table_mut() {
                    tbl.insert("version".to_owned(), toml::Value::String(bumped.clone()));
                    debug!("Set `package.version` to '{}'", bumped);
                } else {
                    error!("`package` section was not a TOML table; cannot set version!");
                    return Err(CrateError::CouldNotSetPackageVersionBecausePackageIsNotATable);
                }
            }

            cargo_toml_path = self.as_ref().join("Cargo.toml");
            new_version_str = bumped;
        } // drop lock

        // D) Finally save to disk outside the lock
        {
            let cargo_toml_arc = self.cargo_toml();
            let mut guard = cargo_toml_arc.lock().await;
            guard.save_to_disk().await?;
        }

        info!(
            "Successfully bumped crate at {:?} from {} to {}",
            cargo_toml_path, old_version_str, new_version_str
        );
        Ok(())
    }
}

#[cfg(test)]
mod test_bump_crate_handle_with_mock {
    use super::*;

    #[traced_test]
    async fn test_bump_patch_ok_with_mock() {
        // 1) Create a fully valid mock crate that simulates everything present
        let mock_crate = MockCrateHandle::fully_valid_config()
            .to_builder()
            .crate_name("patch_ok") // name is optional, but helpful for logs
            .build()
            .unwrap();

        // 2) Wrap it in Arc<AsyncMutex> so it implements CrateHandleInterface in your async code
        let arc_handle = Arc::new(AsyncMutex::new(mock_crate));

        // 3) Perform the patch bump
        {
            let mut guard = arc_handle.lock().await;
            guard.bump(ReleaseType::Patch).await
                 .expect("Expected patch bump to succeed in a fully-valid mock");
        }

        // 4) Confirm the new version in-memory (no real Cargo.toml on disk!)
        {
            let guard = arc_handle.lock().await;
            let new_ver = guard.version().expect("Should parse bumped version from mock");
            assert_eq!(new_ver.to_string(), "1.2.4", "Mock crate's version should become 1.2.4");
        }
    }

    #[traced_test]
    async fn test_bump_fails_if_readme_missing() {
        // 1) Create a mock crate that simulates a missing README.md
        let mock_crate = MockCrateHandle::missing_readme_config();

        let arc_handle = Arc::new(AsyncMutex::new(mock_crate));

        // 2) Attempt to bump => should fail because integrity check sees no README
        let bump_result = {
            let mut guard = arc_handle.lock().await;
            guard.bump(ReleaseType::Minor).await
        };

        // 3) Confirm it fails with the expected error
        match bump_result {
            Err(crate_error) => {
                println!("Got expected error: {:?}", crate_error);
                assert!(format!("{:?}", crate_error).contains("README.md"),
                    "Error should mention missing README file");
            }
            Ok(_) => panic!("Expected missing README to cause a bump failure"),
        }
    }
}
