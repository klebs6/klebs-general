crate::ix!();

#[async_trait]
impl<E> UploadBatchFileCore for OpenAIClientHandle<E>
where
    E: Debug + Send + Sync + From<OpenAIClientError> + From<std::io::Error>, 
{
    type Error = E;

    async fn upload_batch_file_path(
        &self,
        file_path: &Path,
    ) -> Result<OpenAIFile, Self::Error> {
        info!("uploading batch file at path={:?} to online", file_path);

        let create_file_request = CreateFileRequest {
            file:    file_path.into(),
            purpose: FilePurpose::Batch,
        };

        // similarly map the openai error
        let file = self.files().create(create_file_request).await
            .map_err(|api_err| E::from(OpenAIClientError::OpenAIError(api_err)))?;

        Ok(file)
    }
}

#[async_trait]
impl<E: Debug + Send + Sync + From<std::io::Error> + From<OpenAIClientError>> UploadBatchFileExt for OpenAIClientHandle<E> {}

#[cfg(test)]
mod upload_batch_file_core_tests {
    use super::*;
    use futures::executor::block_on;
    use std::path::Path;
    use std::sync::Arc;
    use tempfile::{NamedTempFile, tempdir};
    use tracing::{debug, error, info, trace, warn};

    /// Exhaustive test suite for the `UploadBatchFileCore` implementation on `OpenAIClientHandle`.
    /// We use the mock client to simulate various scenarios (success, error, invalid input, etc).
    #[traced_test]
    async fn test_upload_batch_file_path_success() {
        info!("Beginning test_upload_batch_file_path_success");
        trace!("Constructing mock client...");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();
        debug!("Mock client built: {:?}", mock_client);

        trace!("Creating a temporary file to simulate a local batch file...");
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        debug!("Temp file path: {:?}", temp_file.path());

        trace!("Calling upload_batch_file_path with a valid file path");
        let result = mock_client.upload_batch_file_path(temp_file.path()).await;
        debug!("Result from upload_batch_file_path: {:?}", result);

        // For a valid file path, we expect an Ok result (unless the mock is forced to fail).
        assert!(
            result.is_ok(),
            "Expected upload_batch_file_path to succeed with a valid file path"
        );
        let uploaded_file = result.unwrap();
        debug!("Uploaded file response: {:?}", uploaded_file);
        info!("test_upload_batch_file_path_success passed.");
    }

    #[traced_test]
    async fn test_upload_batch_file_path_invalid_file_path() {
        info!("Beginning test_upload_batch_file_path_invalid_file_path");
        trace!("Constructing mock client...");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();
        debug!("Mock client built: {:?}", mock_client);

        // Provide an invalid path that likely doesn't exist
        let invalid_path = Path::new("/this/path/does/not/exist");
        trace!("Calling upload_batch_file_path with an invalid path: {:?}", invalid_path);
        let result = mock_client.upload_batch_file_path(invalid_path).await;
        debug!("Result from upload_batch_file_path: {:?}", result);

        // We expect this to fail (std::io::Error or something else).
        assert!(
            result.is_err(),
            "Expected upload_batch_file_path to fail with a non-existent file path"
        );
        info!("test_upload_batch_file_path_invalid_file_path passed.");
    }

    #[traced_test]
    async fn test_upload_batch_file_path_openai_api_error() {
        info!("Beginning test_upload_batch_file_path_openai_api_error");
        trace!("Constructing mock client that simulates an OpenAI error...");

        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .fail_on_file_create_openai_error(true)
            .build()
            .unwrap();

        debug!("Mock client built: {:?}", mock_client);

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        debug!("Temp file path: {:?}", temp_file.path());

        trace!("Calling upload_batch_file_path expecting an API error scenario...");
        let result = mock_client.upload_batch_file_path(temp_file.path()).await;
        debug!("Result from upload_batch_file_path: {:?}", result);

        // We expect an Err for an OpenAI error
        assert!(
            result.is_err(),
            "Expected upload_batch_file_path to return an error due to OpenAI API error"
        );
        info!("test_upload_batch_file_path_openai_api_error passed.");
    }

    #[traced_test]
    async fn test_upload_batch_file_path_other_error() {
        info!("Beginning test_upload_batch_file_path_other_error");
        trace!("Constructing mock client that simulates a different kind of error...");

        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            // We specifically set fail_on_file_create_other_error so that the mock
            // forcibly returns an IO error on upload.
            .fail_on_file_create_other_error(true)
            .build()
            .unwrap();

        debug!("Mock client built: {:?}", mock_client);

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        debug!("Temp file path: {:?}", temp_file.path());

        trace!("Calling upload_batch_file_path expecting a non-OpenAI error scenario...");
        let result = mock_client.upload_batch_file_path(temp_file.path()).await;
        debug!("Result from upload_batch_file_path: {:?}", result);

        // We expect an Err for a simulated non-OpenAI error (IO error).
        assert!(
            result.is_err(),
            "Expected upload_batch_file_path to return a non-OpenAI error scenario"
        );
        info!("test_upload_batch_file_path_other_error passed.");
    }
}
