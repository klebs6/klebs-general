// ---------------- [ File: src/extract_house_number_range_from_element.rs ]
crate::ix!();

/// A macro that generates specialized extraction functions for each OSM element variant
/// (Node, Way, Relation, DenseNode) plus a unifying
/// `extract_house_number_range_from_element(...)` top‐level function.
///
/// For each variant, you supply:
/// 1) A name for the generated extraction function.
/// 2) The concrete element type (e.g. `osmpbf::Node`).
/// 3) The variant name (e.g. `Node`, `Way`, `Relation`, `DenseNode`).
/// 4) A closure that transforms a node‐specific error into the variant’s corresponding error.
///
#[macro_export]
macro_rules! generate_house_number_extractors {
    (
        $(
            $fn_name:ident,
            $elem_ty:ty,
            $elem_variant:ident,
            $error_transform:expr
        );+ $(;)?
    ) => {
        $(
            /// Generated function to extract a [`HouseNumberRange`] (if any) from an `$elem_variant`.
            /// Converts node‐specific errors using the supplied closure.
            pub fn $fn_name(elem: &$elem_ty) -> Result<Option<HouseNumberRange>, IncompatibleOsmPbfElement> {
                let id = elem.id();
                trace!(
                    "{}: extracting house number range for {} with id={}",
                    stringify!($fn_name),
                    stringify!($elem_variant),
                    id
                );

                let tag_iter = elem.tags().map(|(k, v)| (k, v));
                let result = extract_house_number_range_from_tags(tag_iter, id)
                    .map_err(|err| match err {
                        IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(node_err) => {
                            // Apply the variant‐specific transformation closure
                            let new_err = $error_transform(id, node_err);
                            error!(
                                "{}: converting node‐specific error into {} error for id={}",
                                stringify!($fn_name),
                                stringify!($elem_variant),
                                id
                            );
                            new_err
                        }
                        other => other,
                    });

                match &result {
                    Ok(Some(_)) => {
                        debug!(
                            "{}: found house‐number range for {} with id={}",
                            stringify!($fn_name),
                            stringify!($elem_variant),
                            id
                        );
                    }
                    Ok(None) => {
                        debug!(
                            "{}: no house‐number range for {} with id={}",
                            stringify!($fn_name),
                            stringify!($elem_variant),
                            id
                        );
                    }
                    Err(e) => {
                        error!(
                            "{}: error extracting range for {} with id={}: {:?}",
                            stringify!($fn_name),
                            stringify!($elem_variant),
                            id,
                            e
                        );
                    }
                }

                result
            }
        )+

        /// A unified function that dispatches to the appropriate
        /// extractor depending on the element variant.
        pub fn extract_house_number_range_from_element<'a>(
            element: &'a osmpbf::Element<'a>
        ) -> Result<Option<HouseNumberRange>, IncompatibleOsmPbfElement> {
            match element {
                $(
                    osmpbf::Element::$elem_variant(e) => $fn_name(e),
                )+
            }
        }
    }
}

/// Example usage of the macro. Here we define four specialized
/// extraction functions plus the unified `extract_house_number_range_from_element`.
generate_house_number_extractors!(
    extract_house_number_range_from_node,
    osmpbf::Node,
    Node,
    // Node has no special error mapping; we pass the node error through unchanged.
    |_, node_err| IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(node_err);

    extract_house_number_range_from_way,
    osmpbf::Way,
    Way,
    // Convert node‐specific error => way‐specific error
    |id, _| IncompatibleOsmPbfElement::IncompatibleOsmPbfWay(
        IncompatibleOsmPbfWay::Incompatible { id }
    );

    extract_house_number_range_from_relation,
    osmpbf::Relation,
    Relation,
    // Convert node‐specific error => relation‐specific error
    |id, _| IncompatibleOsmPbfElement::IncompatibleOsmPbfRelation(
        IncompatibleOsmPbfRelation::Incompatible { id }
    );

    extract_house_number_range_from_dense_node,
    osmpbf::DenseNode,
    DenseNode,
    // Convert node‐specific error => dense‐node‐specific error
    |id, _| IncompatibleOsmPbfElement::IncompatibleOsmPbfDenseNode(
        IncompatibleOsmPbfDenseNode::Incompatible { id }
    );
);

// In src/extract_house_number_range_from_element.rs (or in your tests/integration folder)
#[cfg(test)]
mod extract_house_number_range_from_element_integration_tests {
    use super::*;
    use std::fs::File;

    /// Helper: creates a minimal OSM PBF file with one Node.
    /// The node will have the given parameters.
    async fn create_minimal_osm_pbf(
        path: &Path,
        housenumber: Option<&str>,
        node_id: i64,
    ) -> std::io::Result<()> {
        // We use the common bounding box near Baltimore.
        create_small_osm_pbf_file(
            path,
            (-77_000_000_000, -76_000_000_000, 39_000_000_000, 38_000_000_000),
            "TestCity",
            "TestStreet",
            "11111",
            housenumber,
            39.283,
            -76.616,
            node_id,
        ).await
    }

    #[traced_test]
    async fn test_extract_house_number_range_from_node_valid_range() {
        // Create a minimal osm.pbf file with a Node that has a valid house number range "100-110"
        let tmp_dir = TempDir::new().expect("Failed to create temporary directory");
        let pbf_path = tmp_dir.path().join("test_valid_node.osm.pbf");

        create_minimal_osm_pbf(&pbf_path, Some("100-110"), 1234)
            .await
            .expect("Failed to create minimal OSM PBF file");

        // Open the file with osmpbf ElementReader.
        let file = File::open(&pbf_path).expect("Failed to open test file");
        let reader = ElementReader::new(file);

        let mut found = false;
        reader.for_each(|element| {
            match extract_house_number_range_from_element(&element) {
                Ok(Some(range)) => {
                    // We expect the range to be [100, 110].
                    assert_eq!(*range.start(), 100, "Expected start to be 100");
                    assert_eq!(*range.end(), 110, "Expected end to be 110");
                    found = true;
                }
                Ok(None) => {}, // This element had no housenumber
                Err(e) => panic!("Unexpected error: {:?}", e),
            }
        });
        assert!(found, "Expected at least one Node with a valid house number range");
    }

    #[traced_test]
    async fn test_extract_house_number_range_from_node_no_housenumber() {
        // Create a minimal file with a Node that does not include addr:housenumber.
        let tmp_dir = TempDir::new().expect("Failed to create temporary directory");
        let pbf_path = tmp_dir.path().join("test_no_hn_node.osm.pbf");

        create_minimal_osm_pbf(&pbf_path, None, 2345)
            .await
            .expect("Failed to create minimal OSM PBF file");

        let file = File::open(&pbf_path).expect("Failed to open test file");
        let reader = ElementReader::new(file);

        let mut seen_none = false;

        reader.for_each(|element| {
            match extract_house_number_range_from_element(&element) {
                Ok(None) => seen_none = true,
                Ok(Some(_)) => panic!("Did not expect a house number range"),
                Err(e) => panic!("Unexpected error: {:?}", e),
            }
        });

        assert!(seen_none, "Expected a Node with no housenumber to yield None");
    }

    #[traced_test]
    async fn test_extract_house_number_range_from_node_invalid_housenumber() {
        // Create a minimal file with a Node that has an invalid housenumber (non-numeric)
        let tmp_dir = TempDir::new().expect("Failed to create temporary directory");
        let pbf_path = tmp_dir.path().join("test_invalid_hn_node.osm.pbf");

        // Use an invalid housenumber value like "ABC".
        create_minimal_osm_pbf(&pbf_path, Some("ABC"), 3456)
            .await
            .expect("Failed to create minimal OSM PBF file");

        let file = File::open(&pbf_path).expect("Failed to open test file");
        let reader = ElementReader::new(file);

        let mut saw_error = false;
        reader.for_each(|element| {
            match extract_house_number_range_from_element(&element) {
                Ok(Some(_)) => panic!("Expected an error for invalid housenumber"),
                Ok(None) => panic!("Expected an error for invalid housenumber, not None"),
                Err(e) => {
                    // Check that the error is the expected kind (here we expect an Incompatible error).
                    match e {
                        IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(_) => saw_error = true,
                        _ => panic!("Expected IncompatibleOsmPbfNode error, got {:?}", e),
                    }
                },
            }
        });
        assert!(saw_error, "Expected at least one error for invalid housenumber");
    }
}
