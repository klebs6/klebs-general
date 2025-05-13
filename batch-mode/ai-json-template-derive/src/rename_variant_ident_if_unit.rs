// ---------------- [ File: ai-json-template-derive/src/rename_variant_ident_if_unit.rs ]
crate::ix!();

/// If the `variant_ident`'s string is "Unit", returns a new Ident with the `replacement_name`.
/// Otherwise, returns `variant_ident` unchanged.
pub fn rename_variant_ident_if_unit(
    variant_ident:     &syn::Ident,
    replacement_name:  &str
) -> syn::Ident {
    let original_name = variant_ident.to_string();
    if original_name == "Unit" {
        trace!(
            "rename_variant_ident_if_unit: found 'Unit', using replacement='{}'",
            replacement_name
        );
        syn::Ident::new(replacement_name, variant_ident.span())
    } else {
        trace!(
            "rename_variant_ident_if_unit: variant '{}' not 'Unit', leaving as-is",
            variant_ident
        );
        variant_ident.clone()
    }
}

#[cfg(test)]
mod test_rename_variant_ident_if_unit {
    use super::*;

    #[traced_test]
    fn test_renames_exact_unit() {
        trace!("Starting test_renames_exact_unit");
        let original = Ident::new("Unit", proc_macro2::Span::call_site());
        let replacement = "MyReplacement";
        let renamed = rename_variant_ident_if_unit(&original, replacement);
        debug!("Renamed Ident: '{}'", renamed);
        assert_eq!(renamed.to_string(), replacement, "Should rename 'Unit' to the replacement");
    }

    #[traced_test]
    fn test_does_not_rename_different_case() {
        trace!("Starting test_does_not_rename_different_case");
        let original = Ident::new("UNIT", proc_macro2::Span::call_site());
        let replacement = "FooBar";
        let renamed = rename_variant_ident_if_unit(&original, replacement);
        debug!("Renamed Ident: '{}'", renamed);
        assert_eq!(renamed.to_string(), "UNIT", "Should leave non-matching case ident as-is");
    }

    #[traced_test]
    fn test_does_not_rename_arbitrary_name() {
        trace!("Starting test_does_not_rename_arbitrary_name");
        let original = Ident::new("SomeVariant", proc_macro2::Span::call_site());
        let replacement = "AnyReplacement";
        let renamed = rename_variant_ident_if_unit(&original, replacement);
        debug!("Renamed Ident: '{}'", renamed);
        assert_eq!(renamed.to_string(), "SomeVariant", "Should not rename arbitrary variant");
    }

    #[traced_test]
    fn test_does_not_rename_similar_but_not_exact() {
        trace!("Starting test_does_not_rename_similar_but_not_exact");
        let original = Ident::new("UnitTest", proc_macro2::Span::call_site());
        let replacement = "SomethingElse";
        let renamed = rename_variant_ident_if_unit(&original, replacement);
        info!("Renamed Ident: '{}'", renamed);
        assert_eq!(renamed.to_string(), "UnitTest", "Should not rename 'UnitTest' or other partial matches");
    }
}
