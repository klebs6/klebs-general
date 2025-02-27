// ---------------- [ File: src/openai_client_handle.rs ]
crate::ix!();

pub trait OpenAIConfigInterface = async_openai::config::Config;

pub struct OpenAIClientHandle {
    client: async_openai::Client<OpenAIConfig>,
}

impl OpenAIClientHandle {

    pub fn new() -> Arc<Self> {

        info!("creating new OpenAI Client Handle");

        let openai_api_key 
            = std::env::var("OPENAI_API_KEY")
            .expect("OPENAI_API_KEY environment variable not set");

        // Initialize OpenAI client with your API key
        let config = OpenAIConfig::new().with_api_key(openai_api_key);

        let client = async_openai::Client::with_config(config);

        Arc::new(Self { client })
    }

    delegate!{
        to self.client {
            fn batches(&self) -> async_openai::Batches<OpenAIConfig>;
            fn files(&self) -> async_openai::Files<OpenAIConfig>;
        }
    }

    pub async fn retrieve_batch(&self, batch_id: &str) 
        -> Result<Batch,OpenAIClientError> 
    {
        info!("retrieving batch {} from online", batch_id);

        Ok(self.batches().retrieve(batch_id).await?)
    }

    pub async fn file_content(&self, file_id: &str) -> Result<Bytes,OpenAIClientError> {

        info!("retrieving file {} content from online", file_id);

        let file_content = self.files().content(file_id).await?;
        Ok(file_content)
    }

    pub async fn upload_batch_file(
        &self,
        file_path: impl AsRef<Path>,

    ) -> Result<OpenAIFile, OpenAIClientError> {

        info!("uploading batch file at path={:?} to online", file_path.as_ref());

        let create_file_request = CreateFileRequest {
            file:    file_path.into(),
            purpose: FilePurpose::Batch,
        };

        let file = self.files().create(create_file_request).await?;
        Ok(file)
    }

    pub async fn create_batch(
        &self,
        input_file_id: &str,
    ) -> Result<Batch, OpenAIClientError> {

        info!("creating batch with input_file_id={}", input_file_id);

        let batch_request = BatchRequest {
            input_file_id: input_file_id.to_string(),
            endpoint: BatchEndpoint::V1ChatCompletions,
            completion_window: BatchCompletionWindow::W24H,
            metadata: None,
        };

        let batch = self.batches().create(batch_request).await?;

        Ok(batch)
    }

    pub async fn wait_for_batch_completion(
        &self,
        batch_id: &str,
    ) -> Result<Batch, OpenAIClientError> {

        info!("waiting for batch completion");

        loop {
            let batch = self.retrieve_batch(&batch_id).await?;
            match batch.status {
                BatchStatus::Completed => return Ok(batch),
                BatchStatus::Failed => {
                    return Err(OpenAIClientError::ApiError(OpenAIApiError {
                        message: "Batch failed".to_string(),
                        r#type:  None,
                        param:   None,
                        code:    None,
                    }))
                }
                _ => {
                    println!("Batch status: {:?}", batch.status);
                    sleep(Duration::from_secs(20)).await; // Wait before checking again
                }
            }
        }
    }
}
