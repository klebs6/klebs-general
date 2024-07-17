use serde::*;
use std::fmt;
use std::cmp::Ordering;
use std::hash::{Hash,Hasher};

/// A string type that can either be a static string slice or an owned heap-allocated string.
#[derive(Clone, Eq)]
pub enum StaticOrHeapString {
    Static(&'static str),
    Heap(String),
}

impl PartialEq for StaticOrHeapString {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialOrd for StaticOrHeapString {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.as_str().cmp(other.as_str()))
    }
}

impl Ord for StaticOrHeapString {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl fmt::Debug for StaticOrHeapString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl<'de> Deserialize<'de> for StaticOrHeapString {
    fn deserialize<D>(deserializer: D) -> Result<StaticOrHeapString, D::Error>
    where
        D: Deserializer<'de>,
    {
        let heap_key: String = Deserialize::deserialize(deserializer)?;
        Ok(StaticOrHeapString::Heap(heap_key))
    }
}

impl Serialize for StaticOrHeapString {
    /// Serializes the string, ignoring whether it is static or heap-allocated.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl StaticOrHeapString {
    /// Returns the string slice representation of the enum variant.
    pub fn as_str(&self) -> &str {
        match self {
            StaticOrHeapString::Static(s) => s,
            StaticOrHeapString::Heap(s) => s.as_str(),
        }
    }
}

impl Hash for StaticOrHeapString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_static_or_heap_string_can_serde_roundtrip() {
        // Testing static string round-trip
        let static_str = StaticOrHeapString::Static("hello");
        let serialized = serde_json::to_string(&static_str).unwrap();
        let deserialized: StaticOrHeapString = serde_json::from_str(&serialized).unwrap();
        assert_eq!(static_str, deserialized);

        // Testing heap string round-trip
        let heap_str = StaticOrHeapString::Heap(String::from("world"));
        let serialized = serde_json::to_string(&heap_str).unwrap();
        let deserialized: StaticOrHeapString = serde_json::from_str(&serialized).unwrap();
        assert_eq!(heap_str, deserialized);
    }

    #[test]
    fn test_static_or_heap_string_partial_eq() {
        let static_str1 = StaticOrHeapString::Static("hello");
        let static_str2 = StaticOrHeapString::Static("hello");
        let heap_str1 = StaticOrHeapString::Heap(String::from("hello"));
        let heap_str2 = StaticOrHeapString::Heap(String::from("world"));

        assert_eq!(static_str1, static_str2);
        assert_eq!(static_str1, heap_str1);
        assert_ne!(static_str1, heap_str2);
    }

    #[test]
    fn test_static_or_heap_string_partial_ord() {
        let static_str1 = StaticOrHeapString::Static("apple");
        let static_str2 = StaticOrHeapString::Static("banana");
        let heap_str1 = StaticOrHeapString::Heap(String::from("apple"));
        let heap_str2 = StaticOrHeapString::Heap(String::from("banana"));

        assert!(static_str1 < static_str2);
        assert!(heap_str1 < heap_str2);
        assert!(static_str1 <= heap_str1);
        assert!(static_str2 > heap_str1);
    }

    #[test]
    fn test_static_or_heap_string_ord() {
        let static_str1 = StaticOrHeapString::Static("apple");
        let heap_str1 = StaticOrHeapString::Heap(String::from("apple"));
        let static_str2 = StaticOrHeapString::Static("banana");
        let heap_str2 = StaticOrHeapString::Heap(String::from("banana"));

        assert_eq!(static_str1.cmp(&heap_str1), Ordering::Equal);
        assert_eq!(static_str1.cmp(&static_str2), Ordering::Less);
        assert_eq!(heap_str2.cmp(&static_str1), Ordering::Greater);
    }

    #[test]
    fn test_static_or_heap_string_debug() {
        let static_str = StaticOrHeapString::Static("hello");
        let heap_str = StaticOrHeapString::Heap(String::from("world"));

        assert_eq!(format!("{:?}", static_str), "hello");
        assert_eq!(format!("{:?}", heap_str), "world");
    }

    #[test]
    fn test_static_or_heap_string_as_str() {
        let static_str = StaticOrHeapString::Static("hello");
        let heap_str = StaticOrHeapString::Heap(String::from("world"));

        assert_eq!(static_str.as_str(), "hello");
        assert_eq!(heap_str.as_str(), "world");
    }

    #[test]
    fn test_static_or_heap_string_clone() {
        let static_str = StaticOrHeapString::Static("hello");
        let heap_str = StaticOrHeapString::Heap(String::from("world"));

        let cloned_static = static_str.clone();
        let cloned_heap = heap_str.clone();

        assert_eq!(static_str, cloned_static);
        assert_eq!(heap_str, cloned_heap);
    }

    #[test]
    fn test_static_or_heap_string_hash() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let static_str = StaticOrHeapString::Static("hello");
        let heap_str = StaticOrHeapString::Heap(String::from("hello"));

        let mut hasher1 = DefaultHasher::new();
        static_str.hash(&mut hasher1);
        let hash1 = hasher1.finish();

        let mut hasher2 = DefaultHasher::new();
        heap_str.hash(&mut hasher2);
        let hash2 = hasher2.finish();

        assert_eq!(hash1, hash2);
    }
}
