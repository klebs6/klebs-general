crate::ix!();

/// Attempts to create a `StreetName` from a string (if present). Returns an error
/// if construction fails.
pub fn try_construct_street_name(
    street_raw: Option<&str>,
    element_id: i64,
) -> Result<Option<StreetName>, IncompatibleOsmPbfElement> {
    if let Some(raw_value) = street_raw {
        trace!("try_construct_street_name: Parsing street for element_id={}", element_id);
        match StreetName::new(raw_value) {
            Ok(street) => Ok(Some(street)),
            Err(e) => {
                error!("try_construct_street_name: StreetName construction error for element_id={}: {:?}", element_id, e);
                Err(IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
                    IncompatibleOsmPbfNode::StreetNameConstructionError(e),
                ))
            }
        }
    } else {
        debug!("try_construct_street_name: No street tag for element_id={}", element_id);
        Ok(None)
    }
}
