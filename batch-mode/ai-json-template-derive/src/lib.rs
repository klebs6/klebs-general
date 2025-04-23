// ---------------- [ File: ai-json-template-derive/src/lib.rs ]
#[macro_use] mod imports; use imports::*;

xp!{gather_doc_comments}
xp!{comma_separated_expression}
xp!{classify_field_type}

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
    tracing::trace!("Doc comments (container-level) => {:?}", container_docs_vec);

    match &ast.data {
        // ----------------- Named Struct Path -----------------
        Data::Struct(ds) => {
            tracing::trace!("Found a struct => verifying named fields or error out if not named.");

            match &ds.fields {
                Fields::Named(named_fields) => {
                    // Same logic as before for named structs
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
                        tracing::trace!("Analyzing field `{}` => docs: {:?}", field_name_str, field_docs);

                        let ty = &field.ty;
                        if let Some(expr) = classify_field_type(ty, &field_docs) {
                            tracing::trace!("Successfully classified field type => generating snippet for {}", field_name_str);
                            field_inits.push(quote! {
                                map.insert(#field_name_str.to_string(), #expr);
                            });
                        } else {
                            let type_q = quote!(#ty).to_string();
                            let err_msg = format!("Unsupported field type for AiJsonTemplate: {}", type_q);
                            let err = syn::Error::new(ty.span(), &err_msg);
                            tracing::trace!("ERROR: {}", &err_msg);
                            return err.to_compile_error().into();
                        }
                    }

                    let expanded = quote! {
                        impl AiJsonTemplate for #type_ident {
                            fn to_template() -> serde_json::Value {
                                tracing::trace!("AiJsonTemplate::to_template for named struct {}", #type_name_str);

                                let mut root = serde_json::Map::new();
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
                    tracing::trace!("Exiting AiJsonTemplate for named struct {}", type_name_str);
                    expanded.into()
                },

                // If the struct is not `Fields::Named`, produce an error. 
                // (If you want to support tuple structs, you can adapt similarly to how we handle 
                //  tuple variants below.)
                _ => {
                    let err = syn::Error::new(
                        span,
                        "AiJsonTemplate derive only supports named fields for structs (or handle your tuple struct in the enum logic)."
                    );
                    tracing::trace!("Encountered non-named fields => error");
                    err.to_compile_error().into()
                }
            }
        },

        // ----------------- Enum Path: can have any style of variants -----------------
        Data::Enum(data_enum) => {
            tracing::trace!("Found an enum => generating template for each variant (unit, struct, or tuple).");

            // We’ll accumulate each variant’s JSON object
            let mut variant_exprs = Vec::new();

            for var in &data_enum.variants {
                let var_name_str = var.ident.to_string();
                let var_docs = gather_doc_comments(&var.attrs).join("\n");
                tracing::trace!("Enum variant {} => doc lines: {:?}", var_name_str, var_docs);

                match &var.fields {
                    // ------ Unit variant ------
                    Fields::Unit => {
                        tracing::trace!("Variant {} is unit => building minimal schema", var_name_str);
                        let expr = quote! {
                            {
                                let mut variant_map = serde_json::Map::new();
                                variant_map.insert("variant_name".to_string(), serde_json::Value::String(#var_name_str.to_string()));
                                variant_map.insert("variant_docs".to_string(), serde_json::Value::String(#var_docs.to_string()));
                                variant_map.insert("variant_type".to_string(), serde_json::Value::String("unit".to_string()));
                                serde_json::Value::Object(variant_map)
                            }
                        };
                        variant_exprs.push(expr);
                    }

                    // ------ Struct variant (named fields) ------
                    Fields::Named(named_fields) => {
                        tracing::trace!("Variant {} is a struct variant => gather fields", var_name_str);

                        // For each field, create a schema entry
                        let mut field_inits = Vec::new();
                        for field in &named_fields.named {
                            let field_ident = field.ident.as_ref().unwrap(); // safe because Named
                            let field_name_str = field_ident.to_string();
                            let field_docs = gather_doc_comments(&field.attrs).join("\n");
                            tracing::trace!("  analyzing field `{}` => docs: {:?}", field_name_str, field_docs);

                            let ty = &field.ty;
                            if let Some(expr) = classify_field_type(ty, &field_docs) {
                                field_inits.push(quote! {
                                    map.insert(#field_name_str.to_string(), #expr);
                                });
                            } else {
                                let type_q = quote!(#ty).to_string();
                                let err_msg = format!("Unsupported field type in enum variant {}: {}", var_name_str, type_q);
                                let err = syn::Error::new(ty.span(), &err_msg);
                                tracing::trace!("ERROR: {}", &err_msg);
                                return err.to_compile_error().into();
                            }
                        }

                        let expr = quote! {
                            {
                                let mut variant_map = serde_json::Map::new();
                                variant_map.insert("variant_name".to_string(), serde_json::Value::String(#var_name_str.to_string()));
                                variant_map.insert("variant_docs".to_string(), serde_json::Value::String(#var_docs.to_string()));
                                variant_map.insert("variant_type".to_string(), serde_json::Value::String("struct_variant".to_string()));

                                // build "fields" object
                                let mut map = serde_json::Map::new();
                                #(#field_inits)*

                                variant_map.insert("fields".to_string(), serde_json::Value::Object(map));
                                serde_json::Value::Object(variant_map)
                            }
                        };
                        variant_exprs.push(expr);
                    }

                    // ------ Tuple variant (unnamed fields) ------
                    Fields::Unnamed(unnamed_fields) => {
                        tracing::trace!("Variant {} is a tuple variant => gather fields by index", var_name_str);

                        // For each field, create a schema entry keyed by index
                        // e.g. field_0, field_1
                        let mut field_inits = Vec::new();
                        for (i, field) in unnamed_fields.unnamed.iter().enumerate() {
                            let field_docs = gather_doc_comments(&field.attrs).join("\n");
                            let field_key = format!("field_{}", i);
                            let ty = &field.ty;
                            if let Some(expr) = classify_field_type(ty, &field_docs) {
                                field_inits.push(quote! {
                                    map.insert(#field_key.to_string(), #expr);
                                });
                            } else {
                                let type_q = quote!(#ty).to_string();
                                let err_msg = format!("Unsupported field type in tuple variant {}: {}", var_name_str, type_q);
                                let err = syn::Error::new(ty.span(), &err_msg);
                                tracing::trace!("ERROR: {}", &err_msg);
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

            // Finally, generate the overall enum template
            let expanded = quote! {
                impl AiJsonTemplate for #type_ident {
                    fn to_template() -> serde_json::Value {
                        tracing::trace!("AiJsonTemplate::to_template for enum {}", #type_name_str);

                        let mut root = serde_json::Map::new();
                        root.insert("enum_docs".to_string(), serde_json::Value::String(#container_docs_str.to_string()));
                        root.insert("enum_name".to_string(), serde_json::Value::String(#type_name_str.to_string()));
                        // We use "type":"complex_enum" to indicate it may have struct/tuple/unit variants
                        root.insert("type".to_string(), serde_json::Value::String("complex_enum".to_string()));

                        let variants_array = serde_json::Value::Array(vec![
                            #(#variant_exprs),*
                        ]);
                        root.insert("variants".to_string(), variants_array);

                        serde_json::Value::Object(root)
                    }
                }
            };
            tracing::trace!("Exiting AiJsonTemplate for enum {}", type_name_str);
            expanded.into()
        },

        // ----------------- Union => not supported -----------------
        Data::Union(_) => {
            let err = syn::Error::new(
                span,
                "AiJsonTemplate derive does not support unions."
            );
            tracing::trace!("ERROR: Attempted to derive on a union => not supported.");
            err.to_compile_error().into()
        }
    }
}
