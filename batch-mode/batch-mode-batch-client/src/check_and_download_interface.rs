// ---------------- [ File: src/check_and_download.rs ]
crate::ix!();

#[async_trait]
pub trait CheckAndDownloadInterface
: CheckForAndDownloadOutputAndErrorOnline
+ CheckBatchStatusOnline
+ DownloadOutputFile
+ DownloadErrorFile
{}

#[async_trait]
pub trait CheckForAndDownloadOutputAndErrorOnline {
    async fn check_for_and_download_output_and_error_online(
        &mut self,
        client: &OpenAIClientHandle,
    ) -> Result<(), BatchDownloadError>;
}

#[async_trait]
pub trait CheckBatchStatusOnline {
    async fn check_batch_status_online(
        &self,
        client: &OpenAIClientHandle,
    ) -> Result<BatchOnlineStatus, BatchDownloadError>;
}

#[async_trait]
pub trait DownloadOutputFile {
    async fn download_output_file(
        &mut self,
        client: &OpenAIClientHandle,
    ) -> Result<(), BatchDownloadError>;
}

#[async_trait]
pub trait DownloadErrorFile {
    async fn download_error_file(
        &mut self,
        client: &OpenAIClientHandle,
    ) -> Result<(), BatchDownloadError>;
}

#[async_trait]
impl CheckAndDownloadInterface for BatchFileTriple {}
