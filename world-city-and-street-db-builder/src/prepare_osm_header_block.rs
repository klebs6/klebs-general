// ---------------- [ File: src/prepare_osm_header_block.rs ]
// ---------------- [ File: src/prepare_osm_header_block.rs ]
crate::ix!();

use crate::proto::osmformat;

/// Builds an OSM header block with given bounding box and required features.
pub fn prepare_osm_header_block(bbox: (i64, i64, i64, i64)) -> osmformat::HeaderBlock {
    trace!("prepare_osm_header_block: using bbox={:?}", bbox);

    let (left, right, top, bottom) = bbox;
    let mut headerblock = osmformat::HeaderBlock::new();

    let mut hbbox = osmformat::HeaderBBox::new();
    hbbox.set_left(left);
    hbbox.set_right(right);
    hbbox.set_top(top);
    hbbox.set_bottom(bottom);

    headerblock.bbox = protobuf::MessageField::from_option(Some(hbbox));
    headerblock.required_features.push("OsmSchema-V0.6".to_string());
    headerblock.required_features.push("DenseNodes".to_string());

    debug!("prepare_osm_header_block: HeaderBlock created");
    headerblock
}

#[cfg(test)]
#[disable]
mod test_prepare_osm_header_block {
    use super::*;
    use crate::proto::osmformat; // Ensure the correct path to your osmformat is used
    use protobuf::Message;        // For writing/reading the proto if desired

    #[traced_test]
    fn test_normal_bbox_values() {
        // A typical bounding box near (left=-77, right=-76, top=39, bottom=38) in nano-degrees
        // For instance: (-77_000_000_000, -76_000_000_000, 39_000_000_000, 38_000_000_000)
        let bbox = (-77_000_000_000, -76_000_000_000, 39_000_000_000, 38_000_000_000);
        let header_block = prepare_osm_header_block(bbox);

        // Check the bounding box fields
        assert_eq!(header_block.get_bbox().left(), bbox.0);
        assert_eq!(header_block.get_bbox().right(), bbox.1);
        assert_eq!(header_block.get_bbox().top(), bbox.2);
        assert_eq!(header_block.get_bbox().bottom(), bbox.3);

        // Check required features
        let features = header_block.get_required_features();
        assert!(
            features.contains(&"OsmSchema-V0.6".to_string())
                && features.contains(&"DenseNodes".to_string()),
            "Should include the required features OsmSchema-V0.6 and DenseNodes"
        );
    }

    #[traced_test]
    fn test_zero_bbox_values() {
        // If the bounding box is all zeros, let's confirm it sets them accordingly.
        let bbox = (0, 0, 0, 0);
        let header_block = prepare_osm_header_block(bbox);

        assert_eq!(header_block.get_bbox().left(), 0);
        assert_eq!(header_block.get_bbox().right(), 0);
        assert_eq!(header_block.get_bbox().top(), 0);
        assert_eq!(header_block.get_bbox().bottom(), 0);

        let features = header_block.get_required_features();
        assert_eq!(features.len(), 2, "Expected exactly two required features");
    }

    #[traced_test]
    fn test_negative_or_reversed_bbox() {
        // Some OSM data might have left > right or top < bottom if the data is unusual.
        // In real usage, you'd want to check if that’s correct or not. But let's just confirm
        // it sets exactly what we provide.
        let bbox = (100, 50, -100, -150);
        let header_block = prepare_osm_header_block(bbox);

        assert_eq!(header_block.get_bbox().left(), 100);
        assert_eq!(header_block.get_bbox().right(), 50);
        assert_eq!(header_block.get_bbox().top(), -100);
        assert_eq!(header_block.get_bbox().bottom(), -150);

        // The function doesn't attempt to fix or reorder them, so we just confirm the raw assignment.
        let features = header_block.get_required_features();
        assert_eq!(features[0], "OsmSchema-V0.6");
        assert_eq!(features[1], "DenseNodes");
    }

    #[traced_test]
    fn test_serialization_round_trip() {
        // If you want to confirm the proto can be serialized and read back:
        let bbox = (-123456789, 123456789, 999999999, -999999999);
        let header_block_original = prepare_osm_header_block(bbox);

        let bytes = header_block_original
            .write_to_bytes()
            .expect("Should serialize successfully");

        let header_block_parsed: osmformat::HeaderBlock =
            protobuf::Message::parse_from_bytes(&bytes)
                .expect("Should deserialize successfully");

        let parsed_bbox = header_block_parsed.get_bbox();
        assert_eq!(parsed_bbox.left(), bbox.0);
        assert_eq!(parsed_bbox.right(), bbox.1);
        assert_eq!(parsed_bbox.top(), bbox.2);
        assert_eq!(parsed_bbox.bottom(), bbox.3);

        let features = header_block_parsed.get_required_features();
        assert!(
            features.contains(&"OsmSchema-V0.6".to_string()) 
            && features.contains(&"DenseNodes".to_string())
        );
    }
}
