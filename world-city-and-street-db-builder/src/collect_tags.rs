// ---------------- [ File: src/collect_tags.rs ]
// ---------------- [ File: src/collect_tags.rs ]
crate::ix!();

/// Collect tags into a [`HashMap`], generic over any iterator of
/// `(&str, &str)`.  This allows both real `TagIter<'_>` from `osmpbf`
/// **and** test mocks to be used.
///
pub fn collect_tags<'a,I>(tags: I) -> HashMap<String, String>
where
    I: Iterator<Item = (&'a str, &'a str)>,
{
    tags.map(|(k,v)| (k.to_string(), v.to_string())).collect()
}

#[cfg(test)]
mod collect_tags_tests {
    use super::*;
    use std::collections::HashMap;

    /// Creates a small iterator of `(k, v)` pairs for tests, simulating
    /// typical OSM tag data.  
    fn make_tag_iter<'a>(data: &'a [(&'a str, &'a str)]) -> impl Iterator<Item = (&'a str, &'a str)> {
        data.iter().copied()
    }

    #[traced_test]
    fn test_collect_tags_empty() {
        // No tags => empty map
        let empty_iter = make_tag_iter(&[]);
        let result = collect_tags(empty_iter);
        assert!(result.is_empty(), "Expected empty HashMap");
    }

    #[traced_test]
    fn test_collect_tags_single_pair() {
        // A single key-value
        let data = [("addr:city", "Baltimore")];
        let iter = make_tag_iter(&data);
        let result = collect_tags(iter);
        assert_eq!(result.len(), 1);
        assert_eq!(
            result.get("addr:city"),
            Some(&"Baltimore".to_string()),
            "Should match the single key-value inserted"
        );
    }

    #[traced_test]
    fn test_collect_tags_multiple_pairs() {
        // Typical OSM data with multiple tags
        let data = [
            ("amenity", "restaurant"),
            ("name", "Joe's Diner"),
            ("addr:city", "TestCity"),
        ];
        let iter = make_tag_iter(&data);
        let result = collect_tags(iter);
        assert_eq!(result.len(), 3);
        assert_eq!(result.get("amenity"), Some(&"restaurant".to_string()));
        assert_eq!(result.get("name"), Some(&"Joe's Diner".to_string()));
        assert_eq!(result.get("addr:city"), Some(&"TestCity".to_string()));
    }

    #[traced_test]
    fn test_collect_tags_repeated_keys() {
        // If we have repeated keys, the last one wins (typical HashMap behavior)
        let data = [
            ("highway", "primary"),
            ("highway", "residential"),  // repeated
        ];
        let iter = make_tag_iter(&data);
        let result = collect_tags(iter);
        // We expect only 1 item: "highway" => "residential"
        assert_eq!(result.len(), 1);
        let val = result.get("highway").unwrap();
        assert_eq!(val, "residential");
    }

    #[traced_test]
    fn test_collect_tags_special_chars() {
        // Some keys or values might have punctuation or spaces
        let data = [
            ("foo@bar!", "some/value with spaces"),
            ("UPPERCASE", "MIXED@CASE#1"),
        ];
        let iter = make_tag_iter(&data);
        let result = collect_tags(iter);
        assert_eq!(result.len(), 2);
        assert_eq!(result["foo@bar!"], "some/value with spaces");
        assert_eq!(result["UPPERCASE"], "MIXED@CASE#1");
    }

    #[traced_test]
    fn test_collect_tags_unicode() {
        // Ensures we handle non-ASCII chars
        let data = [
            ("日本語", "値"),
            ("emoji_key", "🌍🚀"),
        ];
        let iter = make_tag_iter(&data);
        let result = collect_tags(iter);
        assert_eq!(result.len(), 2);
        assert_eq!(result["日本語"], "値");
        assert_eq!(result["emoji_key"], "🌍🚀");
    }
}
