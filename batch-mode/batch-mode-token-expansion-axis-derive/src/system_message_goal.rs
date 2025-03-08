// ---------------- [ File: src/system_message_goal.rs ]
crate::ix!();

/// Attempts to parse `#[system_message_goal = "some text"]` or
/// `#[system_message_goal("some text")]` from the attribute list.
/// Returns `Ok(Some(lit_str))` if one is found; otherwise `Ok(None)`.
pub fn parse_system_message_goal(attrs: &[Attribute]) -> SynResult<Option<LitStr>> {
    for attr in attrs {
        // 1) Check that the attribute's path is exactly `system_message_goal`
        if !attr.path().is_ident("system_message_goal") {
            continue;
        }

        // 2) Try the name-value form: #[system_message_goal = "some text"]
        if let Ok(Some(lit_str)) = try_parse_name_value(attr) {
            return Ok(Some(lit_str));
        }

        // 3) If that fails, try the parentheses form: #[system_message_goal("some text")]
        if let Ok(Some(lit_str)) = try_parse_parenthesized(attr) {
            return Ok(Some(lit_str));
        }

        // If both returned Ok(None) or we got parse errors we swallowed,
        // we can continue or bail out here. We'll bail out to avoid re-checking
        // the same attribute multiple times.
        return Ok(None);
    }

    Ok(None) // No system_message_goal attribute found at all
}
