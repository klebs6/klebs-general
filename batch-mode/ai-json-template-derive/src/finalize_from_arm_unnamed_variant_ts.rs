// ---------------- [ File: ai-json-template-derive/src/finalize_from_arm_unnamed_variant_ts.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
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

    // Possibly rename "Unit" => "UnitVariant"
    let raw_vname = variant_ident.to_string();
    let renamed_just_var = if raw_vname == "Unit" {
        syn::Ident::new("UnitVariant", variant_ident.span())
    } else {
        variant_ident.clone()
    };

    let item_exprs   = expansions.item_exprs();
    let just_vals    = expansions.just_vals();
    let conf_vals    = expansions.conf_vals();
    let pattern_vars = expansions.pattern_vars();

    // Construct the final item expression
    let item_ctor = if !item_exprs.is_empty() {
        quote! {
            #parent_enum_ident :: #variant_ident(#(#item_exprs),*)
        }
    } else {
        quote! {
            #parent_enum_ident :: #variant_ident()
        }
    };

    // Justification constructor
    let just_ctor = if !just_vals.is_empty() {
        quote! {
            #justification_ident :: #renamed_just_var {
                #(#just_vals),*
            }
        }
    } else {
        quote! {
            #justification_ident :: #renamed_just_var {}
        }
    };

    // Confidence constructor
    let conf_ctor = if !conf_vals.is_empty() {
        quote! {
            #confidence_ident :: #renamed_just_var {
                #(#conf_vals),*
            }
        }
    } else {
        quote! {
            #confidence_ident :: #renamed_just_var {}
        }
    };

    // Build the match arm pattern
    if !pattern_vars.is_empty() {
        quote! {
            #flat_parent_ident :: #variant_ident { #(#pattern_vars),* } => {
                Self {
                    item: #item_ctor,
                    justification: #just_ctor,
                    confidence: #conf_ctor,
                }
            }
        }
    } else {
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
