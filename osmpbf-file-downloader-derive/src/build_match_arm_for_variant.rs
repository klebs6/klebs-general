crate::ix!();

/// Builds the match arm for a single variant of the enum.
pub fn build_match_arm_for_variant(variant: &Variant) -> proc_macro2::TokenStream {
    let var_ident = &variant.ident;

    match &variant.fields {
        // Unit variant:  e.g. `Greenland`
        syn::Fields::Unit => {
            match parse_geofabrik_single_pair(&variant.attrs) {
                Ok(Some((continent_key, region_file))) => {
                    let url_lit = build_geofabrik_url_literal(variant, &continent_key, &region_file);
                    quote!( Self::#var_ident => #url_lit, )
                }
                // If no attribute or invalid
                _ => syn::Error::new_spanned(
                    variant,
                    "Missing or invalid #[geofabrik(continent=\"region-latest.osm.pbf\")]"
                )
                .to_compile_error(),
            }
        }

        // Single-field tuple variant: e.g. `UnitedStates(USRegion)`
        // We allow an optional `#[geofabrik(...)]` override or else we delegate.
        syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
            match parse_geofabrik_single_pair(&variant.attrs) {
                Ok(Some((continent_key, region_file))) => {
                    let url_lit = build_geofabrik_url_literal(variant, &continent_key, &region_file);
                    quote!( Self::#var_ident(..) => #url_lit, )
                }
                Ok(None) => {
                    // No attribute → delegate to the single field's `download_link()`.
                    quote!( Self::#var_ident(inner) => inner.download_link(), )
                }
                Err(e) => e.to_compile_error(),
            }
        }

        // Single-field named variant: e.g. `UnitedStates { region: USRegion }`
        syn::Fields::Named(fields) if fields.named.len() == 1 => {
            match parse_geofabrik_single_pair(&variant.attrs) {
                Ok(Some((continent_key, region_file))) => {
                    let url_lit = build_geofabrik_url_literal(variant, &continent_key, &region_file);
                    quote!( Self::#var_ident { .. } => #url_lit, )
                }
                Ok(None) => {
                    // No attribute → delegate to the single field's `download_link()`.
                    let field_ident = &fields.named.iter().next().unwrap().ident;
                    quote!( Self::#var_ident { #field_ident } => #field_ident.download_link(), )
                }
                Err(e) => e.to_compile_error(),
            }
        }

        // Other forms (multi-field tuples or named) are not supported by default:
        _ => syn::Error::new_spanned(
            variant,
            "OsmPbfFileDownloader only supports unit variants or single-field variants."
        )
        .to_compile_error(),
    }
}


