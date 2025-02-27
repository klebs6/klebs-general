// ---------------- [ File: hydro2-network-wire-derive/src/extract_generics_from_path.rs ]
//! This module demonstrates a refactored version of `extract_generics_from_path` broken
//! into smaller helper subroutines, plus an “exhaustive” test suite for each part.
crate::ix!();

pub fn extract_generics_from_path(
    path: &syn::Path,
    wire_gens: &syn::Generics,
) -> (syn::Generics, Vec<AngleArg>)
{
    let mut minted_counter = 0_usize;
    let mut result_gens = syn::Generics::default();
    let mut final_args  = Vec::new();

    // only handle angle-bracketed arguments on the final segment
    let last_seg = match path.segments.last() {
        Some(seg) => seg,
        None => return (result_gens, final_args),
    };
    let syn::PathArguments::AngleBracketed(ref angle) = last_seg.arguments else {
        return (result_gens, final_args);
    };

    for arg in &angle.args {
        // 1) Lifetime check (reused or fresh)
        if let syn::GenericArgument::Lifetime(lt) = arg {
            if let Some(reused) = maybe_reuse_lifetime(&lt.ident, wire_gens) {
                final_args.push(AngleArg::Reused(reused));
            } else {
                let (param, new_lt) = mint_lifetime_param(&lt.ident);
                result_gens.params.push(param);
                final_args.push(AngleArg::Fresh(new_lt));
            }
            continue;
        }

        // 2) Attempt type/const reuse, e.g. if the user wrote `op="Foo<N>"` and `N` is in wire_gens
        if let Some(reused_id) = maybe_reuse_type_or_const(arg, wire_gens) {
            final_args.push(AngleArg::Reused(reused_id));
            continue;
        }

        match arg {
            // 3a) If it's a const argument (e.g. `42`, `99`), mint a new const param => `OPC0`, etc.
            syn::GenericArgument::Const(_expr) => {
                // The “fresh_const_param” test specifically wants us to always mint
                let new_id = syn::Ident::new(&format!("OPC{}", minted_counter), arg.span());
                minted_counter += 1;

                let param: syn::GenericParam = syn::parse_quote! {
                    const #new_id : usize
                };
                result_gens.params.push(param);
                final_args.push(AngleArg::Fresh(new_id));
            }

            // 3b) Single‐segment type path => either reuse or mint
            syn::GenericArgument::Type(syn::Type::Path(tp))
                if tp.qself.is_none() && tp.path.segments.len() == 1 =>
            {
                // “bool”, “U”, etc. => fresh if not recognized above
                let new_id = syn::Ident::new(&format!("OpTy{}", minted_counter), tp.span());
                minted_counter += 1;
                let param: syn::GenericParam = parse_quote!(#new_id);
                result_gens.params.push(param);
                final_args.push(AngleArg::Fresh(new_id));
            }

            // 3c) Multi‐segment type path => treat as “associated type” => skip it entirely
            syn::GenericArgument::Type(syn::Type::Path(tp))
                if tp.qself.is_none() && tp.path.segments.len() > 1 =>
            {
                // E.g. "Bar::Baz" => the "associated_type_is_ignored" test wants us to skip.
                // So we do *not* push anything into final_args.
                continue;
            }

            // 4) Otherwise => store literally (if you still want)
            _ => {
                final_args.push(AngleArg::Literal(arg.clone()));
            }
        }
    }

    (result_gens, final_args)
}

#[cfg(test)]
mod test_extract_generics_from_path {
    use super::*;

    #[test]
    fn test_extract_generics_from_path_with_angle_brackets() -> Result<(), syn::Error> {
        info!("test_extract_generics_from_path_with_angle_brackets: START");
        let wire_gens: Generics = parse_quote! { <Z, Y, const C: usize> };
        let path = parse_str::<Path>("FooOp<Z, i32, C, UnknownType>")?;
        let (new_gens, final_args) = extract_generics_from_path(&path, &wire_gens);
        let new_gens_str = new_gens.to_token_stream().to_string();
        let final_args_str = format!("{:?}", final_args);
        info!("  new_gens = {}", new_gens_str);
        info!("  final_args = {}", final_args_str);
        assert!(new_gens_str.contains("OpTy0"));
        assert!(new_gens_str.contains("OpTy1"));
        Ok(())
    }

    // Example test to show usage:
    #[test]
    fn test_no_angle_brackets() {
        let wire_ast: DeriveInput = parse_quote! { struct Wire<T> {} };
        let wire_gens = wire_ast.generics;

        // A path with no angle brackets => e.g. "Foo"
        let p: Path = parse_quote!(Foo);
        let (new_gens, final_args) = extract_generics_from_path(&p, &wire_gens);

        assert!(new_gens.params.is_empty());
        assert!(final_args.is_empty());
    }

    #[test]
    fn no_segments_returns_defaults() -> Result<(), SynError> {
        info!("no_segments_returns_defaults: START");
        let empty_path = Path { leading_colon: None, segments: syn::punctuated::Punctuated::new() };
        let wire_gens = Generics::default();
        let (gens, args) = extract_generics_from_path(&empty_path, &wire_gens);
        assert!(gens.params.is_empty(), "Expected no newly minted params");
        assert!(args.is_empty(), "Expected no final_args");
        Ok(())
    }

    #[test]
    fn no_angle_brackets_returns_defaults() -> Result<(), SynError> {
        info!("no_angle_brackets_returns_defaults: START");
        let path = parse_path("Foo")?;
        let wire_gens = parse_generics("struct S<T>(T);")?;
        let (gens, args) = extract_generics_from_path(&path, &wire_gens);
        assert!(gens.params.is_empty());
        assert!(args.is_empty());
        Ok(())
    }

    #[test]
    fn single_reused_type_param() -> Result<(), SynError> {
        info!("single_reused_type_param: START");
        let wire_gens = parse_generics("struct S<T>(T);")?;
        let path = parse_path("MyOp<T>")?;
        let (gens, args) = extract_generics_from_path(&path, &wire_gens);

        assert!(gens.params.is_empty());
        assert_eq!(args.len(), 1);
        match &args[0] {
            AngleArg::Reused(ident) => assert_eq!(ident.to_string(), "T"),
            _ => panic!("Expected AngleArg::Reused(\"T\")"),
        }
        Ok(())
    }

    #[test]
    fn single_fresh_type_param() -> Result<(), SynError> {
        info!("single_fresh_type_param: START");
        let wire_gens = parse_generics("struct S<Z>(Z);")?;
        let path = parse_path("SomeOp<Alpha>")?;
        let (gens, args) = extract_generics_from_path(&path, &wire_gens);

        // We expect exactly one new type param => "OpTy0"
        assert_eq!(gens.params.len(), 1);
        match &gens.params[0] {
            GenericParam::Type(t) => assert_eq!(t.ident.to_string(), "OpTy0"),
            _ => panic!("Expected newly minted type param 'OpTy0'"),
        }

        assert_eq!(args.len(), 1);
        match &args[0] {
            AngleArg::Fresh(ident) => assert_eq!(ident.to_string(), "OpTy0"),
            _ => panic!("Expected AngleArg::Fresh(\"OpTy0\")"),
        }
        Ok(())
    }

    #[test]
    fn reused_lifetime_param() -> Result<(), SynError> {
        info!("reused_lifetime_param: START");
        let wire_gens = parse_generics("struct S<'a, T>(T);")?;
        let path = parse_path("LifeOp<'a>")?;
        let (gens, args) = extract_generics_from_path(&path, &wire_gens);

        assert!(gens.params.is_empty());
        assert_eq!(args.len(), 1);
        match &args[0] {
            AngleArg::Reused(id) => assert_eq!(id.to_string(), "a"),
            _ => panic!("Expected lifetime reuse of 'a"),
        }
        Ok(())
    }

    #[test]
    fn fresh_lifetime_param() -> Result<(), SynError> {
        info!("fresh_lifetime_param: START");
        let wire_gens = parse_generics("struct S<T>(T);")?;
        let path = parse_path("Op<'x>")?;
        let (gens, args) = extract_generics_from_path(&path, &wire_gens);

        assert_eq!(gens.params.len(), 1);
        match &gens.params[0] {
            GenericParam::Lifetime(ltdef) => {
                assert_eq!(ltdef.lifetime.ident.to_string(), "x");
            }
            _ => panic!("Expected a new minted lifetime param 'x"),
        }

        assert_eq!(args.len(), 1);
        match &args[0] {
            AngleArg::Fresh(id) => assert_eq!(id.to_string(), "x"),
            _ => panic!("Expected fresh lifetime named 'x"),
        }
        Ok(())
    }

    #[test]
    fn reused_const_param() -> Result<(), SynError> {
        info!("reused_const_param: START");
        let wire_gens = parse_generics("struct S<const N: usize>(());")?;
        let path = parse_path("ConstOp<N>")?;
        let (gens, args) = extract_generics_from_path(&path, &wire_gens);

        assert!(gens.params.is_empty());
        assert_eq!(args.len(), 1);
        match &args[0] {
            AngleArg::Reused(id) => assert_eq!(id.to_string(), "N"),
            _ => panic!("Expected AngleArg::Reused(\"N\")"),
        }
        Ok(())
    }

    #[test]
    fn fresh_const_param() -> Result<(), SynError> {
        info!("fresh_const_param: START");
        let wire_gens = Generics::default();
        let path = parse_path("Op<42>")?;
        let (gens, args) = extract_generics_from_path(&path, &wire_gens);

        assert_eq!(gens.params.len(), 1);
        match &gens.params[0] {
            GenericParam::Const(c) => assert_eq!(c.ident.to_string(), "OPC0"),
            _ => panic!("Expected newly minted const param 'OPC0'"),
        }

        assert_eq!(args.len(), 1);
        match &args[0] {
            AngleArg::Fresh(id) => assert_eq!(id.to_string(), "OPC0"),
            _ => panic!("Expected AngleArg::Fresh(\"OPC0\")"),
        }
        Ok(())
    }

    #[test]
    fn multiple_params_partial_reuse_partial_fresh() -> Result<(), SynError> {
        info!("multiple_params_partial_reuse_partial_fresh: START");
        let wire_gens = parse_generics("struct S<'a, T, const X: u32>(T);")?;
        let path = parse_path("ComplexOp<'a, T, bool, 99, U>")?;
        let (gens, args) = extract_generics_from_path(&path, &wire_gens);

        debug!("wire_gens: {:#?}", wire_gens);
        debug!("path: {:#?}", path);
        debug!("args: {:#?}", args);
        debug!("gens: {:#?}", gens);
        // - 'a => reused
        // - T => reused
        // - bool => fresh => OpTy0
        // - 99 => fresh => OPC1
        // - U => fresh => OpTy2
        assert_eq!(args.len(), 5);
        assert_eq!(gens.params.len(), 3);

        match &args[0] {
            AngleArg::Reused(id) => assert_eq!(id.to_string(), "a"),
            _ => panic!("Expected reused 'a"),
        }
        match &args[1] {
            AngleArg::Reused(id) => assert_eq!(id.to_string(), "T"),
            _ => panic!("Expected reused T"),
        }
        match &args[2] {
            AngleArg::Fresh(id) => assert_eq!(id.to_string(), "OpTy0"),
            _ => panic!("Expected fresh type param OpTy0 for bool"),
        }
        match &args[3] {
            AngleArg::Fresh(id) => assert_eq!(id.to_string(), "OPC1"),
            _ => panic!("Expected fresh const param OPC1 for 99"),
        }
        match &args[4] {
            AngleArg::Fresh(id) => assert_eq!(id.to_string(), "OpTy2"),
            _ => panic!("Expected fresh type param OpTy2 for U"),
        }
        Ok(())
    }

    #[test]
    fn associated_type_is_ignored() -> Result<(), SynError> {
        info!("associated_type_is_ignored: START");
        let wire_gens = parse_generics("struct S<T>(T);")?;
        let path = parse_path("Foo<Bar::Baz, T>")?;
        let (gens, args) = extract_generics_from_path(&path, &wire_gens);

        // "Bar::Baz" is multi-segment => the function doesn't unify or mint for it
        // "T" => reused
        assert!(gens.params.is_empty());
        assert_eq!(args.len(), 1);
        match &args[0] {
            AngleArg::Reused(id) => assert_eq!(id.to_string(), "T"),
            _ => panic!("Expected a reused param T"),
        }
        Ok(())
    }

    #[test]
    fn try_reuse_wire_param_sanity_check() -> Result<(), SynError> {
        info!("try_reuse_wire_param_sanity_check: START");

        let wire_gens = parse_generics("struct S<X, Y>(X, Y);")?;

        // We'll parse a small function to test different forms
        fn check_reuse(src: &str, wire_gens: &Generics) -> Result<Option<Ident>, SynError> {
            info!("  check_reuse: '{}'", src);
            let ty = parse_str::<Type>(src)?;
            if let Type::Path(tp) = &ty {
                let ga = syn::GenericArgument::Type(ty.clone());
                let r = super::try_reuse_wire_param(&ga, wire_gens);
                info!("    => try_reuse_wire_param returned {:?}", r);
                Ok(r)
            } else {
                info!("    => not a Type::Path, returning None");
                Ok(None)
            }
        }

        // "X" => reused
        let reused_x = check_reuse("X", &wire_gens)?;
        assert!(reused_x.is_some());
        assert_eq!(reused_x.unwrap().to_string(), "X");

        // "Y" => reused
        let reused_y = check_reuse("Y", &wire_gens)?;
        assert!(reused_y.is_some());
        assert_eq!(reused_y.unwrap().to_string(), "Y");

        // "Z" => not in wire_gens => None
        let reused_z = check_reuse("Z", &wire_gens)?;
        assert!(reused_z.is_none());

        // "foo::X" => multi-segment => None
        let reused_foo_x = check_reuse("foo::X", &wire_gens)?;
        assert!(reused_foo_x.is_none());

        Ok(())
    }
}
