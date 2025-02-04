crate::ix!();

/// Attempts to create a `PostalCode` from a string (if present). Returns an error
/// if construction fails.
pub fn try_construct_postal_code(
    country: Country,
    postcode_raw: Option<&str>,
    element_id: i64,
) -> Result<Option<PostalCode>, IncompatibleOsmPbfElement> {
    if let Some(raw_value) = postcode_raw {
        trace!("try_construct_postal_code: Parsing postcode for element_id={}", element_id);
        match PostalCode::new(country, raw_value) {
            Ok(pc) => Ok(Some(pc)),
            Err(e) => {
                error!("try_construct_postal_code: PostalCode construction error for element_id={}: {:?}", element_id, e);
                Err(IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
                    IncompatibleOsmPbfNode::PostalCodeConstructionError(e),
                ))
            }
        }
    } else {
        debug!("try_construct_postal_code: No postcode tag for element_id={}", element_id);
        Ok(None)
    }
}
