// ---------------- [ File: src/push_history_arms.rs ]
crate::ix!();

/// Builds the arms for `NameHistory` trait methods (add_name_to_history, name_history)
/// and appends them to the EnumArms builder.
pub fn push_history_arms(
    builder: &mut EnumArmsBuilder,
    enum_ident: &syn::Ident,
    var_ident: &syn::Ident,
    pattern: &proc_macro2::TokenStream,
    hist_binding: Option<&syn::Ident>
) {
    let hist = hist_binding.expect("Expected history binding if history=true");
    trace!("Creating history arms for '{}::{}'", enum_ident, var_ident);

    let add_arm = quote! {
        #pattern => {
            trace!("enum '{}' variant '{}' add_name_to_history('{}')",
                   stringify!(#enum_ident), stringify!(#var_ident), name);
            #hist.push(name.to_string());
        }
    };
    let get_arm = quote! {
        #pattern => {
            trace!("enum '{}' variant '{}' name_history()",
                   stringify!(#enum_ident), stringify!(#var_ident));
            #hist.iter()
                 .map(|s| std::borrow::Cow::from(&s[..]))
                 .collect()
        }
    };

    builder.history_arms_add_push(add_arm);
    builder.history_arms_get_push(get_arm);
}
