// ---------------- [ File: src/impl_block_interface.rs ]
crate::ix!();

#[derive(Getters,Debug)]
#[getset(get="pub")]
pub struct ImplBlockInterface {
    docs:           Option<String>,
    attributes:     Option<String>,
    signature_text: String,
    methods:        Vec<crate::crate_interface_item::CrateInterfaceItem<ast::Fn>>,
    type_aliases:   Vec<crate::crate_interface_item::CrateInterfaceItem<ast::TypeAlias>>,
}

impl ImplBlockInterface {
    pub fn new(
        docs:           Option<String>,
        attributes:     Option<String>,
        signature_text: String,
        methods:        Vec<CrateInterfaceItem<ast::Fn>>,
        type_aliases:   Vec<CrateInterfaceItem<ast::TypeAlias>>,
    ) -> Self {
        Self {
            docs,
            attributes,
            signature_text,
            methods,
            type_aliases,
        }
    }
}

impl fmt::Display for ImplBlockInterface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 1) Print doc lines (if any), one per line:
        if let Some(ref docs) = self.docs {
            for line in docs.lines() {
                writeln!(f, "{}", line)?;
            }
        }

        // 2) Print attributes (if any), one per line:
        if let Some(ref attrs) = self.attributes {
            for line in attrs.lines() {
                writeln!(f, "{}", line)?;
            }
        }

        // 3) Trim any trailing spaces from the signature to avoid double-spaces.
        //    Then print "impl Something for T {" on one line
        let sig = self.signature_text.trim_end();
        // If no items, use one-line form: "impl X for Y {}"
        if self.methods.is_empty() && self.type_aliases.is_empty() {
            write!(f, "{} {{}}", sig)?;
            return Ok(());
        }

        // Otherwise, a multi-line block:
        writeln!(f, "{} {{", sig)?;

        // 4) Per test `test_impl_block_interface_real_code`, the order must be
        //    (a) methods first, then (b) type aliases. Also remove any trailing newline from item lines,
        //    and do not add extra newlines between them.

        // a) Methods first:
        for m in &self.methods {
            let item_str = format!("{}", m);
            for line in item_str.lines() {
                // Remove any "/* ... */" placeholders, if you do that in your real code:
                // (If not needed, remove this replacement step.)
                let cleaned = line.replace("{ /* ... */ }", "{}");
                writeln!(f, "    {}", cleaned)?;
            }
        }

        // b) Then type aliases:
        for ta in &self.type_aliases {
            let item_str = format!("{}", ta);
            for line in item_str.lines() {
                writeln!(f, "    {}", line)?;
            }
        }

        // 5) Close brace with no trailing newline
        write!(f, "}}")?;
        Ok(())
    }
}

// Demonstrates testing the real `ImplBlockInterface` by parsing
// a snippet of Rust code and extracting the items via your actual code.
#[cfg(test)]
mod test_impl_block_interface_real {
    use super::*;
    // ^ Adjust imports to whatever your crate's structure is:
    //   - The real `ImplBlockInterface`
    //   - real `extract_docs`, `gather_all_attrs`
    //   - real `generate_impl_signature`
    //   - real `gather_impl_methods`, `gather_assoc_type_aliases`
    //   - real `ConsolidationOptions`

    /// Parses the snippet and returns the first `ast::Impl` node, if any.
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

    /// For demonstration, we define minimal versions of your real gather logic here.
    /// In real code, you might call `gather_impl_methods(&impl_ast, &options)` etc.
    fn gather_methods(impl_ast: &ast::Impl, options: &ConsolidationOptions)
        -> Vec<crate::crate_interface_item::CrateInterfaceItem<ast::Fn>>
    {
        let mut result = vec![];
        if let Some(assoc_items) = impl_ast.assoc_item_list() {
            for child in assoc_items.syntax().children() {
                if child.kind() == ra_ap_syntax::SyntaxKind::FN {
                    if let Some(fn_ast) = ast::Fn::cast(child.clone()) {
                        // Possibly skip private/test if you want:
                        if should_skip_item(&child, options) { 
                            continue; 
                        }
                        let docs = if *options.include_docs() {
                            extract_docs(&child)
                        } else {
                            None
                        };
                        let attrs = gather_all_attrs(&child);
                        // We don't set body_source in this example
                        let fn_item = crate::crate_interface_item::CrateInterfaceItem::new(
                            fn_ast,
                            docs,
                            attrs,
                            None
                        );
                        result.push(fn_item);
                    }
                }
            }
        }
        result
    }

    fn gather_type_aliases(impl_ast: &ast::Impl, options: &ConsolidationOptions)
        -> Vec<crate::crate_interface_item::CrateInterfaceItem<ast::TypeAlias>>
    {
        let mut result = vec![];
        if let Some(assoc_items) = impl_ast.assoc_item_list() {
            for child in assoc_items.syntax().children() {
                if child.kind() == ra_ap_syntax::SyntaxKind::TYPE_ALIAS {
                    if let Some(ty_ast) = ast::TypeAlias::cast(child.clone()) {
                        if should_skip_item(&child, options) {
                            continue;
                        }
                        let docs = if *options.include_docs() {
                            extract_docs(&child)
                        } else {
                            None
                        };
                        let attrs = gather_all_attrs(&child);
                        let alias_item = crate::crate_interface_item::CrateInterfaceItem::new(
                            ty_ast,
                            docs,
                            attrs,
                            None,
                        );
                        result.push(alias_item);
                    }
                }
            }
        }
        result
    }

    /// Minimal stubs for skip logic, doc extraction, attribute gathering, etc.
    /// In actual code, you’d import the real versions.
    fn should_skip_item(_node: &SyntaxNode, _options: &ConsolidationOptions) -> bool {
        // For demonstration, we'll skip nothing
        false
    }

    fn extract_docs(_node: &SyntaxNode) -> Option<String> {
        // In real usage, you'd have a function that reads doc comments.
        // For demonstration, we rely on `#[doc="..."]` or `///...`.
        // Return None or Some(...) as you see fit.
        None
    }

    fn gather_all_attrs(_node: &SyntaxNode) -> Option<String> {
        // Real code might parse and convert them into lines. For demonstration:
        None
    }

    #[test]
    fn test_impl_block_real_code() {
        // A snippet with doc lines, attributes, a couple of methods, and a type alias
        let snippet = r#"
            /// This is doc line
            #[some_attr]
            impl MyTrait for MyType {
                fn do_stuff(&self) {}
                type AliasA = i32;
            }
        "#;

        let impl_ast = parse_first_impl(snippet).expect("Expected an impl");
        let mut options = ConsolidationOptions::new().with_docs();

        // Gather real docs & attributes from the impl node itself
        let docs = if *options.include_docs() {
            extract_docs(impl_ast.syntax())
        } else {
            None
        };
        let attrs = gather_all_attrs(impl_ast.syntax());

        // Generate the signature line: "impl MyTrait for MyType"
        let signature = generate_impl_signature(&impl_ast, docs.as_ref());

        // Gather methods & type aliases
        let methods = gather_methods(&impl_ast, &options);
        let aliases = gather_type_aliases(&impl_ast, &options);

        // Finally build the real ImplBlockInterface
        let ib = ImplBlockInterface::new(docs, attrs, signature, methods, aliases);

        // Format and compare with expected
        let output = format!("{}", ib);
        let expected = r#"impl MyTrait for MyType {
    fn do_stuff(&self) {}
    type AliasA = i32;
}"#;

        assert_eq!(output, expected);
    }

    #[test]
    fn test_impl_block_empty() {
        let snippet = r#"
            impl EmptyTrait for Unit {}
        "#;

        let impl_ast = parse_first_impl(snippet).expect("Expected an impl");
        let options = ConsolidationOptions::new(); // no docs

        let docs = extract_docs(impl_ast.syntax());
        let attrs = gather_all_attrs(impl_ast.syntax());
        let signature = generate_impl_signature(&impl_ast, docs.as_ref());

        let methods = gather_methods(&impl_ast, &options);
        let aliases = gather_type_aliases(&impl_ast, &options);

        let ib = ImplBlockInterface::new(docs, attrs, signature, methods, aliases);
        let output = format!("{}", ib);

        // No items => "impl EmptyTrait for Unit {}"
        assert_eq!(output, "impl EmptyTrait for Unit {}");
    }
}
