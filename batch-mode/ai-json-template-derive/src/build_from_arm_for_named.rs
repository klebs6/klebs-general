crate::ix!();

// ---------------------------------------------------------------------------
//  Subroutine F: Build the final From-arm match snippet
// ---------------------------------------------------------------------------
pub fn build_from_arm_for_named(
    flat_parent_ident:   &syn::Ident,
    parent_enum_ident:   &syn::Ident,
    variant_ident:       &syn::Ident,
    renamed_just_var:    &syn::Ident,
    justification_ident: &syn::Ident,
    confidence_ident:    &syn::Ident,

    pattern_vars_top:    &[proc_macro2::TokenStream],
    pattern_vars_fields: &[proc_macro2::TokenStream],

    just_inits_top:      &[proc_macro2::TokenStream],
    just_inits_fields:   &[proc_macro2::TokenStream],
    conf_inits_top:      &[proc_macro2::TokenStream],
    conf_inits_fields:   &[proc_macro2::TokenStream],
    item_inits_fields:   &[proc_macro2::TokenStream],
) -> proc_macro2::TokenStream
{
    trace!(
        "build_from_arm_for_named: variant='{}', building the final match arm.",
        variant_ident
    );

    // A) Build the “pattern” snippet
    let mut pattern_vars = Vec::new();
    pattern_vars.extend_from_slice(pattern_vars_top);
    pattern_vars.extend_from_slice(pattern_vars_fields);

    // B) Build the “item” constructor
    let item_constructor = if !item_inits_fields.is_empty() {
        // e.g. MyEnum::VarName { alpha: alpha, beta: beta }
        let pairs: Vec<_> = item_inits_fields.iter().collect();
        quote::quote! {
            #parent_enum_ident :: #variant_ident {
                #( #pairs ),*
            }
        }
    } else {
        // no fields
        quote::quote! {
            #parent_enum_ident :: #variant_ident {}
        }
    };

    // C) Build the justification constructor
    let mut j_inits = Vec::new();
    j_inits.extend_from_slice(just_inits_top);
    j_inits.extend_from_slice(just_inits_fields);
    let just_ctor = if !j_inits.is_empty() {
        quote::quote! {
            #justification_ident :: #renamed_just_var {
                #( #j_inits ),*
            }
        }
    } else {
        quote::quote! {
            #justification_ident :: #renamed_just_var {}
        }
    };

    // D) Build the confidence constructor
    let mut c_inits = Vec::new();
    c_inits.extend_from_slice(conf_inits_top);
    c_inits.extend_from_slice(conf_inits_fields);
    let conf_ctor = if !c_inits.is_empty() {
        quote::quote! {
            #confidence_ident :: #renamed_just_var {
                #( #c_inits ),*
            }
        }
    } else {
        quote::quote! {
            #confidence_ident :: #renamed_just_var {}
        }
    };

    // E) Build final match arm
    if !pattern_vars.is_empty() {
        quote::quote! {
            #flat_parent_ident :: #variant_ident { #( #pattern_vars ),* } => {
                Self {
                    item: #item_constructor,
                    justification: #just_ctor,
                    confidence:    #conf_ctor,
                }
            }
        }
    } else {
        // no fields
        quote::quote! {
            #flat_parent_ident :: #variant_ident {} => {
                Self {
                    item: #parent_enum_ident :: #variant_ident {},
                    justification: #justification_ident :: #renamed_just_var {},
                    confidence:    #confidence_ident :: #renamed_just_var {},
                }
            }
        }
    }
}
