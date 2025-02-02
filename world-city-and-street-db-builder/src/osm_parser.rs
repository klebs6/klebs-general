// ---------------- [ File: src/osm_parser.rs ]
crate::ix!();

/// Collect tags into a [`HashMap`], generic over any iterator of
/// `(&str, &str)`.  This allows both real `TagIter<'_>` from `osmpbf`
/// **and** test mocks to be used.
///
pub fn collect_tags<I>(tags: I) -> HashMap<String, String>
where
    I: Iterator<Item = (&'static str, &'static str)>,
{
    tags.map(|(k,v)| (k.to_string(), v.to_string())).collect()
}

/// Parse an OSM PBF file and return a list of [`AddressRecord`].
/// Uses [`osmpbf::ElementReader::from_path`] to read the file,
/// then attempts to build an [`AddressRecord`] from each node/way/etc.
///
pub fn parse_osm_pbf(path: impl AsRef<Path>, country: &Country)
    -> Result<Vec<AddressRecord>, OsmPbfParseError>
{
    let reader = ElementReader::from_path(path)?;
    let mut records = Vec::new();
    let mut count = 0;

    // For each element, we try to parse an `AddressRecord`.
    // If it succeeds, we push it onto our `records` vec.
    reader.for_each(|element| {
        if let Ok(record) = AddressRecord::try_from((element, country)) {
            if count % 1000 == 0 {
                info!("record for osm element, {:?}", record);
            }
            records.push(record);
        }
        count += 1;
    })?;

    Ok(records)
}

#[cfg(test)]
mod osm_parser_tests {
    use super::*;

    // ----------------------------------
    // 1) Unit tests for collect_tags(...)
    // ----------------------------------

    #[test]
    fn test_collect_tags_basic() {
        // We'll define a simple fake TagIter that yields a small set of (k,v).
        let fake_tags = vec![
            ("addr:city", "Baltimore"),
            ("addr:street", "North Avenue"),
        ];
        let fake_iter = FakeTagIter::new(fake_tags);

        // Now we can pass `fake_iter` directly because we've made `collect_tags` generic:
        let map = collect_tags(fake_iter);
        assert_eq!(map.len(), 2, "Should have 2 entries");
        assert_eq!(map.get("addr:city"),   Some(&"Baltimore".to_string()));
        assert_eq!(map.get("addr:street"), Some(&"North Avenue".to_string()));
    }

    #[test]
    fn test_collect_tags_empty() {
        // If we have no tags => empty map
        let fake_iter = FakeTagIter::new(vec![]);
        let map = collect_tags(fake_iter);
        assert!(map.is_empty());
    }

    #[test]
    fn test_collect_tags_overlapping_keys() {
        // If the underlying iterator yields the same key multiple times, 
        // the last inserted pair "wins" in a normal HashMap.
        let fake_tags = vec![
            ("addr:city", "Baltimore"),
            ("addr:city", "OverwrittenCity"),
        ];
        let fake_iter = FakeTagIter::new(fake_tags);

        let map = collect_tags(fake_iter);
        assert_eq!(map.len(), 1, "Duplicates => single final entry");
        assert_eq!(map.get("addr:city"), Some(&"OverwrittenCity".to_string()));
    }

    /// A small helper struct that implements `Iterator<Item=(&'static str,&'static str)>`
    /// so we can pass it to our now-generic `collect_tags(...)`.
    struct FakeTagIter {
        data: std::vec::IntoIter<(&'static str, &'static str)>,
    }

    impl FakeTagIter {
        fn new(tags: Vec<(&'static str, &'static str)>) -> Self {
            Self { data: tags.into_iter() }
        }
    }

    impl Iterator for FakeTagIter {
        type Item = (&'static str, &'static str);

        fn next(&mut self) -> Option<Self::Item> {
            self.data.next()
        }
    }

    // ------------------------------------------------
    // 2) Integration-like tests for parse_osm_pbf(...)
    // ------------------------------------------------

    #[test]
    fn test_parse_osm_pbf_non_existent_file() {
        let path = PathBuf::from("this_file_does_not_exist.osm.pbf");
        let result = parse_osm_pbf(path, &Country::USA);
        assert!(result.is_err(), "Should fail for non-existent file");
        match result.err().unwrap() {
            OsmPbfParseError::OsmPbf(osmpbf_err) => {
                // Good => proceed
                debug!("Got an osmpbf error as expected: {:?}", osmpbf_err);
            }
            other => panic!("Expected OsmPbf parse error, got: {:?}", other),
        }
    }

    #[test]
    fn test_parse_osm_pbf_corrupted_file() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let file_path = temp_dir.path().join("corrupted.osm.pbf");

        // Write some random data that is not a valid OSM PBF
        {
            let mut f = std::fs::File::create(&file_path).unwrap();
            f.write_all(b"This is not a valid OSM PBF").unwrap();
        }

        let result = parse_osm_pbf(&file_path, &Country::USA);
        assert!(result.is_err());
        match result.err().unwrap() {
            OsmPbfParseError::OsmPbf(err) => {
                debug!("Got osmpbf parse error for corrupted file: {:?}", err);
            },
            other => panic!("Expected OsmPbf parse error, got: {:?}", other),
        }
    }

    #[test]
    fn test_parse_osm_pbf_empty_file() {
        // If the file is empty, parse_osm_pbf may yield an error or an empty result,
        // depending on how osmpbf handles empty input.
        let temp_dir = TempDir::new().expect("temp dir");
        let file_path = temp_dir.path().join("empty.osm.pbf");
        std::fs::File::create(&file_path).unwrap(); // zero bytes

        let result = parse_osm_pbf(&file_path, &Country::USA);
        match result {
            Ok(records) => {
                // Possibly returns an empty Vec. 
                assert!(
                    records.is_empty(), 
                    "Empty file => likely no records, but got some? #records={}",
                    records.len()
                );
            },
            Err(OsmPbfParseError::OsmPbf(_)) => {
                // Some versions of osmpbf treat an empty file as an error. Also acceptable.
                info!("Empty file => parse error, acceptable outcome");
            },
            Err(other) => panic!("Unexpected parse error: {:?}", other),
        }
    }

    // An optional test for a minimal valid PBF:
    // You'd need a real fixture or advanced mocking of osmpbf.
    // Here we just show the pattern:
    #[test]
    #[ignore = "Requires a real minimal fixture .osm.pbf"]
    fn test_parse_osm_pbf_minimal_valid_fixture() {
        let fixture_path = PathBuf::from("tests/fixtures/mini.osm.pbf");
        let result = parse_osm_pbf(&fixture_path, &Country::USA).expect("should succeed parsing minimal fixture");
        assert!(!result.is_empty(), "Expected some small # of records in fixture");
    }
}
