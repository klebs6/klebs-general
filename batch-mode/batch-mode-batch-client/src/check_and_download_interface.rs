// ---------------- [ File: src/check_and_download_interface.rs ]
crate::ix!();

#[async_trait]
pub trait CheckAndDownloadInterface<C,E>
: CheckForAndDownloadOutputAndErrorOnline<C,E>
+ CheckBatchStatusOnline<C,E>
+ DownloadOutputFile<C,E>
+ DownloadErrorFile<C,E>
where C: LanguageModelClientInterface<E>
{}

#[async_trait]
pub trait CheckForAndDownloadOutputAndErrorOnline<C,E> 
where C: LanguageModelClientInterface<E>
{
    async fn check_for_and_download_output_and_error_online(
        &mut self,
        client: &C,
    ) -> Result<(), BatchDownloadError>;
}

#[async_trait]
pub trait CheckBatchStatusOnline<C,E> 
where C: LanguageModelClientInterface<E>
{
    async fn check_batch_status_online(
        &self,
        client: &C,
    ) -> Result<BatchOnlineStatus, BatchDownloadError>;
}

#[async_trait]
pub trait DownloadOutputFile<C,E> 
where C: LanguageModelClientInterface<E>
{
    async fn download_output_file(
        &mut self,
        client: &C,
    ) -> Result<(), BatchDownloadError>;
}

#[async_trait]
pub trait DownloadErrorFile<C,E> 
where C: LanguageModelClientInterface<E>
{
    async fn download_error_file(
        &mut self,
        client: &C,
    ) -> Result<(), BatchDownloadError>;
}

#[async_trait]
impl<C,E> CheckAndDownloadInterface<C,E> for BatchFileTriple 
where C: LanguageModelClientInterface<E>,
      BatchDownloadError: From<E>
{}
