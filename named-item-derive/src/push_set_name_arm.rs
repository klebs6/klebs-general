// ---------------- [ File: src/push_set_name_arm.rs ]
crate::ix!();

/// Builds the `set_name()` match arm and appends it to the EnumArms builder.
pub fn push_set_name_arm(
    builder: &mut EnumArmsBuilder,
    cfg: &NamedItemConfig,
    enum_ident: &syn::Ident,
    var_ident: &syn::Ident,
    pattern: &proc_macro2::TokenStream,
    name_binding: Option<&syn::Ident>,
    hist_binding: Option<&syn::Ident>
) {
    let nm = name_binding.expect("Expected name binding in push_set_name_arm");
    trace!("Creating set_name arm for '{}::{}'", enum_ident, var_ident);

    let set_name_logic = if *cfg.history() {
        let hist = hist_binding.expect("Expected history binding if history=true");
        quote! {
            trace!("enum '{}' variant '{}' set_name('{}')",
                   stringify!(#enum_ident), stringify!(#var_ident), name);
            #hist.push(name.to_string());
            if name.is_empty() && name != &*Self::default_name() {
                warn!("Empty name not allowed on '{}::{}'",
                      stringify!(#enum_ident), stringify!(#var_ident));
                return Err(NameError::EmptyName);
            }
            #nm.clear();
            #nm.push_str(name);
            Ok(())
        }
    } else {
        quote! {
            trace!("enum '{}' variant '{}' set_name('{}')",
                   stringify!(#enum_ident), stringify!(#var_ident), name);
            if name.is_empty() && name != &*Self::default_name() {
                warn!("Empty name not allowed on '{}::{}'",
                      stringify!(#enum_ident), stringify!(#var_ident));
                return Err(NameError::EmptyName);
            }
            #nm.clear();
            #nm.push_str(name);
            Ok(())
        }
    };

    let set_name_arm = quote! {
        #pattern => {
            #set_name_logic
        }
    };
    builder.set_name_arms_push(set_name_arm);
}
