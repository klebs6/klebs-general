// ---------------- [ File: ai-json-template-derive/src/parse_doc_expr.rs ]
crate::ix!();

pub fn parse_doc_expr(expr: &Expr, lines: &mut Vec<String>) {
    tracing::trace!("parse_doc_expr: analyzing expr = {:?}", expr);

    match expr {
        // e.g. "some doc" => a literal string
        Expr::Lit(ExprLit {
            lit: Lit::Str(s), ..
        }) => {
            debug!("Found string literal: {:?}", s.value());
            lines.push(s.value());
        }

        // e.g. foo("some doc") => a function call
        Expr::Call(ExprCall { func: _, args, .. }) => {
            tracing::trace!("Found a call expression, will parse function + args");
            // The function might be an ident like `three(...)`
            // We'll collect any string-literal arguments it has.
            for arg in args {
                parse_doc_expr(arg, lines);
            }
            // We do not parse `func` for strings, because the user’s tests show
            // doc(three("value3")) => "three" is an identifier; the string is in the args.
        }

        // e.g. one = "some doc" => an assignment expression
        Expr::Assign(ExprAssign { right, .. }) => {
            tracing::trace!("Found assignment expr => parse the right side");
            parse_doc_expr(right, lines);
        }

        // e.g. parentheses around something: ( "some doc" )
        Expr::Paren(ExprParen { expr: inner, .. }) => {
            tracing::trace!("Found parentheses => parse inside");
            parse_doc_expr(inner, lines);
        }

        // For anything else, we simply skip. e.g. numeric, ident alone, etc.
        _ => {
            tracing::trace!("Skipping non-string expression: {:?}", expr);
        }
    }
}

#[cfg(test)]
mod test_parse_doc_expr_comprehensive {
    use super::*;

    #[traced_test]
    fn it_parses_single_string_literal() {
        trace!("Starting test: it_parses_single_string_literal");

        let expr_src = r#""hello world""#;
        debug!("Creating syn::Expr from source: {}", expr_src);
        let expr: Expr = parse_str(expr_src)
            .expect("Failed to parse single string literal as syn::Expr");

        let mut lines = Vec::new();
        info!("Calling parse_doc_expr for single string literal");
        parse_doc_expr(&expr, &mut lines);

        assert_eq!(lines.len(), 1, "Expected exactly one doc line");
        assert_eq!(lines[0], "hello world");
        debug!("Test passed: got lines={:?}", lines);
    }

    #[traced_test]
    fn it_parses_multiple_string_literals_in_call() {
        trace!("Starting test: it_parses_multiple_string_literals_in_call");

        let expr_src = r#"doc("first line", "second line")"#;
        debug!("Creating syn::Expr from source: {}", expr_src);
        let expr: Expr = parse_str(expr_src)
            .expect("Failed to parse multiple string literals in call as syn::Expr");

        let mut lines = Vec::new();
        info!("Calling parse_doc_expr for multiple string literals in a call");
        parse_doc_expr(&expr, &mut lines);

        assert_eq!(lines.len(), 2, "Expected exactly two doc lines");
        assert_eq!(lines[0], "first line");
        assert_eq!(lines[1], "second line");
        debug!("Test passed: got lines={:?}", lines);
    }

    #[traced_test]
    fn it_parses_assignment_expr_with_string() {
        trace!("Starting test: it_parses_assignment_expr_with_string");

        let expr_src = r#"doc_line = "assigned doc text""#;
        debug!("Creating syn::Expr from source: {}", expr_src);
        let expr: Expr = parse_str(expr_src)
            .expect("Failed to parse assignment expression as syn::Expr");

        let mut lines = Vec::new();
        info!("Calling parse_doc_expr for assignment expr with string");
        parse_doc_expr(&expr, &mut lines);

        assert_eq!(lines.len(), 1, "Expected exactly one doc line");
        assert_eq!(lines[0], "assigned doc text");
        debug!("Test passed: got lines={:?}", lines);
    }

    #[traced_test]
    fn it_parses_nested_parens_string_literal() {
        trace!("Starting test: it_parses_nested_parens_string_literal");

        let expr_src = r#"( ("inside parens") )"#;
        debug!("Creating syn::Expr from source: {}", expr_src);
        let expr: Expr = parse_str(expr_src)
            .expect("Failed to parse nested parentheses expression as syn::Expr");

        let mut lines = Vec::new();
        info!("Calling parse_doc_expr with nested parentheses");
        parse_doc_expr(&expr, &mut lines);

        assert_eq!(lines.len(), 1, "Expected exactly one line from nested parens");
        assert_eq!(lines[0], "inside parens");
        debug!("Test passed: got lines={:?}", lines);
    }

    #[traced_test]
    fn it_skips_non_string_expressions() {
        trace!("Starting test: it_skips_non_string_expressions");

        let expr_src = r#"12345"#;
        debug!("Creating syn::Expr from source: {}", expr_src);
        let expr: Expr = parse_str(expr_src)
            .expect("Failed to parse numeric literal as syn::Expr");

        let mut lines = Vec::new();
        info!("Calling parse_doc_expr with numeric literal, expecting no strings");
        parse_doc_expr(&expr, &mut lines);

        assert_eq!(
            lines.len(),
            0,
            "Expected no doc lines to be collected from numeric literal"
        );
        debug!("Test passed: got lines={:?}", lines);
    }

    #[traced_test]
    fn it_handles_function_call_with_mixed_args() {
        trace!("Starting test: it_handles_function_call_with_mixed_args");

        let expr_src = r#"some_macro(42, "string doc", true, "another doc")"#;
        debug!("Creating syn::Expr from source: {}", expr_src);
        let expr: Expr = parse_str(expr_src)
            .expect("Failed to parse function call with mixed args as syn::Expr");

        let mut lines = Vec::new();
        info!("Calling parse_doc_expr for function call with mixed args");
        parse_doc_expr(&expr, &mut lines);

        assert_eq!(
            lines.len(),
            2,
            "Expected two doc lines (the string arguments only)"
        );
        assert_eq!(lines[0], "string doc");
        assert_eq!(lines[1], "another doc");
        debug!("Test passed: got lines={:?}", lines);
    }

    #[traced_test]
    fn it_ignores_empty_call() {
        trace!("Starting test: it_ignores_empty_call");

        let expr_src = r#"doc()"#;
        debug!("Creating syn::Expr from source: {}", expr_src);
        let expr: Expr = parse_str(expr_src)
            .expect("Failed to parse empty call as syn::Expr");

        let mut lines = Vec::new();
        info!("Calling parse_doc_expr for empty call");
        parse_doc_expr(&expr, &mut lines);

        assert_eq!(lines.len(), 0, "Expected no doc lines in empty call");
        debug!("Test passed: got lines={:?}", lines);
    }

    #[traced_test]
    fn it_handles_nested_function_calls() {
        trace!("Starting test: it_handles_nested_function_calls");

        let expr_src = r#"outer("top doc", inner("nested doc", 123), "final doc")"#;
        debug!("Creating syn::Expr from source: {}", expr_src);
        let expr: Expr = parse_str(expr_src)
            .expect("Failed to parse nested function calls as syn::Expr");

        let mut lines = Vec::new();
        info!("Calling parse_doc_expr for nested function calls");
        parse_doc_expr(&expr, &mut lines);

        assert_eq!(
            lines.len(),
            3,
            "Expected three doc lines from nested function calls"
        );
        assert_eq!(lines[0], "top doc");
        assert_eq!(lines[1], "nested doc");
        assert_eq!(lines[2], "final doc");
        debug!("Test passed: got lines={:?}", lines);
    }

    #[traced_test]
    fn it_skips_boolean_literals() {
        trace!("Starting test: it_skips_boolean_literals");

        let expr_src = r#"true"#;
        debug!("Creating syn::Expr from source: {}", expr_src);
        let expr: Expr = parse_str(expr_src)
            .expect("Failed to parse boolean literal as syn::Expr");

        let mut lines = Vec::new();
        info!("Calling parse_doc_expr with boolean literal, expecting no lines");
        parse_doc_expr(&expr, &mut lines);

        assert_eq!(
            lines.len(),
            0,
            "Expected no lines from boolean literal"
        );
        debug!("Test passed: got lines={:?}", lines);
    }

    #[traced_test]
    fn it_handles_assignments_in_call() {
        trace!("Starting test: it_handles_assignments_in_call");

        let expr_src = r#"doc_fn( note = "alpha", "beta", data=42, detail="gamma" )"#;
        debug!("Creating syn::Expr from source: {}", expr_src);
        let expr: Expr = parse_str(expr_src)
            .expect("Failed to parse call with assignments");

        let mut lines = Vec::new();
        info!("Calling parse_doc_expr for call containing assignments and strings");
        parse_doc_expr(&expr, &mut lines);

        assert_eq!(
            lines.len(),
            3,
            "Expected three doc lines from both assignment and plain string"
        );
        assert_eq!(lines[0], "alpha");
        assert_eq!(lines[1], "beta");
        assert_eq!(lines[2], "gamma");
        debug!("Test passed: got lines={:?}", lines);
    }
}
