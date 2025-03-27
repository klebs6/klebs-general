// ---------------- [ File: workspacer-publish/src/try_publish_crate.rs ]
crate::ix!();

#[async_trait]
pub trait TryPublish {
    type Error;
    async fn try_publish(
        &self,
        dry_run: bool,
    ) -> Result<(), Self::Error>;
}

#[async_trait]
impl TryPublish for CrateHandle {
    type Error = CrateError;

    async fn try_publish(&self, dry_run: bool) -> Result<(), Self::Error> {
        let crate_name    = self.name();
        let crate_version = self.version()?;

        if dry_run {
            info!(
                "DRY RUN: skipping cargo publish for crate {}@{}",
                crate_name, crate_version
            );
            return Ok(());
        }

        // We'll spawn "cargo" from the PATH, but preserve minimal environment so #!/bin/sh works.
        let mut cmd = Command::new("cargo");

        // Instead of clearing everything, let's remove only PATH, then set our own:
        // Or, if you prefer `env_clear()`, at least re-add some system dirs:
        // cmd.env_clear(); // optional, but if you do it, re-add system path:
        // let system_path = "/usr/bin:/bin:/usr/sbin:/sbin";
        // cmd.env("PATH", format!("{}:{}", fake_cargo_dir, system_path));

        // For example, if a test set FAKE_CARGO_PATH = /tmp/fake, we prepend it:
        if let Ok(fake_path) = std::env::var("FAKE_CARGO_PATH") {
            // Prepend the system PATH so that /bin/sh is still found:
            let old_path = std::env::var("PATH").unwrap_or_default();
            let new_path = format!("{fake_path}:{old_path}");
            cmd.env("PATH", new_path);
        }
        // else do nothing => use the normal system PATH

        // Prepare cargo publish command
        cmd.arg("publish").arg("--allow-dirty");

        let cargo_toml_arc = self.cargo_toml();
        let guard = cargo_toml_arc.lock().await;
        let cargo_toml_path = guard.as_ref().display().to_string();
        drop(guard);

        cmd.arg(format!("--manifest-path={}", cargo_toml_path));
        cmd.arg(format!("--package={}", crate_name));

        debug!("Running: {:?}", cmd);

        // Run
        let output = cmd.output().await.map_err(|io_err| {
            error!("IO error when running cargo publish: {}", io_err);
            CrateError::FailedToRunCargoPublish {
                crate_name: crate_name.to_string(),
                crate_version: crate_version.clone(),
                io_err: Arc::new(io_err),
            }
        })?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr_str = String::from_utf8_lossy(&output.stderr);
            let stdout_str = String::from_utf8_lossy(&output.stdout);

            if stderr_str.contains("already exists") || stdout_str.contains("already exists") {
                warn!(
                    "SKIP: cargo says {}@{} already exists. Treating as success.",
                    crate_name, crate_version
                );
                Ok(())
            } else {
                error!("ERROR: publish failed for {}@{} (exit code={:?})",
                       crate_name, crate_version, output.status.code());
                error!("stdout:\n{}", stdout_str);
                error!("stderr:\n{}", stderr_str);

                Err(CrateError::CargoPublishFailedForCrateWithExitCode {
                    crate_name: crate_name.to_string(),
                    crate_version,
                    exit_code: output.status.code(),
                })
            }
        }
    }
}

#[cfg(test)]
mod test_try_publish_crate_handle {
    use super::*;
    use std::path::{Path, PathBuf};
    use std::fs::{File as StdFile};
    use std::io::Write;
    use tempfile::{tempdir, TempDir};
    use tokio::runtime::Runtime;
    use std::os::unix::fs::PermissionsExt;
    use tracing::*;

    /// Minimal struct that implements `HasCargoTomlPathBuf` so we can create a `CrateHandle`.
    #[derive(Clone)]
    struct MockCratePath(PathBuf);

    impl AsRef<Path> for MockCratePath {
        fn as_ref(&self) -> &Path {
            &self.0
        }
    }

    /// Creates a minimal `Cargo.toml` in `dir` so that `CrateHandle::new` works.
    /// Returns the path to the newly created file.
    fn write_minimal_cargo_toml(dir: &Path, crate_name: &str, crate_version: &str) -> PathBuf {
        let cargo_toml_path = dir.join("Cargo.toml");
        let content = format!(
            r#"[package]
name = "{}"
version = "{}"
authors = ["Test <test@example.com>"]
license = "MIT"
"#,
            crate_name, crate_version
        );
        let mut f = StdFile::create(&cargo_toml_path)
            .expect("Failed to create Cargo.toml in temp dir");
        f.write_all(content.as_bytes())
            .expect("Failed to write minimal Cargo.toml");
        cargo_toml_path
    }

    /// Helper to create a `CrateHandle` in a new temp directory, with minimal Cargo.toml.
    async fn setup_crate_handle(crate_name: &str, crate_version: &str) -> (CrateHandle, TempDir) {
        let tmp = tempdir().expect("Failed to create temp directory");
        write_minimal_cargo_toml(tmp.path(), crate_name, crate_version);

        // Construct the handle
        let mock_path = MockCratePath(tmp.path().to_path_buf());
        let handle = CrateHandle::new(&mock_path)
            .await
            .expect("CrateHandle creation failed");
        (handle, tmp)
    }

    /// Creates a fake `cargo` script named exactly `cargo` so our test can find it via `$FAKE_CARGO_PATH`.
    fn create_fake_cargo_script(
        dir: &Path,
        exit_code: i32,
        stdout_msg: Option<&str>,
        stderr_msg: Option<&str>,
    ) -> PathBuf {
        let script_path = dir.join("cargo");
        let script_content = format!(
            "#!/bin/sh\n\
             {}\n\
             {}\n\
             exit {}",
            stdout_msg.map(|m| format!("echo \"{m}\"")).unwrap_or_default(),
            stderr_msg.map(|m| format!("echo \"{m}\" 1>&2")).unwrap_or_default(),
            exit_code
        );

        let mut file = StdFile::create(&script_path)
            .expect("Failed to create fake cargo script");
        file.write_all(script_content.as_bytes())
            .expect("Failed to write cargo script");
        file.sync_all().expect("Failed to sync cargo script file");

        // Make it executable on Unix
        let mut perms = std::fs::metadata(&script_path)
            .expect("metadata read failed")
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&script_path, perms)
            .expect("Failed to set cargo script perms 755");

        script_path
    }

    /// 1) Dry run => skip actual cargo publish => success
    #[traced_test]
    fn test_try_publish_dry_run() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let (handle, _tmp) = setup_crate_handle("dry_run_crate", "0.1.0").await;
            let result = handle.try_publish(true).await;
            assert!(result.is_ok(), "Expected success for dry_run");
        });
    }

    /// 2) A successful publish => cargo script exit code 0 => we expect Ok(()).
    #[traced_test]
    fn test_try_publish_success() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let (handle, tmp) = setup_crate_handle("success_crate", "0.1.0").await;

            // Create a fake cargo that prints "Publishing now..." and exits 0
            let fake_cargo = create_fake_cargo_script(
                tmp.path(),
                0,
                Some("Publishing now..."),
                None,
            );

            // We'll define a minimal path "tmp.path()" so that 'cargo' is found
            // And we only do that for the single command inside `try_publish`.
            // We'll do this by setting FAKE_CARGO_PATH => that environment variable
            // is used in the final code in `impl TryPublish`.
            let new_path = tmp.path().display().to_string();

            // Temporarily set FAKE_CARGO_PATH
            unsafe { std::env::set_var("FAKE_CARGO_PATH", &new_path); }

            let result = handle.try_publish(false).await;

            // Cleanup
            unsafe { std::env::remove_var("FAKE_CARGO_PATH"); }

            assert!(result.is_ok(), "Expected success when cargo exits 0");
        });
    }

    /// 3) "already exists" => cargo prints "already exists" => treat as success
    #[traced_test]
    fn test_try_publish_already_exists() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let (handle, tmp) = setup_crate_handle("already_exists_crate", "0.1.0").await;

            let fake_cargo = create_fake_cargo_script(
                tmp.path(),
                101,
                Some("some stdout text"),
                Some("crate `already_exists_crate@0.1.0` already exists"),
            );

            let new_path = tmp.path().display().to_string();
            unsafe { std::env::set_var("FAKE_CARGO_PATH", &new_path); }

            let result = handle.try_publish(false).await;

            unsafe { std::env::remove_var("FAKE_CARGO_PATH"); }

            // Should interpret "already exists" => Ok(())
            assert!(result.is_ok(), "Expected success if cargo says 'already exists'");
        });
    }

    /// 4) Another failing scenario => cargo prints an error, exit code 101, but no "already exists."
    /// => treat as fatal error => CargoPublishFailedForCrateWithExitCode
    #[traced_test]
    fn test_try_publish_fails_with_exit_code() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let (handle, tmp) = setup_crate_handle("fail_crate", "0.1.0").await;

            let fake_cargo = create_fake_cargo_script(
                tmp.path(),
                101,
                Some("some stdout text"),
                Some("some other error: invalid crate credentials"),
            );

            let new_path = tmp.path().display().to_string();
            unsafe { std::env::set_var("FAKE_CARGO_PATH", &new_path); }

            let result = handle.try_publish(false).await;

            unsafe { std::env::remove_var("FAKE_CARGO_PATH"); }

            assert!(result.is_err(), "Expected error (non-zero exit, no 'already exists')");
            match result {
                Err(CrateError::CargoPublishFailedForCrateWithExitCode {
                    crate_name,
                    crate_version,
                    exit_code,
                }) => {
                    assert_eq!(crate_name, "fail_crate");
                    assert_eq!(crate_version.to_string(), "0.1.0");
                    assert_eq!(exit_code, Some(101));
                }
                other => panic!("Expected CargoPublishFailedForCrateWithExitCode, got: {other:?}"),
            }
        });
    }

    #[traced_test]
    async fn test_try_publish_no_cargo_found() {
        // 1) Create a minimal crate that is actually valid.
        let (handle, tmp) = setup_crate_handle("missing_cargo_crate", "0.1.0").await;

        // Make sure there is a src/lib.rs so cargo doesn’t complain about no targets:
        let src_dir = tmp.path().join("src");
        std::fs::create_dir_all(&src_dir).expect("failed to create src directory");
        let lib_rs_path = src_dir.join("lib.rs");
        std::fs::write(&lib_rs_path, b"// dummy lib").expect("failed to write lib.rs");

        // 2) Force the PATH to be empty. Then "cargo" cannot be found => 
        //    the spawn will fail with an I/O error => CrateError::FailedToRunCargoPublish.
        unsafe { std::env::set_var("FAKE_CARGO_PATH", ""); }

        let result = handle.try_publish(false).await;

        unsafe { std::env::remove_var("FAKE_CARGO_PATH"); }

        match result {
            Err(CrateError::FailedToRunCargoPublish { crate_name, .. }) => {
                assert_eq!(crate_name, "missing_cargo_crate");
            }
            other => panic!(
                "Expected CrateError::FailedToRunCargoPublish, got: {other:?}"
            ),
        }
    }

    /// 6) Check that we do skip for DRY RUN in the presence of a 'fake cargo'
    /// that would fail if actually called.
    #[traced_test]
    fn test_try_publish_dry_run_even_with_fake_cargo() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let (handle, tmp) = setup_crate_handle("no_op_crate", "1.2.3").await;

            // This cargo is a fail if called for real
            let fake_cargo = create_fake_cargo_script(
                tmp.path(),
                101,
                None,
                Some("This would fail if actually invoked"),
            );

            // But we do a DRY RUN => skip calling cargo
            let new_path = tmp.path().display().to_string();
            unsafe { std::env::set_var("FAKE_CARGO_PATH", &new_path); }

            let result = handle.try_publish(true).await;

            unsafe { std::env::remove_var("FAKE_CARGO_PATH"); }

            // Should succeed because DRY RUN
            assert!(result.is_ok());
        });
    }
}
