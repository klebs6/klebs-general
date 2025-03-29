// ---------------- [ File: src/build_bridging_impls.rs ]
crate::ix!();

/// Builds bridging code (`impl OperatorInterface<WireEnum> for FooOp<...>`) for each operator item.
/// Finalizes each `OperatorSpecItem` with the wire generics, then merges them and reconstructs
/// angle brackets from `final_args()` instead of using the item’s `.path()` verbatim.
///
/// This approach ensures that:
///   - `op_item.path().segments.last().unwrap().ident` remains angle‐free after finalize.
///   - In the final “impl <...> for FooOp<...>” snippet, we reconstruct <Z> or <OpTy0, OPC1> from
///     the item’s `final_args()`.
///
/// # Parameters
/// - `enum_ident`: name of your wire enum (e.g. `MyWireIO`)
/// - `op_items`: operator spec items (their `.path` is angle‐free after finalize)
/// - `wire_generics`: generics of the wire type
///
/// # Returns
/// A list of bridging `TokenStream` blocks (one per operator).
pub fn build_bridging_impls(
    enum_ident: &syn::Ident,
    op_items: &mut [OperatorSpecItem],
    wire_generics: &syn::Generics,
) -> Vec<proc_macro2::TokenStream> {
    info!("build_bridging_impls: START");
    info!("  enum_ident = {}", enum_ident);
    info!("  wire_generics = {:?}", wire_generics);
    info!("  op_items initial = {:?}", op_items);

    // Create a map from operator base name => signature ident
    let sig_map = build_operator_signature_map(op_items);
    info!("[build_bridging_impls] sig_map constructed = {:?}", sig_map);

    let mut out = Vec::new();

    for (i, op_item) in op_items.iter_mut().enumerate() {
        info!("[build_bridging_impls] finalizing op_item[{}] => {:?}", i, op_item);

        // Step 1) finalize the operator item with the wire generics => minted vs. reused
        let finalized = op_item.clone().finalize_with_wire_gens(wire_generics);
        info!("[build_bridging_impls] finalized op_item[{}] => {:?}", i, finalized);
        *op_item = finalized;

        // Step 2) build bridging for the single operator
        let bridging = build_single_operator_impl(enum_ident, op_item, &sig_map, wire_generics);
        info!("[build_bridging_impls] bridging[{}] = {:?}", i, bridging.to_string());

        out.push(bridging);
        info!("[build_bridging_impls] out len = {}", out.len());
    }

    info!("[build_bridging_impls] FINISH, returning out of length = {}", out.len());
    out
}

#[cfg(test)]
mod test_build_bridging_impls {
    use super::*;

    #[test]
    fn test_build_bridging_impls_basic() {
        info!("test_build_bridging_impls_basic: START");
        let wire_gens: Generics = parse_quote!{ <Z:SomeTrait> };
        let mut items = vec![
            OperatorSpecItem::new(parse_quote!(AddOp<Z>)),
            OperatorSpecItem::new(parse_quote!(ConstantOp<Foo>)),
        ];
        let enum_ident = syn::Ident::new("MyWireIO", proc_macro2::Span::call_site());
        info!("test_build_bridging_impls_basic: calling build_bridging_impls(...)");
        let out = build_bridging_impls(&enum_ident, &mut items, &wire_gens);
        info!("test_build_bridging_impls_basic: out len = {}", out.len());
        assert_eq!(out.len(), 2, "Expected 2 bridging expansions");

        let s0 = out[0].to_string();
        info!("test_build_bridging_impls_basic: out[0] = {}", s0);
        assert!(
            s0.contains("impl < Z : SomeTrait > hydro2_operator :: OperatorInterface < MyWireIO < Z > > for AddOp < Z >"),
            "Got: {}", s0
        );
    }

    #[test]
    fn test_no_operator_items() {
        info!("test_no_operator_items: START");
        let wire_gens: Generics = parse_quote!{ <T:SomeTrait> };
        let mut items = vec![];
        let enum_ident = syn::Ident::new("EmptyWireIO", proc_macro2::Span::call_site());
        info!("test_no_operator_items: calling build_bridging_impls(...)");
        let out = build_bridging_impls(&enum_ident, &mut items, &wire_gens);
        info!("test_no_operator_items: out len = {}", out.len());
        assert_eq!(out.len(), 0, "Expected no bridging expansions for empty op_items");
    }

    #[test]
    fn test_single_op_no_generics() {
        info!("test_single_op_no_generics: START");
        let wire_gens: Generics = parse_quote!{ <X:SomeTrait> };
        let mut items = vec![
            OperatorSpecItem::new(parse_quote!(AddOp)),
        ];
        let enum_ident = syn::Ident::new("EmptyWireIO", proc_macro2::Span::call_site());
        info!("test_single_op_no_generics: calling build_bridging_impls(...)");
        let out = build_bridging_impls(&enum_ident, &mut items, &wire_gens);
        info!("test_single_op_no_generics: out len = {}", out.len());
        assert_eq!(out.len(), 1);
        let s = out[0].to_string();
        info!("test_single_op_no_generics: out[0] = {}", s);
        assert!(
            s.contains("impl < X : SomeTrait > hydro2_operator :: OperatorInterface < EmptyWireIO < X > > for AddOp"),
            "Got: {}", s
        );
    }

    #[test]
    fn test_multiple_ops() {
        info!("test_multiple_ops: START");
        let wire_gens: Generics = parse_quote!{ <Z:SomeTrait> };
        let mut items = vec![
            OperatorSpecItem::new(parse_quote!(AddOp<Z>)),
            OperatorSpecItem::new(parse_quote!(ConstantOp<Foo>)),
        ];
        let enum_ident = syn::Ident::new("MyWireIO", proc_macro2::Span::call_site());
        info!("test_multiple_ops: calling build_bridging_impls(...)");
        let out = build_bridging_impls(&enum_ident, &mut items, &wire_gens);
        info!("test_multiple_ops: out len = {}", out.len());
        assert_eq!(out.len(), 2, "Expected 2 bridging expansions");

        let s0 = out[0].to_string();
        info!("test_multiple_ops: out[0] = {}", s0);
        assert!(
            s0.contains("impl < Z : SomeTrait > hydro2_operator :: OperatorInterface < MyWireIO < Z > > for AddOp < Z >"),
            "Got: {}", s0
        );

        let s1 = out[1].to_string();
        info!("test_multiple_ops: out[1] = {}", s1);
        assert!(
            s1.contains("impl < Z : SomeTrait , OpTy0 >"),
            "Expected a fresh param minted. Got: {}", s1
        );
        assert!(
            s1.contains("ConstantOp < OpTy0 >"),
            "Got bridging snippet: {}", s1
        );
    }

    #[test]
    fn test_reuse_wire_param() {
        info!("test_reuse_wire_param: START");

        // The user wrote operator "ConstantOp<Z>", and wire also has <Z:AnotherTrait>.
        // So we expect bridging expansions that unify param Z (not create a new param).
        let wire_gens: Generics = parse_quote!{ <Z:AnotherTrait> };
        info!("test_reuse_wire_param: wire_gens = {:?}", wire_gens);

        let mut items = vec![
            OperatorSpecItem::new(parse_quote!(ConstantOp<Z>)),
        ];
        info!("test_reuse_wire_param: items = {:?}", items);

        let enum_ident = syn::Ident::new("MyTestWireIO", proc_macro2::Span::call_site());
        info!("test_reuse_wire_param: enum_ident = {:?}", enum_ident);

        info!("test_reuse_wire_param: calling build_bridging_impls(...)");
        let out = build_bridging_impls(&enum_ident, &mut items, &wire_gens);
        info!("test_reuse_wire_param: out = {:#?}", out);

        assert_eq!(out.len(), 1, "Expected 1 bridging expansion");

        let s = out[0].to_string();
        info!("test_reuse_wire_param: bridging snippet = {}", s);

        // We should see "impl < Z : AnotherTrait > ... for ConstantOp<Z>"
        assert!(
            s.contains("impl < Z : AnotherTrait > hydro2_operator :: OperatorInterface < MyTestWireIO < Z > > for ConstantOp < Z >"),
            "Got bridging snippet: {}", s
        );
    }

    #[test]
    fn test_fresh_param() {
        info!("test_fresh_param: START");
        // The user wrote operator "ConstantOp<u32>", but wire is <Z: AnotherTrait>.
        // So we must create e.g. "OpTy0" for "u32" => "ConstantOp<OpTy0>"
        let wire_gens: Generics = parse_quote!{ <Z:AnotherTrait> };
        let mut items = vec![
            OperatorSpecItem::new(parse_quote!(ConstantOp<u32>)),
        ];
        let enum_ident = syn::Ident::new("SomeWireIO", proc_macro2::Span::call_site());
        info!("test_fresh_param: calling build_bridging_impls(...)");
        let out = build_bridging_impls(&enum_ident, &mut items, &wire_gens);
        info!("test_fresh_param: out = {:?}", out);

        assert_eq!(out.len(), 1);
        let s = out[0].to_string();
        info!("test_fresh_param: bridging snippet = {}", s);
        assert!(
            s.contains("impl < Z : AnotherTrait , OpTy0 >"),
            "fresh param minted. Got: {}", s
        );
        assert!(
            s.contains("ConstantOp < OpTy0 >"),
            "operator uses fresh param. Got: {}", s
        );
    }

    #[test]
    fn test_bridging_impls_does_not_mutate_wire_gens() {
        info!("test_bridging_impls_does_not_mutate_wire_gens: START");
        // Ensure we do not accidentally mutate the original wire generics
        // by verifying the bridging function does not cause side effects on wire_gens.
        let wire_gens: Generics = parse_quote!{ <A: Clone, B: Debug> };
        let wire_gens_clone = wire_gens.clone();

        let mut items = vec![
            OperatorSpecItem::new(parse_quote!(MergeOp<A>)),
            OperatorSpecItem::new(parse_quote!(SplitterOp<u32>)),
        ];
        let enum_ident = syn::Ident::new("AnotherWireIO", proc_macro2::Span::call_site());
        info!("test_bridging_impls_does_not_mutate_wire_gens: calling build_bridging_impls(...)");
        let out = build_bridging_impls(&enum_ident, &mut items, &wire_gens);
        info!("test_bridging_impls_does_not_mutate_wire_gens: out = {:?}", out);

        // bridging expansions might produce new param for u32 => OpTy0
        // But wire_gens should remain <A: Clone, B: Debug>
        assert_eq!(wire_gens, wire_gens_clone, "Expected wire_gens to remain unchanged");
        assert!(!out.is_empty());
    }
}
