// ---------------- [ File: src/parse_city_names.rs ]
crate::ix!();

/// Parses city names from the `(key, value)` pairs. Extracts the city substring
/// after the second colon (`C2Z:US:baltimore => "baltimore"`). Additionally,
/// this step can decode the CBOR values for sanity checks, if desired.
pub fn parse_city_names(kv_pairs: Vec<(String, Vec<u8>)>) -> Vec<String> {
    trace!(
        "parse_city_names: parsing city names from {} key-value pairs",
        kv_pairs.len()
    );

    let mut cities = Vec::new();

    for (key_str, val_bytes) in kv_pairs {
        match extract_city_from_key(&key_str) {
            Some(city) => {
                debug!("parse_city_names: extracted city='{}' from key='{}'", city, key_str);
                // Optionally decode the CBOR to confirm validity:
                if let Err(e) = try_decode_postal_codes(&val_bytes) {
                    // We ignore the contents, but we can log an error if decoding fails
                    warn!(
                        "parse_city_names: postal code decoding failed for city='{}': {}",
                        city, e
                    );
                }
                cities.push(city);
            }
            None => {
                debug!(
                    "parse_city_names: skipping unexpected key='{}' (cannot parse city)",
                    key_str
                );
            }
        }
    }

    cities
}

#[cfg(test)]
#[disable]
mod test_parse_city_names {
    use super::*;
    use std::collections::BTreeSet;

    /// Produces a small valid CBOR that `try_decode_postal_codes` can parse without error.
    /// We can store a simple `CompressedList<PostalCode>` containing one or two items.
    fn make_valid_cbor() -> Vec<u8> {
        let pc = PostalCode::new(Country::USA, "12345").unwrap();
        let clist = CompressedList::from(vec![pc]);
        serde_cbor::to_vec(&clist).expect("Serialization should succeed")
    }

    /// Produces obviously invalid bytes that `try_decode_postal_codes` should fail on.
    fn make_corrupted_cbor() -> Vec<u8> {
        b"not valid cbor".to_vec()
    }

    #[test]
    fn test_empty_input_returns_empty_vec() {
        let kv_pairs = Vec::new();
        let result = parse_city_names(kv_pairs);
        assert!(result.is_empty(), "No pairs => no cities");
    }

    #[test]
    fn test_single_well_formed_key_extracts_city() {
        // key = "C2Z:US:baltimore" => city = "baltimore"
        let kv_pairs = vec![(
            "C2Z:US:baltimore".to_string(),
            make_valid_cbor(), // decode should succeed, but we ignore the result
        )];
        let result = parse_city_names(kv_pairs);
        assert_eq!(result, vec!["baltimore"], "Should parse the city substring");
    }

    #[test]
    fn test_multiple_keys_extract_multiple_cities() {
        let kv_pairs = vec![
            ("C2Z:US:baltimore".to_string(), make_valid_cbor()),
            ("C2Z:US:frederick".to_string(), make_valid_cbor()),
            ("C2Z:US:annapolis".to_string(), make_valid_cbor()),
        ]
        .into_iter()
        .map(|(k, v)| (k, v))
        .collect::<Vec<_>>();

        let mut result = parse_city_names(kv_pairs);
        result.sort();
        assert_eq!(
            result,
            vec!["annapolis", "baltimore", "frederick"],
            "Should parse all city substrings in alphabetical order"
        );
    }

    #[test]
    fn test_malformed_keys_are_skipped() {
        // If extract_city_from_key returns None, we skip the key
        // e.g. "C2Z:US" or "C2Z:US" missing the 3rd part
        let kv_pairs = vec![
            ("C2Z:US".to_string(), make_valid_cbor()),
            ("C2Z:".to_string(), make_valid_cbor()),
            ("SomeIrrelevant:Key".to_string(), make_valid_cbor()),
            // We'll have one valid key so we get at least one city
            ("C2Z:US:baltimore".to_string(), make_valid_cbor()),
        ];

        let result = parse_city_names(kv_pairs);
        assert_eq!(result, vec!["baltimore"], "Only the valid key should be parsed");
    }

    #[test]
    fn test_corrupted_cbor_still_extracts_city_but_logs_warning() {
        // The function logs a warning if the decode fails, but still pushes the city name.
        // We can't directly test the log output, but we confirm it doesn't skip the city.
        let kv_pairs = vec![(
            "C2Z:US:hagerstown".to_string(),
            make_corrupted_cbor(),
        )];

        let result = parse_city_names(kv_pairs);
        assert_eq!(result, vec!["hagerstown"]);
    }

    #[test]
    fn test_duplicate_keys_return_duplicates() {
        // The function does not deduplicate city names. It pushes them in order encountered.
        let kv_pairs = vec![
            ("C2Z:US:baltimore".to_string(), make_valid_cbor()),
            ("C2Z:US:baltimore".to_string(), make_valid_cbor()),
        ];
        let result = parse_city_names(kv_pairs);
        // We expect ["baltimore", "baltimore"]
        assert_eq!(
            result,
            vec!["baltimore", "baltimore"],
            "Should contain duplicates if repeated keys are present"
        );
    }
}
