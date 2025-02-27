// ---------------- [ File: src/load_from_directory.rs ]
crate::ix!();

/// Trait for loading objects from a directory asynchronously.
#[async_trait]
pub trait LoadFromDirectory: Sized {
    type Error;

    async fn load_from_directory(
        dir: impl AsRef<Path> + Send,
    ) -> Result<Vec<Self>, Self::Error>;
}

/// Blanket implementation of `LoadFromDirectory` for any type implementing `LoadFromFile`
/// where the error can be converted from `SaveLoadError`.
#[async_trait]
impl<T> LoadFromDirectory for T
where
    T: LoadFromFile + Send + Sized,
    T::Error: Display + From<SaveLoadError> + From<io::Error> + Send + 'static,
{
    type Error = T::Error;

    async fn load_from_directory(
        dir: impl AsRef<Path> + Send,
    ) -> Result<Vec<Self>, Self::Error> {

        let dir = dir.as_ref();

        if !dir.is_dir() {
            return Err(SaveLoadError::InvalidDirectory {
                dir: dir.to_path_buf()
            }.into());
        }

        let mut results = Vec::new();
        let mut entries = fs::read_dir(dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.is_file() {
                match Self::load_from_file(&path).await {
                    Ok(item) => results.push(item),
                    Err(e) => warn!("Failed to load from file {:?}: {}", path, e),
                }
            }
        }

        Ok(results)
    }
}
