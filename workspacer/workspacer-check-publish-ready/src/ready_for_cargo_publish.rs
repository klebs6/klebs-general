// ---------------- [ File: workspacer-check-publish-ready/src/ready_for_cargo_publish.rs ]
crate::ix!();

/// Trait for checking if a component is ready for Cargo publishing
#[async_trait]
pub trait ReadyForCargoPublish {

    type Error;

    /// Checks if the crate is ready for Cargo publishing
    async fn ready_for_cargo_publish(&self) -> Result<(), Self::Error>;
}

#[async_trait]
impl ReadyForCargoPublish for CrateHandle {
    type Error = CrateError;

    /// Checks if the crate is ready for Cargo publishing
    async fn ready_for_cargo_publish(&self) -> Result<(), Self::Error> {
        trace!("Entering CrateHandle::ready_for_cargo_publish() at path={:?}", self.as_ref());

        // 1) Ask the Cargo.toml to confirm it's ready for publish (required fields, version validity).
        trace!("Calling cargo_toml().ready_for_cargo_publish() ...");
        // This presumably calls your existing logic for required fields (name, license, etc).
        self.cargo_toml().ready_for_cargo_publish().await?;

        // 2) Check README.md, and that `src/` directory has main.rs or lib.rs
        trace!("Ensuring README.md exists");
        self.check_readme_exists()?;

        trace!("Ensuring src/ directory has valid files");
        self.check_src_directory_contains_valid_files()?;

        // 3) Verify the crate is not private
        self.verify_crate_is_not_private()?;

        // 4) Verify the crate version is not yet published on crates.io
        self.verify_crate_version_is_not_yet_published_on_crates_io().await?;

        info!("CrateHandle at path={:?} => fully ready for cargo publish!", self.as_ref());
        Ok(())
    }
}
