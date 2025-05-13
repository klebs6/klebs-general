// ---------------- [ File: ai-json-template-derive/src/flatten_unnamed_field.rs ]
crate::ix!();

#[tracing::instrument(level="trace", skip_all)]
pub fn flatten_unnamed_field(
    field_ident:       &syn::Ident,
    field_ty:          &syn::Type,
    skip_self_just:    bool,
    parent_skip_child: bool
) -> (
    Vec<proc_macro2::TokenStream>, // flattened_decls
    proc_macro2::TokenStream,      // item_init
    proc_macro2::TokenStream,      // just_init
    proc_macro2::TokenStream       // conf_init
)
{
    trace!(
        "flatten_unnamed_field: field='{}', skip_self_just={}, parent_skip_child={}",
        field_ident, skip_self_just, parent_skip_child
    );

    let mut flattened_decls = Vec::new();
    let flattened_type = match compute_flat_type_for_stamped(field_ty, parent_skip_child, field_ty.span()) {
        Ok(ts) => ts,
        Err(e) => {
            return (
                vec![e.to_compile_error()],
                quote::quote!(),
                quote::quote!(),
                quote::quote!()
            );
        }
    };

    // The core field
    flattened_decls.push(quote::quote! {
        #[serde(default)]
        #field_ident:#flattened_type,
    });

    // If parent_skip_child=false => we call From::from(...), else use the field as-is
    let item_init = if parent_skip_child {
        quote::quote!(#field_ident)
    } else {
        quote::quote!(::core::convert::From::from(#field_ident))
    };

    // If skip_self_just=false => add field_justification + field_confidence
    if !skip_self_just {
        let j_id = syn::Ident::new(&format!("{}_justification", field_ident), field_ident.span());
        let c_id = syn::Ident::new(&format!("{}_confidence",    field_ident), field_ident.span());

        flattened_decls.push(quote::quote! {
            #[serde(default)]
            #j_id:String,
            #[serde(default)]
            #c_id:f32,
        });

        let just_init = if parent_skip_child {
            quote::quote!(#j_id:#j_id)
        } else {
            let child_just = child_ty_to_just(field_ty);
            quote::quote! {
                #j_id:#child_just {
                    detail_justification:#j_id,
                    ..::core::default::Default::default()
                }
            }
        };

        let conf_init = if parent_skip_child {
            quote::quote!(#c_id:#c_id)
        } else {
            let child_conf = child_ty_to_conf(field_ty);
            quote::quote! {
                #c_id:#child_conf {
                    detail_confidence:#c_id,
                    ..::core::default::Default::default()
                }
            }
        };

        return (flattened_decls, item_init, just_init, conf_init);
    }

    // If skip_self_just=true => no justification/conf
    (flattened_decls, item_init, quote::quote!(), quote::quote!())
}

#[cfg(test)]
mod test_flatten_unnamed_field_exhaustive {
    use super::*;

    /// Exhaustive tests for the `flatten_unnamed_field` function.
    /// We cover leaf vs. custom types, with all combinations of
    /// `skip_self_just` and `parent_skip_child`, plus error handling.
    ///
    /// As a reminder, `flatten_unnamed_field` returns:
    ///   (Vec<TokenStream2>, TokenStream2, TokenStream2, TokenStream2)
    /// corresponding to
    ///   - `flattened_decls` for the generated struct fields in the flattened type,
    ///   - `item_init` for how to build the final `item.field`,
    ///   - `just_init` for the justification portion,
    ///   - `conf_init` for the confidence portion.
    /// 
    /// The test ensures that each scenario matches expected code expansions.
    /// We do some string-based matching on `item_init.to_string()` to confirm
    /// it includes the usage of `From::from(...)` or not, as appropriate.
    /// 
    #[traced_test]
    fn test_leaf_type_skip_self_just_false_parent_skip_child_false() {
        trace!("Starting test: leaf type, skip_self_just=false, parent_skip_child=false");
        let field_ident = syn::Ident::new("f0", proc_macro2::Span::call_site());
        let field_ty: Type = parse_quote! { bool }; // bool => leaf

        let skip_self_just = false;
        let parent_skip_child = false;
        let (decls, item_init, just_init, conf_init) = flatten_unnamed_field(
            &field_ident,
            &field_ty,
            skip_self_just,
            parent_skip_child
        );

        trace!("decls => {:?}", decls);
        trace!("item_init => {:?}", item_init.to_string());
        trace!("just_init => {:?}", just_init.to_string());
        trace!("conf_init => {:?}", conf_init.to_string());

        // For a leaf type + skip_self_just=false + parent_skip_child=false:
        //   - We expect "f0: bool," + f0_justification + f0_confidence
        //   - item_init should call From::from(f0)
        assert!(decls
            .iter()
            .map(|ts| ts.to_string())
            .any(|s| s.contains("f0 : bool")),
            "Should declare f0: bool"
        );
        assert!(decls
            .iter()
            .any(|ts| ts.to_string().contains("f0_justification : String")),
            "Should declare f0_justification : String"
        );
        assert!(decls
            .iter()
            .any(|ts| ts.to_string().contains("f0_confidence : f32")),
            "Should declare f0_confidence : f32"
        );
        assert!(
            item_init.to_string().contains("From :: from ( f0 )"),
            "Expected item_init to call From::from(f0)"
        );
        assert!(!just_init.is_empty(), "Non-empty just_init is expected");
        assert!(!conf_init.is_empty(), "Non-empty conf_init is expected");
    }

    #[traced_test]
    fn test_leaf_type_skip_self_just_true_parent_skip_child_false() {
        trace!("Starting test: leaf type, skip_self_just=true, parent_skip_child=false");
        let field_ident = syn::Ident::new("f0", proc_macro2::Span::call_site());
        let field_ty: Type = parse_quote! { u32 };

        let skip_self_just = true;
        let parent_skip_child = false;
        let (decls, item_init, just_init, conf_init) = flatten_unnamed_field(
            &field_ident,
            &field_ty,
            skip_self_just,
            parent_skip_child
        );

        trace!("decls => {:?}", decls);
        trace!("item_init => {:?}", item_init.to_string());
        trace!("just_init => {:?}", just_init.to_string());
        trace!("conf_init => {:?}", conf_init.to_string());

        // skip_self_just=true => no justification/conf
        // parent_skip_child=false => we expect item_init => From::from(f0)
        assert!(decls
            .iter()
            .any(|ts| ts.to_string().contains("f0 : u32")),
            "Should declare f0: u32"
        );
        assert!(
            !decls.iter().any(|ts| ts.to_string().contains("justification")),
            "No justification fields for skip_self_just=true"
        );
        assert!(
            !decls.iter().any(|ts| ts.to_string().contains("confidence")),
            "No confidence fields for skip_self_just=true"
        );
        assert!(
            item_init.to_string().contains("From :: from ( f0 )"),
            "Should still call From::from(...) if parent_skip_child=false"
        );
        assert!(just_init.is_empty(), "Should have empty just_init");
        assert!(conf_init.is_empty(), "Should have empty conf_init");
    }

    #[traced_test]
    fn test_leaf_type_skip_self_just_false_parent_skip_child_true() {
        trace!("Starting test: leaf type, skip_self_just=false, parent_skip_child=true");
        let field_ident = syn::Ident::new("f9", proc_macro2::Span::call_site());
        let field_ty: Type = parse_quote! { String };

        let skip_self_just = false;
        let parent_skip_child = true;
        let (decls, item_init, just_init, conf_init) = flatten_unnamed_field(
            &field_ident,
            &field_ty,
            skip_self_just,
            parent_skip_child
        );

        trace!("decls => {:?}", decls);
        trace!("item_init => {:?}", item_init.to_string());
        trace!("just_init => {:?}", just_init.to_string());
        trace!("conf_init => {:?}", conf_init.to_string());

        // skip_self_just=false => we add justification/conf
        // parent_skip_child=true => item_init => direct "f9" usage, not From::from
        let merged_decls = decls.iter().map(|d| d.to_string()).collect::<String>();
        assert!(
            merged_decls.contains("f9 : String"),
            "Should declare f9 : String with no flattening"
        );
        assert!(merged_decls.contains("f9_justification : String"));
        assert!(merged_decls.contains("f9_confidence : f32"));
        let init_str = item_init.to_string();
        assert!(!init_str.contains("From :: from"), "Should not call From::from(...)");
        assert_eq!(init_str, "f9", "Item init should be just 'f9'");
        assert!(!just_init.is_empty());
        assert!(!conf_init.is_empty());
    }

    #[traced_test]
    fn test_leaf_type_skip_self_just_true_parent_skip_child_true() {
        trace!("Starting test: leaf type, skip_self_just=true, parent_skip_child=true");
        let field_ident = syn::Ident::new("f77", proc_macro2::Span::call_site());
        let field_ty: Type = parse_quote! { bool };

        let skip_self_just = true;
        let parent_skip_child = true;
        let (decls, item_init, just_init, conf_init) = flatten_unnamed_field(
            &field_ident,
            &field_ty,
            skip_self_just,
            parent_skip_child
        );

        trace!("decls => {:?}", decls);
        trace!("item_init => {:?}", item_init.to_string());
        trace!("just_init => {:?}", just_init.to_string());
        trace!("conf_init => {:?}", conf_init.to_string());

        // skip_self_just=true => no justification/conf
        // parent_skip_child=true => direct assignment, not From::from
        let merged_decls = decls.iter().map(ToString::to_string).collect::<String>();
        assert!(merged_decls.contains("f77 : bool"), "Should declare f77: bool");
        assert!(!merged_decls.contains("justification"), "No justification for skip_self_just=true");
        assert!(!merged_decls.contains("confidence"), "No confidence for skip_self_just=true");
        assert_eq!(item_init.to_string(), "f77");
        assert!(just_init.is_empty());
        assert!(conf_init.is_empty());
    }

    #[traced_test]
    fn test_custom_type_skip_self_just_false_parent_skip_child_false() {
        trace!("Starting test: custom type, skip_self_just=false, parent_skip_child=false");
        let field_ident = syn::Ident::new("f0", proc_macro2::Span::call_site());
        let field_ty: Type = parse_quote! { Widget };

        let skip_self_just = false;
        let parent_skip_child = false;
        let (decls, item_init, just_init, conf_init) = flatten_unnamed_field(
            &field_ident,
            &field_ty,
            skip_self_just,
            parent_skip_child
        );

        debug!("Collected decls: {:?}", decls);
        debug!("item_init: {}", item_init.to_string());
        debug!("just_init: {}", just_init.to_string());
        debug!("conf_init: {}", conf_init.to_string());

        // skip_self_just=false => we expect f0_justification/conf
        // parent_skip_child=false => we flatten child => e.g. "FlatJustifiedWidget"
        // item_init => From::from(f0)
        let merged_decls = decls.iter().map(ToString::to_string).collect::<String>();
        assert!(merged_decls.contains("f0 : FlatJustifiedWidget"),
            "Should have flattened to FlatJustifiedWidget"
        );
        assert!(merged_decls.contains("f0_justification : String"));
        assert!(merged_decls.contains("f0_confidence : f32"));
        assert!(
            item_init.to_string().contains("From :: from ( f0 )"),
            "Expected item_init to call From::from(f0)"
        );
        assert!(!just_init.is_empty());
        assert!(!conf_init.is_empty());
    }

    #[traced_test]
    fn test_custom_type_skip_self_just_true_parent_skip_child_false() {
        trace!("Starting test: custom type, skip_self_just=true, parent_skip_child=false");
        let field_ident = syn::Ident::new("f123", proc_macro2::Span::call_site());
        let field_ty: Type = parse_quote! { Gizmo };

        let skip_self_just = true;
        let parent_skip_child = false;
        let (decls, item_init, just_init, conf_init) = flatten_unnamed_field(
            &field_ident,
            &field_ty,
            skip_self_just,
            parent_skip_child
        );

        debug!("Collected decls: {:?}", decls);
        debug!("item_init: {}", item_init.to_string());
        debug!("just_init: {}", just_init.to_string());
        debug!("conf_init: {}", conf_init.to_string());

        // skip_self_just=true => no f123_justification/conf
        // parent_skip_child=false => we still flatten child => "FlatJustifiedGizmo"
        // item_init => From::from(f123)
        let merged_decls = decls.iter().map(ToString::to_string).collect::<String>();
        assert!(merged_decls.contains("f123 : FlatJustifiedGizmo"),
            "Should flatten child => FlatJustifiedGizmo"
        );
        assert!(
            !merged_decls.contains("f123_justification"),
            "No justification for skip_self_just=true"
        );
        assert!(
            !merged_decls.contains("f123_confidence"),
            "No confidence for skip_self_just=true"
        );
        assert!(
            item_init.to_string().contains("From :: from ( f123 )"),
            "Expected item_init to call From::from(f123)"
        );
        assert!(just_init.is_empty());
        assert!(conf_init.is_empty());
    }

    #[traced_test]
    fn test_custom_type_skip_self_just_false_parent_skip_child_true() {
        trace!("Starting test: custom type, skip_self_just=false, parent_skip_child=true");
        let field_ident = syn::Ident::new("f88", proc_macro2::Span::call_site());
        let field_ty: Type = parse_quote! { SomeType };

        let skip_self_just = false;
        let parent_skip_child = true;
        let (decls, item_init, just_init, conf_init) = flatten_unnamed_field(
            &field_ident,
            &field_ty,
            skip_self_just,
            parent_skip_child
        );

        debug!("Collected decls: {:?}", decls);
        debug!("item_init: {}", item_init.to_string());
        debug!("just_init: {}", just_init.to_string());
        debug!("conf_init: {}", conf_init.to_string());

        // skip_self_just=false => do "f88_justification"/"f88_confidence"
        // parent_skip_child=true => "f88: SomeType" (no flattening) + item_init => "f88"
        let merged_decls = decls.iter().map(|d| d.to_string()).collect::<String>();
        assert!(merged_decls.contains("f88 : SomeType"));
        assert!(merged_decls.contains("f88_justification : String"));
        assert!(merged_decls.contains("f88_confidence : f32"));
        assert_eq!(item_init.to_string(), "f88", "Should not call From::from(...)");
        assert!(!just_init.is_empty());
        assert!(!conf_init.is_empty());
    }

    #[traced_test]
    fn test_error_path_from_compute_flat_type() {
        trace!("Starting test: error path from compute_flat_type_for_stamped");
        // If "BadType" is recognized and returns Err(...) from compute_flat_type_for_stamped,
        // we expect the function to return a single compile_error tokenstream, with everything else empty.
        let field_ident = syn::Ident::new("f0", proc_macro2::Span::call_site());
        let field_ty: Type = parse_quote! { BadType };

        let (decls, item_init, just_init, conf_init) = flatten_unnamed_field(
            &field_ident,
            &field_ty,
            false, // skip_self_just
            false  // parent_skip_child
        );

        debug!("Collected decls: {:?}", decls);
        debug!("item_init => {:?}", item_init.to_string());
        debug!("just_init => {:?}", just_init.to_string());
        debug!("conf_init => {:?}", conf_init.to_string());

        // We expect exactly one tokenstream in decls => the compile_error
        // plus item_init/just_init/conf_init all empty
        assert_eq!(
            decls.len(),
            1,
            "Should contain exactly one tokenstream for the error compile_error"
        );
        let err_str = decls[0].to_string();
        assert!(
            err_str.contains("compile_error!"),
            "Should produce a compile_error token in the first/only decl"
        );
        assert!(item_init.is_empty(), "Should be empty on error");
        assert!(just_init.is_empty(), "Should be empty on error");
        assert!(conf_init.is_empty(), "Should be empty on error");
    }
}
