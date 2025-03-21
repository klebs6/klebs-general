// ---------------- [ File: src/obtain_pbf_file_for_region.rs ]
crate::ix!();

pub async fn obtain_pbf_file_for_region(
    region:           &WorldRegion,
    target_dir:       impl AsRef<Path> + Send + Sync,
) -> Result<PathBuf, WorldCityAndStreetDbBuilderError> {
    Ok(region.find_file_locally_or_download_into(&target_dir).await?)
}

#[cfg(test)]
mod test_obtain_pbf_file_for_region {
    use super::*;
    use std::path::{PathBuf, Path};
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;
    use async_trait::async_trait;

    /// A minimal trait to simulate or replace the region's `find_file_locally_or_download_into`
    /// call. In real code, this is probably defined on `WorldRegion` itself, so you may 
    /// need to wrap or mock it differently. This is just for demonstration.
    ///
    /// We define a trait identical to the method signature so we can inject mock behaviors.
    #[async_trait]
    pub trait MockRegionDownload {
        async fn find_file_locally_or_download_into(
            &self,
            target_dir: &Path
        ) -> Result<PathBuf, WorldCityAndStreetDbBuilderError>;
    }

    /// A mock `WorldRegion` that implements the `MockRegionDownload` trait.
    /// We can customize the behavior to simulate success/failure/unknown region.
    #[derive(Clone)]
    struct MockRegion {
        behavior: Arc<Mutex<MockBehavior>>,
    }

    /// Various scenarios we might want to simulate.
    enum MockBehavior {
        FileAlreadyExistsLocally { path: PathBuf },
        SuccessfulDownload { path: PathBuf },
        DownloadFailure,
        UnknownRegion,
    }

    #[async_trait]
    impl MockRegionDownload for MockRegion {
        async fn find_file_locally_or_download_into(
            &self,
            _target_dir: &Path
        ) -> Result<PathBuf, WorldCityAndStreetDbBuilderError> {
            let behavior = self.behavior.lock().unwrap();
            match &*behavior {
                MockBehavior::FileAlreadyExistsLocally { path } => {
                    // Simulate returning a local existing path
                    Ok(path.clone())
                }
                MockBehavior::SuccessfulDownload { path } => {
                    // Simulate successful download
                    Ok(path.clone())
                }
                MockBehavior::DownloadFailure => {
                    Err(WorldCityAndStreetDbBuilderError::SimulatedDownloadFailure)
                }
                MockBehavior::UnknownRegion => {
                    // Some code path that might produce an error for unrecognized region
                    // Adjust the variant or error type to match your real usage
                    Err(WorldCityAndStreetDbBuilderError::SimulatedUnknownRegionError)
                }
            }
        }
    }

    /// Extend or wrap the real `obtain_pbf_file_for_region` function so we can pass
    /// our `MockRegionDownload` object. In real usage, the `WorldRegion` itself provides
    /// `find_file_locally_or_download_into`, but here we're demonstrating mocking.
    async fn obtain_pbf_file_for_mock_region(
        mock_region: &MockRegion,
        target_dir: &Path,
    ) -> Result<PathBuf, WorldCityAndStreetDbBuilderError> {
        Ok(mock_region.find_file_locally_or_download_into(target_dir).await?)
    }

    #[traced_test]
    async fn test_already_exists_locally() {
        // Setup: the region claims it already has the file locally
        let region_mock = MockRegion {
            behavior: Arc::new(Mutex::new(
                MockBehavior::FileAlreadyExistsLocally {
                    path: PathBuf::from("/tmp/fake_local_file.osm.pbf"),
                }
            )),
        };

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let result = obtain_pbf_file_for_mock_region(&region_mock, temp_dir.path()).await;
        assert!(result.is_ok(), "Should succeed if the file is found locally");
        let path = result.unwrap();
        assert_eq!(path, PathBuf::from("/tmp/fake_local_file.osm.pbf"),
            "Should return the local file path");
    }

    #[traced_test]
    async fn test_successful_download() {
        // Setup: simulate that we must download the file, and it succeeds
        let region_mock = MockRegion {
            behavior: Arc::new(Mutex::new(
                MockBehavior::SuccessfulDownload {
                    path: PathBuf::from("/tmp/fake_downloaded_file.osm.pbf"),
                }
            )),
        };

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let result = obtain_pbf_file_for_mock_region(&region_mock, temp_dir.path()).await;
        assert!(result.is_ok(), "Should succeed if the download is simulated as successful");
        let path = result.unwrap();
        assert_eq!(path, PathBuf::from("/tmp/fake_downloaded_file.osm.pbf"));
    }

    #[traced_test]
    async fn test_download_failure() {
        // Setup: the mock region fails to download
        let region_mock = MockRegion {
            behavior: Arc::new(Mutex::new(MockBehavior::DownloadFailure)),
        };

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let result = obtain_pbf_file_for_mock_region(&region_mock, temp_dir.path()).await;
        assert!(result.is_err(), "Should fail if the download simulation fails");

        if let Err(WorldCityAndStreetDbBuilderError::SimulatedDownloadFailure) = result {
            // fine
        } else {
            panic!("Expected WorldCityAndStreetDbBuilderError::DownloadError(...)");
        }
    }

    #[traced_test]
    async fn test_unknown_region_error() {
        // Setup: simulate an unrecognized region
        let region_mock = MockRegion {
            behavior: Arc::new(Mutex::new(MockBehavior::UnknownRegion)),
        };

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let result = obtain_pbf_file_for_mock_region(&region_mock, temp_dir.path()).await;
        assert!(result.is_err(), "Should fail if the region is unknown");
        // For demonstration, we match the error variant
        if let Err(WorldCityAndStreetDbBuilderError::SimulatedUnknownRegionError) = result {
            // Good, it matches our simulated unknown region error
        } else {
            panic!("Expected DatabaseConstructionError for unknown region scenario");
        }
    }
}
