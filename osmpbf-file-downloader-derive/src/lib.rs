#[macro_use] mod imports; use imports::*;

xp!{build_geofabrik_url_literal}
xp!{build_match_arm_for_variant}
xp!{get_continent_path}
xp!{parse_geofabrik_single_pair}

/// The main entry point for deriving `OsmPbfFileDownloader`.
/// Looks for `#[geofabrik(continent="region-latest.osm.pbf")]` on each enum variant.
#[proc_macro_derive(OsmPbfFileDownloader, attributes(geofabrik))]
pub fn derive_osm_pbf_file_downloader(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    // Ensure we're deriving on an enum.
    let enum_data = match &ast.data {
        Data::Enum(ed) => ed,
        _ => {
            return syn::Error::new_spanned(
                &ast.ident,
                "OsmPbfFileDownloader can only be derived for enums.",
            )
            .to_compile_error()
            .into();
        }
    };

    // For each variant in the enum, build a match arm.
    let arms = enum_data
        .variants
        .iter()
        .map(build_match_arm_for_variant)
        .collect::<Vec<_>>();

    let ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    // Generate the final implementation.
    let expanded = quote! {
        impl #impl_generics file_downloader::FileDownloader for #ident #ty_generics #where_clause {}
        impl #impl_generics file_downloader::Md5DownloadLink for #ident #ty_generics #where_clause {}
        impl #impl_generics file_downloader::DownloadLink for #ident #ty_generics #where_clause {
            fn download_link(&self) -> &str {
                match self {
                    #(#arms)*
                }
            }
        }
    };

    expanded.into()
}


