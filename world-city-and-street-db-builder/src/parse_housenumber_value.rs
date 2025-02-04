crate::ix!();

/// Parses a non-empty housenumber string as either a single number or a range.
///
/// # Returns
///
/// * `Ok(Some(HouseNumberRange))` on success.
/// * `Ok(None)` if the start-end was reversed or invalid in an ignorable way.
/// * `Err(IncompatibleOsmPbfElement)` if a parse error occurs.
pub fn parse_housenumber_value(
    hn_value: &str,
    element_id: i64,
) -> Result<Option<HouseNumberRange>, IncompatibleOsmPbfElement> {
    trace!(
        "parse_housenumber_value: attempting to parse='{}' (element_id={})",
        hn_value,
        element_id
    );

    if let Some(idx) = hn_value.find('-') {
        let (start_str, rest) = hn_value.split_at(idx);
        // skip the dash
        let end_str = &rest[1..];

        let start_num = parse_integer(start_str.trim(), element_id)?;
        let end_num = parse_integer(end_str.trim(), element_id)?;

        if start_num > end_num {
            debug!(
                "parse_housenumber_value: reversed or invalid range '{}-{}' => ignoring (element_id={})",
                start_num, end_num, element_id
            );
            return Ok(None);
        }

        let range = HouseNumberRange::new(start_num, end_num);
        debug!(
            "parse_housenumber_value: parsed valid range '{}' => {:?} (element_id={})",
            hn_value, range, element_id
        );
        Ok(Some(range))
    } else {
        // single integer
        let single_num = parse_integer(hn_value, element_id)?;
        let range = HouseNumberRange::new(single_num, single_num);
        debug!(
            "parse_housenumber_value: parsed single '{}' => {:?} (element_id={})",
            hn_value, range, element_id
        );
        Ok(Some(range))
    }
}
