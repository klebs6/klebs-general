crate::ix!();

#[async_trait]
impl<P, C> RunLinting for C
where
    C: CrateHandleInterface<P> + HasCargoToml + Named + Sync + Send,    // we rely on cargo_toml(), name(), etc.
    for<'async_trait> P: AsRef<Path> + Send + Sync + 'async_trait,      // the path type inside the crate handle
{
    type Report = LintReport;
    type Error = LintingError;

    async fn run_linting_crate(&self) -> Result<Self::Report, Self::Error> {
        // 1) Lock the CargoToml so we can locate the physical manifest path
        let cargo_toml_arc = self.cargo_toml();
        let cargo_toml_guard = cargo_toml_arc.lock().await;
        let manifest_path = cargo_toml_guard.as_ref().to_path_buf(); // path to Cargo.toml

        // 2) Invoke `cargo clippy` with `--manifest-path` pointing to this crate’s Cargo.toml
        let output = tokio::process::Command::new("cargo")
            .arg("clippy")
            .arg("--manifest-path")
            .arg(&manifest_path)
            .arg("--all-targets")
            .arg("--message-format=short")
            .arg("--quiet")
            .arg("--")
            .arg("-D")
            .arg("warnings")
            .output()
            .await
            .map_err(|io_err| {
                error!(
                    "Failed to spawn cargo clippy for crate='{}': {io_err}",
                    self.name()
                );
                LintingError::CommandError { io: io_err.into() }
            })?;

        // 3) Convert output to `LintReport`
        let report = LintReport::from(output);

        // 4) If linting fails, propagate that as an error
        if !report.success() {
            warn!(
                "Lint failed for crate='{}'. Stderr:\n{}",
                self.name(),
                report.stderr()
            );
            return Err(LintingError::UnknownError {
                stderr: Some(report.stderr().to_owned()),
                stdout: Some(report.stdout().to_owned()),
            });
        }

        // 5) Otherwise, success
        debug!(
            "Lint successful for crate='{}' => {} bytes stdout, {} bytes stderr",
            self.name(),
            report.stdout().len(),
            report.stderr().len()
        );
        Ok(report)
    }
}

