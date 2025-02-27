// ---------------- [ File: hydro2-operator-derive/src/generate_operator_signature_tokens.rs ]
crate::ix!();

/// Generate `impl OperatorSignature` for your hidden type. 
pub fn generate_operator_signature_tokens(
    sig_ident:     &Ident,
    spec:          &OperatorSpec,
    impl_generics: &TokenStream,
    type_generics: &TokenStream,
    where_clause:  &TokenStream,
    full_generics: &syn::Generics,

) -> TokenStream {

    let phantom_field = phantom_data_for_generics(full_generics);

    // produce the signature struct definition:
    let sig_struct = quote! {
        #[doc(hidden)]
        #[derive(Default,Debug,Copy,Clone)]
        pub struct #sig_ident #impl_generics #where_clause {
            #phantom_field
        }
    };

    // For each input0..3 in `spec.inputs()`, if not present, use `()`.
    // For each output0..3 in `spec.outputs()`, if not present, use `()`.
    let input0_ty  = spec.inputs().get(0).cloned().unwrap_or_else(|| syn::parse_quote! { () });
    let input1_ty  = spec.inputs().get(1).cloned().unwrap_or_else(|| syn::parse_quote! { () });
    let input2_ty  = spec.inputs().get(2).cloned().unwrap_or_else(|| syn::parse_quote! { () });
    let input3_ty  = spec.inputs().get(3).cloned().unwrap_or_else(|| syn::parse_quote! { () });
    let output0_ty = spec.outputs().get(0).cloned().unwrap_or_else(|| syn::parse_quote! { () });
    let output1_ty = spec.outputs().get(1).cloned().unwrap_or_else(|| syn::parse_quote! { () });
    let output2_ty = spec.outputs().get(2).cloned().unwrap_or_else(|| syn::parse_quote! { () });
    let output3_ty = spec.outputs().get(3).cloned().unwrap_or_else(|| syn::parse_quote! { () });

    let sig_impl = quote! {
        impl #impl_generics OperatorSignature for #sig_ident #type_generics #where_clause {
            type Input0  = #input0_ty;
            type Input1  = #input1_ty;
            type Input2  = #input2_ty;
            type Input3  = #input3_ty;
            type Output0 = #output0_ty;
            type Output1 = #output1_ty;
            type Output2 = #output2_ty;
            type Output3 = #output3_ty;
        }
    };

    quote!{
        #sig_struct
        #sig_impl
    }
}
