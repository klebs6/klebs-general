// ---------------- [ File: ai-json-template-derive/src/finalize_from_arm_unnamed_variant_ts.rs ]
crate::ix!();

pub fn finalize_from_arm_unnamed_variant_ts(
    parent_enum_ident:   &syn::Ident,
    variant_ident:       &syn::Ident,
    justification_ident: &syn::Ident,
    confidence_ident:    &syn::Ident,
    expansions:          &UnnamedVariantExpansion
) -> TokenStream2
{
    trace!(
        "Constructing from-arm expansions for unnamed variant '{}::{}'",
        parent_enum_ident,
        variant_ident
    );

    let flat_parent_ident = syn::Ident::new(
        &format!("FlatJustified{}", parent_enum_ident),
        parent_enum_ident.span()
    );

    // Possibly rename "Unit" => "UnitVariant" for the justification
    let raw_vname = variant_ident.to_string();
    let renamed_just_var = if raw_vname == "Unit" {
        syn::Ident::new("UnitVariant", variant_ident.span())
    } else {
        variant_ident.clone()
    };

    // Construct the final item expression
    let item_ctor = if !expansions.item_exprs().is_empty() {
        quote! {
            #parent_enum_ident :: #variant_ident(#(#expansions.item_exprs()),*)
        }
    } else {
        quote! {
            #parent_enum_ident :: #variant_ident()
        }
    };

    // Justification constructor
    let just_ctor = if !expansions.just_vals().is_empty() {
        quote! {
            #justification_ident :: #renamed_just_var {
                #(#expansions.just_vals()),*
            }
        }
    } else {
        quote! {
            #justification_ident :: #renamed_just_var {}
        }
    };

    // Confidence constructor
    let conf_ctor = if !expansions.conf_vals().is_empty() {
        quote! {
            #confidence_ident :: #renamed_just_var {
                #(#expansions.conf_vals()),*
            }
        }
    } else {
        quote! {
            #confidence_ident :: #renamed_just_var {}
        }
    };

    // Now build the match arm pattern
    if !expansions.pattern_vars().is_empty() {
        quote! {
            #flat_parent_ident :: #variant_ident { #(#expansions.pattern_vars()),* } => {
                Self {
                    item: #item_ctor,
                    justification: #just_ctor,
                    confidence: #conf_ctor,
                }
            }
        }
    } else {
        // No fields
        quote! {
            #flat_parent_ident :: #variant_ident {} => {
                Self {
                    item: #parent_enum_ident :: #variant_ident(),
                    justification: #justification_ident :: #renamed_just_var {},
                    confidence: #confidence_ident :: #renamed_just_var {},
                }
            }
        }
    }
}
