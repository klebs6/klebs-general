// ---------------- [ File: src/check_and_download_interface.rs ]
crate::ix!();

#[async_trait]
pub trait CheckAndDownloadInterface<E>:
    CheckForAndDownloadOutputAndErrorOnline<E>
    + CheckBatchStatusOnline<E>
    + DownloadOutputFile<E>
    + DownloadErrorFile<E>
{
}

#[async_trait]
pub trait CheckForAndDownloadOutputAndErrorOnline<E> {
    async fn check_for_and_download_output_and_error_online(
        &mut self,
        client: &dyn LanguageModelClientInterface<E>,
    ) -> Result<(), E>;
}

#[async_trait]
pub trait CheckBatchStatusOnline<E> {
    async fn check_batch_status_online(
        &self,
        client: &dyn LanguageModelClientInterface<E>,
    ) -> Result<BatchOnlineStatus, E>;
}

#[async_trait]
pub trait DownloadOutputFile<E> {
    async fn download_output_file(
        &mut self,
        client: &dyn LanguageModelClientInterface<E>,
    ) -> Result<(), E>;
}

#[async_trait]
pub trait DownloadErrorFile<E> {
    async fn download_error_file(
        &mut self,
        client: &dyn LanguageModelClientInterface<E>,
    ) -> Result<(), E>;
}

// ----- The key fix: add a matching `impl<E>` block with the right bounds. -----

#[async_trait]
impl<E> CheckAndDownloadInterface<E> for BatchFileTriple
where
    E: From<BatchDownloadError>
      + From<OpenAIClientError>
      + From<BatchMetadataError>
      + From<std::io::Error> 
      + Debug
      + Display,
{
    // We don’t need any methods here, because this trait 
    // is just the aggregator of the 4 sub‐traits above.
}
