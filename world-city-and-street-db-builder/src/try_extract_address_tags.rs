crate::ix!();

/// Searches a tag map for `addr:city`, `addr:street`, and `addr:postcode`.
/// Returns an error if none are present.
///
/// # Returns
///
/// * `Ok((city_raw, street_raw, postcode_raw))` if at least one is present.
/// * `Err(IncompatibleOsmPbfElement)` if all are absent.
pub fn try_extract_address_tags(
    tags: &std::collections::HashMap<String, String>,
    element_id: i64,
) -> Result<(Option<&str>, Option<&str>, Option<&str>), IncompatibleOsmPbfElement> {
    trace!("try_extract_address_tags: Looking for address tags in element_id={}", element_id);

    let city_raw     = tags.get("addr:city").map(|s| s.as_str());
    let street_raw   = tags.get("addr:street").map(|s| s.as_str());
    let postcode_raw = tags.get("addr:postcode").map(|s| s.as_str());

    if city_raw.is_none() && street_raw.is_none() && postcode_raw.is_none() {
        warn!("try_extract_address_tags: No addr:city/addr:street/addr:postcode for element_id={}", element_id);
        return Err(IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
            IncompatibleOsmPbfNode::Incompatible { id: element_id }
        ));
    }

    Ok((city_raw, street_raw, postcode_raw))
}
