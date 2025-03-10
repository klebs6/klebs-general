// ---------------- [ File: src/openai_client_handle.rs ]
crate::ix!();

pub trait OpenAIConfigInterface = async_openai::config::Config;

pub struct OpenAIClientHandle<E> 
where
    E: Send + Sync + From<OpenAIClientError>,
{
    client: async_openai::Client<OpenAIConfig>,
    _marker: std::marker::PhantomData<E>,
}

#[async_trait]
impl<E> LanguageModelClientInterface<E> for OpenAIClientHandle<E>
where
    // We unify each sub‐trait’s “type Error=E” with the needed bounds:
    E: From<OpenAIClientError>
     + From<std::io::Error>
     + Send
     + Sync,
{
    // No additional methods to define here, because it's just the aggregator.
    // The sub‐traits are already implemented above.
}

impl<E> OpenAIClientHandle<E> 
where
    E: Send + Sync + From<OpenAIClientError>, // so we can do `.map_err(E::from)?`
{
    pub fn new() -> Arc<Self> {

        info!("creating new OpenAI Client Handle");

        let openai_api_key 
            = std::env::var("OPENAI_API_KEY")
            .expect("OPENAI_API_KEY environment variable not set");

        // Initialize OpenAI client with your API key
        let config = OpenAIConfig::new().with_api_key(openai_api_key);

        let client = async_openai::Client::with_config(config);

        Arc::new(Self { 
            client,
            _marker: std::marker::PhantomData::<E>,
        })
    }

    delegate!{
        to self.client {
            fn batches(&self) -> async_openai::Batches<OpenAIConfig>;
            fn files(&self) -> async_openai::Files<OpenAIConfig>;
        }
    }
}

#[async_trait]
impl<E> RetrieveBatchById for OpenAIClientHandle<E>
where
    E: Send + Sync + From<OpenAIClientError>, // so we can do `.map_err(E::from)?`
{
    type Error = E;

    async fn retrieve_batch(&self, batch_id: &str) -> Result<Batch, Self::Error> {
        info!("retrieving batch {} from online", batch_id);

        // The underlying call returns `Result<Batch, OpenAIApiError>` 
        // or `Result<Batch, OpenAIClientError>`? Let’s assume it’s an OpenAI error:
        let batch = self.batches().retrieve(batch_id)
            .await
            .map_err(|openai_err| E::from(OpenAIClientError::OpenAIError(openai_err)))?;

        Ok(batch)
    }
}

#[async_trait]
impl<E> GetBatchFileContent for OpenAIClientHandle<E>
where
    E: Send + Sync + From<OpenAIClientError> + From<std::io::Error>, 
{
    type Error = E;

    async fn file_content(&self, file_id: &str) -> Result<Bytes, Self::Error> {
        info!("retrieving file {} content from online", file_id);

        let bytes = self.files().content(file_id)
            .await
            // If that returns `OpenAIApiError`, do something like:
            .map_err(|api_err| E::from(OpenAIClientError::OpenAIError(api_err)))?;

        Ok(bytes)
    }
}

#[async_trait]
impl<E> UploadBatchFileCore for OpenAIClientHandle<E>
where
    E: Send + Sync + From<OpenAIClientError> + From<std::io::Error>, 
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
impl<E: Send + Sync + From<std::io::Error> + From<OpenAIClientError>> UploadBatchFileExt for OpenAIClientHandle<E> {}

#[async_trait]
impl<E> CreateBatch for OpenAIClientHandle<E>
where
    E: Send + Sync + From<OpenAIClientError>
{
    type Error = E;

    async fn create_batch(&self, input_file_id: &str) -> Result<Batch, Self::Error> {
        info!("creating batch with input_file_id={}", input_file_id);

        let batch_request = BatchRequest {
            input_file_id:     input_file_id.to_string(),
            endpoint:          BatchEndpoint::V1ChatCompletions,
            completion_window: BatchCompletionWindow::W24H,
            metadata: None,
        };

        let batch = self.batches().create(batch_request).await
            .map_err(|api_err| E::from(OpenAIClientError::OpenAIError(api_err)))?;

        Ok(batch)
    }
}

#[async_trait]
impl<E> WaitForBatchCompletion for OpenAIClientHandle<E>
where
    E: Send + Sync + From<OpenAIClientError>
{
    type Error = E;

    async fn wait_for_batch_completion(&self, batch_id: &str)
        -> Result<Batch, Self::Error>
    {
        info!("waiting for batch completion: batch_id={}", batch_id);

        loop {
            let batch = self.retrieve_batch(batch_id).await?;

            match batch.status {
                BatchStatus::Completed => return Ok(batch),
                BatchStatus::Failed => {
                    // Return an error: 
                    let openai_err = OpenAIClientError::ApiError(OpenAIApiError {
                        message: "Batch failed".to_owned(),
                        r#type: None,
                        param:  None,
                        code:   None,
                    });
                    return Err(E::from(openai_err));
                }
                _ => {
                    println!("Batch status: {:?}", batch.status);
                    tokio::time::sleep(std::time::Duration::from_secs(20)).await;
                }
            }
        }
    }
}
