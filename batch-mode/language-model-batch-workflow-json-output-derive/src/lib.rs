// ---------------- [ File: src/lib.rs ]
#[macro_use] mod imports; use imports::*;

xp!{gather_doc_comments}
xp!{comma_separated_expression}

#[proc_macro_derive(AiJsonTemplate)]
pub fn derive_ai_json_template(input: TokenStream) -> TokenStream {
    eprintln!("Entering derive_ai_json_template macro.");

    let ast = parse_macro_input!(input as DeriveInput);
    let struct_ident = &ast.ident;
    let struct_span  = ast.span();
    let struct_name_str = struct_ident.to_string();
    eprintln!("Processing struct: {}", struct_name_str);

    // Gather doc comments from the struct itself
    let struct_docs_vec = gather_doc_comments(&ast.attrs);
    let struct_docs_str = struct_docs_vec.join("\n");
    eprintln!("Gathered struct doc comments = {:?}", struct_docs_vec);

    let fields = match &ast.data {
        Data::Struct(DataStruct { fields: Fields::Named(named), .. }) => {
            eprintln!("Struct has named fields.");
            &named.named
        },
        _ => {
            let err = syn::Error::new(
                struct_span,
                "AiJsonTemplate derive only supports a named struct."
            );
            return err.to_compile_error().into();
        }
    };

    let mut field_inits = Vec::new();
    for field in fields {
        let field_ident = match &field.ident {
            Some(id) => id,
            None => {
                let err = syn::Error::new(
                    field.span(),
                    "Unnamed fields are not supported by AiJsonTemplate."
                );
                return err.to_compile_error().into();
            }
        };
        let field_name_str = field_ident.to_string();
        eprintln!("Analyzing field: {}", field_name_str);

        // doc comments for the field
        let field_docs = gather_doc_comments(&field.attrs).join("\n");
        eprintln!("Field docs => {:?}", field_docs);

        let ty = &field.ty;
        let type_q = quote!(#ty).to_string();
        eprintln!("Field type => {}", type_q);

        if let Some(expr) = classify_field_type(ty, &field_docs) {
            field_inits.push(quote! {
                map.insert(#field_name_str.to_string(), #expr);
            });
        } else {
            let err_msg = format!("Unsupported field type for AiJsonTemplate: {}", type_q);
            eprintln!("ERROR: {}", err_msg);
            let err = syn::Error::new(ty.span(), err_msg);
            return err.to_compile_error().into();
        }
    }

    let expanded = quote! {
        impl AiJsonTemplate for #struct_ident {
            fn to_template() -> serde_json::Value {
                eprintln!("AiJsonTemplate::to_template for struct {}", #struct_name_str);

                let mut root = serde_json::Map::new();
                root.insert("struct_docs".to_string(), serde_json::Value::String(#struct_docs_str.to_string()));
                root.insert("struct_name".to_string(), serde_json::Value::String(#struct_name_str.to_string()));

                let mut map = serde_json::Map::new();
                #(#field_inits)*

                root.insert("fields".to_string(), serde_json::Value::Object(map));
                serde_json::Value::Object(root)
            }
        }
    };

    eprintln!("Exiting derive_ai_json_template macro for {}", struct_name_str);
    expanded.into()
}

fn classify_field_type(ty: &syn::Type, doc_str: &str) -> Option<proc_macro2::TokenStream> {
    eprintln!("classify_field_type => doc_str={:?}, ty=?", doc_str);
    let ty_str = quote!(#ty).to_string().replace(' ', "");
    let doc_lit = proc_macro2::Literal::string(doc_str);

    match ty_str.as_str() {
        "String" => {
            eprintln!("Field is a String. Required = true");
            Some(quote! {
                {
                    let mut obj = serde_json::Map::new();
                    obj.insert("type".to_string(), serde_json::Value::String("string".to_string()));
                    obj.insert("docs".to_string(), serde_json::Value::String(#doc_lit.to_string()));
                    obj.insert("required".to_string(), serde_json::Value::Bool(true));
                    serde_json::Value::Object(obj)
                }
            })
        },
        "Vec<String>" => {
            eprintln!("Field is a Vec<String>. Required = true");
            Some(quote! {
                {
                    let mut obj = serde_json::Map::new();
                    obj.insert("type".to_string(), serde_json::Value::String("array_of_strings".to_string()));
                    obj.insert("docs".to_string(), serde_json::Value::String(#doc_lit.to_string()));
                    obj.insert("required".to_string(), serde_json::Value::Bool(true));
                    serde_json::Value::Object(obj)
                }
            })
        },
        "Option<String>" => {
            eprintln!("Field is an Option<String>. Required = false");
            Some(quote! {
                {
                    let mut obj = serde_json::Map::new();
                    obj.insert("type".to_string(), serde_json::Value::String("string".to_string()));
                    obj.insert("docs".to_string(), serde_json::Value::String(#doc_lit.to_string()));
                    obj.insert("required".to_string(), serde_json::Value::Bool(false));
                    serde_json::Value::Object(obj)
                }
            })
        },
        _ => {
            // assume it's a nested struct implementing AiJsonTemplate
            eprintln!("Treating as nested struct => AiJsonTemplate");
            Some(quote! {
                {
                    let nested = <#ty as AiJsonTemplate>::to_template();
                    let mut obj = serde_json::Map::new();
                    obj.insert("type".to_string(), serde_json::Value::String("nested_struct".to_string()));
                    obj.insert("docs".to_string(), serde_json::Value::String(#doc_lit.to_string()));
                    obj.insert("required".to_string(), serde_json::Value::Bool(true));
                    obj.insert("nested_template".to_string(), nested);
                    serde_json::Value::Object(obj)
                }
            })
        }
    }
}

/// A parser function for parentheses style: `#[doc("some string")]`
fn parse_as_litstr_paren(stream: ParseStream) -> syn::Result<LitStr> {
    eprintln!("parse_as_litstr_paren => trying to parse parentheses style doc attribute");
    stream.parse::<LitStr>()
}

/// A parser function for name-value style: `#[doc = "some string"]`
fn parse_as_litstr_eq(stream: ParseStream) -> syn::Result<LitStr> {
    eprintln!("parse_as_litstr_eq => trying to parse name-value style doc attribute");
    let _: Token![=] = stream.parse()?;
    stream.parse::<LitStr>()
}
