// ---------------- [ File: src/try_reuse_wire_params.rs ]
crate::ix!();

/// If `arg` is a single-segment type path that matches an existing wire param, return Some(ident).
/// Otherwise None.
pub fn try_reuse_wire_param(arg: &GenericArgument, wire_gens: &Generics) -> Option<Ident> {
    info!(
        "[try_reuse_wire_param] Called with arg = {:?}, wire_gens = {:?}",
        arg, wire_gens
    );

    if let GenericArgument::Type(syn::Type::Path(tp)) = arg {
        info!(
            "[try_reuse_wire_param] Detected a Type::Path with segments: {:?}, qself = {:?}",
            tp.path.segments, tp.qself
        );
        if tp.qself.is_none() && tp.path.segments.len() == 1 {
            let seg = &tp.path.segments[0];
            let candidate = &seg.ident;
            info!(
                "[try_reuse_wire_param] Single-segment type path candidate = '{}'",
                candidate
            );

            let found = wire_gens.params.iter().any(|p| match p {
                syn::GenericParam::Type(t) => {
                    let matches = t.ident == *candidate;
                    if matches {
                        info!(
                            "[try_reuse_wire_param] Matched a type param '{}' in wire_gens",
                            candidate
                        );
                    }
                    matches
                }
                syn::GenericParam::Lifetime(lt) => {
                    let matches = lt.lifetime.ident == *candidate;
                    if matches {
                        info!(
                            "[try_reuse_wire_param] Matched a lifetime param '{}' in wire_gens",
                            candidate
                        );
                    }
                    matches
                }
                syn::GenericParam::Const(c) => {
                    let matches = c.ident == *candidate;
                    if matches {
                        info!(
                            "[try_reuse_wire_param] Matched a const param '{}' in wire_gens",
                            candidate
                        );
                    }
                    matches
                }
            });

            if found {
                info!(
                    "[try_reuse_wire_param] Returning Some({}) because it's found in wire_gens",
                    candidate
                );
                return Some(candidate.clone());
            } else {
                info!(
                    "[try_reuse_wire_param] Candidate '{}' not found in wire_gens, returning None",
                    candidate
                );
            }
        } else {
            info!(
                "[try_reuse_wire_param] Either qself is set or path has multiple segments; returning None"
            );
        }
    } else {
        info!(
            "[try_reuse_wire_param] Argument is not Type::Path, returning None."
        );
    }
    None
}

#[cfg(test)]
mod test_try_reuse_wire_param {
    use super::*;
    use syn::{
        parse_quote, GenericArgument, Generics, ItemStruct, Type, TypePath,
        Path, PathSegment, PathArguments,
    };

    /// Construct a representative set of wire generics:
    ///   - One type param (`T`),
    ///   - Another type param (`U`),
    ///   - One const param (`N`).
    fn make_wire_gens() -> Generics {
        info!("[test] Building wire_gens via parse of `Dummy<T, U, const N: usize>`");
        let item: ItemStruct = parse_quote! {
            struct Dummy<T, U, const N: usize>;
        };
        let result = item.generics;
        info!("[test] Resulting wire_gens = {:?}", result);
        result
    }

    /// Helper: build a `GenericArgument::Type(...)` from a single segment like `T`.
    fn single_segment_type_arg(ty_str: &str) -> Result<GenericArgument, syn::Error> {
        info!(
            "[test] Attempting to parse single_segment_type_arg('{}')",
            ty_str
        );
        let ty: Type = syn::parse_str(ty_str)?;
        info!(
            "[test] Parsed '{}' into Type = {:?}. Wrapping in GenericArgument::Type.",
            ty_str, ty
        );
        Ok(GenericArgument::Type(ty))
    }

    /// Negative test: ensure we return `None` for non-Type arguments (e.g. a lifetime).
    #[test]
    fn returns_none_for_lifetimes() {
        info!("[test] >>> returns_none_for_lifetimes >>>");
        let lifetime_arg: GenericArgument = syn::parse_str("'a")
            .expect("Failed to parse `'a` as a lifetime argument.");
        let wire_gens = make_wire_gens();
        let result = try_reuse_wire_param(&lifetime_arg, &wire_gens);
        info!("[test] result = {:?}", result);
        assert!(result.is_none());
    }

    /// Positive test: single-segment type param `T` that is indeed in wire generics.
    #[test]
    fn finds_existing_type_param_t() {
        info!("[test] >>> finds_existing_type_param_t >>>");
        let wire_gens = make_wire_gens();
        let type_arg = single_segment_type_arg("T")
            .expect("failed to parse type argument `T`");
        let result = try_reuse_wire_param(&type_arg, &wire_gens);
        info!("[test] result = {:?}", result);
        assert_eq!(result.as_ref().map(|id| id.to_string()), Some("T".to_string()));
    }

    /// Positive test: single-segment type param `U` that is indeed in wire generics.
    #[test]
    fn finds_existing_type_param_u() {
        info!("[test] >>> finds_existing_type_param_u >>>");
        let wire_gens = make_wire_gens();
        let type_arg = single_segment_type_arg("U")
            .expect("failed to parse type argument `U`");
        let result = try_reuse_wire_param(&type_arg, &wire_gens);
        info!("[test] result = {:?}", result);
        assert_eq!(result.as_ref().map(|id| id.to_string()), Some("U".to_string()));
    }

    /// Negative test: single-segment type param `V` not present in wire generics => `None`.
    #[test]
    fn unknown_type_param_returns_none() {
        info!("[test] >>> unknown_type_param_returns_none >>>");
        let wire_gens = make_wire_gens();
        let type_arg = single_segment_type_arg("V")
            .expect("failed to parse type argument `V`");
        let result = try_reuse_wire_param(&type_arg, &wire_gens);
        info!("[test] result = {:?}", result);
        assert!(result.is_none());
    }

    /// Positive test: single-segment const param `N` that is indeed in wire generics.
    #[test]
    fn finds_existing_const_param_n() {
        info!("[test] >>> finds_existing_const_param_n >>>");
        let wire_gens = make_wire_gens();
        // This is somewhat contrived since `N` is actually a const param,
        // but we parse it as if it's a type path. We do so to test how the function handles it.
        let type_arg = single_segment_type_arg("N")
            .expect("failed to parse type argument `N`");
        let result = try_reuse_wire_param(&type_arg, &wire_gens);
        info!("[test] result = {:?}", result);
        assert_eq!(result.as_ref().map(|id| id.to_string()), Some("N".to_string()));
    }

    /// Negative test: multi-segment path like `foo::T` => no match (returns `None`).
    #[test]
    fn multi_segment_path_returns_none() {
        info!("[test] >>> multi_segment_path_returns_none >>>");
        let wire_gens = make_wire_gens();
        let type_arg: GenericArgument = parse_quote! { foo::T };
        let result = try_reuse_wire_param(&type_arg, &wire_gens);
        info!("[test] result = {:?}", result);
        assert!(result.is_none());
    }

    /// Negative test: `qself` style type => no match.
    #[test]
    fn qself_style_type_returns_none() {
        info!("[test] >>> qself_style_type_returns_none >>>");
        let wire_gens = make_wire_gens();
        let type_arg: GenericArgument = parse_quote! { <T as SomeTrait>::Alias };
        let result = try_reuse_wire_param(&type_arg, &wire_gens);
        info!("[test] result = {:?}", result);
        assert!(result.is_none());
    }

    /// Negative test: something not a `Type::Path`, e.g. a reference type => no match.
    #[test]
    fn reference_type_returns_none() {
        info!("[test] >>> reference_type_returns_none >>>");
        let wire_gens = make_wire_gens();
        let type_arg: GenericArgument = parse_quote! { &T };
        let result = try_reuse_wire_param(&type_arg, &wire_gens);
        info!("[test] result = {:?}", result);
        assert!(result.is_none());
    }

    /// Negative test: if the path is Type::Path but has zero segments => improbable in normal code.
    #[test]
    fn zero_segment_path_returns_none() {
        info!("[test] >>> zero_segment_path_returns_none >>>");
        let wire_gens = make_wire_gens();

        let empty_type_path = TypePath {
            qself: None,
            path: Path {
                leading_colon: None,
                segments: syn::punctuated::Punctuated::new(),
            },
        };
        let type_arg = GenericArgument::Type(Type::Path(empty_type_path));

        let result = try_reuse_wire_param(&type_arg, &wire_gens);
        info!("[test] result = {:?}", result);
        assert!(result.is_none());
    }
}
