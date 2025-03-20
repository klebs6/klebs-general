// ---------------- [ File: src/metadata.rs ]
crate::ix!();

#[derive(Builder,Debug,Clone,Serialize,Deserialize)]
#[builder(setter(into))]
pub struct BatchMetadata {
    batch_id:       String,
    input_file_id:  String,

    #[builder(default)]
    output_file_id: Option<String>,

    #[builder(default)]
    error_file_id:  Option<String>,
}

impl BatchMetadata {

    pub fn with_input_id_and_batch_id(input_id: &str, batch_id: &str) -> Self {
        Self {
            batch_id:       batch_id.to_string(),
            input_file_id:  input_id.to_string(),
            output_file_id: None,
            error_file_id:  None,
        }
    }

    pub fn batch_id(&self) -> &str {
        &self.batch_id
    }

    pub fn input_file_id(&self) -> &str {
        self.input_file_id.as_ref()
    }

    pub fn output_file_id(&self) -> Result<&str,BatchMetadataError> {
        let output_file_id = self.output_file_id.as_ref()
            .ok_or(BatchMetadataError::MissingOutputFileId)?;
        Ok(output_file_id)
    }

    pub fn error_file_id(&self) -> Result<&str,BatchMetadataError> {
        let error_file_id = self.error_file_id.as_ref()
            .ok_or(BatchMetadataError::MissingErrorFileId)?;
        Ok(error_file_id)
    }

    pub fn set_output_file_id(&mut self, new_id: Option<String>) {
        self.output_file_id = new_id;
    }

    pub fn set_error_file_id(&mut self, new_id: Option<String>) {
        self.error_file_id = new_id;
    }
}

#[async_trait]
impl SaveToFile for BatchMetadata {

    type Error = BatchMetadataError;

    async fn save_to_file(
        &self,
        metadata_filename: impl AsRef<Path> + Send,

    ) -> Result<(), Self::Error> {

        info!("saving batch metadata to file {:?}", metadata_filename.as_ref());

        let metadata_json = serde_json::to_string(&self)?;

        std::fs::write(metadata_filename, metadata_json)?;

        Ok(())
    }
}

#[async_trait]
impl LoadFromFile for BatchMetadata {

    type Error = BatchMetadataError;

    async fn load_from_file(metadata_filename: impl AsRef<Path> + Send) 
        -> Result<Self, Self::Error> 
    {
        info!("loading batch metadata from file {:?}", metadata_filename.as_ref());

        let metadata_json = std::fs::read_to_string(metadata_filename)?;
        let metadata      = serde_json::from_str(&metadata_json)?;

        Ok(metadata)
    }
}

#[cfg(test)]
mod batch_metadata_exhaustive_tests {
    use super::*;

    #[traced_test]
    fn with_input_id_and_batch_id_sets_expected_fields() {
        trace!("===== BEGIN TEST: with_input_id_and_batch_id_sets_expected_fields =====");
        let input_id = "some_input_id";
        let batch_id = "some_batch_id";
        let metadata = BatchMetadata::with_input_id_and_batch_id(input_id, batch_id);
        debug!("Constructed metadata: {:?}", metadata);

        pretty_assert_eq!(
            metadata.batch_id(),
            batch_id,
            "batch_id should match the provided value"
        );
        pretty_assert_eq!(
            metadata.input_file_id(),
            input_id,
            "input_file_id should match the provided value"
        );
        assert!(metadata.output_file_id.is_none(), "output_file_id should be None initially");
        assert!(metadata.error_file_id.is_none(), "error_file_id should be None initially");

        trace!("===== END TEST: with_input_id_and_batch_id_sets_expected_fields =====");
    }

    #[traced_test]
    fn set_output_file_id_and_retrieve_it_successfully() {
        trace!("===== BEGIN TEST: set_output_file_id_and_retrieve_it_successfully =====");
        let mut metadata = BatchMetadata::with_input_id_and_batch_id("input_1", "batch_1");
        let new_output_id = Some("output_file_xyz".to_string());
        trace!("Assigning output_file_id={:?}", new_output_id);
        metadata.set_output_file_id(new_output_id);

        match metadata.output_file_id() {
            Ok(id) => {
                debug!("Retrieved output_file_id: {}", id);
                pretty_assert_eq!(id, "output_file_xyz");
            },
            Err(_) => {
                error!("Expected output_file_id to be set, but got an error");
                panic!("Mismatch in output_file_id retrieval");
            }
        }
        trace!("===== END TEST: set_output_file_id_and_retrieve_it_successfully =====");
    }

    #[traced_test]
    fn set_output_file_id_to_none_and_verify_error() {
        trace!("===== BEGIN TEST: set_output_file_id_to_none_and_verify_error =====");
        let mut metadata = BatchMetadata::with_input_id_and_batch_id("input_2", "batch_2");
        metadata.set_output_file_id(None);
        trace!("Set output_file_id to None");

        let result = metadata.output_file_id();
        debug!("Attempting to retrieve output_file_id -> {:?}", result);
        assert!(result.is_err(), "Should fail when output_file_id is None");
        trace!("===== END TEST: set_output_file_id_to_none_and_verify_error =====");
    }

    #[traced_test]
    fn set_error_file_id_and_retrieve_it_successfully() {
        trace!("===== BEGIN TEST: set_error_file_id_and_retrieve_it_successfully =====");
        let mut metadata = BatchMetadata::with_input_id_and_batch_id("input_3", "batch_3");
        let new_error_id = Some("error_file_abc".to_string());
        trace!("Assigning error_file_id={:?}", new_error_id);
        metadata.set_error_file_id(new_error_id);

        match metadata.error_file_id() {
            Ok(id) => {
                debug!("Retrieved error_file_id: {}", id);
                pretty_assert_eq!(id, "error_file_abc");
            },
            Err(_) => {
                error!("Expected error_file_id to be set, but got an error");
                panic!("Mismatch in error_file_id retrieval");
            }
        }
        trace!("===== END TEST: set_error_file_id_and_retrieve_it_successfully =====");
    }

    #[traced_test]
    fn set_error_file_id_to_none_and_verify_error() {
        trace!("===== BEGIN TEST: set_error_file_id_to_none_and_verify_error =====");
        let mut metadata = BatchMetadata::with_input_id_and_batch_id("input_4", "batch_4");
        metadata.set_error_file_id(None);
        trace!("Set error_file_id to None");

        let result = metadata.error_file_id();
        debug!("Attempting to retrieve error_file_id -> {:?}", result);
        assert!(result.is_err(), "Should fail when error_file_id is None");
        trace!("===== END TEST: set_error_file_id_to_none_and_verify_error =====");
    }

    #[traced_test]
    async fn save_to_file_and_load_from_file_round_trip() -> Result<(),BatchMetadataError> {

        trace!("===== BEGIN TEST: save_to_file_and_load_from_file_round_trip =====");

        // Arrange
        let temp_dir = std::env::temp_dir();
        let filename = temp_dir.join("test_batch_metadata.json");
        debug!("Temporary file for metadata: {:?}", filename);

        let mut original = BatchMetadata::with_input_id_and_batch_id("in_10", "batch_10");
        original.set_output_file_id(Some("out_10".into()));
        original.set_error_file_id(Some("err_10".into()));
        trace!("Original metadata: {:?}", original);

        // Act: save to file
        let save_res = original.save_to_file(&filename).await;
        debug!("save_to_file result: {:?}", save_res);
        assert!(save_res.is_ok(), "Saving metadata should succeed");

        // Assert: load from file
        let loaded = BatchMetadata::load_from_file(&filename).await
            .expect("Loading metadata from file should succeed");
        debug!("Loaded metadata: {:?}", loaded);

        // Clean up
        if let Err(e) = fs::remove_file(&filename).await {
            warn!("Failed to remove temp file: {:?}", e);
        }

        // Compare fields
        pretty_assert_eq!(loaded.batch_id(), original.batch_id());
        pretty_assert_eq!(loaded.input_file_id(), original.input_file_id());
        // We unwrap or assert on these, because we set them
        pretty_assert_eq!(loaded.output_file_id().unwrap(), original.output_file_id().unwrap());
        pretty_assert_eq!(loaded.error_file_id().unwrap(), original.error_file_id().unwrap());

        trace!("===== END TEST: save_to_file_and_load_from_file_round_trip =====");
        Ok(())
    }
}
