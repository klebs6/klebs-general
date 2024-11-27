crate::ix!();

#[derive(Clone,Debug,PartialEq)]
pub enum ErrorVariant {

    /// Just an identifier, e.g., `FormatError`
    Basic {
        attrs:          Vec<Attribute>,
        ident:          Ident,
        cmp_neq:        bool,
        display_format: Option<String>,

    }, 

    /// Wrapper around another type, e.g., `IOError(std::io::Error)`
    Wrapped {
        attrs:          Vec<Attribute>,
        ident:          Ident, 
        ty:             Box<Type>,
        cmp_neq:        bool,
        display_format: Option<String>,
    },

    /// Struct variant, e.g., `DeviceNotAvailable { device_name: String }`
    Struct {
        attrs:          Vec<Attribute>,
        ident:          Ident, 
        fields:         Vec<ErrorField>,
        cmp_neq:        bool,
        display_format: Option<String>,

    }
}

impl ErrorVariant {

    pub fn display_format(&self) -> Option<&String> {
        match self {
            ErrorVariant::Basic   { display_format, .. } => display_format.as_ref(),
            ErrorVariant::Wrapped { display_format, .. } => display_format.as_ref(),
            ErrorVariant::Struct  { display_format, .. } => display_format.as_ref(),
        }
    }

    pub fn cmp_neq(&self) -> bool {
        match self {
            ErrorVariant::Basic   { cmp_neq, .. } => *cmp_neq,
            ErrorVariant::Wrapped { cmp_neq, .. } => *cmp_neq,
            ErrorVariant::Struct  { cmp_neq, .. } => *cmp_neq,
        }
    }

    pub fn attrs(&self) -> &Vec<Attribute> {
        match self {
            ErrorVariant::Basic   { attrs, .. } => attrs,
            ErrorVariant::Wrapped { attrs, .. } => attrs,
            ErrorVariant::Struct  { attrs, .. } => attrs,
        }
    }

    pub fn ident(&self) -> &Ident {
        match self {
            ErrorVariant::Basic   { ident, .. } => ident,
            ErrorVariant::Wrapped { ident, .. } => ident,
            ErrorVariant::Struct  { ident, .. } => ident,
        }
    }
}

impl Parse for ErrorVariant {
    fn parse(input: ParseStream) -> SynResult<Self> {
        // Parse attributes
        let mut attrs: Vec<Attribute> = input.call(Attribute::parse_outer)?;

        // Check for `#[cmp_neq]` attribute
        let mut cmp_neq = false;
        let mut display_format = None;

        attrs.retain(|attr| {
            if attr.path().is_ident("cmp_neq") {
                cmp_neq = true;
                false // Remove the `cmp_neq` attribute
            } else if attr.path().is_ident("display") {
                if let Ok(lit_str) = attr.parse_args::<syn::LitStr>() {
                    display_format = Some(lit_str.value());
                }
                false // Remove the `display` attribute
            } else {
                true // Keep other attributes
            }
        });

        // Parse the identifier (variant name)
        let ident: Ident = input.parse()?;
        let lookahead = input.lookahead1();

        if lookahead.peek(syn::token::Brace) {
            let fields;
            braced!(fields in input);
            let punc: Punctuated<ErrorField, Token![,]> =
                fields.parse_terminated(ErrorField::parse, Token![,])?;
            Ok(ErrorVariant::Struct {
                attrs,
                ident,
                fields: punc.into_iter().collect(),
                cmp_neq,
                display_format,
            })
        } else if lookahead.peek(syn::token::Paren) {
            let content;
            syn::parenthesized!(content in input);
            let ty: Type = content.parse()?;
            Ok(ErrorVariant::Wrapped {
                attrs,
                ident,
                ty: Box::new(ty),
                cmp_neq,
                display_format,
            })
        } else {
            Ok(ErrorVariant::Basic { attrs, ident, cmp_neq, display_format })
        }
    }
}

#[cfg(test)]
mod test_error_variant {

    use super::*;
    use syn::{parse_str, Ident};
    use proc_macro2::Span;

    #[test]
    fn test_error_variant_basic() {
        let input_str = "FormatError"; // Add the brace to indicate a basic variant

        match parse_str::<ErrorVariant>(input_str) {
            Ok(parsed_variant) => {
                let basic = ErrorVariant::Basic{
                    attrs:   vec![],
                    ident:   Ident::new("FormatError", Span::call_site()),
                    cmp_neq: false,
                    display_format: None,
                };
                assert_eq!(parsed_variant, basic);
            }
            Err(e) => panic!("Failed to parse: {:?}", e),
        }
    }

    #[test]
    fn test_error_variant_wrapped() {
        let input_str = "IOError(std::io::Error)";

        match parse_str::<ErrorVariant>(input_str) {
            Ok(parsed_variant) => {
                let wrapped =  ErrorVariant::Wrapped{
                    attrs: vec![],
                    ident: Ident::new("IOError", Span::call_site()), 
                    ty:    Box::new(syn::parse_quote!(std::io::Error)),
                    cmp_neq: false,
                    display_format: None,
                };
                assert_eq!(parsed_variant, wrapped);
            }
            Err(e) => panic!("Failed to parse: {:?}", e),
        }
    }

    #[test]
    fn test_error_variant_struct() {
        let input_str = "DeviceNotAvailable { device_name: String }";

        match parse_str::<ErrorVariant>(input_str) {
            Ok(parsed_variant) => {

                let s = ErrorVariant::Struct {
                    attrs:  vec![],
                    ident:  Ident::new("DeviceNotAvailable", Span::call_site()),
                    fields: vec![ErrorField {
                        ident: Ident::new("device_name", Span::call_site()),
                        ty: syn::parse_quote!(String)
                    }],
                    cmp_neq: false,
                    display_format: None,
                };

                assert_eq!(parsed_variant, s);
            }

            Err(e) => panic!("Failed to parse: {:?}", e),
        }
    }
}
