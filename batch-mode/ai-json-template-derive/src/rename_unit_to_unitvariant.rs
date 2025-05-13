// ---------------- [ File: ai-json-template-derive/src/rename_unit_to_unitvariant.rs ]
crate::ix!();

// ---------------------------------------------------------------------------
//  Subroutine B: Possibly rename "Unit" => "UnitVariant"
// ---------------------------------------------------------------------------
pub fn rename_unit_to_unitvariant(variant_ident: &syn::Ident) -> syn::Ident {
    let real_name = variant_ident.to_string();
    if real_name == "Unit" {
        let renamed = syn::Ident::new("UnitVariant", variant_ident.span());
        trace!(
            "Renaming variant '{}' => '{}'",
            real_name,
            renamed.to_string()
        );
        renamed
    } else {
        variant_ident.clone()
    }
}

#[cfg(test)]
mod rename_unit_to_unitvariant_exhaustive_tests {
    use super::*;

    #[traced_test]
    fn it_renames_exact_unit_to_unitvariant() {
        trace!("Preparing test for 'Unit' => should become 'UnitVariant'.");
        let input = Ident::new("Unit", proc_macro2::Span::call_site());
        debug!("Calling rename_unit_to_unitvariant with ident = '{}'", input);
        let result = rename_unit_to_unitvariant(&input);
        info!("Got result ident = '{}'", result);
        assert_eq!(result.to_string(), "UnitVariant", "Expected rename for 'Unit'.");
    }

    #[traced_test]
    fn it_leaves_other_idents_unchanged() {
        trace!("Preparing test for ident that is not 'Unit' => should remain unchanged.");
        let inputs = vec![
            Ident::new("UNIT", proc_macro2::Span::call_site()),
            Ident::new("unit", proc_macro2::Span::call_site()),
            Ident::new("SomeVariant", proc_macro2::Span::call_site()),
            Ident::new("Unitty", proc_macro2::Span::call_site()),
        ];

        for ident in inputs {
            debug!("Testing ident = '{}'", ident);
            let result = rename_unit_to_unitvariant(&ident);
            info!("Got result ident = '{}'", result);
            assert_eq!(result, ident, "Expected no rename when input != 'Unit'.");
        }
    }

    #[traced_test]
    fn it_handles_empty_like_ident() {
        trace!("Preparing test for unusual ident like an empty string. Though not valid in normal code, let's see behavior.");
        // Syn `Ident` cannot truly be empty, but we can test an unusual short name
        let input = Ident::new("_", proc_macro2::Span::call_site());
        debug!("Calling rename_unit_to_unitvariant with ident = '{}'", input);
        let result = rename_unit_to_unitvariant(&input);
        info!("Got result ident = '{}'", result);
        assert_eq!(result, input, "Expected no rename for '_'.");
    }

    #[traced_test]
    fn it_rejects_misleading_partial_match() {
        trace!("Testing ident that partially matches 'Unit' but is not exactly 'Unit' => e.g., 'UnitX'");
        let input = Ident::new("UnitX", proc_macro2::Span::call_site());
        debug!("Calling rename_unit_to_unitvariant with ident = '{}'", input);
        let result = rename_unit_to_unitvariant(&input);
        info!("Got result ident = '{}'", result);
        assert_eq!(result, input, "Expected no rename for partial match of 'Unit'.");
    }
}
