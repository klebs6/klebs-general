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
