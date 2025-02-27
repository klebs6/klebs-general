// ---------------- [ File: src/metadata.rs ]
crate::ix!();

#[derive(Serialize, Deserialize)]
pub struct BatchMetadata {
    batch_id:       String,
    input_file_id:  String,
    output_file_id: Option<String>,
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
