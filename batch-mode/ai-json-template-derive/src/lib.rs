// ---------------- [ File: ai-json-template-derive/src/lib.rs ]
//#![allow(dead_code)]
#![feature(box_patterns)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#[macro_use] mod imports; use imports::*;

xp!{build_enum_variant_expr_with_justification}
xp!{build_enum_variant_fields_map_with_justification}
xp!{build_from_arm_for_named}
xp!{build_from_arm_for_unit_variant}
xp!{build_hashmap_schema}
xp!{build_named_field_child_schema_expr}
xp!{build_named_field_just_conf_placeholders}
xp!{build_top_level_justification_fields_for_variant}
xp!{classify_field_type}
xp!{classify_field_type_for_child}
xp!{comma_separated_expression}
xp!{emit_schema_for_bool}
xp!{emit_schema_for_fallback_nested}
xp!{emit_schema_for_hashmap}
xp!{emit_schema_for_number}
xp!{emit_schema_for_string}
xp!{emit_schema_for_type}
xp!{emit_schema_for_vec}
xp!{expand_enum_with_justification}
xp!{expand_named_struct_with_justification}
xp!{extract_hashmap_inner}
xp!{extract_option_inner}
xp!{extract_vec_inner}
xp!{find_default_variant}
xp!{flattened_field_result}
xp!{gather_doc_comments}
xp!{gather_schemas_and_placeholders_for_named_fields}
xp!{generate_enum_justified}
xp!{generate_justified_structs_for_named}
xp!{generate_manual_default_for_justified_enum}
xp!{generate_manual_default_for_justified_named_struct}
xp!{generate_reverse_from_impl_for_enum_with_justification}
xp!{generate_reverse_from_impl_for_named_with_justification}
xp!{generate_to_template_with_justification_for_enum}
xp!{generate_to_template_with_justification_for_named}
xp!{is_builtin_scalar}
xp!{is_justification_enabled}
xp!{is_leaf_type}
xp!{is_numeric}
xp!{justified_type}
xp!{parse_doc_expr}
xp!{resolve_map_key_type}
xp!{top_level_justification_result}
xp!{unnamed_variant_expansion}

/// This new implementation supports:
///
/// - Named structs (as before),
/// - Enums with any mix of:
///    - Unit variants,
///    - Struct variants (with named fields),
///    - Tuple variants (with unnamed fields).
///
/// It generates a template describing each variant and its fields. Where those fields are
/// themselves `AiJsonTemplate`, the macro nests that template, just like with structs.
/// 
/// Please integrate this **entire function** as-is (and make sure the other helper modules
/// like `gather_doc_comments` and `classify_field_type` are still in scope).
///
#[proc_macro_derive(AiJsonTemplate)]
pub fn derive_ai_json_template(input: TokenStream) -> TokenStream {
    tracing::trace!("Entering derive_ai_json_template macro.");

    let ast = parse_macro_input!(input as DeriveInput);
    let span  = ast.span();
    let type_ident = &ast.ident;
    let type_name_str = type_ident.to_string();
    tracing::trace!("Analyzing type: {}", type_name_str);

    // Gather doc comments from the container itself
    let container_docs_vec = gather_doc_comments(&ast.attrs);
    let container_docs_str = container_docs_vec.join("\n");

    match &ast.data {
        // ----------------- Named Struct Path -----------------
        Data::Struct(ds) => {
            match &ds.fields {
                Fields::Named(named_fields) => {
                    let mut field_inits = Vec::new();
                    for field in &named_fields.named {
                        let field_ident = match &field.ident {
                            Some(id) => id,
                            None => {
                                let err = syn::Error::new(
                                    field.span(),
                                    "Unnamed fields are not supported by AiJsonTemplate for named structs."
                                );
                                return err.to_compile_error().into();
                            }
                        };

                        let field_name_str = field_ident.to_string();
                        let field_docs = gather_doc_comments(&field.attrs).join("\n");
                        let ty = &field.ty;

                        if let Some(expr) = classify_field_type(ty, &field_docs) {
                            field_inits.push(quote! {
                                map.insert(#field_name_str.to_string(), #expr);
                            });
                        } else {
                            let type_q = quote!(#ty).to_string();
                            let err_msg = format!("Unsupported field type for AiJsonTemplate: {}", type_q);
                            let err = syn::Error::new(ty.span(), &err_msg);
                            return err.to_compile_error().into();
                        }
                    }

                    let expanded = quote! {
                        impl AiJsonTemplate for #type_ident {
                            fn to_template() -> serde_json::Value {
                                tracing::trace!("AiJsonTemplate::to_template for named struct {}", #type_name_str);

                                let mut root = serde_json::Map::new();
                                // Include our container docs plus disclaimers
                                root.insert("struct_docs".to_string(), serde_json::Value::String(#container_docs_str.to_string()));
                                root.insert("struct_name".to_string(), serde_json::Value::String(#type_name_str.to_string()));
                                root.insert("type".to_string(), serde_json::Value::String("struct".to_string()));

                                let mut map = serde_json::Map::new();
                                #(#field_inits)*

                                root.insert("fields".to_string(), serde_json::Value::Object(map));
                                serde_json::Value::Object(root)
                            }
                        }
                    };
                    expanded.into()
                },
                _ => {
                    let err = syn::Error::new(
                        span,
                        "AiJsonTemplate derive only supports named fields for structs. Tuple/unnamed not supported here."
                    );
                    err.to_compile_error().into()
                }
            }
        },

        // ----------------- Enum Path -----------------
        Data::Enum(data_enum) => {
            // We'll gather a snippet for each variant
            let mut variant_exprs = Vec::new();
            for var in &data_enum.variants {
                let var_name_str = var.ident.to_string();
                let var_docs = gather_doc_comments(&var.attrs).join("\n");

                match &var.fields {
                    Fields::Unit => {
                        let expr = quote! {
                            {
                                let mut variant_map = serde_json::Map::new();
                                variant_map.insert("variant_name".to_string(), serde_json::Value::String(#var_name_str.to_string()));
                                variant_map.insert("variant_docs".to_string(), serde_json::Value::String(#var_docs.to_string()));
                                variant_map.insert("variant_type".to_string(), serde_json::Value::String("unit_variant".to_string()));
                                serde_json::Value::Object(variant_map)
                            }
                        };
                        variant_exprs.push(expr);
                    }
                    Fields::Named(named_fields) => {
                        let mut field_inits = Vec::new();
                        for field in &named_fields.named {
                            let field_ident = field.ident.as_ref().unwrap();
                            let field_name_str = field_ident.to_string();
                            let fd = gather_doc_comments(&field.attrs).join("\n");
                            let ty = &field.ty;

                            if let Some(expr) = classify_field_type(ty, &fd) {
                                field_inits.push(quote! {
                                    map.insert(#field_name_str.to_string(), #expr);
                                });
                            } else {
                                let type_q = quote!(#ty).to_string();
                                let err_msg = format!("Unsupported field type in enum variant {}: {}", var_name_str, type_q);
                                let err = syn::Error::new(ty.span(), &err_msg);
                                return err.to_compile_error().into();
                            }
                        }

                        let expr = quote! {
                            {
                                let mut variant_map = serde_json::Map::new();
                                variant_map.insert("variant_name".to_string(), serde_json::Value::String(#var_name_str.to_string()));
                                variant_map.insert("variant_docs".to_string(), serde_json::Value::String(#var_docs.to_string()));
                                variant_map.insert("variant_type".to_string(), serde_json::Value::String("struct_variant".to_string()));

                                let mut map = serde_json::Map::new();
                                #(#field_inits)*

                                variant_map.insert("fields".to_string(), serde_json::Value::Object(map));
                                serde_json::Value::Object(variant_map)
                            }
                        };
                        variant_exprs.push(expr);
                    }
                    Fields::Unnamed(unnamed_fields) => {
                        let mut field_inits = Vec::new();
                        for (i, field) in unnamed_fields.unnamed.iter().enumerate() {
                            let field_key = format!("field_{}", i);
                            let fd = gather_doc_comments(&field.attrs).join("\n");
                            let ty = &field.ty;

                            if let Some(expr) = classify_field_type(ty, &fd) {
                                field_inits.push(quote! {
                                    map.insert(#field_key.to_string(), #expr);
                                });
                            } else {
                                let type_q = quote!(#ty).to_string();
                                let err_msg = format!("Unsupported field type in tuple variant {}: {}", var_name_str, type_q);
                                let err = syn::Error::new(ty.span(), &err_msg);
                                return err.to_compile_error().into();
                            }
                        }

                        let expr = quote! {
                            {
                                let mut variant_map = serde_json::Map::new();
                                variant_map.insert("variant_name".to_string(), serde_json::Value::String(#var_name_str.to_string()));
                                variant_map.insert("variant_docs".to_string(), serde_json::Value::String(#var_docs.to_string()));
                                variant_map.insert("variant_type".to_string(), serde_json::Value::String("tuple_variant".to_string()));

                                let mut map = serde_json::Map::new();
                                #(#field_inits)*

                                variant_map.insert("fields".to_string(), serde_json::Value::Object(map));
                                serde_json::Value::Object(variant_map)
                            }
                        };
                        variant_exprs.push(expr);
                    }
                }
            }

            let expanded = quote! {
                impl AiJsonTemplate for #type_ident {
                    fn to_template() -> serde_json::Value {
                        tracing::trace!("AiJsonTemplate::to_template for enum {}", #type_name_str);

                        let mut root = serde_json::Map::new();

                        root.insert("enum_docs".to_string(), serde_json::Value::String(#container_docs_str.to_string()));
                        root.insert("enum_name".to_string(), serde_json::Value::String(#type_name_str.to_string()));
                        root.insert("type".to_string(), serde_json::Value::String("complex_enum".to_string()));

                        let variants_array = serde_json::Value::Array(vec![
                            #(#variant_exprs),*
                        ]);
                        root.insert("variants".to_string(), variants_array);

                        serde_json::Value::Object(root)
                    }
                }
            };
            expanded.into()
        },

        // ----------------- Union => not supported
        Data::Union(_) => {
            let err = syn::Error::new(
                span,
                "AiJsonTemplate derive does not support unions."
            );
            err.to_compile_error().into()
        }
    }
}

/// The main entrypoint for `#[derive(AiJsonTemplateWithJustification)]`.
/// We generate (a) the typed justification structs/enums, and
/// (b) the *FlatJustified* expansions if desired by the test suite.
#[proc_macro_derive(AiJsonTemplateWithJustification, attributes(doc, justify, justify_inner))]
pub fn derive_ai_json_template_with_justification(
    input: proc_macro::TokenStream
) -> proc_macro::TokenStream {

    let ast  = syn::parse_macro_input!(input as syn::DeriveInput);
    let span = ast.span();

    // We'll build up the final expansions in `out`
    let mut out = proc_macro2::TokenStream::new();

    let ty_ident           = &ast.ident;
    let container_docs     = gather_doc_comments(&ast.attrs);
    let _container_doc_str = container_docs.join("\n");

    match &ast.data {

        // ==================== Named Struct ====================
        syn::Data::Struct(ds) => {

            if let syn::Fields::Named(ref named_fields) = ds.fields {
                // (a) typed expansions
                let justified_struct = generate_justified_structs_for_named(ty_ident, named_fields, span);
                out.extend(justified_struct);

                // Optionally generate a specialized "to_template_with_justification()"—you might already do so.
                let to_tpl_ts = generate_to_template_with_justification_for_named(
                    ty_ident,
                    named_fields,
                    &_container_doc_str
                );
                out.extend(to_tpl_ts);

                let from_impl = generate_reverse_from_impl_for_named_with_justification(
                    ty_ident,
                    named_fields,
                    span,
                );
                out.extend(from_impl);

            } else {
                // e.g. unit or tuple struct => produce an error or do something else
                let err = syn::Error::new(
                    span,
                    "AiJsonTemplateWithJustification only supports named structs"
                );
                out.extend(err.to_compile_error());
            }
        }

        // ==================== Enum ====================
        syn::Data::Enum(data_enum) => {

            // (a) typed expansions for justification/conf + JustifiedEnum
            let justified_enum = generate_enum_justified(ty_ident, data_enum, span);
            out.extend(justified_enum);

            // Optionally generate a specialized "to_template_with_justification_for_enum(...)"
            let enum_tpl_ts = generate_to_template_with_justification_for_enum(
                ty_ident,
                data_enum,
                &_container_doc_str
            );
            out.extend(enum_tpl_ts);

            let from_impl = generate_reverse_from_impl_for_enum_with_justification(
                ty_ident,
                data_enum,
                span,
            );
            out.extend(from_impl);
        }

        // ==================== Union => not supported ====================
        syn::Data::Union(_) => {
            let e = syn::Error::new(span, "AiJsonTemplateWithJustification not supported on unions.");
            out.extend(e.to_compile_error());
        }
    }

    if DEBUG_FINAL_EXPANSION {

        eprintln!("=== FINAL EXPANSION for {} ===\n{}", ty_ident, out.to_string());

        assert_tokens_parse_ok(&out);
    }

    out.into()
}

const DEBUG_FINAL_EXPANSION: bool = false;
