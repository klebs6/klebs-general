crate::ix!();

/// Attempts to parse a single `#[geofabrik(continent="region-latest.osm.pbf")]`
/// from the given attributes. Returns:
/// - `Ok(Some((continent, region)))` if a valid attribute is found
/// - `Ok(None)` if no such attribute was found
/// - `Err(syn::Error)` if the attribute is present but invalid
pub fn parse_geofabrik_attr(
    attrs: &[Attribute],
) -> Result<Option<(String, String)>, syn::Error> {
    for attr in attrs {
        // We only care about attributes named `geofabrik`
        if attr.path().is_ident("geofabrik") {
            // With syn 2.0, prefer `parse_args_with(...)` or similar. We'll parse a MetaList.
            let list: MetaList = attr.parse_args_with(MetaList::parse_nested_meta)?;
            if list.nested.len() == 1 {
                if let Some(NestedMeta::Meta(Meta::NameValue(MetaNameValue { path, value, .. }))) =
                    list.nested.first()
                {
                    // The `path` is something like `spain`
                    let continent_ident = path.get_ident().ok_or_else(|| {
                        syn::Error::new_spanned(path, "Expected a simple identifier")
                    })?;
                    let continent_key = continent_ident.to_string();

                    // We expect `value` to be a string literal e.g. `"valencia-latest.osm.pbf"`.
                    match value {
                        syn::Expr::Lit(expr_lit) => match &expr_lit.lit {
                            Lit::Str(lit_str) => {
                                let region_file = lit_str.value();
                                return Ok(Some((continent_key, region_file)));
                            }
                            _ => {
                                return Err(syn::Error::new_spanned(
                                    value,
                                    "Expected string literal in geofabrik attribute",
                                ))
                            }
                        },
                        _ => {
                            return Err(syn::Error::new_spanned(
                                value,
                                "Expected string literal in geofabrik attribute",
                            ))
                        }
                    }
                }
            }

            // If we got here, the attribute was recognized but in the wrong shape.
            return Err(syn::Error::new_spanned(
                attr,
                "Expected #[geofabrik(continent=\"region-latest.osm.pbf\")]",
            ));
        }
    }

    // If there's no `geofabrik` attribute at all, return Ok(None).
    Ok(None)
}
