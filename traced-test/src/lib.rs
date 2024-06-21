extern crate proc_macro;

#[macro_use] mod imports; use imports::*;

xp!{traced_test}

#[proc_macro_attribute]
pub fn traced_test(_attr: TokenStream, item: TokenStream) -> TokenStream {

    // Parse the input tokens into a syntax tree
    let mut function = parse_macro_input!(item as ItemFn);

    ensure_no_test_attribute(&function).unwrap();

    let name = function.sig.ident.to_string();

    let attrs = all_attributes_except_test(&function);

    // Determine if the return type is Result<T, E>
    let function_returns_result = is_return_type_result(&function);

    let new_block = generate_traced_block(function_returns_result, &function.block, &name);

    function.block = Box::new(parse_or_compile_error(new_block));

    // Generate the modified function
    let output_fn = quote! {
        #(#attrs)*
        #[test]
        #function
    };

    output_fn.into()
}

