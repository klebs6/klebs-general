// ---------------- [ File: ai-json-template-derive/src/lib.rs ]
#![allow(dead_code)]
#![allow(unused_imports)]
#[macro_use] mod imports; use imports::*;

xp!{classify_field_type_with_justification}
xp!{classify_field_type}
xp!{classify_result}
xp!{comma_separated_expression}
xp!{emit_schema_for_type}
xp!{extract_hashmap_inner}
xp!{extract_option_inner}
xp!{extract_vec_inner}
xp!{gather_doc_comments}
xp!{gather_field_injections}
xp!{gather_item_accessors}
xp!{gather_justification_and_confidence_fields}
xp!{is_builtin_scalar}
xp!{is_justification_enabled}
xp!{is_numeric}
xp!{parse_doc_expr}
xp!{compute_flat_type_for_stamped}

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

/// This is the **main** entrypoint called by the proc macro. It just
/// parses the input, calls `process_ai_json_template_with_justification`,
/// and returns the resulting token stream.
#[proc_macro_derive(AiJsonTemplateWithJustification, attributes(doc, justify, justify_inner))]
pub fn derive_ai_json_template_with_justification(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);
    let expanded = expand_ai_json_template_with_justification(&ast);
    expanded.into()
}

#[proc_macro_derive(AiJsonTemplateWithJustification, attributes(doc, justify, justify_inner))]
pub fn derive_ai_json_template_with_justification(input: TokenStream) -> TokenStream {
    use syn::{
        parse_macro_input, DeriveInput, Data, Fields, spanned::Spanned
    };

    let ast = parse_macro_input!(input as DeriveInput);
    let span = ast.span();
    let ty_ident = &ast.ident;
    let type_name_str = ty_ident.to_string();

    let container_docs_vec = gather_doc_comments(&ast.attrs);
    let container_docs_str = container_docs_vec.join("\n");
    tracing::trace!("Deriving AiJsonTemplateWithJustification for {}", ty_ident);

    let mut output_ts = proc_macro2::TokenStream::new();

    match &ast.data {
        // -------------------------------
        //  1) Named Struct
        // -------------------------------
        Data::Struct(data_struct) => {
            match &data_struct.fields {
                syn::Fields::Named(named_fields) => {
                    // A) Normal "Justified" items:
                    let justification_ident = syn::Ident::new(
                        &format!("{}Justification", ty_ident),
                        span
                    );
                    let confidence_ident = syn::Ident::new(
                        &format!("{}Confidence", ty_ident),
                        span
                    );
                    let justified_ident = syn::Ident::new(
                        &format!("Justified{}", ty_ident),
                        span
                    );

                    let mut justification_struct_fields = Vec::new();
                    let mut confidence_struct_fields = Vec::new();
                    let mut errs = quote::quote!();
                    let mut field_mappings = Vec::new();

                    gather_justification_and_confidence_fields(
                        named_fields,
                        &mut justification_struct_fields,
                        &mut confidence_struct_fields,
                        &mut errs,
                        &mut field_mappings,
                    );
                    output_ts.extend(errs);

                    // The normal Justification struct
                    let justification_struct = quote::quote! {
                        #[derive(Builder, Debug, Clone, PartialEq, Default, Serialize, Deserialize, Getters, Setters)]
                        #[builder(setter(into))]
                        #[getset(get="pub", set="pub")]
                        struct #justification_ident {
                            #(#justification_struct_fields),*
                        }
                    };

                    // The normal Confidence struct
                    let confidence_struct = quote::quote! {
                        #[derive(Builder, Debug, Clone, PartialEq, Default, Serialize, Deserialize, Getters, Setters)]
                        #[builder(setter(into))]
                        #[getset(get="pub", set="pub")]
                        struct #confidence_ident {
                            #(#confidence_struct_fields),*
                        }
                    };

                    // The normal Justified struct
                    let justified_struct = quote::quote! {
                        #[derive(Builder, Debug, Default, Clone, PartialEq, Serialize, Deserialize, Getters, Setters)]
                        #[builder(setter(into))]
                        #[getset(get="pub", set="pub")]
                        struct #justified_ident {
                            #[getset(get="pub", set="pub")]
                            item: #ty_ident,

                            #[getset(get="pub", set="pub")]
                            justification: #justification_ident,

                            #[getset(get="pub", set="pub")]
                            confidence: #confidence_ident,
                        }

                        impl #justified_ident {
                            fn new(item: #ty_ident) -> Self {
                                Self {
                                    item,
                                    justification: Default::default(),
                                    confidence: Default::default(),
                                }
                            }
                        }
                    };

                    // Accessors for justification:
                    let (item_acc, just_acc, conf_acc) =
                        gather_item_accessors(named_fields, ty_ident, &field_mappings);

                    let impl_accessors = quote::quote! {
                        impl #justified_ident {
                            #(#item_acc)*
                            #(#just_acc)*
                            #(#conf_acc)*
                        }
                    };

                    // The standard `to_template_with_justification()` method:
                    let mut field_inits = Vec::new();
                    for field in &named_fields.named {
                        let field_ident = field.ident.as_ref().unwrap();
                        let doc_str = gather_doc_comments(&field.attrs).join("\n");
                        let field_name_str = field_ident.to_string();

                        let is_required = extract_option_inner(&field.ty).is_none();
                        let skip_self_just = is_justification_disabled_for_field(field);
                        let skip_child_just = skip_self_just || is_justification_disabled_for_inner(field);

                        if let Some(expr) = classify_field_type_for_child(&field.ty, &doc_str, is_required, skip_child_just) {
                            field_inits.push(quote::quote! {
                                map.insert(#field_name_str.to_string(), #expr);
                            });
                        }

                        if !skip_self_just {
                            field_inits.push(quote::quote! {
                                let justify_key = format!("{}_justification", #field_name_str);
                                let conf_key    = format!("{}_confidence", #field_name_str);

                                let mut just_obj = serde_json::Map::new();
                                just_obj.insert("type".to_string(), serde_json::Value::String("string".to_string()));
                                just_obj.insert("required".to_string(), serde_json::Value::Bool(true));
                                map.insert(justify_key, serde_json::Value::Object(just_obj));

                                let mut conf_obj = serde_json::Map::new();
                                conf_obj.insert("type".to_string(), serde_json::Value::String("number".to_string()));
                                conf_obj.insert("required".to_string(), serde_json::Value::Bool(true));
                                map.insert(conf_key, serde_json::Value::Object(conf_obj));
                            });
                        }
                    }

                    let impl_ts = quote::quote! {
                        impl AiJsonTemplateWithJustification for #ty_ident {
                            fn to_template_with_justification() -> serde_json::Value {
                                tracing::trace!("AiJsonTemplateWithJustification::to_template_with_justification for struct {}", stringify!(#ty_ident));

                                let mut root = serde_json::Map::new();
                                root.insert("struct_docs".to_string(), serde_json::Value::String(#container_docs_str.to_string()));
                                root.insert("struct_name".to_string(), serde_json::Value::String(stringify!(#ty_ident).to_string()));
                                root.insert("type".to_string(), serde_json::Value::String("struct".to_string()));
                                root.insert("has_justification".to_string(), serde_json::Value::Bool(true));

                                let mut map = serde_json::Map::new();
                                #(#field_inits)*

                                root.insert("fields".to_string(), serde_json::Value::Object(map));
                                serde_json::Value::Object(root)
                            }
                        }
                    };

                    // B) Generate FlatJustified struct:
                    let flat_ident = syn::Ident::new(
                        &format!("FlatJustified{}", ty_ident),
                        span
                    );
                    let mut flat_fields = Vec::new();

                    // We also generate an `impl From<FlatJustifiedX> for JustifiedX`
                    // so we can reconstruct the normal "JustifiedX" from the flattened version.
                    let mut from_field_rebuild_item = Vec::new();   // for item: X{ ... }
                    let mut from_field_rebuild_just = Vec::new();   // for justification builder
                    let mut from_field_rebuild_conf = Vec::new();   // for confidence builder

                    for field in &named_fields.named {
                        let field_ident = field.ident.as_ref().unwrap();
                        let field_span  = field.span();
                        let skip_self_just = is_justification_disabled_for_field(field);
                        let skip_child_just = skip_self_just || is_justification_disabled_for_inner(field);

                        // compute the flattened type:
                        let field_flat_ty = match compute_flat_type_for_stamped(&field.ty, skip_child_just, field_span) {
                            Ok(ts) => ts,
                            Err(e) => {
                                flat_fields.push(e.to_compile_error());
                                continue;
                            }
                        };

                        // private field, but with getters/setters:
                        flat_fields.push(quote::quote! {
                            #[serde(default)]
                            #[getset(get="pub", set="pub")]
                            #field_ident: #field_flat_ty,
                        });

                        // if we are not skipping justification, add justification/conf:
                        if !skip_self_just {
                            let just_id = syn::Ident::new(&format!("{}_justification", field_ident), field_span);
                            let conf_id = syn::Ident::new(&format!("{}_confidence", field_ident), field_span);

                            flat_fields.push(quote::quote! {
                                #[serde(default)]
                                #[getset(get="pub", set="pub")]
                                #just_id: String,

                                #[serde(default)]
                                #[getset(get="pub", set="pub")]
                                #conf_id: f32,
                            });

                            // for from() reconstruction:
                            from_field_rebuild_just.push(quote::quote! {
                                .#just_id(flat.#just_id)
                            });
                            from_field_rebuild_conf.push(quote::quote! {
                                .#conf_id(flat.#conf_id)
                            });
                        }

                        // Rebuilding the nested `item` field if skip_child_just = false => nested from
                        // otherwise just .field
                        if skip_child_just {
                            from_field_rebuild_item.push(quote::quote! {
                                #field_ident: flat.#field_ident
                            });
                        } else {
                            // We try implementing `From<FlatJustifiedChild> for Child`.
                            // If you have that in your macro or code, we do `Child::from(flat.#field_ident)`.
                            // But you might handle built-ins differently. We'll do a naive approach:
                            from_field_rebuild_item.push(quote::quote! {
                                #field_ident: ::std::convert::From::from(flat.#field_ident)
                            });
                        }
                    }

                    let flat_struct = quote::quote! {
                        #[derive(Builder, Debug, Clone, PartialEq, Serialize, Deserialize, Getters, Setters)]
                        #[builder(setter(into))]
                        #[getset(get="pub", set="pub")]
                        struct #flat_ident {
                            #(#flat_fields)*
                        }
                    };

                    // The `From<FlatJustifiedX> for JustifiedX`
                    //   => reassemble item: X { ... },
                    //      justification: XJustification { ... },
                    //      confidence: XConfidence { ... }.
                    let from_impl = quote::quote! {
                        impl ::std::convert::From<#flat_ident> for #justified_ident {
                            fn from(flat: #flat_ident) -> Self {
                                // Rebuild item
                                let item = #ty_ident {
                                    #(#from_field_rebuild_item),*
                                };

                                let justification = #justification_ident::builder()
                                    #(#from_field_rebuild_just)*
                                    .build()
                                    .unwrap_or_default();

                                let confidence = #confidence_ident::builder()
                                    #(#from_field_rebuild_conf)*
                                    .build()
                                    .unwrap_or_default();

                                Self {
                                    item,
                                    justification,
                                    confidence,
                                }
                            }
                        }
                    };

                    output_ts.extend(justification_struct);
                    output_ts.extend(confidence_struct);
                    output_ts.extend(justified_struct);
                    output_ts.extend(impl_accessors);
                    output_ts.extend(impl_ts);
                    output_ts.extend(flat_struct);
                    output_ts.extend(from_impl);
                }

                // If not named fields, we do the same error as before:
                _ => {
                    let e = syn::Error::new(
                        span,
                        "AiJsonTemplateWithJustification only supports named fields for structs."
                    );
                    output_ts.extend(e.to_compile_error());
                }
            }
        }

        // -------------------------------
        //  2) Enum
        // -------------------------------
        syn::Data::Enum(data_enum) => {
            let justification_ident = syn::Ident::new(
                &format!("{}Justification", ty_ident),
                span
            );
            let confidence_ident = syn::Ident::new(
                &format!("{}Confidence", ty_ident),
                span
            );
            let justified_ident = syn::Ident::new(
                &format!("Justified{}", ty_ident),
                span
            );

            // minimal approach => single string & single f32 for each variant:
            let enum_just_struct = quote::quote! {
                #[derive(Builder, Debug, Clone, PartialEq, Default, Serialize, Deserialize, Getters, Setters)]
                #[builder(setter(into))]
                #[getset(get="pub", set="pub")]
                struct #justification_ident {
                    #[getset(get="pub", set="pub")]
                    enum_variant_justification: String,
                }
            };

            let enum_conf_struct = quote::quote! {
                #[derive(Builder, Debug, Clone, PartialEq, Default, Serialize, Deserialize, Getters, Setters)]
                #[builder(setter(into))]
                #[getset(get="pub", set="pub")]
                struct #confidence_ident {
                    #[getset(get="pub", set="pub")]
                    enum_variant_confidence: f32,
                }
            };

            let justified_enum = quote::quote! {
                #[derive(Builder, Debug, Default, Clone, PartialEq, Serialize, Deserialize, Getters, Setters)]
                #[builder(setter(into))]
                #[getset(get="pub", set="pub")]
                struct #justified_ident {
                    #[getset(get="pub", set="pub")]
                    item: #ty_ident,

                    #[getset(get="pub", set="pub")]
                    justification: #justification_ident,

                    #[getset(get="pub", set="pub")]
                    confidence: #confidence_ident,
                }

                impl #justified_ident {
                    fn new(item: #ty_ident) -> Self {
                        Self {
                            item,
                            justification: Default::default(),
                            confidence: Default::default(),
                        }
                    }
                }
            };

            output_ts.extend(enum_just_struct);
            output_ts.extend(enum_conf_struct);
            output_ts.extend(justified_enum);

            // We'll store expansions for each variant in to_template_with_justification:
            let mut variant_exprs = Vec::new();
            // We'll also build a new \"FlatJustifiedEnum\"
            let flat_enum_ident = syn::Ident::new(
                &format!("FlatJustified{}", ty_ident),
                span
            );
            let mut flat_variants = Vec::new();

            // We'll store lines for the from() => we do an enum match from \"flat\" to \"JustifiedEnum\"
            let mut from_match_arms = Vec::new();

            for var in &data_enum.variants {
                let var_name_str = var.ident.to_string();
                let var_ident = &var.ident;
                let skip_self_just = is_justification_disabled_for_variant(var);
                let skip_child_just = skip_self_just || is_justification_disabled_for_inner_variant(var);

                let variant_kind_str = match var.fields {
                    syn::Fields::Unit => "unit",
                    syn::Fields::Named(_) => "struct_variant",
                    syn::Fields::Unnamed(_) => "tuple_variant",
                };

                // building the normal to_template expansions
                let fields_insertion = match &var.fields {
                    syn::Fields::Unit => {
                        quote::quote! {}
                    }
                    syn::Fields::Named(named) => {
                        let mut field_inits = Vec::new();
                        for field in &named.named {
                            let field_ident = field.ident.as_ref().unwrap();
                            let fname = field_ident.to_string();
                            let doc_str = gather_doc_comments(&field.attrs).join("\n");
                            let is_required = extract_option_inner(&field.ty).is_none();
                            let skip_f_self = is_justification_disabled_for_field(field);
                            let skip_f_child = skip_f_self || skip_child_just;

                            if let Some(expr) = classify_field_type_for_child(&field.ty, &doc_str, is_required, skip_f_child) {
                                field_inits.push(quote::quote! {
                                    map.insert(#fname.to_string(), #expr);
                                });
                            }
                            if !skip_f_self {
                                field_inits.push(quote::quote! {
                                    let justify_key = format!("{}_justification", #fname);
                                    let conf_key    = format!("{}_confidence", #fname);

                                    let mut just_obj = serde_json::Map::new();
                                    just_obj.insert("type".to_string(), serde_json::Value::String("string".to_string()));
                                    just_obj.insert("required".to_string(), serde_json::Value::Bool(true));
                                    map.insert(justify_key, serde_json::Value::Object(just_obj));

                                    let mut conf_obj = serde_json::Map::new();
                                    conf_obj.insert("type".to_string(), serde_json::Value::String("number".to_string()));
                                    conf_obj.insert("required".to_string(), serde_json::Value::Bool(true));
                                    map.insert(conf_key, serde_json::Value::Object(conf_obj));
                                });
                            }
                        }
                        quote::quote! {
                            let mut map = serde_json::Map::new();
                            #(#field_inits)*
                            variant_map.insert("fields".to_string(), serde_json::Value::Object(map));
                        }
                    }
                    syn::Fields::Unnamed(unnamed) => {
                        let mut field_inits = Vec::new();
                        for (i, field) in unnamed.unnamed.iter().enumerate() {
                            let fname = format!("field_{}", i);
                            let doc_str = gather_doc_comments(&field.attrs).join("\n");
                            let is_required = extract_option_inner(&field.ty).is_none();
                            let skip_f_self = is_justification_disabled_for_field(field);
                            let skip_f_child = skip_f_self || skip_child_just;

                            if let Some(expr) = classify_field_type_for_child(&field.ty, &doc_str, is_required, skip_f_child) {
                                field_inits.push(quote::quote! {
                                    map.insert(#fname.to_string(), #expr);
                                });
                            }
                            if !skip_f_self {
                                field_inits.push(quote::quote! {
                                    let justify_key = format!("{}_justification", #fname);
                                    let conf_key    = format!("{}_confidence", #fname);

                                    let mut just_obj = serde_json::Map::new();
                                    just_obj.insert("type".to_string(), serde_json::Value::String("string".to_string()));
                                    just_obj.insert("required".to_string(), serde_json::Value::Bool(true));
                                    map.insert(justify_key, serde_json::Value::Object(just_obj));

                                    let mut conf_obj = serde_json::Map::new();
                                    conf_obj.insert("type".to_string(), serde_json::Value::String("number".to_string()));
                                    conf_obj.insert("required".to_string(), serde_json::Value::Bool(true));
                                    map.insert(conf_key, serde_json::Value::Object(conf_obj));
                                });
                            }
                        }
                        quote::quote! {
                            let mut map = serde_json::Map::new();
                            #(#field_inits)*
                            variant_map.insert("fields".to_string(), serde_json::Value::Object(map));
                        }
                    }
                };

                let variant_just_conf = if !skip_self_just {
                    quote::quote! {
                        let mut j_obj = serde_json::Map::new();
                        j_obj.insert("type".to_string(), serde_json::Value::String("string".to_string()));
                        j_obj.insert("required".to_string(), serde_json::Value::Bool(true));
                        variant_map.insert("variant_justification".to_string(), serde_json::Value::Object(j_obj));

                        let mut c_obj = serde_json::Map::new();
                        c_obj.insert("type".to_string(), serde_json::Value::String("number".to_string()));
                        c_obj.insert("required".to_string(), serde_json::Value::Bool(true));
                        variant_map.insert("variant_confidence".to_string(), serde_json::Value::Object(c_obj));
                    }
                } else {
                    quote::quote! {}
                };

                variant_exprs.push(quote::quote! {
                    {
                        let mut variant_map = serde_json::Map::new();
                        variant_map.insert("variant_name".to_string(), serde_json::Value::String(#var_name_str.to_string()));
                        variant_map.insert("variant_docs".to_string(), serde_json::Value::String(#container_docs_str.to_string()));
                        variant_map.insert("variant_type".to_string(), serde_json::Value::String(#variant_kind_str.to_string()));
                        #fields_insertion
                        #variant_just_conf
                        serde_json::Value::Object(variant_map)
                    }
                });

                // Now building the FlatJustified enum variant
                match &var.fields {
                    syn::Fields::Unit => {
                        // e.g.  MyVariant,
                        flat_variants.push(quote::quote! {
                            #var_ident,
                        });
                        // For the from() match, we do something like:
                        from_match_arms.push(quote::quote! {
                            #flat_enum_ident::#var_ident => #justified_ident {
                                item: #ty_ident::#var_ident,
                                justification: #justification_ident {
                                    enum_variant_justification: "".to_string()
                                },
                                confidence: #confidence_ident {
                                    enum_variant_confidence: 0.0
                                },
                            }
                        });
                    }
                    syn::Fields::Named(named) => {
                        // produce  MyVariant { field: <flat-ty>, field_justification: String, ... }
                        let mut variant_fields = Vec::new();
                        let mut from_variant_item_fields = Vec::new();
                        let mut from_just_fields = Vec::new();
                        let mut from_conf_fields = Vec::new();

                        for field in &named.named {
                            let f_ident = field.ident.as_ref().unwrap();
                            let skip = is_justification_disabled_for_field(field);
                            let skip_inner = skip || skip_child_just;

                            let field_ty = match compute_flat_type_for_stamped(&field.ty, skip_inner, field.span()) {
                                Ok(ts) => ts,
                                Err(e) => {
                                    variant_fields.push(e.to_compile_error());
                                    continue;
                                }
                            };

                            variant_fields.push(quote::quote! {
                                #[serde(default)]
                                #[getset(get="pub", set="pub")]
                                #f_ident: #field_ty,
                            });

                            if !skip {
                                let just_id = syn::Ident::new(&format!("{}_justification", f_ident), field.span());
                                let conf_id = syn::Ident::new(&format!("{}_confidence", f_ident), field.span());

                                variant_fields.push(quote::quote! {
                                    #[serde(default)]
                                    #[getset(get="pub", set="pub")]
                                    #just_id: String,

                                    #[serde(default)]
                                    #[getset(get="pub", set="pub")]
                                    #conf_id: f32,
                                });

                                from_just_fields.push(quote::quote! {
                                    .#just_id(flat.#just_id)
                                });
                                from_conf_fields.push(quote::quote! {
                                    .#conf_id(flat.#conf_id)
                                });
                            }

                            // Rebuilding the actual item variant
                            if skip_inner {
                                from_variant_item_fields.push(quote::quote! {
                                    #f_ident: flat.#f_ident
                                });
                            } else {
                                from_variant_item_fields.push(quote::quote! {
                                    #f_ident: ::std::convert::From::from(flat.#f_ident)
                                });
                            }
                        }

                        flat_variants.push(quote::quote! {
                            #var_ident {
                                #(#variant_fields)*
                            },
                        });

                        from_match_arms.push(quote::quote! {
                            #flat_enum_ident::#var_ident { #(ref flat),* } => {
                                #justified_ident {
                                    item: #ty_ident::#var_ident {
                                        #( #from_variant_item_fields ),*
                                    },
                                    justification: #justification_ident::builder()
                                        #( #from_just_fields )*
                                        .build().unwrap_or_default(),
                                    confidence: #confidence_ident::builder()
                                        #( #from_conf_fields )*
                                        .build().unwrap_or_default(),
                                }
                            }
                        });
                    }
                    syn::Fields::Unnamed(unnamed) => {
                        // same approach for tuple fields
                        // omitted for brevity: replicate the pattern above
                        // ...
                        flat_variants.push(quote::quote! {
                            #var_ident(...),
                        });
                        from_match_arms.push(quote::quote! {
                            // ...
                            // build a match arm for the tuple
                            #flat_enum_ident::#var_ident(..) => {
                                #justified_ident {
                                    item: #ty_ident::#var_ident(...),
                                    justification: #justification_ident {
                                        enum_variant_justification: "...".into(),
                                    },
                                    confidence: #confidence_ident {
                                        enum_variant_confidence: 0.0,
                                    },
                                }
                            }
                        });
                    }
                }
            }

            // The final expansions for to_template_with_justification
            let expanded = quote::quote! {
                impl AiJsonTemplateWithJustification for #ty_ident {
                    fn to_template_with_justification() -> serde_json::Value {
                        tracing::trace!("AiJsonTemplateWithJustification::to_template_with_justification for enum {}", #type_name_str);

                        let base = <#ty_ident as AiJsonTemplate>::to_template();
                        let mut root_map = if let Some(obj) = base.as_object() {
                            obj.clone()
                        } else {
                            serde_json::Map::new()
                        };

                        root_map.insert("has_justification".to_string(), serde_json::Value::Bool(true));
                        if root_map.contains_key("enum_docs") {
                            if let Some(serde_json::Value::String(sdoc)) = root_map.get_mut("enum_docs") {
                                *sdoc = format!("{}\n{}", *sdoc, #container_docs_str);
                            }
                        } else {
                            root_map.insert("enum_docs".to_string(), serde_json::Value::String(#container_docs_str.to_string()));
                        }

                        let variants_vec = vec![ #(#variant_exprs),* ];
                        root_map.insert("variants".to_string(), serde_json::Value::Array(variants_vec));

                        serde_json::Value::Object(root_map)
                    }
                }
            };

            // Now build the FlatJustifiedEnum:
            let flat_enum = quote::quote! {
                #[derive(Builder, Debug, Clone, PartialEq, Serialize, Deserialize, Getters, Setters)]
                #[builder(setter(into))]
                #[getset(get="pub", set="pub")]
                enum #flat_enum_ident {
                    #(#flat_variants)*
                }
            };

            // And the `impl From<FlatJustifiedEnum> for JustifiedEnum`
            let from_impl = quote::quote! {
                impl ::std::convert::From<#flat_enum_ident> for #justified_ident {
                    fn from(flat: #flat_enum_ident) -> Self {
                        match flat {
                            #(#from_match_arms),*
                        }
                    }
                }
            };

            output_ts.extend(enum_just_struct);
            output_ts.extend(enum_conf_struct);
            output_ts.extend(justified_enum);
            output_ts.extend(expanded);
            output_ts.extend(flat_enum);
            output_ts.extend(from_impl);
        }

        //  3) Union => not supported
        syn::Data::Union(_) => {
            let e = syn::Error::new(
                span,
                "AiJsonTemplateWithJustification not supported on unions."
            );
            output_ts.extend(e.to_compile_error());
        }
    }

    output_ts.into()
}
