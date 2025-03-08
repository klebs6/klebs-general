crate::ix!();

#[async_trait]
pub trait LanguageModelClientInterface
: RetrieveBatchById
+ GetBatchFileContent
+ UploadBatchFile
+ CreateBatch
+ WaitForBatchCompletion
{ }

#[async_trait]
pub trait RetrieveBatchById {
    type Error;
    async fn retrieve_batch(&self, batch_id: &str) 
        -> Result<Batch,Self::Error>;
}

#[async_trait]
pub trait GetBatchFileContent {
    type Error;
    async fn file_content(&self, file_id: &str) -> Result<Bytes,Self::Error>;
}

#[async_trait]
pub trait UploadBatchFile {

    type Error;

    async fn upload_batch_file(
        &self,
        file_path: impl AsRef<Path> + Send + Sync,

    ) -> Result<OpenAIFile, Self::Error>;
}

#[async_trait]
pub trait CreateBatch {
    type Error;
    async fn create_batch(
        &self,
        input_file_id: &str,
    ) -> Result<Batch, Self::Error>;
}

#[async_trait]
pub trait WaitForBatchCompletion {
    type Error;
    async fn wait_for_batch_completion(
        &self,
        batch_id: &str,
    ) -> Result<Batch, Self::Error>;
}
