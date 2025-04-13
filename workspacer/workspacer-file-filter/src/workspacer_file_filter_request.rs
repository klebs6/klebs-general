crate::ix!();

#[derive(Serialize, Deserialize, Debug, Builder, Getters, Clone)]
#[builder(setter(into))]
#[getset(get = "pub")]
pub struct AiFileFilterRequest<P>
where
    P: AsRef<Path> + Send + Sync + 'static,
{
    // The path to the actual file we want to filter:
    file_path: P,

    // The original content from that file:
    original_text: String,

    // Possibly other context info or config:
    // e.g. "some user instructions" that we embed
    user_instructions: String,
}

impl<P> AiFileFilterRequest<P>
where
    P: AsRef<Path> + Send + Sync + 'static,
{
    #[tracing::instrument(level = "trace", skip(config))]
    pub async fn async_try_from_path(
        file_path: P,
        user_instructions: &str,
        config: &FileFilterConfig,
    ) -> Result<Self, AiFileFilterError> {
        trace!("Building AiFileFilterRequest from path={:?}", file_path.as_ref());

        // 1) Check if file exists
        let metadata = tokio::fs::metadata(&file_path).await.map_err(|io_err| {
            error!("Cannot read metadata for file: {:?}", file_path.as_ref());
            AiFileFilterError::IoError {
                io_error: std::sync::Arc::new(io_err),
                context: format!("Failed to read metadata for {:?}", file_path.as_ref()),
            }
        })?;

        // If there's a max size, check it
        if let Some(max_sz) = config.max_file_size_bytes() {
            if metadata.len() > *max_sz {
                warn!(
                    "File size {} exceeds max {}; consider skipping or fallback",
                    metadata.len(),
                    max_sz
                );
                // For demonstration, let's proceed anyway,
                // or you can do a fallback approach here if you want
            }
        }

        // 2) Read the file content
        let content = tokio::fs::read_to_string(&file_path).await.map_err(|io_err| {
            AiFileFilterError::IoError {
                io_error: std::sync::Arc::new(io_err),
                context: format!("Failed to read file content from {:?}", file_path.as_ref()),
            }
        })?;

        // 3) Build the request
        Ok(Self {
            file_path,
            original_text: content,
            user_instructions: user_instructions.to_string(),
        })
    }
}

impl<P> std::fmt::Display for AiFileFilterRequest<P>
where
    P: AsRef<Path> + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AiFileFilterRequest for path: {}",
            self.file_path.as_ref().display()
        )
    }
}
