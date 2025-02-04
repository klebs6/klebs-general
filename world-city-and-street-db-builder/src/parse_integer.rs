// ---------------- [ File: src/parse_integer.rs ]
crate::ix!();

/// Parses a string as an unsigned integer (`u32`). Returns a domain error if invalid.
pub fn parse_integer(
    s: &str,
    element_id: i64,
) -> Result<u32, IncompatibleOsmPbfElement> {
    trace!(
        "parse_integer: parsing '{}' as u32 (element_id={})",
        s,
        element_id
    );

    s.parse::<u32>().map_err(|parse_err| {
        error!(
            "parse_integer: unable to parse '{}' as u32 (element_id={}): {}",
            s, element_id, parse_err
        );
        IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
            IncompatibleOsmPbfNode::Incompatible { id: element_id }
        )
    })
}
