// ---------------- [ File: src/try_publish_crate.rs ]
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

    /// Attempts to publish the given crate using `cargo publish`. If an
    /// "already exists" error is found in the output, we treat it as a skip,
    /// returning `Ok(())` instead of a fatal error.
    ///
    /// When `dry_run` is true, we do not invoke `cargo publish` at all,
    /// but instead log a message that we are skipping the actual publish step.
    async fn try_publish(&self, dry_run: bool) -> Result<(), Self::Error> {
        let crate_name    = self.name();
        let crate_version = self.version()?;

        // If dry_run is set, skip the actual cargo publish
        if dry_run {
            info!(
                "DRY RUN: skipping cargo publish for crate {}@{}",
                crate_name, crate_version
            );
            return Ok(());
        }

        // Resolve full path to 'cargo' to avoid "No such file or directory" errors
        let cargo_path = which("cargo").map_err(|which_err| {
            error!("Failed to locate 'cargo' in PATH: {}", which_err);
            CrateError::FailedToRunCargoPublish {
                crate_name: crate_name.to_string(),
                crate_version: crate_version.clone(),
                which_err: Arc::new(which_err),
            }
        })?;

        // We want the path to the crate's Cargo.toml
        let cargo_toml = self.cargo_toml();

        // Prepare `cargo publish` command
        let mut cmd = Command::new(cargo_path);
        cmd.arg("publish")
            .arg("--allow-dirty")
            .arg(&format!("--manifest-path={}", (*cargo_toml).as_ref().display()))
            .arg(&format!("--package={}", crate_name));

        debug!("Running: {:?}", cmd);

        let output = cmd.output().await.map_err(|io_err| {
            error!("IO error when running cargo publish: {}", io_err);
            CrateError::FailedtoRunCargoPublish {
                crate_name:    crate_name.to_string(),
                crate_version: crate_version.clone(),
                io_err:        Arc::new(io_err),
            }
        })?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr_str = String::from_utf8_lossy(&output.stderr);
            let stdout_str = String::from_utf8_lossy(&output.stdout);

            // Check if "already exists" is in either
            if stderr_str.contains("already exists") || stdout_str.contains("already exists") {
                warn!(
                    "SKIP: cargo says {}@{} already exists. Treating as success.",
                    crate_name, crate_version
                );
                Ok(())
            } else {
                error!("ERROR: publish failed for {}@{}", crate_name, crate_version);
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
    use std::fs::{File as StdFile, Permissions};
    use std::io::Write;
    use tempfile::{tempdir, TempDir};
    use tokio::runtime::Runtime;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    /// Minimal struct that implements `HasCargoTomlPathBuf` to allow creating a `CrateHandle`.
    #[derive(Clone)]
    struct MockCratePath(PathBuf);

    impl AsRef<Path> for MockCratePath {
        fn as_ref(&self) -> &Path {
            &self.0
        }
    }

    /// Creates a minimal `Cargo.toml` file in `dir` so that `CrateHandle::new` works.
    /// Returns the path to the newly created file.
    fn write_minimal_cargo_toml(dir: &Path, name: &str, version: &str) -> PathBuf {
        let cargo_toml_path = dir.join("Cargo.toml");
        let content = format!(
            r#"[package]
name = "{}"
version = "{}"
authors = ["Test <test@example.com>"]
license = "MIT"
"#,
            name, version
        );
        let mut f =
            StdFile::create(&cargo_toml_path).expect("Failed to create Cargo.toml in temp dir");
        f.write_all(content.as_bytes())
            .expect("Failed to write minimal Cargo.toml");
        cargo_toml_path
    }

    /// Helper to create a `CrateHandle` in a new temp directory, with minimal Cargo.toml.
    async fn setup_crate_handle(crate_name: &str, crate_version: &str) -> (CrateHandle, TempDir) {
        let tmp = tempdir().expect("Failed to create temp directory");
        let _cargo_toml_path = write_minimal_cargo_toml(tmp.path(), crate_name, crate_version);

        // Construct the handle
        let mock_path = MockCratePath(tmp.path().to_path_buf());
        let handle = CrateHandle::new(&mock_path)
            .await
            .expect("CrateHandle creation failed");
        (handle, tmp)
    }

    /// Creates a fake `cargo` script named exactly `cargo` so `which("cargo")` will find it.
    /// We now:
    /// - Use `#!/bin/sh` instead of `#!/usr/bin/env bash` (for more universal availability).
    /// - Force permissions unconditionally on all platforms to ensure it's executable.
    /// - Flush the file before setting permissions to avoid race conditions.
    fn create_fake_cargo_script(
        dir: &Path,
        exit_code: i32,
        stdout_msg: Option<&str>,
        stderr_msg: Option<&str>,
    ) -> PathBuf {
        // Name it literally `cargo` so `which("cargo")` picks it up.
        let script_path = dir.join("cargo");

        // Use a plain POSIX sh shebang, which should exist on macOS, Linux, etc.
        let script_content = format!(
            "#!/bin/sh\n\
            {}\
            {}\
            exit {}",
            stdout_msg.map(|m| format!("echo \"{}\"\n", m)).unwrap_or_default(),
            stderr_msg.map(|m| format!("echo \"{}\" 1>&2\n", m)).unwrap_or_default(),
            exit_code
        );

        // Create and write the file
        {
            let mut file = StdFile::create(&script_path)
                .expect("Failed to create 'cargo' script file");
            use std::io::Write;
            file.write_all(script_content.as_bytes())
                .expect("Failed to write fake cargo script");
            file.sync_all().expect("Failed to sync file to disk");
        }

        // Unconditionally set executable permission. On Windows, no-ops if not supported,
        // but on macOS/Linux it ensures we get mode 755.
        #[allow(unused_mut)]
        {
            let mut perms = std::fs::metadata(&script_path)
                .expect("Failed to read metadata of cargo script")
                .permissions();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                perms.set_mode(0o755);
            }
            std::fs::set_permissions(&script_path, perms)
                .expect("Failed to set executable permissions on cargo script");
        }

        script_path
    }

    /// Helper to adjust PATH so that our fake cargo is found first.
    /// On Windows, you'd need a different approach (like a .bat file).
    fn prepend_fake_cargo_to_path(fake_cargo: &Path) -> String {
        let fake_cargo_dir = fake_cargo.parent().unwrap();
        let old_path = std::env::var_os("PATH").unwrap_or_default();
        // Put *our* directory first in PATH
        format!("{}:{}",
            fake_cargo_dir.display(),
            old_path.to_string_lossy()
        )
    }

    /// 1) Dry run => we skip the actual cargo publish.
    #[traced_test]
    fn test_try_publish_dry_run() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let (handle, _tmp) = setup_crate_handle("dry_run_crate", "0.1.0").await;

            // If dry_run is true, we do nothing
            let result = handle.try_publish(true).await;
            assert!(result.is_ok(), "Expected success for dry_run");
        });
    }

    /// 2) A successful publish scenario => exit code 0 => we return Ok(()).
    #[traced_test]
    fn test_try_publish_success() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let (handle, tmp) = setup_crate_handle("success_crate", "0.1.0").await;

            // Create a fake cargo that prints something and exits 0
            let fake_cargo_path = create_fake_cargo_script(tmp.path(), 0, Some("Publishing now..."), None);
            let new_path = prepend_fake_cargo_to_path(&fake_cargo_path);

            // Temporarily override PATH
            let old_path = std::env::var("PATH").unwrap_or_default();
            unsafe { std::env::set_var("PATH", &new_path); }

            let result = handle.try_publish(false).await;

            // Restore PATH
            unsafe { std::env::set_var("PATH", old_path); }

            assert!(result.is_ok(), "Expected success when cargo exits 0");
        });
    }

    /// 3) "already exists" scenario => cargo prints "already exists" => we treat as success.
    #[traced_test]
    fn test_try_publish_already_exists() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let (handle, tmp) = setup_crate_handle("already_exists_crate", "0.1.0").await;

            // Fake cargo that prints "already exists" in stderr, exits 101
            let fake_cargo_path = create_fake_cargo_script(
                tmp.path(),
                101,
                Some("some stdout text"),
                Some("crate `already_exists_crate@0.1.0` already exists"),
            );
            let new_path = prepend_fake_cargo_to_path(&fake_cargo_path);

            let old_path = std::env::var("PATH").unwrap_or_default();
            unsafe { std::env::set_var("PATH", &new_path); }

            let result = handle.try_publish(false).await;

            unsafe { std::env::set_var("PATH", old_path); }

            // Should interpret "already exists" => Ok(())
            assert!(result.is_ok(), "Expected success if cargo says 'already exists'");
        });
    }

    /// 4) Another failing scenario => cargo prints an error & exit code != 0, but does NOT mention "already exists."
    ///    => we treat as a fatal error.
    #[traced_test]
    fn test_try_publish_fails_with_exit_code() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let (handle, tmp) = setup_crate_handle("fail_crate", "0.1.0").await;

            // Fake cargo that prints "some other error" and exits 101
            let fake_cargo_path = create_fake_cargo_script(
                tmp.path(),
                101,
                Some("some stdout text"),
                Some("some other error: invalid crate credentials"),
            );
            let new_path = prepend_fake_cargo_to_path(&fake_cargo_path);

            let old_path = std::env::var("PATH").unwrap_or_default();
            unsafe { std::env::set_var("PATH", &new_path); }

            let result = handle.try_publish(false).await;

            unsafe { std::env::set_var("PATH", old_path); }

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

    /// 5) If the command fails to spawn at all (e.g., no cargo found), we get `FailedToRunCargoPublish`.
    #[traced_test]
    fn test_try_publish_no_cargo_found() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let (handle, _tmp) = setup_crate_handle("missing_cargo_crate", "0.1.0").await;

            // Overwrite PATH so we can't find "cargo"
            unsafe { std::env::set_var("PATH", ""); }

            let result = handle.try_publish(false).await;

            // Restore PATH
            unsafe { std::env::remove_var("PATH"); }

            assert!(result.is_err(), "Expected error if we can't run cargo at all");
            match result {
                Err(CrateError::FailedToRunCargoPublish { crate_name, .. }) => {
                    assert_eq!(crate_name, "missing_cargo_crate");
                }
                other => panic!("Expected CrateError::FailedToRunCargoPublish, got: {other:?}"),
            }
        });
    }
}
