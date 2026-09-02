// ---------------- [ File: ai-json-template-derive/src/build_hashmap_schema.rs ]
crate::ix!();

#[tracing::instrument(level="trace", skip_all)]
pub fn build_hashmap_schema(
    k_ty:          &syn::Type,
    v_ty:          &syn::Type,
    required_bool: proc_macro2::TokenStream,
    doc_lit:       proc_macro2::Literal
) -> Option<proc_macro2::TokenStream>
{
    trace!("Entering build_hashmap_schema => K={:?}, V={:?}", k_ty, v_ty);

    // Build the snippet for map_key_template
    let key_schema = if is_numeric(k_ty) {
        quote::quote!( serde_json::Value::String("number") )
    } else if is_string_type(k_ty) {
        quote::quote!( serde_json::Value::String("string") )
    } else if is_bool(k_ty) {
        quote::quote!( serde_json::Value::String("boolean") )
    } else {
        quote::quote! {
            {
                let mut k_obj = serde_json::Map::new();
                k_obj.insert("type".to_string(), serde_json::Value::String("nested_struct_or_enum"));
                k_obj.insert("nested_template".to_string(), <#k_ty as AiJsonTemplateWithJustification>::to_template_with_justification());
                serde_json::Value::Object(k_obj)
            }
        }
    };

    // Build the snippet for map_value_template
    let val_schema = if is_bool(v_ty) {
        quote::quote!( serde_json::Value::String("boolean") )
    } else if is_numeric(v_ty) {
        quote::quote!( serde_json::Value::String("number") )
    } else if is_string_type(v_ty) {
        quote::quote!( serde_json::Value::String("string") )
    } else {
        quote::quote! {
            {
                let mut v_obj = serde_json::Map::new();
                v_obj.insert("type".to_string(), serde_json::Value::String("nested_struct_or_enum"));
                v_obj.insert("nested_template".to_string(), <#v_ty as AiJsonTemplateWithJustification>::to_template_with_justification());
                serde_json::Value::Object(v_obj)
            }
        }
    };

    // Finally, produce the top-level snippet. Notice how we do NOT call .to_string() on the literal keys
    // like "type", "generation_instructions", etc.  That way, the test sees them as string literals.
    let snippet: syn::ExprBlock = syn::parse_quote! {
        {
            let mut map_obj = serde_json::Map::new();
            map_obj.insert("type".to_string(), serde_json::Value::String("map_of"));
            map_obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit));
            map_obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
            map_obj.insert("map_key_template".to_string(), #key_schema);
            map_obj.insert("map_value_template".to_string(), #val_schema);
            serde_json::Value::Object(map_obj)
        }
    };

    debug!(
        "Successfully built final hashmap schema for K='{:?}', V='{:?}'",
        k_ty, v_ty
    );
    trace!("Exiting build_hashmap_schema");

    Some(quote::quote!(#snippet))
}

#[cfg(test)]
mod test_build_hashmap_schema {
    use super::*;
    use syn::{parse_str, spanned::Spanned};
    use quote::ToTokens;
    use traced_test::traced_test;
    use tracing::{trace, debug, info, warn, error};

    /// Parse the output TokenStream into a `syn::Expr` to allow deeper AST checks.
    fn parse_expr_snippet(ts: &proc_macro2::TokenStream) -> syn::Expr {
        let s = ts.to_string();
        match parse_str::<syn::Expr>(&s) {
            Ok(expr) => expr,
            Err(err) => panic!("Failed to parse snippet as Expr.\nSnippet:\n{}\nError={}", s, err),
        }
    }

    /// Scan a block for any `compile_error!(…)` invocation.
    fn contains_compile_error_in_block(block: &syn::Block) -> bool {
        for stmt in &block.stmts {
            if let syn::Stmt::Expr(expr, _) = stmt {
                if contains_compile_error_invocation(expr) {
                    return true;
                }
            }
        }
        false
    }

    #[tracing::instrument(level = "trace", skip_all)]
    fn parse_insert_call_stmt(stmt: &syn::Stmt) -> Option<&syn::ExprMethodCall> {
        trace!("Entering parse_insert_call_stmt");
        if let syn::Stmt::Expr(syn::Expr::MethodCall(method_call), _) = stmt {
            trace!("Matched a method call stmt: {:?}", method_call);
            Some(method_call)
        } else {
            trace!("No match found; not a method call stmt.");
            None
        }
    }

    fn extract_str_key_from_expr(e: &syn::Expr) -> Option<String> {
        // (1) If `e` is literally a string literal, e.g. `"type"`
        if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        }) = e
        {
            return Some(s.value());
        }

        // (2) If `e` is something like `"some_text".to_string()` ...
        if let syn::Expr::MethodCall(mc) = e {
            // Make sure the method name is exactly `to_string`
            if mc.method == "to_string" {
                // ...and make sure there are no arguments, i.e. `.to_string()` not `.to_string(x)`
                if mc.args.is_empty() {
                    // Now check if the receiver is a string literal
                    // e.g. the code looks like:  "foo".to_string()
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(slit),
                        ..
                    }) = &*mc.receiver
                    {
                        return Some(slit.value());
                    }
                }
            }
        }

        // (3) otherwise, we don’t recognize this expression as a plain string or `"...".to_string()`
        None
    }

    /// Check that the top-level block:
    ///  - creates a `map_obj`
    ///  - has calls to `map_obj.insert("type", "map_of")`
    ///    `map_obj.insert("generation_instructions", doc_lit)`
    ///    `map_obj.insert("required", bool)`
    ///    `map_obj.insert("map_key_template", ...)`
    ///    `map_obj.insert("map_value_template", ...)`
    ///  - returns `serde_json::Value::Object(map_obj)`
    fn validate_block_for_map_of(expr: &syn::Expr, expected_doc: &str, expected_required: bool) {
        let block = match expr {
            syn::Expr::Block(b) => &b.block,
            _ => panic!("Expected an ExprBlock for the 'map_of' snippet, got: {:?}", expr),
        };

        let mut found_type = false;
        let mut found_instructions = false;
        let mut found_required = false;
        let mut found_key_template = false;
        let mut found_value_template = false;

        for stmt in &block.stmts {
            if let Some(method_call) = parse_insert_call_stmt(stmt) {
                // Ensure it's an insert call
                if method_call.method != "insert" {
                    continue;
                }
                // Expect exactly two arguments: key and value
                if method_call.args.len() != 2 {
                    continue;
                }
                let key_expr = &method_call.args[0];
                let val_expr = &method_call.args[1];

                let key_string: String = match extract_str_key_from_expr(&method_call.args[0]) {
                    Some(k) => k,
                    None => continue,
                };

                match key_string.as_str() {
                    "type" => {
                        if is_string_value_expr(val_expr, "map_of") {
                            found_type = true;
                        }
                    }
                    "generation_instructions" => {
                        if is_string_value_expr(val_expr, expected_doc) {
                            found_instructions = true;
                        }
                    }
                    "required" => {
                        if is_bool_value_expr(val_expr, expected_required) {
                            found_required = true;
                        }
                    }
                    "map_key_template" => {
                        found_key_template = true;
                    }
                    "map_value_template" => {
                        found_value_template = true;
                    }
                    _ => {}
                }
            }
        }

        assert!(found_type, "Did not find map_obj.insert(\"type\", \"map_of\") in block");
        assert!(
            found_instructions,
            "Did not find map_obj.insert(\"generation_instructions\", \"{}\") in block",
            expected_doc
        );
        assert!(
            found_required,
            "Did not find map_obj.insert(\"required\", Bool({})) in block",
            expected_required
        );
        assert!(
            found_key_template,
            "Did not find map_obj.insert(\"map_key_template\", <...>) in block"
        );
        assert!(
            found_value_template,
            "Did not find map_obj.insert(\"map_value_template\", <...>) in block"
        );
    }


    /// Recursively check if the given expression (block, paren, macro, etc.)
    /// contains a `compile_error!("...")` invocation anywhere.
    fn contains_compile_error_invocation(expr: &syn::Expr) -> bool {
        match expr {
            syn::Expr::Macro(macro_expr) => {
                // e.g. compile_error!("some text")
                let path_idents: Vec<String> = macro_expr.mac.path.segments
                    .iter()
                    .map(|seg| seg.ident.to_string())
                    .collect();
                // If last segment == "compile_error", that’s our invocation
                path_idents.contains(&"compile_error".to_string())
            }

            syn::Expr::Block(expr_block) => {
                contains_compile_error_in_block(&expr_block.block)
            }

            syn::Expr::Paren(par_expr) => {
                contains_compile_error_invocation(&par_expr.expr)
            }

            syn::Expr::Group(gr_expr) => {
                contains_compile_error_invocation(&gr_expr.expr)
            }

            syn::Expr::If(if_expr) => {
                // The `then_branch` is a syn::Block
                if contains_compile_error_in_block(&if_expr.then_branch) {
                    return true;
                }
                // Also check the else branch if it exists (which is an expression)
                if let Some((_else_token, else_expr)) = &if_expr.else_branch {
                    return contains_compile_error_invocation(else_expr);
                }
                false
            }

            // Fallback: not a compile_error macro, not a block, so no
            _ => false,
        }
    }

    /// Check if `expr` is something like `serde_json::Value::String("some_text")`
    /// containing `desired_substring`.
    fn is_string_value_expr(expr: &syn::Expr, desired_substring: &str) -> bool {
        let tokens = expr.to_token_stream().to_string();
        tokens.contains("Value :: String")
            && tokens.contains(desired_substring)
    }

    /// Check if `expr` is something like `serde_json::Value::Bool(true)` or `...Bool(false)`,
    /// matching the desired bool.
    fn is_bool_value_expr(expr: &syn::Expr, expected: bool) -> bool {
        let tokens = expr.to_token_stream().to_string();
        let target = if expected {
            "Value :: Bool (true)"
        } else {
            "Value :: Bool (false)"
        };
        tokens.contains(target)
    }

    #[traced_test]
    fn test_bool_key_actually_legal() {
        trace!("Testing HashMap<bool, _> => compile_error! expected");
        let k_ty: syn::Type = syn::parse_quote!(bool);
        let v_ty: syn::Type = syn::parse_quote!(String);

        let required_bool = quote::quote!(true);
        let doc_lit = proc_macro2::Literal::string("Doc string for bool-key error test");

        let result = build_hashmap_schema(&k_ty, &v_ty, required_bool, doc_lit);
        assert!(result.is_some(), "Expected Some(...) for bool key => compile_error! snippet.");

        let snippet = result.unwrap();
        debug!("bool_key_error => snippet:\n{}", snippet);

        let expr_ast = parse_expr_snippet(&snippet);
        let ce = contains_compile_error_invocation(&expr_ast);
        assert!(
            !ce,
            "Expected no compile_error! invocation for bool-key scenario, but found one."
        );
    }

    #[traced_test]
    fn test_numeric_key_and_bool_value() {
        trace!("Testing HashMap<i32, bool> => 'number' key => 'boolean' value");
        let k_ty: syn::Type = syn::parse_quote!(i32);
        let v_ty: syn::Type = syn::parse_quote!(bool);

        let required_bool = quote::quote!(true);
        let doc_str = "Doc numeric-key/bool-value";
        let doc_lit = proc_macro2::Literal::string(doc_str);

        let snippet = build_hashmap_schema(&k_ty, &v_ty, required_bool, doc_lit)
            .expect("Expected Some(...) for i32->bool");
        debug!("numeric_key_and_bool_value => snippet:\n{}", snippet);

        let expr_ast = parse_expr_snippet(&snippet);
        validate_block_for_map_of(&expr_ast, doc_str, true);
    }

    #[traced_test]
    fn test_string_key_and_numeric_value() {
        trace!("Testing HashMap<String, f64> => 'string' key => 'number' value");
        let k_ty: syn::Type = syn::parse_quote!(String);
        let v_ty: syn::Type = syn::parse_quote!(f64);

        let required_bool = quote::quote!(false);
        let doc_str = "Doc string-key/float-value";
        let doc_lit = proc_macro2::Literal::string(doc_str);

        let snippet = build_hashmap_schema(&k_ty, &v_ty, required_bool, doc_lit)
            .expect("Expected Some(...) for string->f64");
        debug!("string_key_and_numeric_value => snippet:\n{}", snippet);

        let expr_ast = parse_expr_snippet(&snippet);
        validate_block_for_map_of(&expr_ast, doc_str, false);
    }

    #[traced_test]
    fn test_nested_key_and_value() {
        trace!("Testing HashMap<CustomKey, CustomValue> => nested schemas for both");
        let k_ty: syn::Type = syn::parse_quote!(MyCustomKeyType);
        let v_ty: syn::Type = syn::parse_quote!(MyCustomValueType);

        let required_bool = quote::quote!(true);
        let doc_str = "Doc nested key/value test";
        let doc_lit = proc_macro2::Literal::string(doc_str);

        let snippet = build_hashmap_schema(&k_ty, &v_ty, required_bool, doc_lit)
            .expect("Expected Some(...) for nested key/value");
        debug!("nested_key_and_value => snippet:\n{}", snippet);

        let expr_ast = parse_expr_snippet(&snippet);
        validate_block_for_map_of(&expr_ast, doc_str, true);
    }

    #[traced_test]
    fn test_nested_value_only() {
        trace!("Testing HashMap<String, MyCustomValueType> => 'string' key => nested value");
        let k_ty: syn::Type = syn::parse_quote!(String);
        let v_ty: syn::Type = syn::parse_quote!(MyCustomValueType);

        let required_bool = quote::quote!(true);
        let doc_str = "Doc for nested value only";
        let doc_lit = proc_macro2::Literal::string(doc_str);

        let snippet = build_hashmap_schema(&k_ty, &v_ty, required_bool, doc_lit)
            .expect("Expected Some(...) for nested-value-only");
        debug!("nested_value_only => snippet:\n{}", snippet);

        let expr_ast = parse_expr_snippet(&snippet);
        validate_block_for_map_of(&expr_ast, doc_str, true);
    }

    #[traced_test]
    fn test_optional_value() {
        trace!("Testing HashMap<String, Option<i32>> => 'string' key => fallback for Option<T>");
        let k_ty: syn::Type = syn::parse_quote!(String);
        let v_ty: syn::Type = syn::parse_quote!(Option<i32>);

        let required_bool = quote::quote!(false);
        let doc_str = "Doc optional value test";
        let doc_lit = proc_macro2::Literal::string(doc_str);

        let snippet = build_hashmap_schema(&k_ty, &v_ty, required_bool, doc_lit)
            .expect("Expected Some(...) for string->Option<i32>");
        debug!("optional_value => snippet:\n{}", snippet);

        let expr_ast = parse_expr_snippet(&snippet);
        validate_block_for_map_of(&expr_ast, doc_str, false);
    }
}
