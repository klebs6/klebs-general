crate::ix!();

pub(crate) fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(segment) = path.segments.last() {
            return segment.ident == "Option";
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, Type};

    #[test]
    fn test_is_option_type_with_option() {
        let ty: Type = parse_quote!(Option<String>);
        assert_eq!(is_option_type(&ty), true);
    }

    #[test]
    fn test_is_option_type_with_non_option() {
        let ty: Type = parse_quote!(String);
        assert_eq!(is_option_type(&ty), false);
    }

    #[test]
    fn test_is_option_type_with_nested_option() {
        let ty: Type = parse_quote!(Option<Option<String>>);
        assert_eq!(is_option_type(&ty), true);
    }

    #[test]
    fn test_is_option_type_with_result_containing_option() {
        let ty: Type = parse_quote!(Result<Option<String>, Error>);
        assert_eq!(is_option_type(&ty), false);
    }

    #[test]
    fn test_is_option_type_with_reference_option() {
        let ty: Type = parse_quote!(&Option<String>);
        assert_eq!(is_option_type(&ty), false);
    }

    #[test]
    fn test_is_option_type_with_std_option() {
        let ty: Type = parse_quote!(std::option::Option<String>);
        assert_eq!(is_option_type(&ty), true);
    }
}

