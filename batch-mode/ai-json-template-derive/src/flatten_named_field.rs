// ---------------- [ File: ai-json-template-derive/src/flatten_named_field.rs ]
crate::ix!();

/// A little helper that collects expansions for one named field.
/// Returns:
///   - `flattened_decls`: lines for the Flat variant (e.g. `#[serde(default)]alpha: AlphaType,`).
///   - `item_init`: how to initialize `item.field` in the final `From<...>` arm.
///   - `just_init`: how to initialize `justification.field_justification`.
///   - `conf_init`: how to initialize `confidence.field_confidence`.
///
/// If `skip_self_just == true`, we skip top-level justification/conf for this field.
/// If `parent_skip_child == true`, we do **not** flatten the child type, so we just do a direct assignment.
pub fn flatten_named_field(
    field_ident: &Ident,
    field_ty: &syn::Type,
    skip_self_just: bool,
    parent_skip_child: bool,
) -> (
    Vec<TokenStream2>, // flattened_decls
    TokenStream2,      // item_init
    TokenStream2,      // just_init
    TokenStream2       // conf_init
)
{
    let mut flattened_decls = Vec::new();

    // Flatten the type
    let flattened_type = match compute_flat_type_for_stamped(field_ty, parent_skip_child, field_ty.span()) {
        Ok(ts) => ts,
        Err(e) => {
            return (vec![e.to_compile_error()], quote!(), quote!(), quote!());
        }
    };

    // 1) Declare the flattened field
    flattened_decls.push(quote! {
        #[serde(default)]
        #field_ident:#flattened_type,
    });

    // 2) item init
    let item_init = if parent_skip_child {
        quote! { #field_ident }
    } else {
        quote! { ::core::convert::From::from(#field_ident) }
    };

    // 3) optional justification/conf
    if !skip_self_just {
        let j_id = Ident::new(
            &format!("{}_justification", field_ident),
            field_ident.span()
        );
        let c_id = Ident::new(
            &format!("{}_confidence", field_ident),
            field_ident.span()
        );

        flattened_decls.push(quote! {
            #[serde(default)]
            #j_id:String,
            #[serde(default)]
            #c_id:f32,
        });

        let just_init = if parent_skip_child {
            quote! { #j_id:#j_id }
        } else {
            let child_just = child_ty_to_just(field_ty);
            quote! {
                #j_id:#child_just {
                    detail_justification:#j_id,
                    ..::core::default::Default::default()
                }
            }
        };
        let conf_init = if parent_skip_child {
            quote! { #c_id:#c_id }
        } else {
            let child_conf = child_ty_to_conf(field_ty);
            quote! {
                #c_id:#child_conf {
                    detail_confidence:#c_id,
                    ..::core::default::Default::default()
                }
            }
        };

        (flattened_decls, item_init, just_init, conf_init)
    } else {
        (flattened_decls, item_init, quote!(), quote!())
    }
}

#[cfg(test)]
mod test_flatten_named_field_exhaustive {
    use super::*;

    /// A helper to remove all whitespace from `TokenStream::to_string()`,
    /// making it easier to test for a substring match regardless of spacing.
    fn ts_to_minimal_string(ts: &TokenStream2) -> String {
        ts.to_string().replace(' ', "").replace('\n', "")
    }

    #[traced_test]
    fn test_skip_self_just_false_parent_skip_child_false_builtin_scalar() {
        trace!("Testing flatten_named_field with skip_self_just=false, parent_skip_child=false, builtin scalar type");
        let field_ident = syn::Ident::new("my_field", proc_macro2::Span::call_site());
        let field_ty: Type = parse_quote! { u8 };

        let (decls, item_init, just_init, conf_init) = flatten_named_field(
            &field_ident,
            &field_ty,
            /* skip_self_just = */ false,
            /* parent_skip_child = */ false,
        );

        debug!("Flattened declarations: {:?}", decls);
        debug!("item_init: {:?}", item_init.to_string());
        debug!("just_init: {:?}", just_init.to_string());
        debug!("conf_init: {:?}", conf_init.to_string());

        // We expect two sets of declarations: the main field, plus justification/conf fields.
        let decls_str: Vec<String> = decls.iter().map(|ts| ts.to_string()).collect();
        pretty_assert_eq!(
            decls_str.len(),
            2,
            "We expect 2 sets of field declarations here (the main field + justification/conf)."
        );

        // Check that the main field is defined as something like "my_field: u8,"
        assert!(
            decls_str[0].contains("my_field : u8"),
            "Should define the main u8 field"
        );
        // Check that justification/conf is defined
        assert!(
            decls_str[1].contains("my_field_justification : String")
                && decls_str[1].contains("my_field_confidence : f32"),
            "Should define justification/confidence fields"
        );

        // item_init should do a From::from(...) call. We ignore whitespace for matching.
        let item_init_str = ts_to_minimal_string(&item_init);
        assert!(
            item_init_str.contains("::core::convert::From::from(my_field)"),
            "Expected item_init to contain ::core::convert::From::from(my_field). Got: {item_init_str}"
        );

        // just_init => we store a 'detail_justification' in e.g. "u8Justification"
        let just_init_str = ts_to_minimal_string(&just_init);
        assert!(
            just_init_str.contains("my_field_justification:u8Justification"),
            "Expected just_init to reference u8Justification"
        );

        // conf_init => we store 'detail_confidence' in e.g. "u8Confidence"
        let conf_init_str = ts_to_minimal_string(&conf_init);
        assert!(
            conf_init_str.contains("my_field_confidence:u8Confidence"),
            "Expected conf_init to reference u8Confidence"
        );
    }

    #[traced_test]
    fn test_skip_self_just_false_parent_skip_child_true_custom_type() {
        trace!("Testing flatten_named_field with skip_self_just=false, parent_skip_child=true, custom type");
        let field_ident = syn::Ident::new("some_custom", proc_macro2::Span::call_site());
        let field_ty: Type = parse_quote! { SomeCustomType };

        let (decls, item_init, just_init, conf_init) = flatten_named_field(
            &field_ident,
            &field_ty,
            /* skip_self_just = */ false,
            /* parent_skip_child = */ true,
        );

        debug!("Flattened declarations: {:?}", decls);
        debug!("item_init: {:?}", item_init.to_string());
        debug!("just_init: {:?}", just_init.to_string());
        debug!("conf_init: {:?}", conf_init.to_string());

        // Because parent_skip_child=true => direct assignment for item_init
        let item_init_str = ts_to_minimal_string(&item_init);
        assert_eq!(
            item_init_str, "some_custom",
            "Direct assignment expected for item_init"
        );

        // Decls must contain the main field plus justification/conf
        let decls_str: Vec<String> = decls.iter().map(|ts| ts.to_string()).collect();
        assert_eq!(
            decls_str.len(),
            2,
            "Expect main field + justification/conf for skip_self_just=false"
        );
        assert!(
            decls_str[0].contains("some_custom : SomeCustomType"),
            "Main field should be 'some_custom: SomeCustomType'"
        );
        assert!(
            decls_str[1].contains("some_custom_justification : String")
                && decls_str[1].contains("some_custom_confidence : f32"),
            "Should define justification/conf fields for 'some_custom'"
        );

        // just_init => "some_custom_justification: some_custom_justification"
        let just_init_str = ts_to_minimal_string(&just_init);
        assert!(
            just_init_str.contains("some_custom_justification:some_custom_justification"),
            "We expect direct assignment for just_init"
        );

        // conf_init => "some_custom_confidence: some_custom_confidence"
        let conf_init_str = ts_to_minimal_string(&conf_init);
        assert!(
            conf_init_str.contains("some_custom_confidence:some_custom_confidence"),
            "We expect direct assignment for conf_init"
        );
    }

    #[traced_test]
    fn test_skip_self_just_true_parent_skip_child_false_string_type() {
        trace!("Testing flatten_named_field with skip_self_just=true, parent_skip_child=false, String type");
        let field_ident = syn::Ident::new("label", proc_macro2::Span::call_site());
        let field_ty: Type = parse_quote! { String };

        let (decls, item_init, just_init, conf_init) = flatten_named_field(
            &field_ident,
            &field_ty,
            /* skip_self_just = */ true,
            /* parent_skip_child = */ false,
        );

        debug!("Flattened declarations: {:?}", decls);
        debug!("item_init: {:?}", item_init.to_string());
        debug!("just_init: {:?}", just_init.to_string());
        debug!("conf_init: {:?}", conf_init.to_string());

        // skip_self_just=true => no top-level justification/conf fields
        // parent_skip_child=false => we do a From::from(...) for the item
        let decls_str: Vec<String> = decls.iter().map(|ts| ts.to_string()).collect();
        assert_eq!(
            decls_str.len(),
            1,
            "Only the main field should be declared (no justification/conf)."
        );
        assert!(
            decls_str[0].contains("label : String"),
            "Should define 'label: String'"
        );

        // item_init => ::core::convert::From::from(label)
        let item_init_str = ts_to_minimal_string(&item_init);
        assert!(
            item_init_str.contains("::core::convert::From::from(label)"),
            "Expected item_init to contain From::from(label). Got: {item_init_str}"
        );

        // No justification or confidence expansions
        assert!(just_init.is_empty(), "Should be empty if skip_self_just=true");
        assert!(conf_init.is_empty(), "Should be empty if skip_self_just=true");
    }

    #[traced_test]
    fn test_skip_self_just_true_parent_skip_child_true_numeric_type() {
        trace!("Testing flatten_named_field with skip_self_just=true, parent_skip_child=true, numeric type");
        let field_ident = syn::Ident::new("amount", proc_macro2::Span::call_site());
        let field_ty: Type = parse_quote! { i32 };

        let (decls, item_init, just_init, conf_init) = flatten_named_field(
            &field_ident,
            &field_ty,
            /* skip_self_just = */ true,
            /* parent_skip_child = */ true,
        );

        debug!("Flattened declarations: {:?}", decls);
        debug!("item_init: {:?}", item_init.to_string());
        debug!("just_init: {:?}", just_init.to_string());
        debug!("conf_init: {:?}", conf_init.to_string());

        // skip_self_just=true => no justification/conf
        // parent_skip_child=true => direct assignment
        let decls_str: Vec<String> = decls.iter().map(|ts| ts.to_string()).collect();
        assert_eq!(decls_str.len(), 1, "Only the main field is declared");
        assert!(
            decls_str[0].contains("amount : i32"),
            "Should define 'amount: i32'"
        );

        let item_init_str = ts_to_minimal_string(&item_init);
        assert_eq!(
            item_init_str, "amount",
            "Direct assignment expected for item_init"
        );
        assert!(just_init.is_empty(), "No justification expansions");
        assert!(conf_init.is_empty(), "No confidence expansions");
    }

    #[traced_test]
    fn test_error_scenario_child_type_unflattenable() {
        trace!("Testing flatten_named_field with unflattenable child type => sees how the code behaves with 'BadType'");
        let field_ident = syn::Ident::new("bad_field", proc_macro2::Span::call_site());
        let field_ty: Type = parse_quote! { BadType };

        let (decls, item_init, just_init, conf_init) = flatten_named_field(
            &field_ident,
            &field_ty,
            /* skip_self_just = */ false,
            /* parent_skip_child = */ false,
        );

        debug!("Flattened declarations: {:?}", decls);
        debug!("item_init: {:?}", item_init.to_string());
        debug!("just_init: {:?}", just_init.to_string());
        debug!("conf_init: {:?}", conf_init.to_string());

        // The current code transforms "BadType" into something like "FlatJustifiedBadType"
        // (and also adds top-level justification/conf). It does *not* produce a compile_error in
        // compute_flat_type_for_stamped. So let's verify that "FlatJustifiedBadType" is indeed declared.
        let decls_str: Vec<String> = decls.iter().map(|ts| ts.to_string()).collect();
        assert_eq!(
            decls_str.len(),
            2,
            "We expect main field + justification/conf fields"
        );

        // 1) Check the main field references "FlatJustifiedBadType"
        assert!(
            decls_str[0].contains("bad_field : :: std :: option :: Option <")
                == false,
            "Should not interpret 'BadType' as an Option. Actual line: {}",
            decls_str[0]
        );
        assert!(
            decls_str[0].contains("bad_field : FlatJustifiedBadType")
                || decls_str[0].contains("bad_field : ::core::something::FlatJustifiedBadType"),
            "Should define 'bad_field : FlatJustifiedBadType'. Actual: {}",
            decls_str[0]
        );

        // 2) Check the justification/conf fields
        assert!(
            decls_str[1].contains("bad_field_justification : String")
                && decls_str[1].contains("bad_field_confidence : f32"),
            "Should define justification/conf fields for 'bad_field'"
        );

        // item_init => ":: core :: convert :: From :: from (bad_field)"
        let item_init_str = ts_to_minimal_string(&item_init);
        assert!(
            item_init_str.contains("::core::convert::From::from(bad_field)"),
            "Expected item_init to contain ::core::convert::From::from(bad_field)"
        );

        // just_init => references "BadTypeJustification"
        let just_init_str = ts_to_minimal_string(&just_init);
        assert!(
            just_init_str.contains("BadTypeJustification"),
            "Expected just_init to reference 'BadTypeJustification'."
        );

        // conf_init => references "BadTypeConfidence"
        let conf_init_str = ts_to_minimal_string(&conf_init);
        assert!(
            conf_init_str.contains("BadTypeConfidence"),
            "Expected conf_init to reference 'BadTypeConfidence'."
        );
    }

    #[traced_test]
    fn test_skip_self_just_false_parent_skip_child_false_option_type() {
        trace!("Testing flatten_named_field with skip_self_just=false, parent_skip_child=false, Option<T> type");
        let field_ident = syn::Ident::new("maybe_val", proc_macro2::Span::call_site());
        let field_ty: Type = parse_quote! { Option<MyStruct> };

        let (decls, item_init, just_init, conf_init) = flatten_named_field(
            &field_ident,
            &field_ty,
            /* skip_self_just = */ false,
            /* parent_skip_child = */ false,
        );

        debug!("Flattened declarations: {:?}", decls);
        debug!("item_init: {:?}", item_init.to_string());
        debug!("just_init: {:?}", just_init.to_string());
        debug!("conf_init: {:?}", conf_init.to_string());

        // Because parent_skip_child=false => we flatten Option<MyStruct> to Option<FlatJustifiedMyStruct>
        // and produce top-level justification/conf fields
        let decls_str: Vec<String> = decls.iter().map(|ts| ts.to_string()).collect();
        assert_eq!(
            decls_str.len(),
            2,
            "We expect main field + justification/conf fields"
        );

        // The main field: "maybe_val : :: std :: option :: Option < FlatJustifiedMyStruct >"
        assert!(
            decls_str[0].contains("maybe_val : :: std :: option :: Option < FlatJustifiedMyStruct >"),
            "Should define 'maybe_val : Option<FlatJustifiedMyStruct>'. Actual: {}",
            decls_str[0]
        );

        // justification/conf fields
        assert!(
            decls_str[1].contains("maybe_val_justification : String")
                && decls_str[1].contains("maybe_val_confidence : f32"),
            "Should define justification/conf fields for 'maybe_val'"
        );

        let item_init_str = ts_to_minimal_string(&item_init);
        assert!(
            item_init_str.contains("::core::convert::From::from(maybe_val)"),
            "Expected item_init to contain From::from(maybe_val). Got: {item_init_str}"
        );

        // just_init => "maybe_val_justification : OptionJustification { ... }"
        let just_init_str = ts_to_minimal_string(&just_init);
        assert!(
            just_init_str.contains("maybe_val_justification:OptionJustification"),
            "Expected just_init to wrap an Option justification"
        );

        // conf_init => "maybe_val_confidence : OptionConfidence { ... }"
        let conf_init_str = ts_to_minimal_string(&conf_init);
        assert!(
            conf_init_str.contains("maybe_val_confidence:OptionConfidence"),
            "Expected conf_init to wrap an Option confidence"
        );
    }
}
