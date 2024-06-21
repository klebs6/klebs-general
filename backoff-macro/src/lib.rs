#![warn(dead_code)]
#![warn(unused_imports)]
extern crate proc_macro;

#[macro_use] mod imports; use imports::*;

xp!{extract_error_type}
xp!{extract_output_type}

// Example usage in your macro
#[proc_macro_attribute]
pub fn backoff(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_attrs = &input.attrs;
    let fn_vis = &input.vis;
    let fn_sig = &input.sig;
    let fn_block = &input.block;

    let output_type = match &input.sig.output {
        ReturnType::Type(_, ty) => &**ty,
        ReturnType::Default => {
            return Error::new_spanned(&input.sig.output, "expected a return type")
                .to_compile_error()
                .into();
        }
    };

    // Extract error type
    let error_type = match extract_error_type(output_type) {
        Ok(ty) => ty,
        Err(e) => return e.to_compile_error().into(),
    };

    let expanded = if let Type::Reference(_) = error_type {
        quote! {
            #(#fn_attrs)*
            #fn_vis #fn_sig {
                let result: #output_type = backoff::future::retry(backoff::ExponentialBackoff::default(), || async {
                    #fn_block
                }).await;

                result
            }
        }

    } else {

        quote! {
            #(#fn_attrs)*
            #fn_vis #fn_sig {
                let result: #output_type = backoff::future::retry(backoff::ExponentialBackoff::default(), || async {
                    let res: #output_type = (async { #fn_block }).await;
                    res.map_err(|e| #error_type::from(e))
                }).await;

                result
            }
        }
    };

    TokenStream::from(expanded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_extract_error_type() {
        let output_type: Type = parse_quote!(Result<(), DummyError>);
        let error_type = extract_error_type(&output_type).expect("Failed to extract error type");
        assert_eq!(quote!(#error_type).to_string(), quote!(DummyError).to_string());

        let output_type: Type = parse_quote!(Result<(), ComplexError>);
        let error_type = extract_error_type(&output_type).expect("Failed to extract error type");
        assert_eq!(quote!(#error_type).to_string(), quote!(ComplexError).to_string());

        let output_type: Type = parse_quote!(Result<(), &'static str>);
        let error_type = extract_error_type(&output_type).expect("Failed to extract error type");
        assert_eq!(quote!(#error_type).to_string(), quote!(&'static str).to_string());

        let output_type: Type = parse_quote!(Option<()>);
        match extract_error_type(&output_type) {
            Ok(_) => panic!("Expected error, but got Ok"),
            Err(e) => println!("Error: {:?}", e),
        }
    }
}
