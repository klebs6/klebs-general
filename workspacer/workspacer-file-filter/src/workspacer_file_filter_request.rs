// ---------------- [ File: workspacer-file-filter/src/workspacer_file_filter_request.rs ]
crate::ix!();

#[derive(Serialize, Deserialize, Debug, Builder, Getters, Clone)]
#[builder(setter(into))]
#[getset(get = "pub")]
pub struct AiFileFilterRequest<P>
where
    P: AsRef<Path> + std::fmt::Debug + Send + Sync + 'static
{
    file_path:         P,
    original_text:     String,
    user_instructions: String,
}

impl<P> AiFileFilterRequest<P>
where
    P: AsRef<Path> + std::fmt::Debug + Send + Sync + 'static,
{
    pub fn snake_path_stem(&self) -> String {
        filename_to_snake(&self.file_path)
    }

    pub async fn async_try_from_path(
        file_path:         P,
        user_instructions: &str,
        config:            &FileFilterConfig,
    ) -> Result<Self, AiFileFilterError>
    {
        trace!("Building AiFileFilterRequest from path={:?}", file_path.as_ref());

        // 1) Check if file exists
        let metadata = tokio::fs::metadata(&file_path).await.map_err(|io_err| {
            error!("Cannot read metadata for file: {:?}", file_path.as_ref());
            AiFileFilterError::IoError {
                io_error: std::sync::Arc::new(io_err),
                context: format!("Failed to read metadata for {:?}", file_path.as_ref()),
            }
        })?;

        // 2) If there's a max size, check it
        if let Some(max_sz) = config.max_file_size_bytes() {
            if metadata.len() > *max_sz {
                warn!(
                    "File size {} exceeds max {}; consider skipping or fallback",
                    metadata.len(),
                    max_sz
                );
            }
        }

        // 3) Read the file content
        let content = tokio::fs::read_to_string(&file_path).await.map_err(|io_err| {
            AiFileFilterError::IoError {
                io_error: std::sync::Arc::new(io_err),
                context: format!("Failed to read file content from {:?}", file_path.as_ref()),
            }
        })?;

        // 4) Build the request
        Ok(Self {
            file_path,
            original_text: content,
            user_instructions: user_instructions.to_string(),
        })
    }
}

impl<P> std::fmt::Display for AiFileFilterRequest<P>
where
    P: AsRef<Path> + std::fmt::Debug + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AiFileFilterRequest for path: {}",
            self.file_path.as_ref().display()
        )
    }
}

impl<P> Named for AiFileFilterRequest<P>
where
    P: AsRef<Path> + std::fmt::Debug + Send + Sync + 'static,
{
    fn name(&self) -> std::borrow::Cow<'_, str> {
        // Provide any naming scheme you like:
        std::borrow::Cow::Owned(format!("file_filter_request_{}", self.snake_path_stem()))
    }
}

impl<P> HasAssociatedOutputName for AiFileFilterRequest<P>
where
    P: AsRef<Path> + std::fmt::Debug + Send + Sync + 'static,
{
    fn associated_output_name(&self) -> std::borrow::Cow<'_, str> {
        // Provide a stable "base name" so the .json expansion can be recognized:
        std::borrow::Cow::Owned(format!("filtered_{}", self.snake_path_stem()))
    }
}
