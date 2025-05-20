crate::ix!();

///TODO: this might not be useful in this crate. maybe remove it.
pub fn find_default_variant(
    data_enum: &syn::DataEnum
) -> syn::Result<Option<&syn::Variant>> {
    let mut default_variant: Option<&syn::Variant> = None;

    for variant in &data_enum.variants {
        for attr in &variant.attrs {
            if attr.path().is_ident("default") {
                if default_variant.is_some() {
                    // Found two or more `#[default]` attributes => error
                    return Err(syn::Error::new(
                        attr.span(),
                        "Multiple variants have `#[default]`; only one is allowed."
                    ));
                }
                default_variant = Some(variant);
            }
        }
    }
    Ok(default_variant)
}
