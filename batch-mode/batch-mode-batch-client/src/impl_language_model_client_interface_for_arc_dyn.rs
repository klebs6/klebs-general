crate::ix!();

//-----------------------------------------[impl for dyn]
#[async_trait]
impl<E> LanguageModelClientInterface<E>
    for std::sync::Arc<dyn LanguageModelClientInterface<E>>
{ }

#[async_trait]
impl<E> RetrieveBatchById
    for std::sync::Arc<dyn LanguageModelClientInterface<E>>
{
    type Error = E;
    async fn retrieve_batch(&self, batch_id: &str) -> Result<Batch, Self::Error> {
        self.as_ref().retrieve_batch(batch_id).await
    }
}

#[async_trait]
impl<E> GetBatchFileContent
    for std::sync::Arc<dyn LanguageModelClientInterface<E>>
{
    type Error = E;

    async fn file_content(&self, file_id: &str) -> Result<Bytes, Self::Error> {
        self.as_ref().file_content(file_id).await
    }
}

#[async_trait]
impl<E> UploadBatchFileCore
    for std::sync::Arc<dyn LanguageModelClientInterface<E>>
{
    type Error = E;

    // and so on, forwarding each trait method:
    async fn upload_batch_file_path(
        &self,
        file_path: &std::path::Path
    ) -> Result<OpenAIFile, Self::Error> {
        self.as_ref().upload_batch_file_path(file_path).await
    }
}

#[async_trait]
impl<E> CreateBatch
    for std::sync::Arc<dyn LanguageModelClientInterface<E>>
{
    type Error = E;

    async fn create_batch(
        &self,
        input_file_id: &str,
    ) -> Result<Batch, Self::Error> {
        self.as_ref().create_batch(input_file_id).await
    }
}

#[async_trait]
impl<E> WaitForBatchCompletion
    for std::sync::Arc<dyn LanguageModelClientInterface<E>>
{
    type Error = E;

    async fn wait_for_batch_completion(
        &self,
        batch_id: &str,
    ) -> Result<Batch, Self::Error> {
        self.as_ref().wait_for_batch_completion(batch_id).await
    }
}
