// ---------------- [ File: src/retrieve_housenumber_value.rs ]
crate::ix!();

/// Retrieves the `addr:housenumber` value from the collected tags, if present and non-empty.
///
/// # Returns
///
/// * `Ok(None)` if the housenumber key is absent or empty.
/// * `Ok(Some(&str))` containing a trimmed housenumber string otherwise.
/// * `Err(...)` if the data is invalid in a way that must produce an error.
pub fn retrieve_housenumber_value(
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
