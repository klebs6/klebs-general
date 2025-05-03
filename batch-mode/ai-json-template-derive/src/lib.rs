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

#[proc_macro_derive(AiJsonTemplateWithJustification, attributes(doc, justify, justify_inner))]
pub fn derive_ai_json_template_with_justification(input: TokenStream) -> TokenStream {
    use syn::{parse_macro_input, DeriveInput, Data, Fields, spanned::Spanned};

    let ast = parse_macro_input!(input as DeriveInput);
    let span = ast.span();
    let ty_ident = &ast.ident;
    let type_name_str = ty_ident.to_string();

    let container_docs_vec = gather_doc_comments(&ast.attrs);
    let container_docs_str = container_docs_vec.join("\n");
    tracing::trace!("Deriving AiJsonTemplateWithJustification for {}", ty_ident);

    // We'll append disclaimers at the container level as well:
    let disclaimers = "\nIMPORTANT:\n\
        - Provide all justification/confidence fields with correct data.\n\
        - Numeric fields must be real JSON numbers (no quotes), typically in [0..1] if it is a confidence.\n\
        - For optional fields, either fill them or set them to null.\n\
        - If this is an enum, pick exactly one variant. Justify that choice.\n\
        - Return strictly one JSON object with no extra keys.\n";

    let container_docs_enhanced = format!("{}\n{}", container_docs_str, disclaimers);

    let mut output_ts = proc_macro2::TokenStream::new();

    match &ast.data {
        // Named structs
        Data::Struct(data_struct) => {
            match &data_struct.fields {
                syn::Fields::Named(named_fields) => {
                    // same logic as existing: create Justification & Confidence structs, plus the wrapper
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

                    let mut just_fields = Vec::new();
                    let mut conf_fields = Vec::new();
                    let mut errs = quote::quote!();
                    let mut field_mappings = Vec::new();

                    gather_justification_and_confidence_fields(
                        named_fields,
                        &mut just_fields,
                        &mut conf_fields,
                        &mut errs,
                        &mut field_mappings,
                    );
                    output_ts.extend(errs);

                    let justification_struct = quote::quote! {
                        #[derive(Builder, Debug, Clone, PartialEq, Default, Serialize, Deserialize, Getters, Setters)]
                        #[builder(setter(into))]
                        #[getset(get="pub", set="pub")]
                        pub struct #justification_ident {
                            #(#just_fields),*
                        }
                    };
                    let confidence_struct = quote::quote! {
                        #[derive(Builder, Debug, Clone, PartialEq, Default, Serialize, Deserialize, Getters, Setters)]
                        #[builder(setter(into))]
                        #[getset(get="pub", set="pub")]
                        pub struct #confidence_ident {
                            #(#conf_fields),*
                        }
                    };
                    let justified_struct = quote::quote! {
                        #[derive(Builder, Debug, Default, Clone, PartialEq, Serialize, Deserialize, Getters, Setters)]
                        #[builder(setter(into))]
                        #[getset(get="pub", set="pub")]
                        pub struct #justified_ident {
                            item: #ty_ident,
                            justification: #justification_ident,
                            confidence: #confidence_ident,
                        }

                        impl #justified_ident {
                            pub fn new(item: #ty_ident) -> Self {
                                Self {
                                    item,
                                    justification: Default::default(),
                                    confidence: Default::default(),
                                }
                            }
                        }
                    };

                    let (item_acc, just_acc, conf_acc) =
                        gather_item_accessors(named_fields, ty_ident, &field_mappings);

                    let impl_accessors = quote::quote! {
                        impl #justified_ident {
                            #(#item_acc)*
                            #(#just_acc)*
                            #(#conf_acc)*
                        }
                    };

                    // For the container-level disclaimers:
                    let container_msg = format!(
                        "{}\n(This struct has justification & confidence for each field. Fill them carefully, set numeric fields as real JSON numbers, etc.)",
                        container_docs_enhanced
                    );

                    // We'll build the to_template_with_justification similarly to original
                    let mut field_inits = Vec::new();
                    for field in &named_fields.named {
                        let field_ident = field.ident.as_ref().unwrap();
                        let doc_str = gather_doc_comments(&field.attrs).join("\n");
                        let field_name_str = field_ident.to_string();

                        // logic from classify_field_type_for_child
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
                                just_obj.insert(
                                    "generation_instructions".to_string(),
                                    serde_json::Value::String(
                                        format!("Explain or justify your choice for the field '{}'. Must be a non-empty string.", #field_name_str)
                                    )
                                );
                                map.insert(justify_key, serde_json::Value::Object(just_obj));

                                let mut conf_obj = serde_json::Map::new();
                                conf_obj.insert("type".to_string(), serde_json::Value::String("number".to_string()));
                                conf_obj.insert("required".to_string(), serde_json::Value::Bool(true));
                                conf_obj.insert(
                                    "generation_instructions".to_string(),
                                    serde_json::Value::String(
                                        format!("Confidence in '{}', as a real JSON number in [0..1].", #field_name_str)
                                    )
                                );
                                map.insert(conf_key, serde_json::Value::Object(conf_obj));
                            });
                        }
                    }

                    let impl_ts = quote::quote! {
                        impl AiJsonTemplateWithJustification for #ty_ident {
                            fn to_template_with_justification() -> serde_json::Value {
                                tracing::trace!("AiJsonTemplateWithJustification::to_template_with_justification for struct {}", stringify!(#ty_ident));

                                let mut root = serde_json::Map::new();
                                root.insert("struct_docs".to_string(), serde_json::Value::String(#container_msg.to_string()));
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

                    output_ts.extend(justification_struct);
                    output_ts.extend(confidence_struct);
                    output_ts.extend(justified_struct);
                    output_ts.extend(impl_accessors);
                    output_ts.extend(impl_ts);
                }
                _ => {
                    let e = syn::Error::new(
                        span,
                        "AiJsonTemplateWithJustification only supports named fields for structs."
                    );
                    output_ts.extend(e.to_compile_error());
                }
            }
        },

        // Enums
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

            // Minimal approach => a single textual justification and single numeric confidence
            let enum_just_struct = quote::quote! {
                #[derive(Builder, Debug, Clone, PartialEq, Default, Serialize, Deserialize, Getters, Setters)]
                #[builder(setter(into))]
                #[getset(get="pub", set="pub")]
                pub struct #justification_ident {
                    enum_variant_justification: String,
                }
            };
            let enum_conf_struct = quote::quote! {
                #[derive(Builder, Debug, Clone, PartialEq, Default, Serialize, Deserialize, Getters, Setters)]
                #[builder(setter(into))]
                #[getset(get="pub", set="pub")]
                pub struct #confidence_ident {
                    enum_variant_confidence: f32,
                }
            };
            let justified_enum = quote::quote! {
                #[derive(Builder, Debug, Default, Clone, PartialEq, Serialize, Deserialize, Getters, Setters)]
                #[builder(setter(into))]
                #[getset(get="pub", set="pub")]
                pub struct #justified_ident {
                    item: #ty_ident,
                    justification: #justification_ident,
                    confidence: #confidence_ident,
                }

                impl #justified_ident {
                    pub fn new(item: #ty_ident) -> Self {
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

            // Next, the expansions for each variant in to_template_with_justification
            let mut variant_exprs = Vec::new();
            for var in &data_enum.variants {
                let var_name_str = var.ident.to_string();
                let var_docs = gather_doc_comments(&var.attrs).join("\n");
                let skip_self_just = is_justification_disabled_for_variant(var);
                let skip_child_just = skip_self_just || is_justification_disabled_for_inner_variant(var);

                let variant_kind_str = match var.fields {
                    syn::Fields::Unit => "unit",
                    syn::Fields::Named(_) => "struct_variant",
                    syn::Fields::Unnamed(_) => "tuple_variant",
                };

                let fields_insertion = match &var.fields {
                    syn::Fields::Unit => quote::quote! {},
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
                                    just_obj.insert("generation_instructions".to_string(),
                                        serde_json::Value::String(
                                            format!("Explain or justify your choice for the field '{}'. Non-empty string required.", #fname)
                                        )
                                    );
                                    map.insert(justify_key, serde_json::Value::Object(just_obj));

                                    let mut conf_obj = serde_json::Map::new();
                                    conf_obj.insert("type".to_string(), serde_json::Value::String("number".to_string()));
                                    conf_obj.insert("required".to_string(), serde_json::Value::Bool(true));
                                    conf_obj.insert("generation_instructions".to_string(),
                                        serde_json::Value::String(
                                            format!("Confidence in '{}', as a real JSON number in [0..1].", #fname)
                                        )
                                    );
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
                                    just_obj.insert("generation_instructions".to_string(),
                                        serde_json::Value::String(
                                            format!("Explain or justify your choice for the field '{}'. Non-empty string required.", #fname)
                                        )
                                    );
                                    map.insert(justify_key, serde_json::Value::Object(just_obj));

                                    let mut conf_obj = serde_json::Map::new();
                                    conf_obj.insert("type".to_string(), serde_json::Value::String("number".to_string()));
                                    conf_obj.insert("required".to_string(), serde_json::Value::Bool(true));
                                    conf_obj.insert("generation_instructions".to_string(),
                                        serde_json::Value::String(
                                            format!("Confidence in '{}', as a real JSON number in [0..1].", #fname)
                                        )
                                    );
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
                        j_obj.insert("generation_instructions".to_string(),
                            serde_json::Value::String(
                                format!("Explain your choice for variant '{}'. Non-empty justification string required.", #var_name_str)
                            )
                        );
                        variant_map.insert("variant_justification".to_string(), serde_json::Value::Object(j_obj));

                        let mut c_obj = serde_json::Map::new();
                        c_obj.insert("type".to_string(), serde_json::Value::String("number".to_string()));
                        c_obj.insert("required".to_string(), serde_json::Value::Bool(true));
                        c_obj.insert("generation_instructions".to_string(),
                            serde_json::Value::String(
                                format!("Confidence in choosing variant '{}', as a real JSON number in [0..1].", #var_name_str)
                            )
                        );
                        variant_map.insert("variant_confidence".to_string(), serde_json::Value::Object(c_obj));
                    }
                } else {
                    quote::quote! {}
                };

                let expr = quote! {
                    {
                        let mut variant_map = serde_json::Map::new();
                        variant_map.insert("variant_name".to_string(), serde_json::Value::String(#var_name_str.to_string()));
                        variant_map.insert("variant_docs".to_string(), serde_json::Value::String(#var_docs.to_string()));
                        variant_map.insert("variant_type".to_string(), serde_json::Value::String(#variant_kind_str.to_string()));

                        #fields_insertion
                        #variant_just_conf
                        serde_json::Value::Object(variant_map)
                    }
                };
                variant_exprs.push(expr);
            }

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

                        // Insert disclaimers
                        root_map.insert("has_justification".to_string(), serde_json::Value::Bool(true));
                        let doc_plus = format!("{}\n(This enum has justification. Choose exactly one variant. Do not mention unselected variants. Provide numeric fields as real JSON numbers, etc.)", #container_docs_enhanced);
                        if root_map.contains_key("enum_docs") {
                            if let Some(serde_json::Value::String(sdoc)) = root_map.get_mut("enum_docs") {
                                *sdoc = format!("{}\n{}", *sdoc, doc_plus);
                            }
                        } else {
                            root_map.insert("enum_docs".to_string(), serde_json::Value::String(doc_plus));
                        }

                        let variants_vec = vec![ #(#variant_exprs),* ];
                        root_map.insert("variants".to_string(), serde_json::Value::Array(variants_vec));

                        serde_json::Value::Object(root_map)
                    }
                }
            };
            output_ts.extend(expanded);
        },

        // Union => not supported
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
