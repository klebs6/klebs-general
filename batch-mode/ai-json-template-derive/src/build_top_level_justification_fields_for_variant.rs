// ---------------- [ File: ai-json-template-derive/src/build_top_level_justification_fields_for_variant.rs ]
crate::ix!();

// ---------------------------------------------------------------------------
//  Subroutine C: Build top-level "enum_variant_just/conf" fields, pattern vars, inits
// ---------------------------------------------------------------------------
pub fn build_top_level_just_fields_for_variant(
    variant_ident: &syn::Ident,
    skip_self_just: bool,
) -> TopLevelJustResult
{
    trace!(
        "build_top_level_just_fields_for_variant: variant='{}', skip_self_just={}",
        variant_ident,
        skip_self_just
    );

    if skip_self_just {
        // no top-level justification/conf
        return TopLevelJustResultBuilder::default()
            .field_decls_top(vec![])
            .pattern_vars_top(vec![])
            .just_inits_top(vec![])
            .conf_inits_top(vec![])
            .build()
            .unwrap();
    }

    debug!(
        "Inserting top-level enum_variant_justification/enum_variant_confidence for variant '{}'",
        variant_ident
    );

    TopLevelJustResultBuilder::default()
        .field_decls_top(vec![
            quote::quote! {
                #[serde(default)]
                enum_variant_justification:String
            },
            quote::quote! {
                #[serde(default)]
                enum_variant_confidence:f32
            }
        ])
        .pattern_vars_top(vec![
            quote::quote! { enum_variant_justification },
            quote::quote! { enum_variant_confidence },
        ])
        .just_inits_top(vec![
            quote::quote! { variant_justification: enum_variant_justification },
        ])
        .conf_inits_top(vec![
            quote::quote! { variant_confidence: enum_variant_confidence },
        ])
        .build()
        .unwrap()
}
