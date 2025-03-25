// ---------------- [ File: workspacer-crate/src/gather_bin_target_names.rs ]
crate::ix!();

// Example default impl that tries to parse cargo_toml.raw_toml() or something
#[async_trait]
impl GatherBinTargetNames for CrateHandle 
{
    type Error = CrateError;

    async fn gather_bin_target_names(&self) -> Result<Vec<String>, Self::Error> {
        Ok(self.cargo_toml().lock().await.gather_bin_target_names().await?)
    }
}
