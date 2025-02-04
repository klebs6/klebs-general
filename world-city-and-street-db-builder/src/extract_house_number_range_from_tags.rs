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
