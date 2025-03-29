// ---------------- [ File: workspacer-publish/src/try_publish_crate.rs ]
crate::ix!();

#[async_trait]
pub trait TryPublish {
    type Error;
    async fn try_publish(&self, dry_run: bool) -> Result<(), Self::Error>;
}

#[async_trait]
impl TryPublish for CrateHandle {
    type Error = CrateError;

    async fn try_publish(&self, dry_run: bool) -> Result<(), Self::Error> {
        use tracing::{trace, info, error, warn, debug};

        trace!("Entered CrateHandle::try_publish");
        let crate_name    = self.name();
        let crate_version = self.version()?;

        if dry_run {
            info!("DRY RUN: skipping cargo publish for {}@{}", crate_name, crate_version);
            return Ok(());
        }

        // We'll do the simplest approach: 
        // `cargo publish --allow-dirty --registry=mock` 
        // and rely on your local "CARGO_REGISTRIES_MOCK_INDEX" 
        // pointing to the rocket-based server. 
        // If you want the normal crates.io, you'd do no registry 
        // or pass `--registry=crates-io`.

        let mut cmd = std::process::Command::new("cargo");
        cmd.arg("publish")
            .arg("--allow-dirty");

        // If an env variable says "USE_MOCK_REGISTRY=1", we do `--registry=mock`
        if std::env::var("USE_MOCK_REGISTRY").unwrap_or_default() == "1" {
            cmd.arg("--registry=mock");
        }

        let cargo_toml_arc = self.cargo_toml();
        let guard = cargo_toml_arc.lock().await;
        let cargo_toml_path = guard.as_ref().display().to_string();
        drop(guard);

        // We can do `--manifest-path=some/path`
        cmd.arg(format!("--manifest-path={}", cargo_toml_path));

        cmd.arg(format!("--package={}", crate_name));

        debug!("Running: {:?}", cmd);

        let output = cmd.output().map_err(|io_err| {
            error!("IO error when spawning cargo publish: {}", io_err);
            CrateError::FailedToRunCargoPublish {
                crate_name: crate_name.to_string(),
                crate_version: crate_version.clone(),
                io_err: std::sync::Arc::new(io_err),
            }
        })?;

        let stderr_str = String::from_utf8_lossy(&output.stderr);
        let stdout_str = String::from_utf8_lossy(&output.stdout);
        let exit_code = output.status.code();

        if output.status.success() {
            info!("cargo publish succeeded for {}@{}", crate_name, crate_version);
            return Ok(());
        }

        // If cargo prints "already exists," treat as success
        if stderr_str.contains("already exists") || stdout_str.contains("already exists") {
            warn!("SKIP: cargo says {}@{} already exists => success", crate_name, crate_version);
            return Ok(());
        }

        error!("ERROR: publish failed for {}@{} (exit code={:?})",
            crate_name, crate_version, exit_code);
        error!("stdout:\n{}", stdout_str);
        error!("stderr:\n{}", stderr_str);

        Err(CrateError::CargoPublishFailedForCrateWithExitCode {
            crate_name: crate_name.to_string(),
            crate_version,
            exit_code,
        })
    }
}
