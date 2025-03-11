crate::ix!();

fn parse_doc_expr(expr: &Expr, lines: &mut Vec<String>) {
    trace!("parse_doc_expr: analyzing expr = {:?}", expr);

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
            trace!("Found a call expression, will parse function + args");
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
            trace!("Found assignment expr => parse the right side");
            parse_doc_expr(right, lines);
        }

        // e.g. parentheses around something: ( "some doc" )
        Expr::Paren(ExprParen { expr: inner, .. }) => {
            trace!("Found parentheses => parse inside");
            parse_doc_expr(inner, lines);
        }

        // For anything else, we simply skip. e.g. numeric, ident alone, etc.
        _ => {
            trace!("Skipping non-string expression: {:?}", expr);
        }
    }
}

pub fn gather_doc_comments(attrs: &[Attribute]) -> Vec<String> {
    info!("Starting gather_doc_comments on {} attributes", attrs.len());
    let mut lines = Vec::new();

    for attr in attrs {
        trace!("Examining attribute: {:?}", attr);

        // We only care about `#[doc(...)]` or `#[doc = ...]`.
        if !attr.path().is_ident("doc") {
            debug!("Skipping non-doc attribute");
            continue;
        }

        match &attr.meta {
            // `#[doc = "some doc line"]`
            Meta::NameValue(MetaNameValue {
                value: Expr::Lit(ExprLit { lit: Lit::Str(s), .. }),
                ..
            }) => {
                debug!("Found name-value doc attribute => {:?}", s.value());
                lines.push(s.value());
            }

            // Fallback for NameValue doc attributes that don't match the literal string pattern.
            Meta::NameValue(other) => {
                warn!("Unhandled name-value doc attribute format: {:?}", other);
            }

            // `#[doc(...)]` => Could be e.g. doc("some doc") or doc(one="1", two("2")) etc.
            Meta::List(MetaList { tokens, .. }) => {
                trace!("Found list style doc => parsing comma-separated expressions");
                match parse2::<CommaSeparatedExpressions>(tokens.clone()) {
                    Ok(cse) => {
                        for expr in cse.expressions() {
                            parse_doc_expr(&expr, &mut lines);
                        }
                    }
                    Err(e) => {
                        warn!("Failed parsing doc(...) tokens as expressions: {:?}", e);
                    }
                }
            }

            // Just `#[doc]` (no tokens) => skip
            Meta::Path(_) => {
                trace!("Found bare path doc => ignoring");
            }
        }
    }

    trace!("Completed gather_doc_comments with lines: {:?}", lines);
    lines
}

#[cfg(test)]
mod test_gather_doc_comments {
    use super::*;

    /// Helper to call `gather_doc_comments` and compare to an expected Vec.
    fn assert_gather_equals(attrs: Vec<Attribute>, expected: &[&str]) {
        let lines = gather_doc_comments(&attrs);
        assert_eq!(lines, expected, "gather_doc_comments mismatch");
    }

    /// 1) Single name-value style: `#[doc = "some doc line"]`
    #[traced_test]
    fn test_doc_name_value_single() {
        let attrs: Vec<Attribute> = parse_quote! {
            #[doc = "some doc line"]
        };
        assert_gather_equals(attrs, &["some doc line"]);
    }

    /// 2) Single parentheses style: `#[doc("some doc line")]`
    #[traced_test]
    fn test_doc_paren_single() {
        let attrs: Vec<Attribute> = parse_quote! {
            #[doc("some doc line")]
        };
        assert_gather_equals(attrs, &["some doc line"]);
    }

    /// 3) Multiple doc attributes (both name-value and parentheses).
    #[traced_test]
    fn test_doc_multiple_mixed() {
        let attrs: Vec<Attribute> = parse_quote! {
            #[doc = "Line 1"]
            #[doc("Line 2")]
            #[doc = "Line 3"]
        };
        // The function should parse each doc attribute in sequence:
        //   => ["Line 1", "Line 2", "Line 3"]
        assert_gather_equals(attrs, &["Line 1", "Line 2", "Line 3"]);
    }

    /// 4) No `#[doc(...)]` at all => should return an empty vector.
    #[traced_test]
    fn test_no_doc_attrs() {
        let attrs: Vec<Attribute> = parse_quote! {
            #[some_other_attr]
            #[allow(unused)]
        };
        assert_gather_equals(attrs, &[]);
    }

    /// 5) Complex doc list with multiple nested tokens.
    ///
    /// `#[doc(one="1", two="2", three("3"))]`
    ///   => might produce multiple nested items, only the literal strings get captured.
    #[traced_test]
    fn test_doc_list_multiple_nested() {
        let attrs: Vec<Attribute> = parse_quote! {
            #[doc(one = "value1", two = "value2", three("value3"))]
        };
        // The function calls parse_nested_meta for each sub-item,
        // capturing each `= "valueX"` or `("valueX")`. We expect "value1", "value2", "value3".
        // However, your function specifically looks for `nested.value()?.parse::<LitStr>()`,
        // so each sub-attribute like `one = "value1"` or `three("value3")` matches.
        // => lines: ["value1", "value2", "value3"]
        let lines = gather_doc_comments(&attrs);
        // Depending on how you parse, you might get them in the order they're listed:
        assert_eq!(lines, ["value1", "value2", "value3"]);
    }

    /// 6) Invalid doc attribute => no lines are extracted.
    #[traced_test]
    fn test_doc_invalid_falls_back() {
        // e.g. doc(something = 123, else)
        // This snippet is half-baked but shows that
        // the function won't extract anything from numeric or non-string-literal values.
        let attrs: Vec<Attribute> = parse_quote! {
            #[doc(something = 123, else)]
        };
        // The function tries `nested.value()?.parse::<LitStr>()`, which fails for `= 123`.
        // => no lines.
        assert_gather_equals(attrs, &[]);
    }

    /// 7) Combined valid + invalid sub-items => only valid string-literal sub-items appear.
    #[traced_test]
    fn test_doc_partial_valid_and_invalid() {
        let attrs: Vec<Attribute> = parse_quote! {
            #[doc(one=123, two="valid string", three)]
        };
        // 'one=123' => parse::<LitStr> fails
        // 'two="valid string"' => parse::<LitStr> succeeds => "valid string"
        // 'three' => parse::<LitStr> fails
        // => lines => ["valid string"]
        assert_gather_equals(attrs, &["valid string"]);
    }

    /// 8) Multiple `#[doc = "line"]` spread out => ensures each is captured.
    #[traced_test]
    fn test_doc_name_value_spread() {
        let attrs: Vec<Attribute> = parse_quote! {
            #[doc = "Line A"]
            #[allow(dead_code)]
            #[doc = "Line B"]
            #[traced_test]
            #[doc = "Line C"]
        };
        // => ["Line A", "Line B", "Line C"]
        assert_gather_equals(attrs, &["Line A", "Line B", "Line C"]);
    }

    /// 9) Parentheses style repeated => ensure we collect them all.
    #[traced_test]
    fn test_doc_paren_spread() {
        let attrs: Vec<Attribute> = parse_quote! {
            #[doc("A1")]
            #[doc("A2")]
            #[doc("A3")]
        };
        assert_gather_equals(attrs, &["A1", "A2", "A3"]);
    }

    /// 10) Complex real-world snippet => ensures the function doesn't panic if there's weird spacing.
    #[traced_test]
    fn test_doc_spaced_snippet() {
        let attrs: Vec<Attribute> = parse_quote! {
            #[doc      =       "some spaced doc"]
            #[doc      (    "some   spaced paren doc"   )]
        };
        assert_gather_equals(attrs, &["some spaced doc", "some   spaced paren doc"]);
    }
}
