// ---------------- [ File: src/compressed_list.rs ]
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

    /// A simple helper struct to test non-String serialization.
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[traced_test]
    fn test_compress_and_decompress_strings_basic() {
        let set: BTreeSet<String> = ["Hello","World","Baltimore"].iter().map(|s| s.to_string()).collect();
        let bytes = compress_set_to_cbor(&set);
        assert!(!bytes.is_empty(), "CBOR must not be empty");

        let decompressed: Vec<String> = decompress_cbor_to_list(&bytes);
        let decompressed_set: BTreeSet<String> = decompressed.into_iter().collect();
        assert_eq!(set, decompressed_set, "Should match the original set");
    }

    #[traced_test]
    fn test_compress_and_decompress_strings_with_duplicates() {
        // BTreeSet automatically removes duplicates, so final set is "Hello", "Hello", "World" => "Hello", "World"
        let input = vec!["Hello","Hello","World","World"];
        let set: BTreeSet<String> = input.iter().map(|s| s.to_string()).collect();
        assert_eq!(set.len(), 2);

        let bytes = compress_set_to_cbor(&set);
        assert!(!bytes.is_empty());

        let decompressed: Vec<String> = decompress_cbor_to_list(&bytes);
        let de_set: BTreeSet<String> = decompressed.into_iter().collect();
        assert_eq!(de_set.len(), 2, "Duplicates remain collapsed");
        assert_eq!(set, de_set);
    }

    #[traced_test]
    fn test_compress_and_decompress_strings_with_punctuation() {
        // Confirm punctuation-laden strings remain intact after round trip
        let set: BTreeSet<String> = ["Hello!!", "Baltimore--City", "Wi-Fi", "N/A"].iter().map(|s| s.to_string()).collect();
        let bytes = compress_set_to_cbor(&set);

        let decompressed: Vec<String> = decompress_cbor_to_list(&bytes);
        let de_set: BTreeSet<String> = decompressed.into_iter().collect();
        assert_eq!(set, de_set, "Punctuation-laden strings remain intact");
    }

    #[traced_test]
    fn test_compress_and_decompress_empty() {
        let set: BTreeSet<String> = BTreeSet::new();
        let bytes = compress_set_to_cbor(&set);
        assert!(!bytes.is_empty(), "CBOR of empty set should still produce some bytes");

        let decompressed: Vec<String> = decompress_cbor_to_list(&bytes);
        assert!(decompressed.is_empty(), "Decompressed result should be empty");
    }

    #[traced_test]
    fn test_compress_and_decompress_nonstring_points() {
        // Ensure we can handle arbitrary serializable types
        let mut set = BTreeSet::new();
        set.insert(Point { x: 1, y: 2 });
        set.insert(Point { x: -10, y: 0 });
        set.insert(Point { x: 100, y: 999 });

        let bytes = compress_set_to_cbor(&set);
        assert!(!bytes.is_empty());

        let decompressed: Vec<Point> = decompress_cbor_to_list(&bytes);
        let de_set: BTreeSet<Point> = decompressed.into_iter().collect();
        assert_eq!(set, de_set, "All points should round-trip correctly");
    }

    #[traced_test]
    fn test_compress_and_decompress_large_set() {
        // If we want to test performance or memory usage in moderate scale
        // We'll do e.g. 1000 items. (Be mindful if there's a big cost.)
        let mut set = BTreeSet::new();
        for i in 0..1000 {
            set.insert(i.to_string());
        }

        let bytes = compress_set_to_cbor(&set);
        assert!(!bytes.is_empty(), "Should produce a non-empty CBOR payload");

        let decompressed: Vec<String> = decompress_cbor_to_list(&bytes);
        let de_set: BTreeSet<_> = decompressed.into_iter().collect();
        assert_eq!(set, de_set, "Should properly handle a thousand items");
    }

    #[traced_test]
    fn test_decompress_corrupted_cbor() {
        // Let's pass random bytes or partial data => should yield an empty Vec
        let corrupted_bytes: Vec<u8> = vec![0xde, 0xad, 0xbe, 0xef];
        let result: Vec<String> = decompress_cbor_to_list(&corrupted_bytes);
        assert!(result.is_empty(), "Corrupted CBOR yields an empty list");
    }

    #[traced_test]
    fn test_decompress_partial_data() {
        // Suppose a valid header for a small CBOR array, but truncated 
        // => also yields empty 
        let partial_bytes: Vec<u8> = vec![0x83, 0x62, 0x48]; 
        // 0x83 => an array of length 3, 0x62 => next item is a text string of length 2 => "H" but truncated
        let result: Vec<String> = decompress_cbor_to_list(&partial_bytes);
        assert!(result.is_empty(), "Partial/truncated data => empty");
    }
}
