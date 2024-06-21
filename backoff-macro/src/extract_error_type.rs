crate::ix!();

fn extract_type_path(ty: &Type) -> Result<&TypePath, Error> {
    match ty {
        Type::Path(type_path) => Ok(type_path),
        Type::Reference(type_reference) => match &*type_reference.elem {
            Type::Path(type_path) => Ok(type_path),
            _ => Err(Error::new_spanned(&type_reference.elem, "expected a TypePath within the reference")),
        },
        _ => Err(Error::new_spanned(ty, "expected a TypePath")),
    }
}


fn extract_last_segment(type_path: &TypePath) -> Result<&PathSegment, Error> {
    type_path.path.segments.last().ok_or_else(|| {
        Error::new_spanned(type_path, "expected at least one segment in the path")
    })
}

fn extract_angle_bracketed_args(segment: &PathSegment) -> Result<&AngleBracketedGenericArguments, Error> {
    match &segment.arguments {
        PathArguments::AngleBracketed(arguments) => Ok(arguments),
        _ => Err(Error::new_spanned(&segment.arguments, "expected angle bracketed arguments")),
    }
}

fn extract_last_generic_argument(arguments: &AngleBracketedGenericArguments) -> Result<&GenericArgument, Error> {
    arguments.args.last().ok_or_else(|| {
        Error::new_spanned(arguments, "expected at least one generic argument")
    })
}

fn extract_error_type_from_generic_argument(arg: &GenericArgument) -> Result<&Type, Error> {
    match arg {
        GenericArgument::Type(ty) => Ok(ty),
        _ => Err(Error::new_spanned(arg, "expected a Type in the generic argument")),
    }
}

pub fn extract_error_type(output_type: &Type) -> Result<&Type, Error> {

    let type_path = extract_type_path(output_type)?;
    let segment   = extract_last_segment(type_path)?;

    // Ensure the outer type is `Result`
    if segment.ident != "Result" {
        return Err(Error::new_spanned(segment, "expected a Result type"));
    }

    let arguments = extract_angle_bracketed_args(segment)?;
    if arguments.args.len() != 2 {
        return Err(Error::new_spanned(arguments, "expected Result type to have two generic arguments"));
    }

    let generic_argument = extract_last_generic_argument(arguments)?;
    extract_error_type_from_generic_argument(generic_argument)
}

#[cfg(test)]
mod type_extractor_tests {
    use super::*;
    use syn::{parse_quote, TypePath, PathSegment, AngleBracketedGenericArguments, GenericArgument, Type};
    use quote::ToTokens;

    fn normalize_quote<T: ToTokens>(tokens: &T) -> String {
        tokens.to_token_stream().to_string().replace(" ", "").replace("::", "::")
    }

    #[test]
    fn test_extract_type_path() {
        let type_path: Type = parse_quote!(Result<(), DummyError>);
        assert!(extract_type_path(&type_path).is_ok());

        let type_path: Type = parse_quote!(Option<()>);
        assert!(extract_type_path(&type_path).is_ok());

        let type_path: Type = parse_quote!(&'static str);
        assert!(extract_type_path(&type_path).is_ok());

        let type_path: Type = parse_quote!(i32);
        assert!(extract_type_path(&type_path).is_ok());

        let type_path: Type = parse_quote!(Result<&'static str, i32>);
        assert!(extract_type_path(&type_path).is_ok());

        let type_path: Type = parse_quote!(Result<usize, Box<dyn std::error::Error>>);
        assert!(extract_type_path(&type_path).is_ok());
    }

    #[test]
    fn test_extract_last_segment() {
        let type_path: TypePath = parse_quote!(Result<(), DummyError>);
        let segment = extract_last_segment(&type_path).expect("Failed to extract last segment");
        assert_eq!(segment.ident.to_string(), "Result");

        let type_path: TypePath = parse_quote!(Option<()>);
        let segment = extract_last_segment(&type_path).expect("Failed to extract last segment");
        assert_eq!(segment.ident.to_string(), "Option");

        let type_path: TypePath = parse_quote!(Result<&'static str, i32>);
        let segment = extract_last_segment(&type_path).expect("Failed to extract last segment");
        assert_eq!(segment.ident.to_string(), "Result");

        let type_path: TypePath = parse_quote!(Result<usize, Box<dyn std::error::Error>>);
        let segment = extract_last_segment(&type_path).expect("Failed to extract last segment");
        assert_eq!(segment.ident.to_string(), "Result");
    }

    #[test]
    fn test_extract_angle_bracketed_args() {
        let segment: PathSegment = parse_quote!(Result<(), DummyError>);
        let args = extract_angle_bracketed_args(&segment).expect("Failed to extract angle bracketed args");
        assert_eq!(args.args.len(), 2);

        let segment: PathSegment = parse_quote!(Option<()>);
        let args = extract_angle_bracketed_args(&segment).expect("Failed to extract angle bracketed args");
        assert_eq!(args.args.len(), 1);

        let segment: PathSegment = parse_quote!(Result<&'static str, i32>);
        let args = extract_angle_bracketed_args(&segment).expect("Failed to extract angle bracketed args");
        assert_eq!(args.args.len(), 2);

        let segment: PathSegment = parse_quote!(Result<usize, Box<dyn std::error::Error>>);
        let args = extract_angle_bracketed_args(&segment).expect("Failed to extract angle bracketed args");
        assert_eq!(args.args.len(), 2);
    }

    #[test]
    fn test_extract_last_generic_argument() {
        let args: AngleBracketedGenericArguments = parse_quote!(<(), DummyError>);
        let arg = extract_last_generic_argument(&args).expect("Failed to extract last generic argument");
        assert_eq!(normalize_quote(&arg), "DummyError");

        let args: AngleBracketedGenericArguments = parse_quote!(<&'static str, i32>);
        let arg = extract_last_generic_argument(&args).expect("Failed to extract last generic argument");
        assert_eq!(normalize_quote(&arg), "i32");

        let args: AngleBracketedGenericArguments = parse_quote!(<usize, Box<dyn std::error::Error>>);
        let arg = extract_last_generic_argument(&args).expect("Failed to extract last generic argument");
        assert_eq!(normalize_quote(&arg), "Box<dynstd::error::Error>");
    }

    #[test]
    fn test_extract_error_type_from_generic_argument() {
        let arg: GenericArgument = parse_quote!(DummyError);
        let ty = extract_error_type_from_generic_argument(&arg).expect("Failed to extract error type from generic argument");
        assert_eq!(normalize_quote(&ty), "DummyError");

        let arg: GenericArgument = parse_quote!(&'static str);
        let ty = extract_error_type_from_generic_argument(&arg).expect("Failed to extract error type from generic argument");
        assert_eq!(normalize_quote(&ty), "&'staticstr");

        let arg: GenericArgument = parse_quote!(Box<dyn std::error::Error>);
        let ty = extract_error_type_from_generic_argument(&arg).expect("Failed to extract error type from generic argument");
        assert_eq!(normalize_quote(&ty), "Box<dynstd::error::Error>");
    }

    #[test]
    fn test_extract_error_type() {
        let output_type: Type = parse_quote!(Result<(), DummyError>);
        let error_type = extract_error_type(&output_type).expect("Failed to extract error type");
        assert_eq!(normalize_quote(&error_type), "DummyError");

        let output_type: Type = parse_quote!(Result<(), &'static str>);
        let error_type = extract_error_type(&output_type).expect("Failed to extract error type");
        assert_eq!(normalize_quote(&error_type), "&'staticstr");

        let output_type: Type = parse_quote!(Result<usize, Box<dyn std::error::Error>>);
        let error_type = extract_error_type(&output_type).expect("Failed to extract error type");
        assert_eq!(normalize_quote(&error_type), "Box<dynstd::error::Error>");

        let output_type: Type = parse_quote!(Option<()>);
        assert!(extract_error_type(&output_type).is_err(), "Expected extract_error_type to return an error for Option");
    }
}
