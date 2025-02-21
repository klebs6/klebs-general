crate::ix!();

#[async_trait]
impl TryPublish for CrateHandle
{
    type Error = CrateError;

    /// Attempts to publish the given crate using `cargo publish`. If an
    /// "already exists" error is found in the output, we treat it as a skip,
    /// returning `Ok(())` instead of a fatal error.
    async fn try_publish(
        &self,
    ) -> Result<(), Self::Error>
    {
        let crate_name    = self.name();
        let crate_version = self.version()?;

        // We want the path to the crate's Cargo.toml
        let cargo_toml = self.cargo_toml();

        let mut cmd = Command::new("cargo");

        cmd.arg("publish")
           .arg("--allow-dirty")
           .arg(&format!("--manifest-path={}", cargo_toml.as_ref().display()))
           .arg(&format!("--package={}", crate_name));

        debug!("Running: {:?}", cmd);
        let output = cmd.output().await.map_err(|io_err| {
            CrateError::FailedtoRunCargoPublish { 
                crate_name:    crate_name.to_string(), 
                crate_version: crate_version.clone(), 
                io_err:        Arc::new(io_err) 
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
                    exit_code: output.status.code()
                })
            }
        }
    }
}
