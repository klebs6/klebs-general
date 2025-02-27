// ---------------- [ File: src/build_network_io_enum.rs ]
crate::ix!();

/// This version of `build_network_io_enum` ensures that when the *original* operator type
/// has generic parameters (e.g. `ConstantOp<T>`), the corresponding *IO type* (`ConstantOpIO`)
/// is also instantiated with those same parameters (`ConstantOpIO<T>`) in the final enum
/// variant.
///
/// For example:
///
/// ```ignore
/// // If user wrote op="ConstantOp<T>" ...
/// // We'll generate an enum variant:
/// //   ConstantOpIO(ConstantOpIO<T>)
/// // rather than (ConstantOpIO)
/// ```
///
/// # Explanation
///
/// 1. For each operator item:
///    - We determine the variant name by appending `IO` to the last segment of the
///      operator path (e.g. `AddOp` -> `AddOpIO`).
///    - We also build the *inner type* by renaming the last segment to `SomethingIO` and
///      reusing any generics from `op_item.final_args()`.
/// 2. That final_args array is turned into `<T, OpTy0, ...>` (if non-empty) via
///    `build_operator_type_args`.
/// 3. This way, if the user originally wrote `ConstantOp<T>`, the final enum variant
///    becomes `ConstantOpIO(ConstantOpIO<T>)`.
///
/// 4. We keep a `#[default] None` variant for convenience.
///
/// 5. The caller also passes the wire struct’s generics in textual form (`impl_generics`,
///    `ty_generics`, `where_clause`) to place on the enum itself, so it can be something like:
///
/// ```ignore
/// pub enum MyWireIO<T> where T: Debug {
///     #[default] None,
///     ConstantOpIO(ConstantOpIO<T>),
///     ...
/// }
/// ```
///
/// 6. The user’s code can then construct e.g.:
///   `MyWireIO::<SomeType>::ConstantOpIO(ConstantOpIO::<SomeType>::new(...))`
///
/// # Important
///
/// The existing error `error[E0107]: missing generics for enum '...'` typically means
/// that the user’s macro expansions are referencing an IO type (e.g. `ConstantOpIO`)
/// that also expects generics in its own definition. For instance, if `ConstantOpIO` is
/// defined as `enum ConstantOpIO<T> { ... }` in `hydro2_basic_operators`,
/// then you must supply `<T>` when constructing it. Our final code below handles that by
/// passing the same angle args `<T>` we extracted from the user’s input operator spec.
pub fn build_network_io_enum(
    enum_ident:    &syn::Ident,
    op_items:      &[OperatorSpecItem],
    impl_generics: &proc_macro2::TokenStream,
    ty_generics:   &proc_macro2::TokenStream,
    where_clause:  &proc_macro2::TokenStream
) -> (proc_macro2::TokenStream,Vec<syn::Ident>)
{
    info!("[build_network_io_enum] START");
    info!("  enum_ident = {}", enum_ident);
    info!("  impl_generics = {}", impl_generics.to_string());
    info!("  ty_generics   = {}", ty_generics.to_string());
    info!("  where_clause  = {}", where_clause.to_string());
    info!("  op_items len  = {}", op_items.len());

    let mut variants       = Vec::new();
    let mut variant_idents = Vec::new();

    for (i, op_item) in op_items.iter().enumerate() {
        info!("[build_network_io_enum] processing op_item[{}] => {:?}", i, op_item);

        // Ensure the operator path has at least one segment
        let last_seg = match op_item.path().segments.last() {
            Some(seg) => seg,
            None => {
                let msg = format!(
                    "Path {} had no segments?",
                    op_item.path().to_token_stream()
                );
                info!("[build_network_io_enum] returning syn::Error => {}", msg);
                return (syn::Error::new(op_item.path().span(), msg).to_compile_error(), vec![]);
            }
        };

        // The enum variant name => "AddOpIO"
        let variant_ident = syn::Ident::new(
            &format!("{}IO", last_seg.ident),
            last_seg.ident.span(),
        );

        // Build a new path for the *IO type* by renaming the last segment from e.g. "AddOp"
        // to "AddOpIO"
        let mut io_path = op_item.path().clone();
        if let Some(io_seg_last) = io_path.segments.last_mut() {
            let old_id = &io_seg_last.ident;
            let new_id = syn::Ident::new(&format!("{}IO", old_id), old_id.span());
            io_seg_last.ident = new_id;
        }

        // Convert op_item.final_args() into angle brackets <...> if non-empty
        let minted_brackets = build_operator_type_args(op_item.final_args());

        info!(
            "[build_network_io_enum] final variant => {}({}{})",
            variant_ident,
            io_path.to_token_stream(),
            minted_brackets.to_string()
        );

        // e.g. AddOpIO(AddOpIO<T>)
        variants.push(quote::quote! {
            #variant_ident ( #io_path #minted_brackets )
        });

        variant_idents.push(variant_ident);
    }

    info!(
        "[build_network_io_enum] constructing final enum definition for '{}'",
        enum_ident
    );

    // Our enum has a default "None" variant plus all the constructed variants.
    let out = quote::quote! {
        #[derive(Default,PartialEq,Eq,Clone,Debug)]
        pub enum #enum_ident #impl_generics #where_clause {
            #[default]
            None,
            #( #variants ),*
        }
    };

    info!("[build_network_io_enum] FINISH, out = {}", out.to_string());
    (out,variant_idents)
}

#[cfg(test)]
mod test_build_network_io_enum {
    use super::*;

    #[test]
    fn test_build_network_io_enum_basics() {
        info!("test_build_network_io_enum_basics: START");
        let op1 = OperatorSpecItem::new(parse_quote!(AddOp));
        let op2 = OperatorSpecItem::new(parse_quote!(ConstantOp<T>));
        let op3 = OperatorSpecItem::new(parse_quote!(BarOp));
        let items = vec![op1, op2, op3];

        info!("test_build_network_io_enum_basics: calling build_network_io_enum(...)");
        let (enum_ts,_) = build_network_io_enum(
            &parse_quote!(MyWireIO),
            &items,
            &quote!{},
            &quote!{},
            &quote!{}
        );
        info!("test_build_network_io_enum_basics: out = {}", enum_ts.to_string());
        let s = enum_ts.to_string();
        assert!(s.contains("enum MyWireIO"));
        assert!(s.contains("AddOpIO (AddOpIO)"));
        assert!(s.contains("ConstantOpIO (ConstantOpIO < T >)"));
        assert!(s.contains("BarOpIO (BarOpIO)"));
    }

    /// Helper to parse an enum from a `TokenStream` string without unwrap/expect.
    fn try_parse_enum(ts: &str) -> Result<syn::ItemEnum, syn::Error> {
        syn::parse_str::<syn::ItemEnum>(ts)
    }

    #[test]
    fn test_build_network_io_enum_no_operator_items() {
        info!("test_no_operator_items: START");
        let items: Vec<OperatorSpecItem> = vec![];
        let (enum_ts,_) = build_network_io_enum(
            &parse_quote!(EmptyWireIO),
            &items,
            &quote!{},
            &quote!{},
            &quote!{}
        );
        let s = enum_ts.to_string();
        info!("test_no_operator_items: out = {}", s);
        assert!(s.contains("enum EmptyWireIO"), "Enum name missing in output");
        assert!(s.contains("None ,"), "Missing default None variant for empty op_items");

        match try_parse_enum(&s) {
            Ok(parsed_enum) => {
                assert_eq!(
                    parsed_enum.variants.len(),
                    1,
                    "Expected exactly one variant (None) in empty operator enum"
                );
            }
            Err(e) => panic!("Failed to parse back into syn::ItemEnum: {:?}", e),
        }
    }

    #[test]
    fn test_single_operator_item() {
        info!("test_single_operator_item: START");
        let op = OperatorSpecItem::new(parse_quote!(AddOp));
        let items = vec![op];
        let (enum_ts,_) = build_network_io_enum(
            &parse_quote!(MyWireIO),
            &items,
            &quote!{},
            &quote!{},
            &quote!{}
        );
        let s = enum_ts.to_string();
        info!("test_single_operator_item: out = {}", s);
        assert!(s.contains("enum MyWireIO"), "Enum name missing in output");
        assert!(s.contains("AddOpIO (AddOpIO)"), "Missing variant AddOpIO(AddOp)");
        assert!(s.contains("# [default]"), "Missing #[default] on None variant");

        match try_parse_enum(&s) {
            Ok(parsed_enum) => {
                assert_eq!(parsed_enum.variants.len(), 2, "Should have 2 variants");
            }
            Err(e) => panic!("Could not parse the generated enum: {:?}", e),
        }
    }

    #[test]
    fn test_multiple_operator_items() {
        info!("test_multiple_operator_items: START");
        let op1 = OperatorSpecItem::new(parse_quote!(AddOp));
        let op2 = OperatorSpecItem::new(parse_quote!(MulOp));
        let op3 = OperatorSpecItem::new(parse_quote!(ConstantOp));
        let items = vec![op1, op2, op3];
        let (enum_ts,_) = build_network_io_enum(
            &parse_quote!(TestWireIO),
            &items,
            &quote!{},
            &quote!{},
            &quote!{}
        );
        let s = enum_ts.to_string();
        info!("test_multiple_operator_items: out = {}", s);
        assert!(s.contains("enum TestWireIO"));
        assert!(s.contains("None ,"),       "No default None variant found");
        assert!(s.contains("AddOpIO (AddOpIO)"));
        assert!(s.contains("MulOpIO (MulOpIO)"));
        assert!(s.contains("ConstantOpIO (ConstantOpIO)"));

        match try_parse_enum(&s) {
            Ok(parsed_enum) => {
                assert_eq!(parsed_enum.variants.len(), 4);
            }
            Err(e) => panic!("Failed parsing: {:?}", e),
        }
    }

    #[test]
    fn test_nested_path_operator_item() {
        info!("test_nested_path_operator_item: START");
        let op = OperatorSpecItem::new(parse_quote!(my_crate::ops::AddOp));
        let items = vec![op];
        let (enum_ts,_) = build_network_io_enum(
            &parse_quote!(NestedWireIO),
            &items,
            &quote!{},
            &quote!{},
            &quote!{}
        );
        let s = enum_ts.to_string();
        info!("test_nested_path_operator_item: out = {}", s);
        assert!(s.contains("enum NestedWireIO"), "Missing enum declaration");
        assert!(s.contains("AddOpIO (my_crate :: ops :: AddOpIO)"), "Nested path variant mismatch");

        match try_parse_enum(&s) {
            Ok(parsed_enum) => {
                assert_eq!(parsed_enum.variants.len(), 2);
                let var_names: Vec<_> = parsed_enum
                    .variants
                    .iter()
                    .map(|v| v.ident.to_string())
                    .collect();
                assert!(var_names.contains(&"AddOpIO".to_string()));
            }
            Err(e) => panic!("Failed parsing nested-path enum: {:?}", e),
        }
    }

    #[test]
    fn test_enum_with_impl_ty_generics() {
        info!("test_enum_with_impl_ty_generics: START");
        // Suppose the wire type is MyWireIO<T>
        // We pass in <T: Clone> and <T>, plus a where clause
        let items = vec![
            OperatorSpecItem::new(parse_quote!(FooOp)),
            OperatorSpecItem::new(parse_quote!(BarOp))
        ];
        let (enum_ts,_) = build_network_io_enum(
            &parse_quote!(MyWireIO),
            &items,
            &quote! { <T: Clone> },
            &quote! { <T> },
            &quote! { where T: std::fmt::Debug }
        );

        let s = enum_ts.to_string();
        info!("test_enum_with_impl_ty_generics: out = {}", s);
        assert!(s.contains("enum MyWireIO < T : Clone > where T : std :: fmt :: Debug"));
        assert!(s.contains("None ,"), "Missing #[default] None variant");
        assert!(s.contains("FooOpIO (FooOpIO)"));
        assert!(s.contains("BarOpIO (BarOpIO)"));

        match try_parse_enum(&s) {
            Ok(parsed_enum) => {
                // We want 3 variants: None, FooOpIO, BarOpIO
                assert_eq!(parsed_enum.variants.len(), 3);
            }
            Err(e) => panic!("Failed to parse generics-based enum: {:?}", e),
        }
    }

    #[test]
    fn test_enum_where_clause_only() {
        info!("test_enum_where_clause_only: START");
        // No explicit impl generics or ty generics, but has a where clause
        let items = vec![OperatorSpecItem::new(parse_quote!(ExampleOp))];
        let (enum_ts,_) = build_network_io_enum(
            &parse_quote!(AnotherWireIO),
            &items,
            &quote!{},
            &quote!{},
            &quote!{ where i32: Send }
        );

        let s = enum_ts.to_string();
        info!("test_enum_where_clause_only: out = {}", s);
        assert!(s.contains("where i32 : Send"), "Where clause not found in output");

        match try_parse_enum(&s) {
            Ok(parsed_enum) => {
                // We want 2 variants: None and ExampleOpIO
                assert_eq!(parsed_enum.variants.len(), 2);
                let var_names: Vec<_> = parsed_enum
                    .variants
                    .iter()
                    .map(|v| v.ident.to_string())
                    .collect();
                assert!(var_names.contains(&"ExampleOpIO".to_string()), "Missing ExampleOpIO variant");
            }
            Err(e) => {
                panic!("Failed to parse AnotherWireIO: {:?}", e);
            }
        }
    }

    #[test]
    fn test_error_no_segments_in_path() {
        info!("test_error_no_segments_in_path: START");

        // Instead of parse_quote!(), manually build an empty path:
        // leading_colon = None, and an empty Punctuated for segments.
        let path_without_segments = syn::Path {
            leading_colon: None,
            segments: syn::punctuated::Punctuated::new(),
        };

        let op = OperatorSpecItem::new(path_without_segments);
        let items = vec![op];

        let (enum_ts,_) = build_network_io_enum(
            &parse_quote!(BrokenWireIO),
            &items,
            &quote!{},
            &quote!{},
            &quote!{}
        );
        let s = enum_ts.to_string();
        info!("test_error_no_segments_in_path: out = {}", s);

        // Check that our emitted TokenStream contains the error message "had no segments?"
        assert!(
            s.contains("had no segments?"),
            "Expected error message in output token stream"
        );

        // Confirm that parsing it back as an enum fails:
        match try_parse_enum(&s) {
            Ok(_) => panic!("Expected parse to fail due to no path segments, but it succeeded!"),
            Err(_) => {
                info!("test_error_no_segments_in_path: parse failed as expected");
            }
        }
    }
}
