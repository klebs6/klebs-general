// ---------------- [ File: src/push_name_arm.rs ]
crate::ix!();

/// Builds the `name()` match arm and appends it to the EnumArms builder.
pub fn push_name_arm(
    builder: &mut EnumArmsBuilder,
    enum_ident: &syn::Ident,
    var_ident: &syn::Ident,
    pattern: &proc_macro2::TokenStream,
    name_binding: Option<&syn::Ident>
) {
    let nm = name_binding.expect("Expected name binding to exist");
    trace!("Creating name arm for '{}::{}'", enum_ident, var_ident);

    let name_arm = quote! {
        #pattern => std::borrow::Cow::from(#nm)
    };
    builder.name_arms_push(name_arm);
}
