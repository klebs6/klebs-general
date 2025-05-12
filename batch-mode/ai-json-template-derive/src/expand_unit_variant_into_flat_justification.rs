// ---------------- [ File: ai-json-template-derive/src/expand_unit_variant_into_flat_justification.rs ]
crate::ix!();

/// Refactored version of `expand_unit_variant_into_flat_justification`,
/// now split into smaller subroutines with robust tracing logs.
pub fn expand_unit_variant_into_flat_justification(
    parent_enum_ident: &syn::Ident,
    variant_ident:     &syn::Ident,
    justification_ident: &syn::Ident,
    confidence_ident:    &syn::Ident,
    skip_self_just:      bool
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream)
{
    trace!(
        "expand_unit_variant_into_flat_justification: parent_enum='{}', variant='{}', skip_self_just={}",
        parent_enum_ident, variant_ident, skip_self_just
    );

    // 1) Determine final names for the flat variant and the justification variant.
    let renamed_var_for_flat = rename_variant_ident_if_unit(variant_ident, "UnitVariant");
    let renamed_var_for_just = rename_variant_ident_if_unit(variant_ident, "UnitVariant");

    // 2) Construct the "FlatJustifiedFoo" ident for our match arms.
    let flat_parent_ident = build_flat_parent_ident(parent_enum_ident);

    // 3) Build the flat variant snippet (the piece inside `pub enum FlatJustifiedFoo { ... }`).
    let flat_variant_ts = build_flat_unit_variant_ts(skip_self_just, &renamed_var_for_flat);

    // 4) Build the corresponding `From<FlatJustifiedFoo> for JustifiedFoo` match arm snippet.
    let from_arm_ts = build_from_arm_for_unit_variant(
        skip_self_just,
        parent_enum_ident,
        variant_ident,
        justification_ident,
        confidence_ident,
        &flat_parent_ident,
        &renamed_var_for_just
    );

    (flat_variant_ts, from_arm_ts)
}

#[cfg(test)]
mod test_expand_unit_variant_into_flat_justification {
    use super::*;

    #[traced_test]
    fn test_skip_self_just_true() {
        let parent = Ident::new("MyEnum", proc_macro2::Span::call_site());
        let variant = Ident::new("Unit", proc_macro2::Span::call_site());
        let just = Ident::new("MyEnumJustification", proc_macro2::Span::call_site());
        let conf = Ident::new("MyEnumConfidence", proc_macro2::Span::call_site());
        let (flat_ts, from_ts) = expand_unit_variant_into_flat_justification(
            &parent, &variant, &just, &conf, /*skip_self_just=*/ true
        );

        // We check that there's no `enum_variant_justification` in flat_ts.
        let flat_str = flat_ts.to_string();
        assert!(flat_str.contains("Unit") && !flat_str.contains("justification"));
        // For the from_ts, we ensure it references the empty struct pattern:
        let from_str = from_ts.to_string();
        assert!(from_str.contains("MyEnum :: Unit"));
        assert!(from_str.contains("MyEnumJustification :: UnitVariant"));
        assert!(from_str.contains("MyEnumConfidence :: UnitVariant"));
    }

    #[traced_test]
    fn test_skip_self_just_false() {
        let parent = Ident::new("MyEnum", proc_macro2::Span::call_site());
        let variant = Ident::new("SpecialCase", proc_macro2::Span::call_site());
        let just = Ident::new("MyEnumJustification", proc_macro2::Span::call_site());
        let conf = Ident::new("MyEnumConfidence", proc_macro2::Span::call_site());
        let (flat_ts, from_ts) = expand_unit_variant_into_flat_justification(
            &parent, &variant, &just, &conf, /*skip_self_just=*/ false
        );

        let flat_str = flat_ts.to_string();
        assert!(flat_str.contains("enum_variant_justification"));
        assert!(flat_str.contains("enum_variant_confidence"));
        // from_ts check
        let from_str = from_ts.to_string();
        assert!(from_str.contains("enum_variant_justification"));
        assert!(from_str.contains("enum_variant_confidence"));
    }
}
