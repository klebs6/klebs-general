crate::ix!();

impl<H> IsPrivate for H
where H: HasCargoToml
{
    type Error = CrateError;
    /// Checks if the crate is private by reading the 'publish' field
    /// or 'publish = false' or 'package.publish = false' in Cargo.toml.
    /// Returns `Ok(true)` if private, `Ok(false)` if not private.
    fn is_private(&self) -> Result<bool, Self::Error>
    {
        let pkg_section = self
            .cargo_toml()
            .get_package_section()?;

        // The crate might specify "publish = false", or an array of allowed registries.
        // We'll say "private" if there's an explicit false or if "publish" is missing altogether
        // but typically "private" is recognized if "publish" = false in the package section.
        if let Some(publish_val) = pkg_section.get("publish") {
            // Could be boolean or array
            match publish_val {
                toml::Value::Boolean(b) => {
                    if !b {
                        return Ok(true);
                    }
                }
                // If there's an array of registries, we consider it public enough
                // for crates.io if "crates-io" is in that array or if it's empty, etc.
                toml::Value::Array(_) => {
                    // That might be considered public, so we skip marking it private
                }
                _ => {}
            }
        }

        // Check for "package.private" if it exists (rare in old cargo, but let's consider it).
        if let Some(private_val) = pkg_section.get("private").and_then(|val| val.as_bool()) {
            if private_val {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
