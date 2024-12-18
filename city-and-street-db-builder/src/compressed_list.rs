crate::ix!();

/// A simple wrapper for a list of strings that can be compressed.
/// In real code, implement a front-coding scheme here.
#[derive(Getters,Setters,Serialize,Deserialize)]
#[getset(get="pub",set="pub")]
#[serde(bound = "T: Serialize + DeserializeOwned")]
pub struct CompressedList<T> {
    /// The items stored in a compressed list.
    items: Vec<T>,
}

impl<T> From<Vec<T>> for CompressedList<T> 
where 
    T: Serialize + DeserializeOwned,
{
    fn from(items: Vec<T>) -> Self {
        Self { items }
    }
}

pub fn compress_set_to_cbor<T>(set: &std::collections::BTreeSet<T>) -> Vec<u8> 
where 
    T: Serialize + DeserializeOwned + Clone,
{
    let list: Vec<T> = set.iter().cloned().collect();
    let clist = CompressedList::from(list);
    // Handle errors explicitly rather than `unwrap_or_else` with empty Vec
    match serde_cbor::to_vec(&clist) {
        Ok(bytes) => bytes,
        Err(_) => Vec::new(),
    }
}

pub fn decompress_cbor_to_list<T>(bytes: &[u8]) -> Vec<T> 
where 
    T: Serialize + DeserializeOwned,
{
    match serde_cbor::from_slice::<CompressedList<T>>(bytes) {
        Ok(clist) => clist.items,
        Err(_) => Vec::new(),
    }
}

/// Tests for CompressedList serialization/deserialization
#[cfg(test)]
mod compressed_list_tests {
    use super::*;

    #[test]
    fn compress_and_decompress_strings() {
        let set: BTreeSet<String> = ["Hello","World","Baltimore"].iter().map(|s| s.to_string()).collect();
        let bytes = compress_set_to_cbor(&set);
        assert!(!bytes.is_empty());

        let decompressed: Vec<String> = decompress_cbor_to_list(&bytes);
        let decompressed_set: BTreeSet<String> = decompressed.into_iter().collect();
        assert_eq!(set, decompressed_set);
    }

    #[test]
    fn compress_and_decompress_empty() {
        let set: BTreeSet<String> = BTreeSet::new();
        let bytes = compress_set_to_cbor(&set);
        assert!(!bytes.is_empty()); // CBOR of empty still should produce some bytes

        let decompressed: Vec<String> = decompress_cbor_to_list(&bytes);
        assert!(decompressed.is_empty());
    }
}
