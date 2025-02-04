crate::ix!();

/// Attempts to construct an [`AddressRecord`] from a stream of OSM-style tags.
/// Returns an error if no address-related tags are found or if any field fails
/// to parse.
///
/// # Arguments
///
/// * `tags_iter`  - Iterator of key-value pairs representing OSM tags.
/// * `country`    - The country associated with the address record.
/// * `element_id` - Unique identifier (e.g., node id).
///
/// # Returns
///
/// * `Ok(AddressRecord)` if a valid record can be built.
/// * `Err(IncompatibleOsmPbfElement)` otherwise.
pub fn try_build_address_record_from_tags<'a>(
    tags_iter: impl Iterator<Item = (&'a str, &'a str)>,
    country: Country,
    element_id: i64,
) -> Result<AddressRecord, IncompatibleOsmPbfElement> {
    trace!("try_build_address_record_from_tags: Start for element_id={}", element_id);

    let tags = collect_tags(tags_iter);
    debug!(
        "try_build_address_record_from_tags: Collected {} tags for element_id={}",
        tags.len(),
        element_id
    );

    // 1. Extract city/street/postcode tags or return an error if all missing.
    let (city_raw, street_raw, postcode_raw) = try_extract_address_tags(&tags, element_id)?;

    // 2. Parse each tag into its strongly-typed representation.
    let city     = try_construct_city_name(city_raw, element_id)?;
    let street   = try_construct_street_name(street_raw, element_id)?;
    let postcode = try_construct_postal_code(country, postcode_raw, element_id)?;

    // 3. Assemble the final [`AddressRecord`].
    let record = try_assemble_address_record(city, street, postcode, element_id)?;

    info!("try_build_address_record_from_tags: Successfully built AddressRecord for element_id={}", element_id);
    Ok(record)
}
