// ---------------- [ File: src/extract_house_number_range_from_tags.rs ]
crate::ix!();

/// Attempts to parse a house number or house‐number range from typical OSM tags:
///   - `addr:housenumber = "123"`        => returns Range(123..=123)
///   - `addr:housenumber = "100-150"`    => returns Range(100..=150)
///   - If none is found or unparseable, returns `Ok(None)`.
///
/// Adjust to handle e.g. “100;102;104” or advanced syntax if desired.
pub fn extract_house_number_range_from_tags<'a,I>(
    tags_iter: I,
    element_id: i64,
) -> Result<Option<HouseNumberRange>, IncompatibleOsmPbfElement> 
where
    I: Iterator<Item=(&'a str, &'a str)>
{
    let tags: HashMap<_,_> = tags_iter.map(|(k,v)|(k.to_owned(),v.to_owned())).collect();

    let hn_value = match tags.get("addr:housenumber") {
        None => return Ok(None), // no housenumber => nothing to do
        Some(val) if val.trim().is_empty() => return Ok(None),
        Some(val) => val.trim(),
    };

    // If it contains a dash => treat as "start-end"
    if let Some(idx) = hn_value.find('-') {
        let (start_str, rest) = hn_value.split_at(idx);
        // rest includes the dash as first char => skip it
        let end_str = &rest[1..];
        let start_num: u32 = start_str.trim().parse().map_err(|_| 
            IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
                IncompatibleOsmPbfNode::Incompatible { id: element_id }
            )
        )?;
        let end_num: u32 = end_str.trim().parse().map_err(|_| 
            IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
                IncompatibleOsmPbfNode::Incompatible { id: element_id }
            )
        )?;
        if start_num > end_num {
            // In real usage you might flip them or skip. We'll skip here:
            return Ok(None);
        }

        let range = HouseNumberRange::new(start_num,end_num);

        Ok(Some(range))

    } else {
        // Single number
        let single: u32 = hn_value.parse().map_err(|_|
            IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
                IncompatibleOsmPbfNode::Incompatible { id: element_id }
            )
        )?;

        let range = HouseNumberRange::new(single,single);

        Ok(Some(range))
    }
}
