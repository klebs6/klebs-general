// ---------------- [ File: src/extract_house_number_range_from_tags.rs ]
crate::ix!();

/// Attempts to parse a house number or house‐number range from typical OSM tags:
///   - `addr:housenumber = "123"`        => returns Range(123..=123)
///   - `addr:housenumber = "100-150"`    => returns Range(100..=150)
///   - If none is found or unparseable, returns `Ok(None)`.
///
/// # Arguments
///
/// * `tags_iter`  - An iterator over (key, value) tag pairs.
/// * `element_id` - The ID of the OSM element from which the tags are drawn.
///
/// # Returns
///
/// * `Ok(Some(HouseNumberRange))` if a valid range was found.
/// * `Ok(None)` if no parseable house number is present.
/// * `Err(IncompatibleOsmPbfElement)` if an error prevents us from parsing.
pub fn extract_house_number_range_from_tags<'a, I>(
    tags_iter: I,
    element_id: i64,
) -> Result<Option<HouseNumberRange>, IncompatibleOsmPbfElement>
where
    I: Iterator<Item = (&'a str, &'a str)>,
{
    trace!(
        "extract_house_number_range_from_tags: start (element_id={})",
        element_id
    );

    let tags = collect_tags(tags_iter);
    debug!(
        "extract_house_number_range_from_tags: collected {} tags (element_id={})",
        tags.len(),
        element_id
    );

    match retrieve_housenumber_value(&tags, element_id)? {
        None => {
            debug!(
                "extract_house_number_range_from_tags: no housenumber tag found (element_id={})",
                element_id
            );
            Ok(None)
        }
        Some(raw_value) => parse_housenumber_value(raw_value, element_id),
    }
}

/// Retrieves the `addr:housenumber` value from the collected tags, if present and non-empty.
///
/// # Returns
///
/// * `Ok(None)` if the housenumber key is absent or empty.
/// * `Ok(Some(&str))` containing a trimmed housenumber string otherwise.
/// * `Err(...)` if the data is invalid in a way that must produce an error.
fn retrieve_housenumber_value(
    tags: &HashMap<String, String>,
    element_id: i64,
) -> Result<Option<&str>, IncompatibleOsmPbfElement> {
    trace!(
        "retrieve_housenumber_value: checking for addr:housenumber (element_id={})",
        element_id
    );

    match tags.get("addr:housenumber") {
        None => Ok(None),
        Some(val) if val.trim().is_empty() => Ok(None),
        Some(val) => {
            debug!(
                "retrieve_housenumber_value: found housenumber='{}' (element_id={})",
                val, element_id
            );
            Ok(Some(val.trim()))
        }
    }
}

/// Parses a non-empty housenumber string as either a single number or a range.
///
/// # Returns
///
/// * `Ok(Some(HouseNumberRange))` on success.
/// * `Ok(None)` if the start-end was reversed or invalid in an ignorable way.
/// * `Err(IncompatibleOsmPbfElement)` if a parse error occurs.
fn parse_housenumber_value(
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

/// Parses a string as an unsigned integer (`u32`). Returns a domain error if invalid.
fn parse_integer(
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

