// ---------------- [ File: src/maybe_reuse_type_or_const.rs ]
crate::ix!();

/// Checks if a `GenericArgument` is a single-segment type or const param recognized in `wire_gens`.
/// If so, returns `Some(ident)`. Otherwise returns `None`.
///
/// This is the same logic you used in `try_reuse_wire_param`.
pub fn maybe_reuse_type_or_const(
    arg: &GenericArgument,
    wire_gens: &Generics,
) -> Option<Ident> {
    // (Implementation identical to your existing `try_reuse_wire_param` except we
    // rename it for clarity.)
    match arg {
        GenericArgument::Type(syn::Type::Path(tp)) => {
            // Single-segment path check
            if tp.qself.is_none() && tp.path.segments.len() == 1 {
                let candidate = &tp.path.segments[0].ident;
                // Check if candidate is in wire_gens
                let found = wire_gens.params.iter().any(|p| match p {
                    GenericParam::Type(TypeParam { ident, .. }) => *ident == *candidate,
                    GenericParam::Const(ConstParam { ident, .. }) => *ident == *candidate,
                    _ => false,
                });
                if found {
                    return Some(candidate.clone());
                }
            }
        }
        // If it’s a const param in a form we can parse, we might do more checks, but typically
        // we rely on your existing logic. For brevity, we skip that here.
        _ => {}
    }
    None
}

#[cfg(test)]
mod test_maybe_reuse_type_or_const {
    use super::*;

    #[test]
    fn test_maybe_reuse_type_or_const_notfound() {
        let wire_ast: DeriveInput = parse_quote! { struct Wire<A> {} };
        let wire_gens = wire_ast.generics;

        // We'll parse a GenericArgument::Type for "B"
        let type_arg: syn::GenericArgument = parse_quote!(B);
        let res = maybe_reuse_type_or_const(&type_arg, &wire_gens);
        assert!(res.is_none(), "Should not find 'B' in <A>");
    }

    #[test]
    fn test_maybe_reuse_type_or_const_found_type() {
        let wire_ast: DeriveInput = parse_quote! { struct Wire<A, const N: usize> {} };
        let wire_gens = wire_ast.generics;

        let type_arg: syn::GenericArgument = parse_quote!(A);
        let res = maybe_reuse_type_or_const(&type_arg, &wire_gens);
        assert_eq!(res.unwrap().to_string(), "A");
    }

    #[test]
    fn test_maybe_reuse_type_or_const_found_const() {
        let wire_ast: DeriveInput = parse_quote! { struct Wire<A, const N: usize> {} };
        let wire_gens = wire_ast.generics;

        // We'll parse an argument "N" (like a type param). This is a bit contrived but
        // matches how the code tries to unify single-segment consts.
        let type_arg: syn::GenericArgument = parse_quote!(N);
        let res = maybe_reuse_type_or_const(&type_arg, &wire_gens);
        assert_eq!(res.unwrap().to_string(), "N");
    }
}
