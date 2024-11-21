crate::ix!();

// Function to handle enums
pub fn derive_ai_descriptor_for_enum(
    enum_name: &syn::Ident,
    data_enum: &syn::DataEnum,
    attrs: &[syn::Attribute],
) -> TokenStream2 {

    let variants = &data_enum.variants;

    // Check for #[ai(Display)] attribute
    let derive_display = has_ai_display_attribute(attrs);

    let variant_arms = generate_variant_arms(enum_name, variants);

    // Generate the impl block
    let gen = generate_ai_descriptor_impl(enum_name, &variant_arms);

    // If #[ai(Display)] is present, implement Display
    let display_impl = generate_display_impl(derive_display, enum_name);

    quote! {
        #gen
        #display_impl
    }
}
