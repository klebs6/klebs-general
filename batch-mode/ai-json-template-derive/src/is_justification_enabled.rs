// ---------------- [ File: ai-json-template-derive/src/is_justification_enabled.rs ]
crate::ix!();

/**
 * Checks whether `#[justify = false]` OR `#[justify(false)]` appears on a field.
 * If so, we return `true` meaning “justification is disabled.”
 */
#[tracing::instrument(level = "trace", skip_all)]
pub fn is_justification_disabled_for_field(field: &Field) -> bool {
    trace!("Checking if #[justify=false] or #[justify(false)] is present on field {:?}", field.ident);

    for attr in &field.attrs {
        trace!("Inspecting attribute path: {:?}", attr.path().segments.last().map(|s| s.ident.to_string()));
        if attr.path().is_ident("justify") {
            match &attr.meta {
                // -----------------------
                // 1) NameValue style: #[justify = false]
                // -----------------------
                Meta::NameValue(MetaNameValue {
                    value: Expr::Lit(ExprLit { lit: Lit::Bool(lb), .. }),
                    ..
                }) => {
                    trace!("Found a justify attribute with boolean literal = {}", lb.value());
                    if lb.value() == false {
                        trace!("=> #[justify = false] found on field {:?}", field.ident);
                        return true;
                    }
                }

                // -----------------------
                // 2) List style: #[justify(false)] => parse tokens as (false)
                // -----------------------
                Meta::List(meta_list) => {
                    // meta_list.tokens could be something like `TokenStream [ Ident(false) ]`.
                    let tokens_str = meta_list.tokens.to_string();
                    trace!("Found list style for #[justify(...)], tokens => '{}'", tokens_str);
                    // The simplest approach is to see if tokens_str == "false"
                    if tokens_str == "false" {
                        trace!("=> #[justify(false)] found on field {:?}", field.ident);
                        return true;
                    }
                }

                // -----------------------
                // 3) Everything else
                // -----------------------
                other => {
                    debug!("Skipping non-boolean-literal justify attribute: {:?}", other);
                }
            }
        }
    }

    trace!("=> #[justify=false] or #[justify(false)] not found on field {:?}", field.ident);
    false
}

/**
 * Checks whether `#[justify_inner = false]` OR `#[justify_inner(false)]` is present on a field.
 */
#[tracing::instrument(level = "trace", skip_all)]
pub fn is_justification_disabled_for_inner(field: &Field) -> bool {
    trace!("Checking if #[justify_inner=false] or #[justify_inner(false)] is present on field {:?}", field.ident);

    for attr in &field.attrs {
        trace!("Inspecting attribute path: {:?}", attr.path().segments.last().map(|s| s.ident.to_string()));
        if attr.path().is_ident("justify_inner") {
            match &attr.meta {
                // -----------------------
                // 1) NameValue style: #[justify_inner = false]
                // -----------------------
                Meta::NameValue(MetaNameValue {
                    value: Expr::Lit(ExprLit { lit: Lit::Bool(lb), .. }),
                    ..
                }) => {
                    trace!("Found a justify_inner attribute with boolean literal = {}", lb.value());
                    if lb.value() == false {
                        trace!("=> #[justify_inner = false] found on field {:?}", field.ident);
                        return true;
                    }
                }

                // -----------------------
                // 2) List style: #[justify_inner(false)]
                // -----------------------
                Meta::List(meta_list) => {
                    let tokens_str = meta_list.tokens.to_string();
                    trace!("Found list style for #[justify_inner(...)], tokens => '{}'", tokens_str);
                    if tokens_str == "false" {
                        trace!("=> #[justify_inner(false)] found on field {:?}", field.ident);
                        return true;
                    }
                }

                other => {
                    debug!("Skipping non-boolean-literal justify_inner attribute: {:?}", other);
                }
            }
        }
    }

    trace!("=> #[justify_inner=false] or #[justify_inner(false)] not found on field {:?}", field.ident);
    false
}

/**
 * Checks whether `#[justify = false]` or `#[justify(false)]` appears on an enum variant.
 */
#[tracing::instrument(level = "trace", skip_all)]
pub fn is_justification_disabled_for_variant(variant: &Variant) -> bool {
    trace!("Checking if #[justify=false] or #[justify(false)] is present on variant {:?}", variant.ident);

    for attr in &variant.attrs {
        trace!("Inspecting attribute path: {:?}", attr.path().segments.last().map(|s| s.ident.to_string()));
        if attr.path().is_ident("justify") {
            match &attr.meta {
                // -----------------------
                // 1) NameValue style: #[justify = false]
                // -----------------------
                Meta::NameValue(MetaNameValue {
                    value: Expr::Lit(ExprLit { lit: Lit::Bool(lb), .. }),
                    ..
                }) => {
                    trace!("Found a justify attribute with boolean literal = {}", lb.value());
                    if lb.value() == false {
                        trace!("=> #[justify = false] found on variant {:?}", variant.ident);
                        return true;
                    }
                }

                // -----------------------
                // 2) List style: #[justify(false)]
                // -----------------------
                Meta::List(meta_list) => {
                    let tokens_str = meta_list.tokens.to_string();
                    trace!("List style for #[justify(...)], tokens => '{}'", tokens_str);
                    if tokens_str == "false" {
                        trace!("=> #[justify(false)] found on variant {:?}", variant.ident);
                        return true;
                    }
                }

                other => {
                    debug!("Skipping non-boolean-literal justify attribute: {:?}", other);
                }
            }
        }
    }

    trace!("=> #[justify=false] or #[justify(false)] not found on variant {:?}", variant.ident);
    false
}

/**
 * Checks whether `#[justify_inner = false]` or `#[justify_inner(false)]` appears on an enum variant.
 */
#[tracing::instrument(level = "trace", skip_all)]
pub fn is_justification_disabled_for_inner_variant(variant: &Variant) -> bool {
    trace!("Checking if #[justify_inner=false] or #[justify_inner(false)] is present on variant {:?}", variant.ident);

    for attr in &variant.attrs {
        trace!("Inspecting attribute path: {:?}", attr.path().segments.last().map(|s| s.ident.to_string()));
        if attr.path().is_ident("justify_inner") {
            match &attr.meta {
                // -----------------------
                // 1) NameValue style: #[justify_inner = false]
                // -----------------------
                Meta::NameValue(MetaNameValue {
                    value: Expr::Lit(ExprLit { lit: Lit::Bool(lb), .. }),
                    ..
                }) => {
                    trace!("Found a justify_inner attribute with boolean literal = {}", lb.value());
                    if lb.value() == false {
                        trace!("=> #[justify_inner = false] found on variant {:?}", variant.ident);
                        return true;
                    }
                }

                // -----------------------
                // 2) List style: #[justify_inner(false)]
                // -----------------------
                Meta::List(meta_list) => {
                    let tokens_str = meta_list.tokens.to_string();
                    trace!("List style for #[justify_inner(...)], tokens => '{}'", tokens_str);
                    if tokens_str == "false" {
                        trace!("=> #[justify_inner(false)] found on variant {:?}", variant.ident);
                        return true;
                    }
                }

                other => {
                    debug!("Skipping non-boolean-literal justify_inner attribute: {:?}", other);
                }
            }
        }
    }

    trace!("=> #[justify_inner=false] or #[justify_inner(false)] not found on variant {:?}", variant.ident);
    false
}

#[cfg(test)]
mod tests_is_justification_enabled {
    use super::*;

    // Helper to parse a named struct snippet into its **first field**:
    fn first_field_of_struct(input: &str) -> syn::Field {
        let item_struct: ItemStruct = parse_quote!(#input);
        match &item_struct.fields {
            Fields::Named(named) => {
                named.named.iter().next().expect("No fields in struct?").clone()
            }
            _ => panic!("Not a named struct"),
        }
    }

    // Helper to parse an enum snippet, returning the **first variant**:
    fn first_variant_of_enum(input: &str) -> Variant {
        let item_enum: ItemEnum = parse_quote!(#input);
        item_enum.variants
            .iter()
            .next()
            .expect("No variants in enum?")
            .clone()
    }

    // ----------------------------------------------------------
    // Tests for is_justification_disabled_for_field
    // ----------------------------------------------------------
    #[test]
    fn test_field_justify_eq_false() {
        let field = first_field_of_struct(r#"
            struct S {
                #[justify = false]
                x: i32
            }
        "#);
        assert!(is_justification_disabled_for_field(&field));
    }

    #[test]
    fn test_field_justify_paren_false() {
        let field = first_field_of_struct(r#"
            struct S {
                #[justify(false)]
                x: i32
            }
        "#);
        assert!(is_justification_disabled_for_field(&field));
    }

    #[test]
    fn test_field_justify_true_ignored() {
        let field = first_field_of_struct(r#"
            struct S {
                #[justify = true]
                x: i32
            }
        "#);
        assert!(!is_justification_disabled_for_field(&field));
    }

    #[test]
    fn test_field_no_attr() {
        let field = first_field_of_struct(r#"
            struct S {
                x: i32
            }
        "#);
        assert!(!is_justification_disabled_for_field(&field));
    }

    // ----------------------------------------------------------
    // Tests for is_justification_disabled_for_inner (fields)
    // ----------------------------------------------------------
    #[test]
    fn test_field_justify_inner_eq_false() {
        let field = first_field_of_struct(r#"
            struct S {
                #[justify_inner = false]
                x: i32
            }
        "#);
        assert!(is_justification_disabled_for_inner(&field));
    }

    #[test]
    fn test_field_justify_inner_paren_false() {
        let field = first_field_of_struct(r#"
            struct S {
                #[justify_inner(false)]
                x: i32
            }
        "#);
        assert!(is_justification_disabled_for_inner(&field));
    }

    #[test]
    fn test_field_justify_inner_true_ignored() {
        let field = first_field_of_struct(r#"
            struct S {
                #[justify_inner = true]
                x: i32
            }
        "#);
        assert!(!is_justification_disabled_for_inner(&field));
    }

    #[test]
    fn test_field_justify_inner_no_attr() {
        let field = first_field_of_struct(r#"
            struct S {
                x: i32
            }
        "#);
        assert!(!is_justification_disabled_for_inner(&field));
    }

    // ----------------------------------------------------------
    // Tests for is_justification_disabled_for_variant
    // ----------------------------------------------------------
    #[test]
    fn test_variant_justify_eq_false() {
        let var = first_variant_of_enum(r#"
            enum E {
                #[justify = false]
                A,
                B
            }
        "#);
        assert!(is_justification_disabled_for_variant(&var));
    }

    #[test]
    fn test_variant_justify_paren_false() {
        let var = first_variant_of_enum(r#"
            enum E {
                #[justify(false)]
                A,
                B
            }
        "#);
        assert!(is_justification_disabled_for_variant(&var));
    }

    #[test]
    fn test_variant_justify_true_ignored() {
        let var = first_variant_of_enum(r#"
            enum E {
                #[justify = true]
                A,
                B
            }
        "#);
        assert!(!is_justification_disabled_for_variant(&var));
    }

    #[test]
    fn test_variant_justify_no_attr() {
        let var = first_variant_of_enum(r#"
            enum E {
                A,
                B
            }
        "#);
        assert!(!is_justification_disabled_for_variant(&var));
    }

    // ----------------------------------------------------------
    // Tests for is_justification_disabled_for_inner_variant
    // ----------------------------------------------------------
    #[test]
    fn test_variant_justify_inner_eq_false() {
        let var = first_variant_of_enum(r#"
            enum E {
                #[justify_inner = false]
                A,
                B
            }
        "#);
        assert!(is_justification_disabled_for_inner_variant(&var));
    }

    #[test]
    fn test_variant_justify_inner_paren_false() {
        let var = first_variant_of_enum(r#"
            enum E {
                #[justify_inner(false)]
                A,
                B
            }
        "#);
        assert!(is_justification_disabled_for_inner_variant(&var));
    }

    #[test]
    fn test_variant_justify_inner_true_ignored() {
        let var = first_variant_of_enum(r#"
            enum E {
                #[justify_inner = true]
                A,
                B
            }
        "#);
        assert!(!is_justification_disabled_for_inner_variant(&var));
    }

    #[test]
    fn test_variant_justify_inner_no_attr() {
        let var = first_variant_of_enum(r#"
            enum E {
                A,
                B
            }
        "#);
        assert!(!is_justification_disabled_for_inner_variant(&var));
    }
}
