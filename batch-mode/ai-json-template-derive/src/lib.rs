// ---------------- [ File: ai-json-template-derive/src/lib.rs ]
#[macro_use] mod imports; use imports::*;

xp!{gather_doc_comments}
xp!{comma_separated_expression}
xp!{classify_field_type}

#[proc_macro_derive(AiJsonTemplate)]
pub fn derive_ai_json_template(input: TokenStream) -> TokenStream {
    trace!("Entering derive_ai_json_template macro.");

    let ast = parse_macro_input!(input as DeriveInput);
    let struct_ident = &ast.ident;
    let struct_span  = ast.span();
    let struct_name_str = struct_ident.to_string();
    trace!("Processing struct: {}", struct_name_str);

    // Gather doc comments from the struct itself
    let struct_docs_vec = gather_doc_comments(&ast.attrs);
    let struct_docs_str = struct_docs_vec.join("\n");
    trace!("Gathered struct doc comments = {:?}", struct_docs_vec);

    let fields = match &ast.data {
        Data::Struct(DataStruct { fields: Fields::Named(named), .. }) => {
            trace!("Struct has named fields.");
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
        trace!("Analyzing field: {}", field_name_str);

        // doc comments for the field
        let field_docs = gather_doc_comments(&field.attrs).join("\n");
        trace!("Field docs => {:?}", field_docs);

        let ty = &field.ty;
        let type_q = quote!(#ty).to_string();
        trace!("Field type => {}", type_q);

        if let Some(expr) = classify_field_type(ty, &field_docs) {
            field_inits.push(quote! {
                map.insert(#field_name_str.to_string(), #expr);
            });
        } else {
            let err_msg = format!("Unsupported field type for AiJsonTemplate: {}", type_q);
            trace!("ERROR: {}", err_msg);
            let err = syn::Error::new(ty.span(), err_msg);
            return err.to_compile_error().into();
        }
    }

    let expanded = quote! {
        impl AiJsonTemplate for #struct_ident {
            fn to_template() -> serde_json::Value {
                tracing::trace!("AiJsonTemplate::to_template for struct {}", #struct_name_str);

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

    trace!("Exiting derive_ai_json_template macro for {}", struct_name_str);
    expanded.into()
}
