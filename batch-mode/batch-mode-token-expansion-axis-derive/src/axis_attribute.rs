// ---------------- [ File: src/axis_attribute.rs ]
crate::ix!();

/// Parse out `#[axis("axis_name => axis_description")]`.
pub fn parse_axis_attribute(attr: &Attribute) -> Result<Option<(String, String)>, syn::Error> {
    if !attr.path().is_ident("axis") {
        return Ok(None);
    }

    // parse the entire attribute as one string literal
    let raw_value: syn::LitStr = attr.parse_args()?;
    let value_str = raw_value.value();

    // Split on "=>"
    let parts: Vec<&str> = value_str.splitn(2, "=>").map(str::trim).collect();
    if parts.len() != 2 {
        return Err(syn::Error::new_spanned(
            raw_value,
            r#"Expected format: "axis_name => axis_description""#,
        ));
    }

    let axis_name = parts[0].trim_matches('"').to_string();
    let axis_description = parts[1].trim_matches('"').to_string();
    Ok(Some((axis_name, axis_description)))
}
