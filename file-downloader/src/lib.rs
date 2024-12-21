#![allow(unused_variables)]
#[macro_use] mod imports; use imports::*;

x!{compute_and_verify_md5}
x!{download_file}
x!{download_file_with_md5}
x!{errors}
x!{extract_md5_from_filename}
x!{fetch_md5_for_link}
x!{file_utils}
x!{filename_with_md5}
x!{find_local_file}
x!{find_file_locally_or_download}
x!{get_extension_intelligent}

#[async_trait]
pub trait FileDownloader {

    /// Return the associated OSM PBF download link
    fn download_link(&self) -> &str;

    fn md5_download_link(&self) -> Option<Cow<'_,str>> {
        Some(Cow::Owned(format!("{}.md5", self.download_link())))
    }

    /// Obtain the associated PBF file locally, downloading if necessary.
    /// By default, this uses the `find_or_download` function provided by this crate.
    async fn find_file_locally_or_download(&self, directory: impl AsRef<Path> + Send + Sync) -> Result<PathBuf, DownloadError> {
        find_file_locally_or_download(
            self.download_link(), 
            self.md5_download_link().as_deref(), 
            directory
        ).await
    }
}
