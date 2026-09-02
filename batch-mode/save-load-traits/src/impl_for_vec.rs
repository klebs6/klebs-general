// ---------------- [ File: save-load-traits/src/impl_for_vec.rs ]
crate::ix!();

/// Implements SaveToFile for `Vec<T>`, writing its contents as JSON array,
/// where `T: Serialize + DeserializeOwned + Send + Sync + 'static`.
#[async_trait]
impl<T> SaveToFile for Vec<T>
where
    T: Serialize + DeserializeOwned + Send + Sync + 'static,
{
    type Error = SaveLoadError;

    async fn save_to_file(
        &self,
        filename: impl AsRef<Path> + Send,
    ) -> Result<(), Self::Error> {
        let json_data = serde_json::to_string_pretty(&self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        let mut file = File::create(filename).await?;
        file.write_all(json_data.as_bytes()).await?;
        Ok(())
    }
}

/// Implements LoadFromFile for `Vec<T>`, reading its contents from JSON array,
/// where `T: Serialize + DeserializeOwned + Send + Sync + 'static`.
#[async_trait]
impl<T> LoadFromFile for Vec<T>
where
    T: Serialize + DeserializeOwned + Send + Sync + 'static,
{
    type Error = SaveLoadError;

    async fn load_from_file(filename: impl AsRef<Path> + Send)
        -> Result<Self, Self::Error>
    {
        let mut file = File::open(filename).await?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).await?;

        let list_of_items: Vec<T> = serde_json::from_slice(&buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(list_of_items)
    }
}

/// Exhaustive test suite verifying we can store/load any `Vec<T>` using these
/// blanket impls. In real usage, `T` must implement `Serialize + DeserializeOwned + Send + Sync`.
#[cfg(test)]
mod test_vec_t_save_load {
    use super::*;
    use tokio::fs;
    use traced_test::traced_test;
    use std::path::PathBuf;

    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    struct ExampleItem {
        name: String,
        value: i32,
    }

    #[traced_test]
    async fn test_save_load_vec_example_item() {
        let items = vec![
            ExampleItem { name: "First".into(), value: 10 },
            ExampleItem { name: "Second".into(), value: 20 },
        ];
        let tmpfile = PathBuf::from("test_vec_example_item.json");

        items.save_to_file(&tmpfile).await.expect("save failed");
        let loaded_vec = <Vec<ExampleItem>>::load_from_file(&tmpfile)
            .await
            .expect("load failed");
        assert_eq!(loaded_vec, items, "Roundtrip mismatch for Vec<ExampleItem>");
        fs::remove_file(&tmpfile).await.ok();
    }

    /// Demonstrates we can also store/load basic types if needed, e.g. `Vec<String>`.
    #[traced_test]
    async fn test_save_load_vec_string() {
        let data = vec!["alpha".to_string(), "beta".to_string()];
        let tmpfile = PathBuf::from("test_vec_string.json");

        data.save_to_file(&tmpfile).await.expect("save failed");
        let loaded = <Vec<String>>::load_from_file(&tmpfile)
            .await
            .expect("load failed");
        assert_eq!(loaded, data, "Roundtrip mismatch for Vec<String>");
        fs::remove_file(&tmpfile).await.ok();
    }

    #[traced_test]
    async fn test_load_non_existent_file() {
        let result = <Vec<ExampleItem>>::load_from_file("no_such_vec_123.json").await;
        assert!(result.is_err(), "Should fail for a nonexistent file");
    }

    #[traced_test]
    async fn test_load_corrupt_json() {
        let tmpfile = PathBuf::from("test_vec_t_corrupt.json");
        fs::write(&tmpfile, b"not valid json")
            .await
            .expect("write fail");
        let result = <Vec<ExampleItem>>::load_from_file(&tmpfile).await;
        assert!(result.is_err(), "Should fail for corrupt JSON");
        fs::remove_file(&tmpfile).await.ok();
    }
}

/// Exhaustive test suite to verify reading/writing Vec<String> from/to file.
#[cfg(test)]
mod test_vec_string_save_and_load {
    use super::*;
    use tokio::fs;

    /// Test saving and loading an empty Vec<String>.
    #[traced_test]
    async fn test_save_load_empty_vec() {
        let tmpfile = PathBuf::from("test_vec_string_empty.json");
        let empty_vec: Vec<String> = vec![];

        empty_vec.save_to_file(&tmpfile).await.expect("save failed");
        let loaded_vec = <Vec<String> as LoadFromFile>::load_from_file(&tmpfile)
            .await
            .expect("load failed");
        assert_eq!(loaded_vec, empty_vec, "Loaded vec must match saved vec");

        fs::remove_file(&tmpfile).await.unwrap_or(());
    }

    /// Test saving and loading a non-empty Vec<String>.
    #[traced_test]
    async fn test_save_load_nonempty_vec() {
        let tmpfile = PathBuf::from("test_vec_string_nonempty.json");
        let sample_vec = vec!["Alpha".to_string(), "Beta".to_string(), "Gamma".to_string()];

        sample_vec.save_to_file(&tmpfile).await.expect("save failed");
        let loaded_vec = <Vec<String> as LoadFromFile>::load_from_file(&tmpfile)
            .await
            .expect("load failed");
        assert_eq!(loaded_vec, sample_vec, "Loaded vec must match saved vec");

        fs::remove_file(&tmpfile).await.unwrap_or(());
    }

    /// Test saving a Vec<String> containing special characters and verifying the round trip.
    #[traced_test]
    async fn test_save_load_special_chars() {
        let tmpfile = PathBuf::from("test_vec_string_special.json");
        let special_vec = vec![
            "Line\nbreak".to_string(),
            "Tab\tchar".to_string(),
            "Unicode: λ -> λ".to_string()
        ];

        special_vec.save_to_file(&tmpfile).await.expect("save failed");
        let loaded = <Vec<String> as LoadFromFile>::load_from_file(&tmpfile)
            .await
            .expect("load failed");
        assert_eq!(loaded, special_vec, "Loaded special-chars must match saved data");

        fs::remove_file(&tmpfile).await.unwrap_or(());
    }

    /// Test that loading from a non-existent file returns an error.
    #[traced_test]
    async fn test_load_non_existent_file_error() {
        let tmpfile = PathBuf::from("this_file_should_not_exist_123.json");
        let load_result = <Vec<String> as LoadFromFile>::load_from_file(&tmpfile).await;
        assert!(load_result.is_err(), "Expected an error when file does not exist");
    }

    /// Test that loading a malformed JSON file yields an error.
    #[traced_test]
    async fn test_load_malformed_json() {
        let tmpfile = PathBuf::from("test_vec_string_malformed.json");
        let contents = b"This is not valid JSON at all.";
        {
            let mut f = tokio::fs::File::create(&tmpfile).await.expect("create fail");
            use tokio::io::AsyncWriteExt;
            f.write_all(contents).await.expect("write fail");
        }

        let load_result = <Vec<String> as LoadFromFile>::load_from_file(&tmpfile).await;
        assert!(load_result.is_err(), "Expected an error for malformed JSON input");

        fs::remove_file(&tmpfile).await.unwrap_or(());
    }
}
