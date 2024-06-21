crate::ix!();

#[derive(Clone,Debug,PartialEq)]
pub enum ErrorVariant {

    /// Just an identifier, e.g., `FormatError`
    Basic(Ident), 

    /// Wrapper around another type, e.g., `IOError(std::io::Error)`
    Wrapped(Ident, Box<Type>), 

    /// Struct variant, e.g., `DeviceNotAvailable { device_name: String }`
    Struct(Ident, Vec<ErrorField>), 
}

impl ErrorVariant {

    pub fn ident(&self) -> Ident {
        match self {
            ErrorVariant::Basic(ident)      => ident.clone(),
            ErrorVariant::Wrapped(ident, _) => ident.clone(),
            ErrorVariant::Struct(ident, _)  => ident.clone(),
        }
    }
}

impl Parse for ErrorVariant {

    fn parse(input: ParseStream) -> SynResult<Self> {

        let ident: Ident = input.parse()?;
        let lookahead = input.lookahead1();

        if lookahead.peek(syn::token::Brace) {
            let fields;
            braced!(fields in input);
            let punc: Punctuated<ErrorField, Token![,]> = fields.parse_terminated(ErrorField::parse, Token![,])?;
            Ok(ErrorVariant::Struct(ident, punc.into_iter().collect()))
        } else if lookahead.peek(syn::token::Paren) {
            let content;
            syn::parenthesized!(content in input);
            let ty: Type = content.parse()?;
            Ok(ErrorVariant::Wrapped(ident, Box::new(ty)))
        } else {
            Ok(ErrorVariant::Basic(ident))
        }
    }
}

#[cfg(test)]
mod test_error_variant {

    use super::*;
    use syn::{parse_str, Ident, Type};
    use proc_macro2::Span;

    #[test]
    fn test_error_variant_basic() {
        let input_str = "FormatError"; // Add the brace to indicate a basic variant

        match parse_str::<ErrorVariant>(input_str) {
            Ok(parsed_variant) => {
                assert_eq!(parsed_variant, ErrorVariant::Basic(Ident::new("FormatError", Span::call_site())));
            }
            Err(e) => panic!("Failed to parse: {:?}", e),
        }
    }

    #[test]
    fn test_error_variant_wrapped() {
        let input_str = "IOError(std::io::Error)";

        match parse_str::<ErrorVariant>(input_str) {
            Ok(parsed_variant) => {
                assert_eq!(parsed_variant, ErrorVariant::Wrapped(Ident::new("IOError", Span::call_site()), Box::new(syn::parse_quote!(std::io::Error))));
            }
            Err(e) => panic!("Failed to parse: {:?}", e),
        }
    }

    #[test]
    fn test_error_variant_struct() {
        let input_str = "DeviceNotAvailable { device_name: String }";

        match parse_str::<ErrorVariant>(input_str) {
            Ok(parsed_variant) => {
                assert_eq!(parsed_variant, ErrorVariant::Struct(
                        Ident::new("DeviceNotAvailable", Span::call_site()),
                        vec![ErrorField {
                            ident: Ident::new("device_name", Span::call_site()),
                            ty: syn::parse_quote!(String)
                        }]
                ));
            }
            Err(e) => panic!("Failed to parse: {:?}", e),
        }
    }
}
