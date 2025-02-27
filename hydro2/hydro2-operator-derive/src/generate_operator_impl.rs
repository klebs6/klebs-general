// ---------------- [ File: hydro2-operator-derive/src/generate_operator_impl.rs ]
crate::ix!();

pub fn generate_operator_impl(
    struct_ident:  &Ident,
    operator_spec: &OperatorSpec,
    io_enum_ident: &Ident,
    impl_generics: &TokenStream,
    type_generics: &TokenStream,
    where_clause:  &TokenStream,
) -> TokenStream {

    let input_count  = operator_spec.inputs().len();
    let output_count = operator_spec.outputs().len();

    let opcode_expr  = operator_spec.opcode_expr();
    let execute_fn   = operator_spec.execute_fn();

    let (extract_inputs_code, call_expr, store_outputs_code) =
        build_execute_body(operator_spec, io_enum_ident);

    // Build the arms for each function
    let input_port_arms   = build_input_port_type_arms(operator_spec.inputs());
    let output_port_arms  = build_output_port_type_arms(operator_spec.outputs());
    let input_req_arms    = build_input_port_required_arms(operator_spec.inputs());
    let output_req_arms   = build_output_port_required_arms(operator_spec.outputs());

    quote! {
        #[async_trait::async_trait]
        impl #impl_generics Operator<#io_enum_ident #type_generics> for #struct_ident #type_generics
        #where_clause
        {
            fn opcode(&self) -> std::sync::Arc<dyn OpCode> {
                Arc::new(#opcode_expr)
            }

            fn input_count(&self) -> usize {
                #input_count
            }

            fn output_count(&self) -> usize {
                #output_count
            }

            fn input_port_type_str(&self, port: usize) -> Option<&'static str> {
                match port {
                    #( #input_port_arms, )*
                    _ => None,
                }
            }

            fn output_port_type_str(&self, port: usize) -> Option<&'static str> {
                match port {
                    #( #output_port_arms, )*
                    _ => None,
                }
            }

            fn input_port_connection_required(&self, port: usize) -> bool {
                match port {
                    #( #input_req_arms, )*
                    _ => false,
                }
            }

            /// NOTE: typically, we do not require that outputs from any given operator be connected.
            /// perhaps this design choice may be changed in the future
            fn output_port_connection_required(&self, port: usize) -> bool {
                false
                /*
                match port {
                    #( #output_req_arms, )*
                    _ => false,
                }
                */
            }

            async fn execute(
                &self,
                execute_inputs: [Option<&#io_enum_ident #type_generics>; 4],
                execute_outputs: &mut [Option<#io_enum_ident #type_generics>; 4]
            ) -> NetResult<()> {
                #extract_inputs_code
                #call_expr
                #store_outputs_code
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod test_generate_operator_impl {
    use super::*;

    //------------------------------------------------------------------
    // 4) Tests for generate_operator_impl
    //------------------------------------------------------------------
    #[traced_test]
    fn test_generate_operator_impl_no_in_out() {
        // 0 inputs, 0 outputs
        let spec = OperatorSpecBuilder::default()
            .execute_fn::<syn::Path>(parse_quote!(noop))
            .opcode_expr::<syn::Path>(parse_quote!(BasicOpCode::Nothing))
            .inputs(vec![])
            .outputs(vec![])
            .build()
            .unwrap();

        let impl_ts = generate_operator_impl(
            &Ident::new("NoIOOperator", proc_macro2::Span::call_site()),
            &spec,
            &Ident::new("NoIOOperatorIO", proc_macro2::Span::call_site()),
            &quote! { <T> },
            &quote! { <T> },
            &quote! {},
        );

        let impl_str = normalize_whitespace(&impl_ts.to_string());

        debug!("impl_str: {:#?}", impl_str);

        // We expect an impl block containing:
        //   impl <T> Operator<NoIOOperatorIO<T>> for NoIOOperator<T> { ... }
        assert!(impl_str.contains("impl < T > Operator < NoIOOperatorIO < T > > for NoIOOperator < T >"));
        // input_count => 0
        assert!(impl_str.contains("fn input_count (& self) -> usize { 0usize }"));
        // output_count => 0
        assert!(impl_str.contains("fn output_count (& self) -> usize { 0usize }"));
        // opcode => OpCode::Nothing
        assert!(impl_str.contains("fn opcode (& self) -> std :: sync :: Arc < dyn OpCode > { Arc :: new (BasicOpCode :: Nothing) }"));

        // in the `execute` body, we should see no input extraction or output storage,
        // only the call to `self.noop(...).await`.
        assert!(impl_str.contains("let _ = self . noop () . await ? ;"));
    }

    #[traced_test]
    fn test_generate_operator_impl_multiple_in_out() {
        // 2 inputs, 2 outputs
        let spec = OperatorSpecBuilder::default()
            .execute_fn::<syn::Path>(parse_quote!(some_execute))
            .opcode_expr::<syn::Path>(parse_quote!(BasicOpCode::Custom))
            .inputs(vec![parse_quote! { A }, parse_quote! { B }])
            .outputs(vec![parse_quote! { X }, parse_quote! { Y }])
            .build()
            .unwrap();

        let impl_ts = generate_operator_impl(
            &Ident::new("MultiIOOp", proc_macro2::Span::call_site()),
            &spec,
            &Ident::new("MultiIOOpIO", proc_macro2::Span::call_site()),
            &quote! { <'a, T> },
            &quote! { <'a, T> },
            &quote! {},
        );

        let impl_str = normalize_whitespace(&impl_ts.to_string());

        debug!("impl_str: {:#?}", impl_str);

        // We expect:
        //   fn opcode(...) -> dyn OpCode { BasicOpCode::Custom }
        assert!(impl_str.contains("-> std :: sync :: Arc < dyn OpCode > { Arc :: new (BasicOpCode :: Custom) }"));
        //   fn input_count(...) -> usize { 2 }
        assert!(impl_str.contains("-> usize { 2usize }"));
        //   fn output_count(...) -> usize { 2 }
        assert!(impl_str.contains("-> usize { 2usize }"));

        // We also see partial lines from build_execute_body:
        //   let input0_val = match ...
        //   let input1_val = match ...
        //   let (output_result0, output_result1) = self.some_execute(input0_val, input1_val).await?;
        assert!(impl_str.contains("input0_val"));
        assert!(impl_str.contains("input1_val"));
        assert!(impl_str.contains("(output_result0 , output_result1) = self . some_execute (input0_val , input1_val) . await ? ;"));
    }
}
