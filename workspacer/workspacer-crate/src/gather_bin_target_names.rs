// ---------------- [ File: src/gather_bin_target_names.rs ]
crate::ix!();

// Example default impl that tries to parse cargo_toml.raw_toml() or something
impl GatherBinTargetNames for CrateHandle 
{
    type Error = CrateError;

    fn gather_bin_target_names(&self) -> Result<Vec<String>, Self::Error> {
        Ok(self.cargo_toml().gather_bin_target_names()?)
    }
}
