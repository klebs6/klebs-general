// ---------------- [ File: src/impl_default_macro.rs ]
crate::ix!();

#[macro_export]
macro_rules! impl_default_save_to_file_traits {
    ($ty:ty) => {
        #[async_trait]
        impl LoadFromFile for $ty {
            type Error = SaveLoadError;

            async fn load_from_file(
                filename: impl AsRef<std::path::Path> + Send,
            ) -> Result<Self, Self::Error> {
                debug!("Attempting to load `{}` from file: {:?}", stringify!($ty), filename.as_ref());

                let content = match tokio::fs::read_to_string(filename.as_ref()).await {
                    Ok(c) => {
                        trace!("File content successfully read for `{}`: {}", stringify!($ty), c);
                        c
                    },
                    Err(e) => {
                        error!("Failed to read file for `{}`: {:?}", stringify!($ty), e);
                        return Err(SaveLoadError::IoError(e));
                    }
                };

                match serde_json::from_str(&content) {
                    Ok(instance) => {
                        info!("Successfully deserialized `{}` from file: {:?}", stringify!($ty), filename.as_ref());
                        Ok(instance)
                    },
                    Err(e) => {
                        error!("Deserialization error for `{}`: {:?}", stringify!($ty), e);
                        return Err(SaveLoadError::JsonParseError(e.into()));
                    }
                }
            }
        }

        #[async_trait]
        impl SaveToFile for $ty {
            type Error = SaveLoadError;

            async fn save_to_file(
                &self,
                filename: impl AsRef<std::path::Path> + Send,
            ) -> Result<(), Self::Error> {
                debug!("Attempting to save `{}` to file: {:?}", stringify!($ty), filename.as_ref());

                let serialized = match serde_json::to_string_pretty(self) {
                    Ok(json_str) => {
                        trace!("Successfully serialized `{}` to JSON: {}", stringify!($ty), json_str);
                        json_str
                    },
                    Err(e) => {
                        error!("Serialization error for `{}`: {:?}", stringify!($ty), e);
                        return Err(SaveLoadError::JsonParseError(e.into()));
                    }
                };

                if let Err(e) = tokio::fs::write(filename.as_ref(), &serialized).await {
                    error!("Failed to write `{}` to file: {:?}", stringify!($ty), e);
                    return Err(SaveLoadError::IoError(e));
                }

                info!("Successfully saved `{}` to file: {:?}", stringify!($ty), filename.as_ref());
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod default_impl_save_to_file_traits_when_serde_test {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestData {
        foo: String,
        bar: i32,
    }

    // We now use our macro that returns `SaveLoadError` instead of `serde_json::Error`.
    impl_default_save_to_file_traits!(TestData);

    #[traced_test]
    async fn test_save_and_load() {
        let data = TestData {
            foo: "Hello".to_string(),
            bar: 42,
        };

        let dir = tempdir().expect("Failed to create temp directory");
        let file_path = dir.path().join("test.json");

        info!("Attempting to save `TestData` to temporary file.");
        data.save_to_file(&file_path)
            .await
            .expect("Failed to save data");

        info!("Attempting to load `TestData` back from the same file.");
        let loaded = TestData::load_from_file(&file_path)
            .await
            .expect("Failed to load data");
        assert_eq!(data, loaded, "Loaded data did not match saved data.");
    }

    #[traced_test]
    async fn test_load_from_non_existent_file() {
        let non_existent_path = "definitely_does_not_exist.json";
        warn!("Trying to load `TestData` from a non-existent file. Expecting an error.");
        let result = TestData::load_from_file(non_existent_path).await;
        match result {
            Err(SaveLoadError::IoError(_)) => {
                info!("Got expected IoError for non-existent file.")
            },
            other => panic!("Expected an IoError, got {:?}", other),
        }
    }

    #[traced_test]
    async fn test_save_invalid_path() {
        let dir = tempdir().expect("Failed to create temp directory");
        let invalid_file_path = dir.path(); // Using the directory path instead of a file.

        let data = TestData {
            foo: "Hello".to_string(),
            bar: 42,
        };

        warn!("Trying to save `TestData` to a directory path instead of a file. Expecting an error.");
        let result = data.save_to_file(&invalid_file_path).await;
        match result {
            Err(SaveLoadError::IoError(_)) => {
                info!("Got expected IoError for invalid save path.")
            },
            other => panic!("Expected an IoError, got {:?}", other),
        }
    }

    #[traced_test]
    async fn test_load_corrupt_file() {
        let dir = tempdir().expect("Failed to create temp directory");
        let file_path = dir.path().join("corrupt.json");

        debug!("Writing invalid JSON to file to test parse errors.");
        tokio::fs::write(&file_path, b"not valid json")
            .await
            .expect("Failed to write corrupt data to file");

        warn!("Trying to load `TestData` from a corrupt JSON file. Expecting a JsonParseError.");
        let result = TestData::load_from_file(&file_path).await;
        match result {
            Err(SaveLoadError::JsonParseError(_)) => {
                info!("Got expected JsonParseError for corrupt file.")
            },
            other => panic!("Expected a JsonParseError, got {:?}", other),
        }
    }
}
