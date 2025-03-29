// ---------------- [ File: src/push_aliases_arms.rs ]
crate::ix!();

/// Builds the arms for `NamedAlias` trait methods (add_alias, aliases, clear_aliases)
/// and appends them to the EnumArms builder.
pub fn push_aliases_arms(
    builder: &mut EnumArmsBuilder,
    enum_ident: &syn::Ident,
    var_ident: &syn::Ident,
    pattern: &proc_macro2::TokenStream,
    alias_binding: Option<&syn::Ident>
) {
    use tracing::trace;

    let al = alias_binding.expect("Expected alias binding if aliases=true");
    trace!("Creating alias arms for '{}::{}'", enum_ident, var_ident);

    let add_alias_arm = quote! {
        #pattern => {
            trace!("enum '{}' variant '{}' add_alias('{}')",
                   stringify!(#enum_ident), stringify!(#var_ident), alias);
            #al.push(alias.to_string());
        }
    };
    let get_aliases_arm = quote! {
        #pattern => {
            trace!("enum '{}' variant '{}' aliases()",
                   stringify!(#enum_ident), stringify!(#var_ident));
            #al.iter()
               .map(|s| std::borrow::Cow::from(&s[..]))
               .collect()
        }
    };
    let clear_aliases_arm = quote! {
        #pattern => {
            trace!("enum '{}' variant '{}' clear_aliases()",
                   stringify!(#enum_ident), stringify!(#var_ident));
            #al.clear();
        }
    };

    builder.aliases_arms_add_push(add_alias_arm);
    builder.aliases_arms_get_push(get_aliases_arm);
    builder.aliases_arms_clear_push(clear_aliases_arm);
}
