// ---------------- [ File: ai-json-template-derive/src/gather_unnamed_variant_expansions.rs ]
crate::ix!();

pub fn gather_unnamed_variant_expansions(
    parent_enum_ident:         &syn::Ident,
    variant_ident:             &syn::Ident,
    unnamed_fields:            &FieldsUnnamed,
    skip_self_just:           bool,
    skip_child_just:          bool,
    flatten_unnamed_field_fn: &impl Fn(&syn::Ident, &syn::Type, bool, bool)
        -> (Vec<TokenStream2>, TokenStream2, TokenStream2, TokenStream2),
    skip_field_self_just_fn:  &impl Fn(&Field) -> bool,
    is_leaf_type_fn:          &impl Fn(&syn::Type) -> bool,
) -> UnnamedVariantExpansion {
    debug!(
        "Gathering expansions for unnamed variant '{}::{}'",
        parent_enum_ident,
        variant_ident
    );

    let mut expansions = UnnamedVariantExpansionBuilder::default()
        .field_declarations(vec![])
        .pattern_vars(vec![])
        .item_exprs(vec![])
        .just_vals(vec![])
        .conf_vals(vec![])
        .build()
        .unwrap();

    // Top-level justification/conf if not skipped
    if !skip_self_just {
        expansions.field_declarations_mut().push(quote! {
            #[serde(default)]
            enum_variant_justification:String
        });
        expansions.field_declarations_mut().push(quote! {
            #[serde(default)]
            enum_variant_confidence:f32
        });
        expansions.pattern_vars_mut().push(quote! { enum_variant_justification });
        expansions.pattern_vars_mut().push(quote! { enum_variant_confidence });
        expansions.just_vals_mut().push(quote! { variant_justification: enum_variant_justification });
        expansions.conf_vals_mut().push(quote! { variant_confidence: enum_variant_confidence });
    }

    // Now each unnamed field
    for (idx, field) in unnamed_fields.unnamed.iter().enumerate() {
        let field_ident = syn::Ident::new(&format!("f{}", idx), field.span());

        let skip_f_self = skip_field_self_just_fn(field);
        let child_skip  = skip_f_self || skip_child_just || is_leaf_type_fn(&field.ty);

        let (field_decls, i_init, j_init, c_init) =
            flatten_unnamed_field_fn(&field_ident, &field.ty, skip_f_self, child_skip);

        expansions.field_declarations_mut().extend(field_decls);
        expansions.pattern_vars_mut().push(quote! { #field_ident });

        if !i_init.is_empty() {
            expansions.item_exprs_mut().push(i_init);
        }
        if !j_init.is_empty() {
            expansions.just_vals_mut().push(j_init);
        }
        if !c_init.is_empty() {
            expansions.conf_vals_mut().push(c_init);
        }
    }

    expansions
}

#[cfg(test)]
mod test_gather_unnamed_variant_expansions {
    use super::*;

    #[traced_test]
    fn test_no_fields() {
        trace!("Starting test_no_fields: zero unnamed fields");
        let parent_enum_ident: Ident = parse_quote! { TestEnum };
        let variant_ident: Ident = parse_quote! { ZeroFields };
        let fields_unnamed = match parse_quote! {
            enum Example {
                ZeroFields()
            }
        } {
            syn::Item::Enum(e) => {
                if let Fields::Unnamed(u) = &e.variants[0].fields {
                    u.clone()
                } else {
                    panic!("Expected unnamed fields for ZeroFields variant");
                }
            }
            _ => panic!("Parsed item was not an enum"),
        };

        let skip_self_just = false;
        let skip_child_just = false;

        let flatten_unnamed_field_fn = |_fid: &Ident, _t: &syn::Type, _skip_self: bool, _skip_child: bool|
            (vec![], quote!(), quote!(), quote!());

        let skip_field_self_just_fn = |_field: &syn::Field| false;
        let is_leaf_type_fn = |_ty: &syn::Type| false;

        let expansion = gather_unnamed_variant_expansions(
            &parent_enum_ident,
            &variant_ident,
            &fields_unnamed,
            skip_self_just,
            skip_child_just,
            &flatten_unnamed_field_fn,
            &skip_field_self_just_fn,
            &is_leaf_type_fn,
        );

        assert_eq!(expansion.field_declarations().len(), 0, "No field_declarations expected");
        assert_eq!(expansion.pattern_vars().len(), 0, "No pattern_vars expected");
        assert_eq!(expansion.item_exprs().len(), 0, "No item_exprs expected");
        assert_eq!(expansion.just_vals().len(), 0, "No just_vals expected");
        assert_eq!(expansion.conf_vals().len(), 0, "No conf_vals expected");
        info!("test_no_fields passed successfully");
    }

    #[traced_test]
    fn test_single_field_skip_self_just() {
        trace!("Starting test_single_field_skip_self_just: one unnamed field, skip_self_just=true");
        let parent_enum_ident: Ident = parse_quote! { TestEnum };
        let variant_ident: Ident = parse_quote! { SingleField };
        let fields_unnamed = match parse_quote! {
            enum Example {
                SingleField(u32)
            }
        } {
            syn::Item::Enum(e) => {
                if let Fields::Unnamed(u) = &e.variants[0].fields {
                    u.clone()
                } else {
                    panic!("Expected unnamed fields for SingleField variant");
                }
            }
            _ => panic!("Parsed item was not an enum"),
        };

        let skip_self_just = true;
        let skip_child_just = false;

        let flatten_unnamed_field_fn = |fid: &Ident, _t: &syn::Type, skip_s: bool, skip_c: bool| {
            debug!("flatten_unnamed_field_fn: field={}, skip_s={}, skip_c={}", fid, skip_s, skip_c);
            let declarations = vec![quote! { #[serde(default)] #fid: u32 }];
            let item_init = quote! { #fid };
            (declarations, item_init, quote!(), quote!())
        };

        let skip_field_self_just_fn = |_field: &syn::Field| false;
        let is_leaf_type_fn = |_ty: &syn::Type| false;

        let expansion = gather_unnamed_variant_expansions(
            &parent_enum_ident,
            &variant_ident,
            &fields_unnamed,
            skip_self_just,
            skip_child_just,
            &flatten_unnamed_field_fn,
            &skip_field_self_just_fn,
            &is_leaf_type_fn,
        );

        // Because skip_self_just=true, we expect no top-level justification/conf
        assert_eq!(expansion.field_declarations().len(), 1, "One field_declaration expected");
        assert_eq!(expansion.pattern_vars().len(), 1, "One pattern_var expected");
        assert_eq!(expansion.item_exprs().len(), 1, "One item_expr expected");
        assert_eq!(expansion.just_vals().len(), 0, "No just_vals expected");
        assert_eq!(expansion.conf_vals().len(), 0, "No conf_vals expected");
        info!("test_single_field_skip_self_just passed successfully");
    }

    #[traced_test]
    fn test_single_field_including_self_just() {
        trace!("Starting test_single_field_including_self_just: one unnamed field, skip_self_just=false");
        let parent_enum_ident: Ident = parse_quote! { TestEnum };
        let variant_ident: Ident = parse_quote! { SingleField };
        let fields_unnamed = match parse_quote! {
            enum Example {
                SingleField(bool)
            }
        } {
            syn::Item::Enum(e) => {
                if let Fields::Unnamed(u) = &e.variants[0].fields {
                    u.clone()
                } else {
                    panic!("Expected unnamed fields for SingleField variant");
                }
            }
            _ => panic!("Parsed item was not an enum"),
        };

        let skip_self_just = false;
        let skip_child_just = false;

        let flatten_unnamed_field_fn = |fid: &Ident, _t: &syn::Type, skip_s: bool, skip_c: bool| {
            debug!("flatten_unnamed_field_fn: field={}, skip_s={}, skip_c={}", fid, skip_s, skip_c);
            // Emulate we produce a single field plus justification/conf
            let declarations = vec![
                quote! { #[serde(default)] #fid: bool },
                quote! { #[serde(default)] #fid _justification: String },
                quote! { #[serde(default)] #fid _confidence: f32 },
            ];
            let item_init = quote! { #fid };
            let just_init = quote! { #fid _justification: #fid _justification };
            let conf_init = quote! { #fid _confidence: #fid _confidence };
            (declarations, item_init, just_init, conf_init)
        };

        let skip_field_self_just_fn = |_field: &syn::Field| false;
        let is_leaf_type_fn = |_ty: &syn::Type| false;

        let expansion = gather_unnamed_variant_expansions(
            &parent_enum_ident,
            &variant_ident,
            &fields_unnamed,
            skip_self_just,
            skip_child_just,
            &flatten_unnamed_field_fn,
            &skip_field_self_just_fn,
            &is_leaf_type_fn,
        );

        // Because skip_self_just=false, top-level enum_variant_justification/conf are inserted
        assert_eq!(expansion.field_declarations().len(), 5, "Five field_declarations (top-level + field) expected");
        assert_eq!(expansion.pattern_vars().len(), 3, "3 pattern vars => enum_variant_justification, enum_variant_confidence, field");
        assert_eq!(expansion.item_exprs().len(), 1, "One item_expr => the field itself");
        assert_eq!(expansion.just_vals().len(), 2, "Two just_vals => top-level + field justification");
        assert_eq!(expansion.conf_vals().len(), 2, "Two conf_vals => top-level + field confidence");
        info!("test_single_field_including_self_just passed successfully");
    }

    #[traced_test]
    fn test_multiple_fields_skip_child_just() {
        trace!("Starting test_multiple_fields_skip_child_just: multiple unnamed fields, skip_child_just=true");
        let parent_enum_ident: Ident = parse_quote! { ComplexEnum };
        let variant_ident: Ident = parse_quote! { ManyFields };
        let fields_unnamed = match parse_quote! {
            enum Example {
                ManyFields(u32, String, bool)
            }
        } {
            syn::Item::Enum(e) => {
                if let Fields::Unnamed(u) = &e.variants[0].fields {
                    u.clone()
                } else {
                    panic!("Expected unnamed fields for ManyFields variant");
                }
            }
            _ => panic!("Parsed item was not an enum"),
        };

        let skip_self_just = false;
        let skip_child_just = true;

        let flatten_unnamed_field_fn = |fid: &Ident, _t: &syn::Type, skip_s: bool, skip_c: bool| {
            debug!("flatten_unnamed_field_fn: field={}, skip_s={}, skip_c={}", fid, skip_s, skip_c);
            let declarations = vec![quote! { #[serde(default)] #fid: i64 }];
            let item_init = quote! { #fid };
            let just_init = if skip_s {
                quote!()
            } else {
                quote!(field_justification: format!("justification_for_{}", stringify!(#fid)))
            };
            let conf_init = if skip_s {
                quote!()
            } else {
                quote!(field_confidence: 0.75f32)
            };
            (declarations, item_init, just_init, conf_init)
        };

        let skip_field_self_just_fn = |_field: &syn::Field| false;
        let is_leaf_type_fn = |_ty: &syn::Type| false;

        let expansion = gather_unnamed_variant_expansions(
            &parent_enum_ident,
            &variant_ident,
            &fields_unnamed,
            skip_self_just,
            skip_child_just,
            &flatten_unnamed_field_fn,
            &skip_field_self_just_fn,
            &is_leaf_type_fn,
        );

        // top-level justification/conf (skip_self_just=false) => 2 declarations
        // plus 3 fields => each 1 declaration => total 2 + 3 = 5
        assert_eq!(expansion.field_declarations().len(), 5, "Expected 5 total field_declarations");
        // pattern_vars => 2 for top-level + 3 for fields => 5
        assert_eq!(expansion.pattern_vars().len(), 5, "Expected 5 pattern_vars");
        // item_exprs => 3 fields
        assert_eq!(expansion.item_exprs().len(), 3, "Expected 3 item_exprs");
        // just_vals => 1 top-level, plus each field (skip_s is false, but skip_child_just is true => does not block field justification if skip_s is false)
        // Actually, each field skip_s is false => so each field has a justification, plus top-level => total 1 + 3 = 4
        assert_eq!(expansion.just_vals().len(), 4, "Expected 4 just_vals");
        // same for conf_vals => 4
        assert_eq!(expansion.conf_vals().len(), 4, "Expected 4 conf_vals");
        info!("test_multiple_fields_skip_child_just passed successfully");
    }

    #[traced_test]
    fn test_multiple_fields_no_skip() {
        trace!("Starting test_multiple_fields_no_skip: multiple unnamed fields, no skipping");
        let parent_enum_ident: Ident = parse_quote! { AnotherEnum };
        let variant_ident: Ident = parse_quote! { SomeVariant };
        let fields_unnamed = match parse_quote! {
            enum Example {
                SomeVariant(f32, bool, String)
            }
        } {
            syn::Item::Enum(e) => {
                if let Fields::Unnamed(u) = &e.variants[0].fields {
                    u.clone()
                } else {
                    panic!("Expected unnamed fields for SomeVariant");
                }
            }
            _ => panic!("Parsed item was not an enum"),
        };

        let skip_self_just = false;
        let skip_child_just = false;

        // For demonstration, we force each field to produce 2 declarations (field, justification/conf)
        let flatten_unnamed_field_fn = |fid: &Ident, _t: &syn::Type, skip_s: bool, _skip_c: bool| {
            debug!("flatten_unnamed_field_fn: field={}, skip_s={}", fid, skip_s);
            if skip_s {
                // not used in these tests, but let's handle it
                return (vec![quote! { #[serde(default)] #fid: i8 }], quote! { #fid }, quote!(), quote!());
            }
            let declarations = vec![
                quote! { #[serde(default)] #fid: String },
                quote! { #[serde(default)] #fid _just: String },
                quote! { #[serde(default)] #fid _conf: f32 },
            ];
            let item_init = quote! { #fid };
            let just_init = quote! { #fid _just: format!("justify_{}", stringify!(#fid)) };
            let conf_init = quote! { #fid _conf: 0.99f32 };
            (declarations, item_init, just_init, conf_init)
        };

        let skip_field_self_just_fn = |_field: &syn::Field| false;
        let is_leaf_type_fn = |_ty: &syn::Type| false;

        let expansion = gather_unnamed_variant_expansions(
            &parent_enum_ident,
            &variant_ident,
            &fields_unnamed,
            skip_self_just,
            skip_child_just,
            &flatten_unnamed_field_fn,
            &skip_field_self_just_fn,
            &is_leaf_type_fn,
        );

        // top-level justification/conf => 2
        // each field => 3 declarations => total fields=3 => 3*3=9 => 2 + 9 = 11
        assert_eq!(expansion.field_declarations().len(), 11, "Expected 11 field_declarations");
        // pattern_vars => 2 top-level + 3 fields => 5
        assert_eq!(expansion.pattern_vars().len(), 5, "Expected 5 pattern_vars");
        // item_exprs => 3 fields
        assert_eq!(expansion.item_exprs().len(), 3, "Expected 3 item_exprs");
        // just_vals => 1 top-level + 3 fields => 4
        assert_eq!(expansion.just_vals().len(), 4, "Expected 4 just_vals");
        // conf_vals => same => 4
        assert_eq!(expansion.conf_vals().len(), 4, "Expected 4 conf_vals");
        info!("test_multiple_fields_no_skip passed successfully");
    }
}
