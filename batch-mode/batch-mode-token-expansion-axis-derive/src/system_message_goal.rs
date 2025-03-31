// ---------------- [ File: batch-mode-token-expansion-axis-derive/src/system_message_goal.rs ]
crate::ix!();

/// Attempts to parse `#[system_message_goal = "some text"]` or
/// `#[system_message_goal("some text")]` from the attribute list.
/// Returns `Ok(Some(lit_str))` if one is found; otherwise `Ok(None)`.
#[tracing::instrument(level = "trace", skip(attrs))]
pub fn parse_system_message_goal(attrs: &[Attribute]) -> SynResult<Option<LitStr>> {
    trace!("Looking for a #[system_message_goal(...)] or #[system_message_goal = ...] attribute.");

    for attr in attrs {
        // Check that this attribute is named `system_message_goal`
        if !attr.path().is_ident("system_message_goal") {
            trace!("Skipping non-system_message_goal attribute: {:?}", attr.path());
            continue;
        }

        // First, try the name-value style: #[system_message_goal = "some text"]
        if let Ok(Some(lit_str)) = try_parse_name_value(attr) {
            debug!("Parsed system_message_goal via name-value syntax: {:?}", lit_str.value());
            return Ok(Some(lit_str));
        }

        // Then, try the parenthesized/single-literal style: #[system_message_goal("some text")]
        if let Ok(Some(lit_str)) = try_parse_parenthesized(attr) {
            debug!("Parsed system_message_goal via parentheses syntax: {:?}", lit_str.value());
            return Ok(Some(lit_str));
        }

        // If we got here, that means neither parse worked, so we treat it as "not found".
        // (One might raise an error instead, but the code uses a default if not found.)
        warn!("Found a #[system_message_goal] attribute that didn't parse as name-value or literal; using default.");
        return Ok(None);
    }

    trace!("No #[system_message_goal(...)] attribute found; returning None.");
    Ok(None)
}
