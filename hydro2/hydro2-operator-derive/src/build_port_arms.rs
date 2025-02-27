// ---------------- [ File: hydro2-operator-derive/src/build_port_arms.rs ]
crate::ix!();

/// Return a Vec of match arms for `fn input_port_type_str(&self, port: usize) -> Option<&'static str>`.
/// Each arm is like:
///    0 => Some(std::any::type_name::<i32>()),
///    1 => Some(std::any::type_name::<Option<T>>()),
///    ...
pub fn build_input_port_type_arms(input_types: &[Type]) -> Vec<TokenStream> {
    input_types
        .iter()
        .enumerate()
        .map(|(i, ty)| {
            let i_lit = LitInt::new(&format!("{}", i), ty.span());
            quote! {
                #i_lit => Some(::std::any::type_name::<#ty>())
            }
        })
        .collect()
}

/// Return a Vec of match arms for `fn output_port_type_str(&self, port: usize) -> Option<&'static str>`.
/// Each arm is like:
///    0 => Some(std::any::type_name::<i32>()),
///    1 => Some(std::any::type_name::<Option<T>>()),
///    ...
pub fn build_output_port_type_arms(output_types: &[Type]) -> Vec<TokenStream> {
    output_types
        .iter()
        .enumerate()
        .map(|(i, ty)| {
            let i_lit = LitInt::new(&format!("{}", i), ty.span());
            quote! {
                #i_lit => Some(::std::any::type_name::<#ty>())
            }
        })
        .collect()
}

/// Helper to check if a type is `Option<T>` by textual pattern:
fn is_type_option(ty: &Type) -> bool {
    let s = ty.to_token_stream().to_string();
    // e.g. "Option < Foo >" or "Option<Foo>"
    s.starts_with("Option <") || s.starts_with("Option<")
}

/// Return arms for `fn input_port_connection_required(&self, port: usize) -> bool`.
/// We return `true` if not Option<T>.
pub fn build_input_port_required_arms(input_types: &[Type]) -> Vec<TokenStream> {
    input_types
        .iter()
        .enumerate()
        .map(|(i, ty)| {
            let i_lit = LitInt::new(&format!("{}", i), ty.span());
            // required => NOT Option
            let is_required = !is_type_option(ty);
            quote! {
                #i_lit => #is_required
            }
        })
        .collect()
}

/// Return arms for `fn output_port_connection_required(&self, port: usize) -> bool`.
/// We return `true` if not Option<T>.
pub fn build_output_port_required_arms(output_types: &[Type]) -> Vec<TokenStream> {
    output_types
        .iter()
        .enumerate()
        .map(|(i, ty)| {
            let i_lit = LitInt::new(&format!("{}", i), ty.span());
            let is_required = !is_type_option(ty);
            quote! {
                #i_lit => #is_required
            }
        })
        .collect()
}

#[cfg(test)]
mod test_port_functions {
    use super::*;
    use syn::parse_quote;
    use syn::Type;
    use proc_macro2::TokenStream;

    fn strip_spaces(ts: &TokenStream) -> String {
        ts.to_string().split_whitespace().collect()
    }

    #[test]
    fn test_is_type_option() {
        let types: Vec<Type> = vec![
            parse_quote!(i32),
            parse_quote!(Option<String>),
            parse_quote!(Vec<u8>),
            parse_quote!(Option<CustomType>),
        ];
        let arms = build_input_port_required_arms(&types);
        let arms_str: Vec<String> = arms.iter().map(|ts| strip_spaces(ts)).collect();
        // e.g. "0=>true", "1=>false", "2=>true", "3=>false"
        assert_eq!(
            arms_str,
            vec!["0=>true", "1=>false", "2=>true", "3=>false"],
            "Expected i32 => true, Option => false, etc."
        );
    }

    #[test]
    fn test_build_input_port_type_arms() {
        let types: Vec<Type> = vec![
            parse_quote!(i32),
            parse_quote!(Option<Foo>),
        ];
        let arms = build_input_port_type_arms(&types);
        let arms_str: Vec<String> = arms.iter().map(|ts| strip_spaces(ts)).collect();

        // For example, the code might generate:
        //   0=>Some(::std::any::type_name::<i32>())
        //   1=>Some(::std::any::type_name::<Option<Foo>>())
        //
        // After stripping spaces, compare with the expected strings:
        assert_eq!(arms_str.len(), 2);
        assert_eq!(
            arms_str[0],
            "0=>Some(::std::any::type_name::<i32>())"
        );
        assert_eq!(
            arms_str[1],
            "1=>Some(::std::any::type_name::<Option<Foo>>())"
        );
    }

    #[test]
    fn test_build_output_port_type_arms() {
        let types: Vec<Type> = vec![
            parse_quote!(Option<Bar>),
            parse_quote!(Vec<i32>),
        ];
        let arms = build_output_port_type_arms(&types);
        let arms_str: Vec<String> = arms.iter().map(|ts| strip_spaces(ts)).collect();

        // Expect:
        // 0=>Some(::std::any::type_name::<Option<Bar>>())
        // 1=>Some(::std::any::type_name::<Vec<i32>>())
        assert_eq!(arms_str.len(), 2);
        assert_eq!(
            arms_str[0],
            "0=>Some(::std::any::type_name::<Option<Bar>>())"
        );
        assert_eq!(
            arms_str[1],
            "1=>Some(::std::any::type_name::<Vec<i32>>())"
        );
    }
}
