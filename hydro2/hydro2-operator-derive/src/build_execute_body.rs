// ---------------- [ File: hydro2-operator-derive/src/build_execute_body.rs ]
crate::ix!();

pub fn build_execute_body(
    spec:          &OperatorSpec,
    io_enum_ident: &Ident,

) -> (TokenStream, TokenStream, TokenStream) {

    let input_count        = spec.inputs().len();
    let output_count       = spec.outputs().len();
    let mut extract_inputs = Vec::new();
    let mut input_vars     = Vec::new();

    for i in 0..input_count {

        let var_name     = Ident::new(&format!("input{}_val", i), Span::call_site());
        let variant_name = Ident::new(&format!("Input{}", i), Span::call_site());

        input_vars.push(var_name.clone());

        extract_inputs.push(quote! {
            let #var_name = match execute_inputs[#i] {
                Some(#io_enum_ident::#variant_name(v)) => v,
                _ => return Err(NetworkError::InvalidPinAssignment),
            };
        });
    }

    let call_target = spec.execute_fn();

    let call_args = if input_vars.is_empty() {
        quote!()
    } else {
        quote! { #( #input_vars ),* }
    };

    // If we have N outputs, we either do: 
    // 0 => `let _ = self.execute_fn(args).await?;`
    // 1 => `let output_result0 = self.execute_fn(args).await?;`
    // 2+ => `(out0, out1, ...) = self.execute_fn(args).await?;`
    let call_expr = match output_count {
        0 => quote! {
            let _ = self.#call_target(#call_args).await?;
        },
        1 => quote! {
            let output_result0 = self.#call_target(#call_args).await?;
        },
        _ => {
            let out_idents: Vec<_> = (0..output_count)
                .map(|i| Ident::new(&format!("output_result{}", i), Span::call_site()))
                .collect();
            quote! {
                let (#(#out_idents),*) = self.#call_target(#call_args).await?;
            }
        }
    };

    let mut store_outputs = Vec::new();
    for i in 0..output_count {
        let variant_name = Ident::new(&format!("Output{}", i), Span::call_site());
        let out_name = Ident::new(&format!("output_result{}", i), Span::call_site());
        store_outputs.push(quote! {
            execute_outputs[#i] = Some(#io_enum_ident::#variant_name(#out_name));
        });
    }

    let extract_inputs_code = quote! { #( #extract_inputs )* };
    let store_outputs_code  = quote! { #( #store_outputs )* };

    (extract_inputs_code, call_expr, store_outputs_code)
}

#[cfg(test)]
mod test_build_execute_body {

    use super::*;

    //------------------------------------------------------------------
    // 3) Tests for build_execute_body (direct calls)
    //------------------------------------------------------------------
    #[test]
    fn test_build_execute_body_zero_in_out() {
        let spec = OperatorSpecBuilder::default()
            .execute_fn::<syn::Path>(parse_quote!(do_nothing))
            .opcode_expr::<syn::Path>(parse_quote!(OpCode::NoOp))
            .inputs(vec![])
            .outputs(vec![])
            .build()
            .unwrap();

        let io_enum_ident = Ident::new("ZeroIO", proc_macro2::Span::call_site());
        let (extract_ts, call_ts, store_ts) = build_execute_body(&spec, &io_enum_ident);

        let extract_str = normalize_whitespace(&extract_ts.to_string());
        let call_str    = normalize_whitespace(&call_ts.to_string());
        let store_str   = normalize_whitespace(&store_ts.to_string());

        //println!("extract_str={}", extract_str);
        //println!("call_str={}", call_str);
        //println!("store_str={}", store_str);
        // No input extraction
        assert!(extract_str.is_empty(), "Expected empty extraction, got: {extract_str}");
        // 0 outputs => "let _ = self.do_nothing(...).await?;"
        assert!(call_str.contains("let _ = self . do_nothing () . await ? ;"));
        // No store
        assert!(store_str.is_empty(), "Expected empty store, got: {store_str}");
    }

    #[test]
    fn test_build_execute_body_single_in_out() {
        let spec = OperatorSpecBuilder::default()
            .execute_fn::<syn::Path>(parse_quote!(process_one))
            .opcode_expr::<syn::Path>(parse_quote!(OpCode::UnaryOp))
            .inputs(vec![parse_quote! { i32 }])
            .outputs(vec![parse_quote! { String }])
            .build()
            .unwrap();

        let io_enum_ident = Ident::new("UnaryIO", proc_macro2::Span::call_site());
        let (extract_ts, call_ts, store_ts) = build_execute_body(&spec, &io_enum_ident);

        let extract_str = normalize_whitespace(&extract_ts.to_string());
        let call_str    = normalize_whitespace(&call_ts.to_string());
        let store_str   = normalize_whitespace(&store_ts.to_string());

        //println!("extract_str={}", extract_str);
        //println!("call_str={}", call_str);
        //println!("store_str={}", store_str);

        // We expect something like:
        // let input0_val = match execute_inputs[0] {
        //     Some(UnaryIO::Input0(v)) => v,
        //     _ => return Err(NetworkError::InvalidPinAssignment),
        // };
        assert!(extract_str.contains("let input0_val = match execute_inputs [0usize]"));
        assert!(extract_str.contains("Some (UnaryIO :: Input0 (v)) => v"));

        // Single output => "let output_result0 = self.process_one(input0_val).await?;"
        assert!(call_str.contains("let output_result0 = self . process_one (input0_val) . await ? ;"));

        // Single output => "execute_outputs[0] = Some(UnaryIO::Output0(output_result0));"
        assert!(store_str.contains("execute_outputs [0usize] = Some (UnaryIO :: Output0 (output_result0))"));
    }

    #[test]
    fn test_build_execute_body_multiple_in_out() {
        // 2 inputs, 3 outputs
        let spec = OperatorSpecBuilder::default()
            .execute_fn::<syn::Path>(parse_quote!(multi_op))
            .opcode_expr::<syn::Path>(parse_quote!(OpCode::MultiThing))
            .inputs(vec![parse_quote! { &str }, parse_quote! { bool }])
            .outputs(vec![parse_quote! { i32 }, parse_quote! { String }, parse_quote! { Vec<u8> }])
            .build()
            .unwrap();

        let io_enum_ident = Ident::new("MultiIO", proc_macro2::Span::call_site());
        let (extract_ts, call_ts, store_ts) = build_execute_body(&spec, &io_enum_ident);

        let extract_str = normalize_whitespace(&extract_ts.to_string());
        let call_str    = normalize_whitespace(&call_ts.to_string());
        let store_str   = normalize_whitespace(&store_ts.to_string());

        //println!("extract_str={}", extract_str);
        //println!("call_str={}", call_str);
        //println!("store_str={}", store_str);

        // Input lines:
        //   let input0_val = match ...
        //   let input1_val = match ...
        assert!(extract_str.contains("input0_val"));
        assert!(extract_str.contains("input1_val"));
        assert!(extract_str.contains("MultiIO :: Input0 (v)"));
        assert!(extract_str.contains("MultiIO :: Input1 (v)"));

        // 3 outputs => "let (output_result0, output_result1, output_result2) = self.multi_op(...).await?;"
        assert!(call_str.contains("let (output_result0 , output_result1 , output_result2) = self . multi_op (input0_val , input1_val) . await ? ;"));

        // Then storing them:
        //   execute_outputs[0] = Some(MultiIO::Output0(output_result0));
        //   ...
        //   execute_outputs[2] = Some(MultiIO::Output2(output_result2));
        assert!(store_str.contains("execute_outputs [0usize] = Some (MultiIO :: Output0 (output_result0))"));
        assert!(store_str.contains("execute_outputs [1usize] = Some (MultiIO :: Output1 (output_result1))"));
        assert!(store_str.contains("execute_outputs [2usize] = Some (MultiIO :: Output2 (output_result2))"));
    }
}
