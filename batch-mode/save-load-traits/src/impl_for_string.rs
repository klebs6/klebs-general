// ---------------- [ File: save-load-traits/src/impl_for_string.rs ]
crate::ix!();

#[async_trait]
impl SaveToFile for String {
    type Error = SaveLoadError;

    async fn save_to_file(
        &self,
        filename: impl AsRef<Path> + Send
    ) -> Result<(), Self::Error> {
        let json_data = serde_json::to_string_pretty(&self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let mut file = File::create(filename).await?;
        file.write_all(json_data.as_bytes()).await?;
        Ok(())
    }
}

/// Implements LoadFromFile for Vec<String>, reading its contents from a raw JSON file.
#[async_trait]
impl LoadFromFile for String {
    type Error = SaveLoadError;

    async fn load_from_file(filename: impl AsRef<Path> + Send)
        -> Result<Self, Self::Error>
    {
        let mut file = File::open(filename).await?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).await?;
        let string: String =
            serde_json::from_slice(&buf)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(string)
    }
}

#[cfg(test)]
mod string_file_persistence_behavior {
    use super::*;
    use tracing::{info, debug, trace};
    use traced_test::traced_test;
    use tempfile::NamedTempFile;
    use std::path::Path;

    #[traced_test]
    async fn roundtrip_save_and_load_preserves_exact_string() {
        let original = String::from("Hello, world!");
        info!("Testing roundtrip save and load for string: {}", original);
        let tmp = NamedTempFile::new().expect("unable to create temporary file");
        let path = tmp.path().to_path_buf();
        trace!("Saving string to file at {:?}", path);
        original.save_to_file(&path).await.expect("save_to_file failed");
        trace!("Loading string from file at {:?}", path);
        let loaded = String::load_from_file(&path).await.expect("load_from_file failed");
        info!("Loaded string: {}", loaded);
        assert_eq!(loaded, original);
    }

    #[traced_test]
    async fn save_creates_file_with_pretty_json_representation() {
        let original = String::from("Test JSON");
        debug!("Original string for JSON formatting test: {}", original);
        let tmp = NamedTempFile::new().expect("unable to create temporary file");
        let path = tmp.path().to_path_buf();
        original.save_to_file(&path).await.expect("save_to_file failed");
        let content = tokio::fs::read_to_string(&path).await.expect("failed to read file");
        let expected = serde_json::to_string_pretty(&original).expect("serde_json formatting failed");
        debug!("File content: {}", content);
        assert_eq!(content, expected);
    }

    #[traced_test]
    async fn load_from_nonexistent_path_returns_not_found_error() {
        let nonexistent = Path::new("unlikely_to_exist_12345.json");
        let result = String::load_from_file(&nonexistent).await;
        assert!(result.is_err(), "Expected error for nonexistent file");
        let err = result.unwrap_err();
        debug!("Received error: {:?}", err);
    }

    #[traced_test]
    async fn load_with_invalid_json_data_returns_invalid_data_error() {
        let tmp = NamedTempFile::new().expect("unable to create temporary file");
        let path = tmp.path().to_path_buf();
        // write invalid JSON directly
        tokio::fs::write(&path, b"not a valid json").await.expect("failed to write invalid data");
        let result = String::load_from_file(&path).await;
        assert!(result.is_err(), "Expected error for invalid JSON data");
        let err = result.unwrap_err();
        debug!("Received error: {:?}", err);
    }
}
