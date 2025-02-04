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
