crate::ix!();

/// Builds the final URL literal from a `continent_key` and `region_file`.
/// Returns a `quote!`d literal expression. Errors if the continent key
/// is unknown according to `get_continent_path`.
pub fn build_geofabrik_url_literal(
    variant: &Variant,
    continent_key: &str,
    region_file: &str,
) -> proc_macro2::TokenStream {
    match get_continent_path(continent_key) {
        Ok(Some(base_path)) => {
            let final_url = format!("https://download.geofabrik.de/{}/{}", base_path, region_file);
            let literal = proc_macro2::Literal::string(&final_url);
            quote!( #literal )
        },
        Ok(None) => {
            let final_url = format!("https://download.geofabrik.de/{}", region_file);
            let literal = proc_macro2::Literal::string(&final_url);
            quote!( #literal )
        }
        Err(_) => {
            // If the user typed a continent that's not in your `get_continent_path`,
            // produce a compiler error at that location.
            syn::Error::new(
                variant.span(),
                format!("Unknown continent key: '{}'", continent_key),
            ).to_compile_error()
        }
    }
}
