crate::ix!();

/// Attempt to parse a single assignment expression from `#[geofabrik(...)]`.
///
/// For example, `#[geofabrik(spain="valencia-latest.osm.pbf")]` is parsed as:
///   * left side: spain
///   * right side: "valencia-latest.osm.pbf"
///
/// Returns:
///   - `Ok(Some((continent_key, region_file)))` if a valid assignment is found
///   - `Ok(None)` if no `geofabrik` attribute was found
///   - `Err(e)` if the `geofabrik` attribute was present but malformed
pub fn parse_geofabrik_single_pair(
    attrs: &[Attribute],
) -> syn::Result<Option<(String, String)>> {
    for attr in attrs {
        if attr.path().is_ident("geofabrik") {
            // The contents inside the parentheses is typically:  spain = "valencia-latest.osm.pbf"
            // We'll parse that as an expression, e.g. Expr::Assign(...).
            let expr: Expr = attr.parse_args()?;

            if let Expr::Assign(ExprAssign { left, right, .. }) = expr {
                // Expecting left side to be e.g. spain
                if let Expr::Path(ExprPath { path, .. }) = *left {
                    if let Some(ident) = path.get_ident() {
                        let continent_key = ident.to_string();

                        // Right side should be a string literal: "valencia-latest.osm.pbf"
                        if let Expr::Lit(ExprLit { lit: Lit::Str(lit_str), .. }) = *right {
                            let region_file = lit_str.value();
                            return Ok(Some((continent_key, region_file)));
                        }
                    }
                }
            }
            return Err(syn::Error::new_spanned(
                attr,
                "Expected #[geofabrik(continent=\"region-latest.osm.pbf\")] as a single assignment."
            ));
        }
    }
    // No `geofabrik` attribute found at all
    Ok(None)
}
