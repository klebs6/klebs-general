crate::ix!();

/// Additional trait used by `ReadyForCargoPublish` to check if the crate version
/// is already published on crates.io. If yes, we return an error.
#[async_trait]
pub trait VerifyCrateVersionIsNotYetPublishedOnCratesIo {
    type Error;
    async fn verify_crate_version_is_not_yet_published_on_crates_io(&self) -> Result<(), Self::Error>;
}

#[async_trait]
impl VerifyCrateVersionIsNotYetPublishedOnCratesIo for CrateHandle {
    type Error = CrateError;

    async fn verify_crate_version_is_not_yet_published_on_crates_io(&self) -> Result<(), Self::Error> {
        trace!("Verifying if crate version is already published on crates.io");

        let version             = self.version()?;
        let crate_name          = self.name().to_string();
        let is_published_result = is_crate_version_published_on_crates_io(&crate_name, &version).await;

        // We must convert `Result<bool, WorkspaceError>` -> `Result<bool, CrateError>`.
        let is_published = match is_published_result {
            Ok(flag) => flag,
            Err(ws_err) => {
                warn!("Error calling is_crate_version_published_on_crates_io: {:?}", ws_err);
                // Attempt to extract a CrateError, otherwise wrap in IoError
                match ws_err {
                    WorkspaceError::CrateError(ce) => {
                        error!("Underlying cause was a CrateError => returning that");
                        return Err(ce);
                    },
                    other => {
                        error!("Non-CrateError from crates.io check => wrapping in IoError");
                        return Err(CrateError::IoError {
                            io_error: Arc::new(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("crates.io check returned {other:?}")
                            )),
                            context: "While checking crates.io publish readiness".to_string(),
                        });
                    }
                }
            },
        };

        if is_published {
            error!("Crate version is already published on crates.io => not publish-ready");
            return Err(CrateError::CrateAlreadyPublishedOnCratesIo {
                crate_name:    crate_name.clone(),
                crate_version: version.clone(),
            });
        }

        info!("CrateHandle at path={:?} => name={}, version={} is NOT yet published on crates.io => OK",
            self.as_ref(), crate_name, version);

        Ok(())
    }
}
