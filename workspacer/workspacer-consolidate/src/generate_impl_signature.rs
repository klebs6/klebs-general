// ---------------- [ File: src/generate_impl_signature.rs ]
crate::ix!();

// Return something like "impl SomeTrait for T" WITHOUT braces
pub fn generate_impl_signature(impl_ast: &ast::Impl, docs: Option<&String>) -> String {
    let doc_part = docs
        .filter(|d| !d.trim().is_empty())
        .map(|d| format!("{d}\n"))
        .unwrap_or_default();

    let generic_params = impl_ast
        .generic_param_list()
        .map(|gp| gp.syntax().text().to_string())
        .unwrap_or_default();
    let where_clause = impl_ast
        .where_clause()
        .map(|wc| wc.syntax().text().to_string())
        .unwrap_or_default();
    let trait_part = impl_ast
        .trait_()
        .map_or("".to_string(), |tr| tr.syntax().text().to_string());
    let self_ty = impl_ast
        .self_ty()
        .map_or("???".to_string(), |ty| ty.syntax().text().to_string());

    let signature_line = if trait_part.is_empty() {
        format!("impl{generic_params} {self_ty} {where_clause}")
    } else {
        format!("impl{generic_params} {trait_part} for {self_ty} {where_clause}")
    };

    format!("{doc_part}{signature_line}")
}

// A test suite for the `generate_impl_signature` function. We rely on
// `ra_ap_syntax` to parse some Rust snippets and then extract the `ast::Impl`
// node to pass into our function.
#[cfg(test)]
mod test_generate_impl_signature {
    use super::*;
    use ra_ap_syntax::{ast, SourceFile};

    /// Helper to parse a Rust snippet containing exactly one `impl` block.
    /// Returns the `ast::Impl` node if found, or `None` otherwise.
    fn parse_first_impl(snippet: &str) -> Option<ast::Impl> {
        let parse = SourceFile::parse(snippet,Edition::Edition2024);
        let file_syntax = parse.tree().syntax().clone();
        for node in file_syntax.descendants() {
            if let Some(impl_node) = ast::Impl::cast(node) {
                return Some(impl_node);
            }
        }
        None
    }

    // ------------------------------------------------------------------------
    // Test Cases
    // ------------------------------------------------------------------------

    /// 1) Trait impl with no generics, no where-clause, no doc comments
    #[test]
    fn test_trait_impl_no_generics_no_where_no_docs() {
        let snippet = r#"
            impl Display for MyType {
                fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
                    Ok(())
                }
            }
        "#;
        let impl_ast = parse_first_impl(snippet).expect("Expected an impl node");
        let result = generate_impl_signature(&impl_ast, None);
        assert_eq!(result, "impl Display for MyType");
    }

    /// 2) Inherent impl (no trait specified), so it should produce e.g. "impl MyStruct"
    #[test]
    fn test_inherent_impl() {
        let snippet = r#"
            impl MyStruct {
                fn do_something(&self) {}
            }
        "#;
        let impl_ast = parse_first_impl(snippet).expect("Expected an impl node");
        let result = generate_impl_signature(&impl_ast, None);
        assert_eq!(result, "impl MyStruct");
    }

    /// 3) Impl with generic parameters, no where-clause, no docs
    #[test]
    fn test_generic_impl() {
        let snippet = r#"
            impl<T> SomeTrait for Container<T> {
                fn do_something(&self) {}
            }
        "#;
        let impl_ast = parse_first_impl(snippet).expect("Expected an impl node");
        let result = generate_impl_signature(&impl_ast, None);
        // Typically we expect "impl<T> SomeTrait for Container<T>"
        assert_eq!(result, "impl<T> SomeTrait for Container<T>");
    }

    /// 4) Impl with where clause, no docs
    #[test]
    fn test_impl_with_where_clause() {
        let snippet = r#"
            impl<U> AnotherTrait for Wrapper<U>
            where U: Debug
            {
                fn stuff(&self) {}
            }
        "#;
        let impl_ast = parse_first_impl(snippet).expect("Expected an impl node");
        let result = generate_impl_signature(&impl_ast, None);
        // We typically expect "impl<U> AnotherTrait for Wrapper<U> where U: Debug"
        assert_eq!(result, "impl<U> AnotherTrait for Wrapper<U> where U: Debug");
    }

    /// 5) If the impl is missing a self_ty, our function defaults to "???".
    /// This is contrived but tests that code path.
    #[test]
    fn test_impl_missing_self_ty() {
        // We'll contrive a snippet that fails to parse a self_ty. For example,
        // a broken snippet or partial snippet. Note that ra_ap_syntax might
        // parse incorrectly, but let's see. If `self_ty()` is None, we get "???".
        let snippet = r#"
            impl<T> SomeTrait for
        "#;
        // This snippet is incomplete and likely won't parse into a valid impl.
        // Depending on the parser, we might get an `ast::Impl` but no `self_ty`.
        let impl_ast = parse_first_impl(snippet).expect("Expected an impl node (incomplete parse?)");
        let result = generate_impl_signature(&impl_ast, None);
        assert!(
            result.contains("???"),
            "Expected ??? if self_ty is missing, got: {result}"
        );
    }

    /// 6) If there's no trait, we do an inherent impl for the recognized `self_ty`.
    #[test]
    fn test_no_trait_but_has_generics_where() {
        let snippet = r#"
            impl<T> Wrapper<T>
            where T: Clone
            {
                fn do_it(&self) {}
            }
        "#;
        let impl_ast = parse_first_impl(snippet).expect("Expected an impl node");
        let result = generate_impl_signature(&impl_ast, None);
        // Should be "impl<T> Wrapper<T> where T: Clone"
        assert_eq!(result, "impl<T> Wrapper<T> where T: Clone");
    }

    /// 7) Doc string present => it gets printed before the signature, plus a newline.
    #[test]
    fn test_docs() {
        let snippet = r#"
            /// Some doc comment
            impl MyThing {
                fn usage(&self) {}
            }
        "#;
        let impl_ast = parse_first_impl(snippet).expect("Expected an impl node");

        let doc_text = "/// Some doc comment";
        let result = generate_impl_signature(&impl_ast, Some(&doc_text.to_string()));
        let expected = format!("{doc_text}\nimpl MyThing");
        assert_eq!(result, expected);
    }

    /// 8) If docs is just whitespace or empty, we skip them entirely (no new line).
    #[test]
    fn test_docs_are_empty() {
        let snippet = r#"
            impl AnotherOne {
                fn something(&self) {}
            }
        "#;
        let impl_ast = parse_first_impl(snippet).expect("Expected an impl node");

        // doc string is purely whitespace
        let doc_text = "   ";
        let result = generate_impl_signature(&impl_ast, Some(&doc_text.to_string()));
        // We expect no doc line, so just "impl AnotherOne"
        assert_eq!(result, "impl AnotherOne");
    }

    /// 9) If there's a trait but no generics or where clause => "impl Trait for SelfTy"
    #[test]
    fn test_trait_simple() {
        let snippet = r#"
            impl PartialEq for Foo {
                fn eq(&self, other: &Self) -> bool { true }
            }
        "#;
        let impl_ast = parse_first_impl(snippet).expect("Expected an impl node");
        let result = generate_impl_signature(&impl_ast, None);
        assert_eq!(result, "impl PartialEq for Foo");
    }

    /// 10) Complex example: trait with generics + where clause + doc lines
    #[test]
    fn test_complex_with_doc() {
        let snippet = r#"
            #[doc = "More complex impl"]
            impl<T, U> SomeTrait<T, U> for (T, U) 
            where T: Debug,
                  U: Clone,
            {
                fn do_work(&self) {}
            }
        "#;
        let impl_ast = parse_first_impl(snippet).expect("Expected an impl node");
        // We'll pass a doc string ourselves to see how it's displayed
        let doc_text = "/// This is a doc\n/// Another line";
        let result = generate_impl_signature(&impl_ast, Some(&doc_text.to_string()));
        // Typically: doc lines + newline + "impl<T, U> SomeTrait<T, U> for (T, U) where T: Debug, U: Clone"
        let expected = r#"/// This is a doc
/// Another line
impl<T, U> SomeTrait<T, U> for (T, U) where T: Debug, U: Clone"#;
        assert_eq!(result, expected);
    }
}
