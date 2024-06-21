crate::ix!();

#[derive(Clone,Debug,PartialEq)]
pub struct ErrorField {
    pub ident: Ident,
    pub ty:    Type,
}

impl Parse for ErrorField {

    fn parse(input: ParseStream) -> SynResult<Self> {

        let ident: Ident = input.parse()?;

        input.parse::<Token![:]>()?;

        let ty: Type = input.parse()?;

        Ok(ErrorField { ident, ty })
    }
}

#[cfg(test)]
mod test_error_field {

    use super::*;
    use syn::{parse_str, Ident, Type};
    use proc_macro2::Span;

    #[test]
    fn test_error_field_parse() {
        let input_str = "device_name: String";

        match parse_str::<ErrorField>(input_str) {
            Ok(parsed_field) => {
                assert_eq!(parsed_field.ident, Ident::new("device_name", Span::call_site()));
                
                // This part is a bit tricky because syn::Type doesn't implement PartialEq
                // One way to test it is to convert both to a string and compare those
                let expected_type: Type = parse_str("String").expect("Failed to parse type");
                assert_eq!(format!("{:?}", parsed_field.ty), format!("{:?}", expected_type));
            }
            Err(e) => panic!("Failed to parse: {:?}", e),
        }
    }
}
