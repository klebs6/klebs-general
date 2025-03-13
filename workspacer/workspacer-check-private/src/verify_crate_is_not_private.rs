// ---------------- [ File: workspacer-check-private/src/verify_crate_is_not_private.rs ]
crate::ix!();

/// Additional trait used by `ReadyForCargoPublish` to check if the crate is private.
/// If it is private, we return an error.
pub trait VerifyCrateIsNotPrivate {
    type Error;
    fn verify_crate_is_not_private(&self) -> Result<(), Self::Error>;
}

// ---------------------------------------------------------------------
// Implementation: VerifyCrateIsNotPrivate for CrateHandle
// ---------------------------------------------------------------------
impl VerifyCrateIsNotPrivate for CrateHandle {
    type Error = CrateError;

    fn verify_crate_is_not_private(&self) -> Result<(), Self::Error> {
        trace!("Checking crate privacy via is_private()");
        let is_private = self.is_private()?;
        if is_private {
            error!("Crate is marked private => cannot be published. Aborting.");
            return Err(CrateError::CrateIsPrivate {
                crate_path: self.as_ref().to_path_buf(),
            });
        }
        info!("Crate is not private => OK");
        Ok(())
    }
}
