crate::ix!();

pub fn generate_flat_variant_for_variant(
    enum_ident:                 &Ident,
    variant:                    &syn::Variant,
    justification_ident:        &Ident,
    confidence_ident:           &Ident,
    skip_variant_self_just_fn:  &impl Fn(&syn::Variant) -> bool,
    skip_variant_child_just_fn: &impl Fn(&syn::Variant) -> bool,
    skip_field_self_just_fn:    &impl Fn(&syn::Field) -> bool,
    is_leaf_type_fn:            &impl Fn(&syn::Type) -> bool,
    flatten_named_field_fn:     &impl Fn(&Ident, &syn::Type, bool, bool)
        -> (Vec<TokenStream2>, TokenStream2, TokenStream2, TokenStream2),
    flatten_unnamed_field_fn:   &impl Fn(&Ident, &syn::Type, bool, bool)
        -> (Vec<TokenStream2>, TokenStream2, TokenStream2, TokenStream2),
) -> (TokenStream2, TokenStream2) {
    trace!(
        "generate_flat_variant_for_variant called for '{}::{}'",
        enum_ident,
        variant.ident
    );

    let skip_self_just  = skip_variant_self_just_fn(variant);
    let skip_child_just = skip_self_just || skip_variant_child_just_fn(variant);

    match &variant.fields {
        Fields::Unit => {
            expand_unit_variant_into_flat_justification(
                enum_ident,
                &variant.ident,
                justification_ident,
                confidence_ident,
                skip_self_just
            )
        }
        Fields::Named(named_fields) => {
            expand_named_variant_into_flat_justification(
                enum_ident,
                &variant.ident,
                named_fields,
                justification_ident,
                confidence_ident,
                skip_self_just,
                skip_child_just,
                flatten_named_field_fn,
                skip_field_self_just_fn,
                is_leaf_type_fn
            )
        }
        Fields::Unnamed(unnamed_fields) => {
            expand_unnamed_variant_into_flat_justification(
                enum_ident,
                &variant.ident,
                unnamed_fields,
                justification_ident,
                confidence_ident,
                skip_self_just,
                skip_child_just,
                flatten_unnamed_field_fn,
                skip_field_self_just_fn,
                is_leaf_type_fn
            )
        }
    }
}
