crate::ix!();

/// A specialized SaveToFile/LoadFromFile impl for HashMap<K, V>,
/// so you can avoid the universal blanket approach and thus not conflict with
/// your macro-based `impl_default_save_to_file_traits!` expansions.
///
/// This requires that K,V: Serialize + DeserializeOwned. Also note that
/// we require K: Eq + Hash. Then you can do:
///
///   let map: HashMap<String, MyStruct> = ...;
///   map.save_to_file("some.json").await?;
///
#[async_trait]
impl<K, V> SaveToFile for HashMap<K, V>
where
    K: Serialize + DeserializeOwned + Eq + Hash + Send + Sync + 'static,
    V: Serialize + DeserializeOwned + Send + Sync + 'static,
{
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

#[async_trait]
impl<K, V> LoadFromFile for HashMap<K, V>
where
    K: Serialize + DeserializeOwned + Eq + Hash + Send + Sync + 'static,
    V: Serialize + DeserializeOwned + Send + Sync + 'static,
{
    type Error = SaveLoadError;

    async fn load_from_file(filename: impl AsRef<Path> + Send)
        -> Result<Self, Self::Error>
    {
        let mut file = File::open(filename).await?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).await?;
        let map: HashMap<K, V> =
            serde_json::from_slice(&buf)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(map)
    }
}

/// A small test verifying that this specialized HashMap impl works.
/// In real usage, K,V must each implement Serialize + Deserialize + Eq + Hash (for K).
#[cfg(test)]
mod test_hashmap_save_load {
    use super::*;
    use tokio::fs;
    use traced_test::traced_test;
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    struct FakeLeaf {
        label: String,
        id: u32,
    }

    #[traced_test]
    async fn test_save_load_hashmap_basic() {
        let mut my_map: HashMap<String, FakeLeaf> = HashMap::new();
        my_map.insert(
            "Alpha".to_string(),
            FakeLeaf { label: "AlphaLeaf".into(), id: 1 }
        );
        my_map.insert(
            "Beta".to_string(),
            FakeLeaf { label: "BetaLeaf".into(), id: 2 }
        );

        let tmpfile = PathBuf::from("test_hashmap.json");
        my_map.save_to_file(&tmpfile).await.expect("save failed");

        let loaded_map = HashMap::<String, FakeLeaf>::load_from_file(&tmpfile)
            .await
            .expect("load failed");
        assert_eq!(loaded_map, my_map, "Roundtrip mismatch for HashMap<String, FakeLeaf>");

        fs::remove_file(&tmpfile).await.ok();
    }

    #[traced_test]
    async fn test_load_non_existent_map() {
        let path = PathBuf::from("unlikely_map_123.json");
        let result = HashMap::<String, FakeLeaf>::load_from_file(&path).await;
        assert!(result.is_err(), "Should error on non-existent file");
    }

    #[traced_test]
    async fn test_load_malformed_map_json() {
        let tmpfile = PathBuf::from("test_malformed.json");
        fs::write(&tmpfile, b"NOT VALID JSON").await.expect("write fail");
        let result = HashMap::<String, FakeLeaf>::load_from_file(&tmpfile).await;
        assert!(result.is_err(), "Should error on malformed JSON");
        fs::remove_file(&tmpfile).await.ok();
    }
}

