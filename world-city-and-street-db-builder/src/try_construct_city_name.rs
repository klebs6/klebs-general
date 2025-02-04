crate::ix!();

/// Attempts to create a `CityName` from a string (if present). Returns an error
/// if construction fails.
pub fn try_construct_city_name(
    city_raw: Option<&str>,
    element_id: i64,
) -> Result<Option<CityName>, IncompatibleOsmPbfElement> {
    if let Some(raw_value) = city_raw {
        trace!("try_construct_city_name: Parsing city for element_id={}", element_id);
        match CityName::new(raw_value) {
            Ok(city) => Ok(Some(city)),
            Err(e) => {
                error!("try_construct_city_name: CityName construction error for element_id={}: {:?}", element_id, e);
                Err(IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
                    IncompatibleOsmPbfNode::CityNameConstructionError(e),
                ))
            }
        }
    } else {
        debug!("try_construct_city_name: No city tag for element_id={}", element_id);
        Ok(None)
    }
}
