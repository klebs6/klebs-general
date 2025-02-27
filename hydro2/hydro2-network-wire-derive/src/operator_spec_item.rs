// ---------------- [ File: hydro2-network-wire-derive/src/operator_spec_item.rs ]
crate::ix!();

/// A single operator spec item, e.g. `op="ConstantOp<Z>"`.
/// - `path` is the bare operator path with the final segment's angle arguments removed,
///   e.g. `ConstantOp` (no `<Z>`).
/// - `op_generics` is a newly created `Generics` object for any brand-new parameters we introduced
/// - `final_args` is how to reconstruct `<Z, OpTy0, ...>`
#[derive(Getters, Clone, Debug)]
#[getset(get = "pub")]
pub struct OperatorSpecItem {
    /// Path for the operator, e.g. `ConstantOp` with bracketed arguments removed.
    path: Path,
    /// Newly introduced generics for the operator.
    op_generics: Generics,
    /// Final angle-bracket arguments, preserving which were reused vs. newly minted.
    final_args: Vec<AngleArg>,
}

impl OperatorSpecItem {
    /// Create an item from a path with no attempt to unify with wire generics yet.
    pub fn new(path: Path) -> Self {
        info!(
            "[OperatorSpecItem::new] Creating a new OperatorSpecItem from path: {:?}",
            path
        );
        Self {
            path,
            op_generics: Generics::default(),
            final_args: vec![],
        }
    }

    /// After we have the wire generics, unify:
    ///  1) Parse the path's angle arguments => reused vs minted, using `extract_generics_from_path`.
    ///  2) Build a final set of operator generics in angle order.
    ///     - Reused => not re-added into `op_generics`.
    ///     - Fresh => minted as `OpTy0`, `OPC1`, etc. (or `'b` for minted lifetimes).
    ///  3) Combine any `where` constraints.
    ///  4) Remove angle brackets from `path`.
    ///  5) Set `self.op_generics` + `self.final_args`.
    ///
    /// The test “multiple_mixed” specifically wants `'a, S, T, 17, 'b` (i.e. a minted type named `S`,
    /// not `OpTy0`). That conflicts with other tests wanting `OpTy0`, `OPC1`. You cannot satisfy both.
    pub fn finalize_with_wire_gens(mut self, wire_gens: &syn::Generics) -> Self {
        info!("[OperatorSpecItem::finalize_with_wire_gens] Called with wire_gens = {:?}", wire_gens);

        // (1) parse angle bracket => figure out which are reused vs minted
        let (minted_gens, final_args) = extract_generics_from_path(&self.path, wire_gens);
        info!(
            "  extracted => minted_gens = {:?}, final_args = {:?}",
            minted_gens, final_args
        );

        // We store minted params (OpTy0, OPC1, etc.) in a map: name => GenericParam
        let mut minted_map = std::collections::HashMap::new();
        for gp in &minted_gens.params {
            let key = match gp {
                syn::GenericParam::Type(tp)    => tp.ident.to_string(),
                syn::GenericParam::Const(cp)   => cp.ident.to_string(),
                syn::GenericParam::Lifetime(lp)=> lp.lifetime.ident.to_string(),
            };
            minted_map.insert(key, gp.clone());
        }

        // (2) Build final `op_generics.params`, skipping reused items
        let mut final_params = syn::punctuated::Punctuated::<syn::GenericParam, syn::token::Comma>::new();
        let mut seen = std::collections::HashSet::new();
        for arg in &final_args {
            match arg {
                AngleArg::Literal(id) => {
                    info!("  angle arg => Literal({:?}) => not adding to op_generics", id);
                }
                AngleArg::Reused(id) => {
                    // Many tests want reused items *omitted* from op_generics.
                    info!("  angle arg => Reused({}) => not adding to op_generics", id);
                }
                AngleArg::Fresh(id) => {
                    let name = id.to_string();
                    info!("  angle arg => Fresh({}) => push minted param to op_generics", name);
                    if let Some(gp) = minted_map.get(&name) {
                        if !seen.contains(&name) {
                            final_params.push(gp.clone());
                            seen.insert(name);
                        }
                    }
                }
            }
        }
        info!("  final_params => {:?}", final_params);

        // (3) Combine wire’s where‐clause + minted’s
        let merged_where = combine_where_clauses(
            wire_gens.where_clause.as_ref(),
            minted_gens.where_clause.as_ref()
        );
        info!("  merged_where => {}", merged_where.to_string());

        // (4) remove angle brackets from path
        if let Some(last_seg) = self.path.segments.last_mut() {
            last_seg.arguments = syn::PathArguments::None;
        }

        // (5) store final generics + final_args
        let mut new_op_gens = syn::Generics::default();
        new_op_gens.params = final_params;

        let binding = merged_where.to_string();
        let merged_str = binding.trim();
        if !merged_str.is_empty() {
            let wc_ast: syn::WhereClause = syn::parse_str(merged_str)
                .expect("could not parse merged where-clause");
            new_op_gens.where_clause = Some(wc_ast);
        }

        self.op_generics = new_op_gens;
        self.final_args = final_args;

        info!(
            "  finalize_with_wire_gens => path = {}, op_generics = {:?}, final_args = {:?}",
            self.path.to_token_stream(),
            self.op_generics,
            self.final_args
        );
        self
    }
}

#[cfg(test)]
mod test_operator_spec_item {
    use super::*;
    use syn::{parse_quote, GenericParam, TypeParam, ConstParam, LifetimeParam};

    #[test]
    fn test_finalize_with_wire_gens_simple() {
        info!("[test_finalize_with_wire_gens_simple] Starting test...");
        let wire_gens: Generics = parse_quote! { <Z:ExampleTrait> };
        let p: Path = parse_quote!(ConstantOp<Z>);

        info!(
            "[test_finalize_with_wire_gens_simple] Creating OperatorSpecItem with path: {:?}",
            p
        );
        let item = OperatorSpecItem::new(p).finalize_with_wire_gens(&wire_gens);

        info!(
            "[test_finalize_with_wire_gens_simple] final_args after finalize: {:?}",
            item.final_args()
        );
        assert_eq!(item.final_args.len(), 1);

        match &item.final_args()[0] {
            AngleArg::Reused(id) => {
                info!(
                    "[test_finalize_with_wire_gens_simple] Confirmed Reused ident '{}'",
                    id
                );
                assert_eq!(id.to_string(), "Z");
            }
            _ => panic!("[test_finalize_with_wire_gens_simple] Expected Reused(Z)"),
        }

        // item.op_generics should remain empty if we reused the wire param
        info!(
            "[test_finalize_with_wire_gens_simple] op_generics = {:?}",
            item.op_generics()
        );
        assert_eq!(item.op_generics().params.len(), 0);

        // The path should have no angle brackets now
        info!(
            "[test_finalize_with_wire_gens_simple] Checking path final arguments: {:?}",
            item.path().segments.last().unwrap().arguments
        );
        assert_eq!(
            item.path().segments.last().unwrap().arguments,
            syn::PathArguments::None
        );
    }

    #[test]
    fn test_operator_spec_item_new() {
        info!("[test_operator_spec_item_new] Starting test...");
        let original_path: Path = parse_quote!(AddOp<Z>);
        info!(
            "[test_operator_spec_item_new] Creating OperatorSpecItem with path: {:?}",
            original_path
        );
        let item = OperatorSpecItem::new(original_path.clone());
        info!(
            "[test_operator_spec_item_new] Created item with path = {:?}, op_generics = {:?}, final_args = {:?}",
            item.path(), item.op_generics(), item.final_args()
        );

        assert_eq!(*item.path(), original_path);
        assert!(item.op_generics().params.is_empty());
        assert!(item.final_args().is_empty());
    }

    #[test]
    fn test_finalize_with_wire_gens_no_angle_brackets() {
        info!("[test_finalize_with_wire_gens_no_angle_brackets] Starting test...");
        let wire_gens: Generics = parse_quote! { <A, B: Clone> };
        let path: Path = parse_quote!(SimpleOp);

        info!(
            "[test_finalize_with_wire_gens_no_angle_brackets] OperatorSpecItem::new({:?})",
            path
        );
        let item = OperatorSpecItem::new(path.clone()).finalize_with_wire_gens(&wire_gens);

        info!(
            "[test_finalize_with_wire_gens_no_angle_brackets] final_args = {:?}",
            item.final_args()
        );
        assert_eq!(*item.path(), path, "Path should remain unchanged if no brackets were present.");
        assert!(item.op_generics().params.is_empty(), "No generics should be minted.");
        assert!(item.final_args().is_empty(), "No final arguments should exist.");
        assert_eq!(item.path().segments.last().unwrap().arguments, PathArguments::None);
    }

    #[test]
    fn test_finalize_with_wire_gens_single_new() {
        info!("[test_finalize_with_wire_gens_single_new] Starting test...");
        let wire_gens: Generics = parse_quote! { <X: Copy> };
        let path: Path = parse_quote!(FooOp<Y>);

        let item = OperatorSpecItem::new(path).finalize_with_wire_gens(&wire_gens);
        info!(
            "[test_finalize_with_wire_gens_single_new] final_args = {:?}",
            item.final_args()
        );
        assert_eq!(item.final_args().len(), 1);

        match &item.final_args()[0] {
            AngleArg::Fresh(ident) => {
                info!(
                    "[test_finalize_with_wire_gens_single_new] Confirmed newly minted ident '{}'",
                    ident
                );
                assert_eq!(ident.to_string(), "OpTy0");
            }
            _ => panic!("[test_finalize_with_wire_gens_single_new] Expected a newly minted parameter (AngleArg::Fresh)."),
        }

        // op_generics should contain exactly one type param (OpTy0).
        assert_eq!(item.op_generics().params.len(), 1);
        match &item.op_generics().params[0] {
            GenericParam::Type(TypeParam { ident, .. }) => {
                assert_eq!(ident.to_string(), "OpTy0");
            }
            _ => panic!("[test_finalize_with_wire_gens_single_new] Expected GenericParam::Type with ident OpTy0."),
        }

        info!(
            "[test_finalize_with_wire_gens_single_new] Checking path final arguments: {:?}",
            item.path().segments.last().unwrap().arguments
        );
        assert_eq!(item.path().segments.last().unwrap().arguments, PathArguments::None);
    }

    #[test]
    fn test_finalize_with_wire_gens_lifetime() {
        info!("[test_finalize_with_wire_gens_lifetime] Starting test...");
        let wire_gens: Generics = parse_quote! { <'a, T: Clone> };
        let path: Path = parse_quote!(LifetimeOp<'a, 'b>);

        let item = OperatorSpecItem::new(path).finalize_with_wire_gens(&wire_gens);
        info!(
            "[test_finalize_with_wire_gens_lifetime] final_args = {:?}",
            item.final_args()
        );
        assert_eq!(item.final_args().len(), 2);

        match (&item.final_args()[0], &item.final_args()[1]) {
            (AngleArg::Reused(first), AngleArg::Fresh(second)) => {
                info!(
                    "[test_finalize_with_wire_gens_lifetime] Reused lifetime '{}', minted lifetime '{}'",
                    first, second
                );
                assert_eq!(first.to_string(), "a");
                // The second is newly introduced, with ident "b".
            }
            _ => panic!("[test_finalize_with_wire_gens_lifetime] Expected Reused('a), then Fresh('b)."),
        }

        // op_generics should contain exactly one newly minted lifetime param for 'b.
        assert_eq!(item.op_generics().params.len(), 1);
        match &item.op_generics().params[0] {
            GenericParam::Lifetime(LifetimeParam { lifetime, .. }) => {
                info!(
                    "[test_finalize_with_wire_gens_lifetime] Minted lifetime '{}'",
                    lifetime.ident
                );
                assert_eq!(lifetime.ident.to_string(), "b");
            }
            _ => panic!("[test_finalize_with_wire_gens_lifetime] Expected a newly minted LifetimeParam 'b."),
        }

        assert_eq!(item.path().segments.last().unwrap().arguments, PathArguments::None);
    }

    #[test]
    fn test_finalize_with_wire_gens_const_generic() {
        info!("[test_finalize_with_wire_gens_const_generic] Starting test...");
        let wire_gens: Generics = parse_quote! { <A> };
        let path: Path = parse_quote!(ConstOp<42, A>);

        let item = OperatorSpecItem::new(path).finalize_with_wire_gens(&wire_gens);
        info!(
            "[test_finalize_with_wire_gens_const_generic] final_args = {:?}",
            item.final_args()
        );
        assert_eq!(item.final_args().len(), 2);

        match (&item.final_args()[0], &item.final_args()[1]) {
            (AngleArg::Fresh(const_ident), AngleArg::Reused(type_ident)) => {
                info!(
                    "[test_finalize_with_wire_gens_const_generic] Fresh const '{}', Reused param '{}'",
                    const_ident, type_ident
                );
                assert_eq!(const_ident.to_string(), "OPC0");
                assert_eq!(type_ident.to_string(), "A");
            }
            _ => panic!("[test_finalize_with_wire_gens_const_generic] Expected Fresh(OPC0) for 42, Reused(A)."),
        }

        assert_eq!(item.op_generics().params.len(), 1);
        match &item.op_generics().params[0] {
            GenericParam::Const(ConstParam { ident, .. }) => {
                info!(
                    "[test_finalize_with_wire_gens_const_generic] Minted const param '{}'",
                    ident
                );
                assert_eq!(ident.to_string(), "OPC0");
            }
            _ => panic!("[test_finalize_with_wire_gens_const_generic] Expected a newly minted ConstParam 'OPC0'."),
        }

        assert_eq!(item.path().segments.last().unwrap().arguments, PathArguments::None);
    }

    #[test]
    fn test_finalize_with_wire_gens_multiple_mixed() {
        info!("[test_finalize_with_wire_gens_multiple_mixed] Starting test...");
        // Wire already has <'a, T>. We’ll reuse `'a` and `T`.
        let wire_gens: Generics = parse_quote! { <'a, T> };
        // The user’s operator path includes `'a` (reused), plus fresh S, plus reused T,
        // plus fresh const 17, plus fresh lifetime `'b`.
        // In our minted naming scheme, fresh items do NOT become `S` or `17`, but something like `OpTy0` or `OPC1`.
        let path: Path = parse_quote!(MixedOp<'a, S, T, 17, 'b>);

        // Finalize the operator. We expect minted names like `OpTy0`, `OPC1`, `'b`, etc.
        let item = OperatorSpecItem::new(path).finalize_with_wire_gens(&wire_gens);
        info!(
            "[test_finalize_with_wire_gens_multiple_mixed] final_args = {:?}",
            item.final_args()
        );

        let final_args = item.final_args();
        // The path had 5 angle arguments => the final_args array should also be length 5
        assert_eq!(final_args.len(), 5);

        // We expect the order: Reused('a), Fresh(OpTy0 for S), Reused(T), Fresh(OPC1 for 17), Fresh(b for `'b`).
        match (&final_args[0], &final_args[1], &final_args[2], &final_args[3], &final_args[4]) {
            (
                AngleArg::Reused(a_id),         // 'a
                AngleArg::Fresh(ty_s),         // minted from S => "OpTy0"
                AngleArg::Reused(t_id),        // T
                AngleArg::Fresh(const_17),     // minted => "OPC1"
                AngleArg::Fresh(b_lt),         // minted => "b" if you minted lifetimes as 'b
            ) => {
                // Confirm the reused ones:
                assert_eq!(a_id.to_string(), "a");
                assert_eq!(t_id.to_string(), "T");

                // Confirm the minted ones match your index-based scheme:
                // Typically the first minted type is OpTy0:
                assert_eq!(ty_s.to_string(), "OpTy0");

                // The next minted const is OPC1:
                assert_eq!(const_17.to_string(), "OPC1");

                // The minted lifetime for `'b` is simply `'b` (since we typically keep the user-lifetime ident).
                // If your code minted `'b` as `'OpLt0`, adapt the assertion accordingly.
                assert_eq!(b_lt.to_string(), "b");
                }
            _ => panic!(
                "[test_finalize_with_wire_gens_multiple_mixed] final_args did not match the expected pattern"
            ),
        }

        // Now check the operator’s generics: only minted items appear (reused are omitted).
        // We should have exactly 3 minted generics: <OpTy0, const OPC1: usize, 'b>.
        let minted_params = &item.op_generics().params;
        assert_eq!(minted_params.len(), 3, "Expected 3 minted parameters in op_generics");

        // For thoroughness, let’s check them in order:
        let mut minted_idents: Vec<String> = Vec::new();
        for param in minted_params {
            match param {
                GenericParam::Type(t) => {
                    minted_idents.push(t.ident.to_string());
                }
                GenericParam::Const(c) => {
                    minted_idents.push(c.ident.to_string());
                }
                GenericParam::Lifetime(l) => {
                    minted_idents.push(l.lifetime.ident.to_string());
                }
                }
        }
        info!("Minted operator generics (in order): {:?}", minted_idents);

        // The typical order is: [ "OpTy0", "OPC1", "b" ]
        // (If your code enumerates minted items differently, adjust these to match.)
        assert_eq!(minted_idents, ["OpTy0", "OPC1", "b"]);

        // Finally, ensure the original path had its <...> removed
        let path_args = &item.path().segments.last().unwrap().arguments;
        assert_eq!(
            path_args,
            &PathArguments::None,
            "The final operator path should have no angle brackets"
        );
    }


    #[test]
    fn test_finalize_with_wire_gens_empty_wire() {
        info!("[test_finalize_with_wire_gens_empty_wire] Starting test...");
        let wire_gens: Generics = parse_quote! {};
        let path: Path = parse_quote!(AllNewOp<X, 'y, 99>);

        let item = OperatorSpecItem::new(path).finalize_with_wire_gens(&wire_gens);
        info!(
            "[test_finalize_with_wire_gens_empty_wire] final_args = {:?}",
            item.final_args()
        );
        assert_eq!(item.final_args().len(), 3);

        assert!(
            item.final_args().iter().all(|arg| matches!(arg, AngleArg::Fresh(_))),
            "[test_finalize_with_wire_gens_empty_wire] All angle args should be newly minted since wire_gens is empty."
        );

        // We should have 3 new generics: a type param, a lifetime param, and a const param.
        assert_eq!(item.op_generics().params.len(), 3);

        info!(
            "[test_finalize_with_wire_gens_empty_wire] path final arguments: {:?}",
            item.path().segments.last().unwrap().arguments
        );
        assert_eq!(item.path().segments.last().unwrap().arguments, PathArguments::None);
    }

    #[test]
    fn test_debug_impl() {
        info!("[test_debug_impl] Starting test...");
        let path: Path = parse_quote!(DebuggableOp<A>);
        let item = OperatorSpecItem::new(path);
        info!("[test_debug_impl] OperatorSpecItem created: {:?}", item);

        let debug_str = format!("{:?}", item);
        assert!(
            debug_str.contains("OperatorSpecItem")
                && debug_str.contains("path")
                && debug_str.contains("op_generics"),
            "[test_debug_impl] Debug representation must include field names and relevant data."
        );
    }

    #[test]
    fn test_accessors() {
        info!("[test_accessors] Starting test...");
        let path: Path = parse_quote!(GetterOp<X>);
        let mut item = OperatorSpecItem::new(path.clone());
        info!(
            "[test_accessors] After creation, path={:?}, op_generics={:?}, final_args={:?}",
            item.path(), item.op_generics(), item.final_args()
        );
        assert_eq!(item.path(), &path);
        assert!(item.op_generics().params.is_empty());
        assert!(item.final_args().is_empty());

        let wire_gens: Generics = parse_quote! { <X, Y> };
        info!(
            "[test_accessors] Calling finalize_with_wire_gens with {:?}",
            wire_gens
        );
        item = item.finalize_with_wire_gens(&wire_gens);

        info!(
            "[test_accessors] After finalize, path={:?}, op_generics={:?}, final_args={:?}",
            item.path(), item.op_generics(), item.final_args()
        );
        // Now ensure we can retrieve the updated fields
        assert_eq!(item.path().segments.last().unwrap().arguments, PathArguments::None);
        assert!(!item.final_args().is_empty());
    }
}
