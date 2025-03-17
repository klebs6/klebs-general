// ---------------- [ File: src/combine_impls_into_final_macro.rs ]
crate::ix!();

/// Combine the various generated token streams into a final unified output.
///
/// We simply concatenate the items in the provided vector.
pub fn combine_impls_into_final_macro(chunks: Vec<TokenStream2>) -> TokenStream2 {
    trace!("combine_impls_into_final_macro: start.");
    let merged = quote! {
        #[allow(non_snake_case)]
        #( #chunks )*
    };
    trace!("combine_impls_into_final_macro: done.");
    merged
}

#[cfg(test)]
mod test_combine_impls_into_final_macro {
    use super::*;

    #[traced_test]
    fn merges_multiple_code_chunks() {
        info!("Starting merges_multiple_code_chunks test.");

        let chunk1: TokenStream2 = quote! { struct A; };
        let chunk2: TokenStream2 = quote! { struct B; };
        let chunk3: TokenStream2 = quote! { fn c() {} };

        let combined = combine_impls_into_final_macro(vec![chunk1, chunk2, chunk3]);
        let code = combined.to_string();
        info!("Merged code: {}", code);

        assert!(code.contains("struct A ; struct B ; fn c"), "Should contain all pieces in sequence.");
    }
}
