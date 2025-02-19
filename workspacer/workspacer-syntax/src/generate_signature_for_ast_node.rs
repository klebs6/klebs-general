// ---------------- [ File: workspacer-syntax/src/generate_signature_for_ast_node.rs ]
crate::ix!();

/// A trait to generate a "signature" string (or declaration line)
/// for different AST nodes like `Fn`, `Struct`, `Enum`, etc.
pub trait GenerateSignature: fmt::Debug + Clone {
    /// Generate a textual signature, optionally embedding doc lines
    /// (passed in from your doc-extraction routine).
    fn generate_signature(&self, docs: Option<&String>) -> String;
}

/// If you want doc lines above the signature, convert each line from `docs`
/// into `/// ...` lines (if they aren’t already). Then you can insert them
/// before or within the final signature string.
pub fn format_docs_for_signature(docs: Option<&String>) -> String {
    if let Some(d) = docs {
        let trimmed = d.trim();
        if trimmed.is_empty() {
            "".to_string()
        } else {
            // Add a newline after them so the signature is on a new line
            format!("{trimmed}\n")
        }
    } else {
        "".to_string()
    }
}

/// Minimal post-processing for spacing around `->` and `where`.
/// For example, it ensures `) ->` has a space, and `> where` has a space.
pub fn post_process_spacing(signature: &str) -> String {
    signature
        .replace(")->", ") ->")
        .replace(">where", "> where")
}

// --------------------------------------------------------------------
// Implementation for `ast::Fn`
// --------------------------------------------------------------------
impl GenerateSignature for ast::Fn {
    fn generate_signature(&self, docs: Option<&String>) -> String {
        // 1) Convert doc lines into something we can prepend
        let doc_comment_str = format_docs_for_signature(docs);

        // 2) Visibility, e.g. "pub "
        let vis_str = self
            .visibility()
            .map(|v| format!("{} ", v.syntax().text()))
            .unwrap_or_default();

        // 3) Possibly "async "
        let async_str = if let Some(token) = self.async_token() {
            format!("{} ", token.text()) // e.g. "async "
        } else {
            "".to_string()
        };

        // 4) `fn`
        let fn_keyword = "fn";

        // 5) The function name or <anon>
        let name_str = self
            .name()
            .map(|n| n.text().to_string())
            .unwrap_or_else(|| "<anon>".to_string());

        // 6) Gather generics, e.g. <T: Debug>
        let generic_params = self
            .generic_param_list()
            .map(|gp| gp.syntax().text().to_string())
            .unwrap_or_default();

        // 7) Gather parameters
        let mut param_texts = Vec::new();
        if let Some(plist) = self.param_list() {
            for param in plist.params() {
                // Check if param is a `SelfParam` (e.g. &self) or normal `Param`
                if let Some(self_param) = ast::SelfParam::cast(param.syntax().clone()) {
                    // e.g. "&self", "&mut self", "self"
                    let amp_str = if self_param.amp_token().is_some() { "&" } else { "" };
                    let lifetime_str = self_param
                        .lifetime()
                        .map(|lt| lt.syntax().text().to_string())
                        .unwrap_or_default();
                    let mut_str = if self_param.mut_token().is_some() { "mut " } else { "" };

                    // combine them into e.g. "&'a mut self"
                    // handle spacing carefully so we don't get double spaces
                    if !amp_str.is_empty() || !lifetime_str.is_empty() || !mut_str.is_empty() {
                        // e.g. "&'a " + "mut " + "self"
                        let middle = if !lifetime_str.is_empty() {
                            format!("{} ", lifetime_str)
                        } else {
                            "".to_string()
                        };
                        let param_str = format!("{}{}{}self", amp_str, middle, mut_str);
                        param_texts.push(param_str.trim().to_string());
                    } else {
                        param_texts.push("self".to_string());
                    }
                } else if let Some(normal_param) = ast::Param::cast(param.syntax().clone()) {
                    // normal param with pattern + type
                    let pat_str = normal_param
                        .pat()
                        .map(|p| p.syntax().text().to_string())
                        .unwrap_or_default();
                    let ty_str = normal_param
                        .ty()
                        .map(|t| t.syntax().text().to_string())
                        .unwrap_or_default();

                    if !pat_str.is_empty() && !ty_str.is_empty() {
                        param_texts.push(format!("{}: {}", pat_str, ty_str));
                    } else if !ty_str.is_empty() {
                        param_texts.push(ty_str);
                    } else if !pat_str.is_empty() {
                        param_texts.push(pat_str);
                    } else {
                        param_texts.push("<unknown_param>".to_string());
                    }
                } else {
                    // fallback if neither a SelfParam nor normal Param
                    param_texts.push("<unrecognized_param>".to_string());
                }
            }
        }
        let params_str = param_texts.join(", ");

        // 8) Return type, e.g. " -> String"
        let ret_str = if let Some(ret_type) = self.ret_type() {
            if let Some(ty_node) = ret_type.ty() {
                format!(" -> {}", ty_node.syntax().text())
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        };

        // 9) Where clause, e.g. " where T: Debug"
        let where_str = if let Some(wc) = self.where_clause() {
            format!(" {}", wc.syntax().text())
        } else {
            "".to_string()
        };

        // Construct the raw function signature
        let raw_sig = format!(
            "{doc_comment_str}{vis_str}{async_str}{fn_keyword} {name_str}{generic_params}({params_str}){ret_str}{where_str}"
        );

        post_process_spacing(&raw_sig)
    }
}

// --------------------------------------------------------------------
// Implementation for `ast::Struct`
// --------------------------------------------------------------------
impl GenerateSignature for ast::Struct {
    fn generate_signature(&self, docs: Option<&String>) -> String {
        let doc_comment_str = format_docs_for_signature(docs);

        // Possibly pub
        let vis_str = self
            .visibility()
            .map(|v| format!("{} ", v.syntax().text()))
            .unwrap_or_default();

        let name = self
            .name()
            .map(|n| n.to_string())
            .unwrap_or_else(|| "<unknown_struct>".to_string());

        let generic_params_raw = self
            .generic_param_list()
            .map(|g| g.syntax().text().to_string())
            .unwrap_or_default();

        let where_clause_raw = self
            .where_clause()
            .map(|wc| wc.syntax().text().to_string())
            .unwrap_or_default();
        let where_clause = if where_clause_raw.is_empty() {
            "".to_string()
        } else {
            format!(" {}", where_clause_raw)
        };

        // optional: gather fields for display
        let fields_text = if let Some(fl) = self.field_list() {
            match fl {
                ast::FieldList::RecordFieldList(rfl) => {
                    let all_fields: Vec<String> = rfl
                        .fields()
                        .map(|field| {
                            let fname = field
                                .name()
                                .map(|n| n.text().to_string())
                                .unwrap_or_default();
                            let fty = field
                                .ty()
                                .map(|t| t.syntax().text().to_string())
                                .unwrap_or_default();
                            format!("    {}: {},", fname, fty)
                        })
                        .collect();
                    format!("{{\n{}\n}}", all_fields.join("\n"))
                }
                ast::FieldList::TupleFieldList(tfl) => {
                    let all_fields: Vec<String> = tfl
                        .fields()
                        .map(|field| {
                            let vis = field
                                .visibility()
                                .map(|v| format!("{} ", v.syntax().text()))
                                .unwrap_or_default();
                            let fty = field
                                .ty()
                                .map(|t| t.syntax().text().to_string())
                                .unwrap_or_default();
                            format!("    {}{},", vis, fty)
                        })
                        .collect();
                    format!("(\n{}\n);", all_fields.join("\n"))
                }
            }
        } else {
            // fallback: no fields => e.g. "struct Foo;"
            "{ /* ... */ }".to_string()
        };

        let core = format!(
            "{vis_str}struct {name}{generic_params_raw}{where_clause} {fields_text}",
        );
        let final_sig = format!("{doc_comment_str}{core}");
        post_process_spacing(&final_sig)
    }
}

// --------------------------------------------------------------------
// Implementation for `ast::Trait`
// --------------------------------------------------------------------
impl GenerateSignature for ast::Trait {
    fn generate_signature(&self, docs: Option<&String>) -> String {
        let doc_comment_str = format_docs_for_signature(docs);

        let vis_str = self
            .visibility()
            .map(|v| format!("{} ", v.syntax().text()))
            .unwrap_or_default();

        let name = self
            .name()
            .map(|n| n.text().to_string())
            .unwrap_or_else(|| "<unknown_trait>".to_string());

        let generic_params_raw = self
            .generic_param_list()
            .map(|gp| gp.syntax().text().to_string())
            .unwrap_or_default();

        let where_clause_raw = self
            .where_clause()
            .map(|wc| wc.syntax().text().to_string())
            .unwrap_or_default();

        let where_clause = if where_clause_raw.is_empty() {
            "".to_string()
        } else {
            format!(" {}", where_clause_raw)
        };

        let core = format!(
            "{vis_str}trait {name}{generic_params_raw}{where_clause} ",
        );
        let final_sig = format!("{doc_comment_str}{core}");
        post_process_spacing(&final_sig)
    }
}

// --------------------------------------------------------------------
// Implementation for `ast::Enum`
// --------------------------------------------------------------------
impl GenerateSignature for ast::Enum {
    fn generate_signature(&self, docs: Option<&String>) -> String {
        let doc_comment_str = format_docs_for_signature(docs);

        let vis_str = self
            .visibility()
            .map(|v| format!("{} ", v.syntax().text()))
            .unwrap_or_default();

        let name = self
            .name()
            .map(|n| n.text().to_string())
            .unwrap_or_else(|| "<unknown_enum>".to_string());

        let generic_params_raw = self
            .generic_param_list()
            .map(|g| g.syntax().text().to_string())
            .unwrap_or_default();

        let where_clause_raw = self
            .where_clause()
            .map(|wc| wc.syntax().text().to_string())
            .unwrap_or_default();
        let where_clause = if where_clause_raw.is_empty() {
            "".to_string()
        } else {
            format!(" {}", where_clause_raw)
        };

        // optionally gather enum variants, but for brevity we skip
        let core = format!(
            "{vis_str}enum {name}{generic_params_raw}{where_clause} ",
        );
        let final_sig = format!("{doc_comment_str}{core}");
        post_process_spacing(&final_sig)
    }
}

// --------------------------------------------------------------------
// Implementation for `ast::MacroRules`
// --------------------------------------------------------------------
impl GenerateSignature for ast::MacroRules {
    fn generate_signature(&self, docs: Option<&String>) -> String {
        let doc_comment_str = format_docs_for_signature(docs);

        let name = self
            .name()
            .map(|n| n.to_string())
            .unwrap_or_else(|| "<unknown_macro>".to_string());

        let core = format!("macro_rules! {name} ");
        let final_sig = format!("{doc_comment_str}{core}");
        post_process_spacing(&final_sig)
    }
}

// --------------------------------------------------------------------
// Implementation for `ast::TypeAlias`
// --------------------------------------------------------------------
impl GenerateSignature for ast::TypeAlias {
    fn generate_signature(&self, docs: Option<&String>) -> String {
        let doc_comment_str = format_docs_for_signature(docs);

        let name = self
            .name()
            .map(|n| n.text().to_string())
            .unwrap_or_else(|| "<unknown_type_alias>".to_string());

        let generic_params_raw = self
            .generic_param_list()
            .map(|g| g.syntax().text().to_string())
            .unwrap_or_default();

        let where_clause_raw = self
            .where_clause()
            .map(|wc| wc.syntax().text().to_string())
            .unwrap_or_default();
        let where_clause = if where_clause_raw.is_empty() {
            "".to_string()
        } else {
            format!(" {}", where_clause_raw)
        };

        // Get the aliased type
        let aliased_type = self
            .ty()
            .map(|ty| ty.syntax().text().to_string())
            .unwrap_or_else(|| "<unknown_aliased_type>".to_string());

        // Possibly `pub `
        let visibility = self
            .visibility()
            .map(|v| format!("{} ", v.syntax().text()))
            .unwrap_or_default();

        let core = format!(
            "{visibility}type {name}{generic_params_raw}{where_clause} = {aliased_type};",
        );

        let final_sig = format!("{doc_comment_str}{core}");
        post_process_spacing(&final_sig)
    }
}



#[cfg(test)]
mod test_generate_signature_robustness {
    use super::*;
    use ra_ap_syntax::{SourceFile, AstNode, Edition};

    /// Helper: parse a snippet of code, return the first node of type T we find.
    fn parse_first_node_of_type<T: AstNode>(code: &str) -> T {
        let file = SourceFile::parse(code, Edition::Edition2021);
        let syntax = file.syntax_node();
        syntax
            .descendants()
            .find_map(T::cast)
            .expect("Should parse and find a node of desired AST type.")
    }

    /// Helper: unify doc lines as a single string, as though we extracted them from the AST
    fn docs_from_lines(lines: &[&str]) -> String {
        lines.join("\n")
    }

    // -------------------------------- Fn Tests --------------------------------

    #[test]
    fn test_fn_signature_no_params_no_return() {
        let code = r#"
            pub fn simple_fn() {}
        "#;
        let fn_node: ast::Fn = parse_first_node_of_type(code);

        let signature = fn_node.generate_signature(None);
        assert!(signature.contains("pub fn simple_fn()"), "Signature: {signature}");
        assert!(signature.contains("{ /* ... */ }"), "Should have curly placeholder body");
    }

    #[test]
    fn test_fn_signature_with_params_and_return() {
        let code = r#"
            pub fn add(a: i32, b: i32) -> i32 { a + b }
        "#;
        let fn_node: ast::Fn = parse_first_node_of_type(code);

        let signature = fn_node.generate_signature(None);
        assert!(signature.contains("pub fn add(a: i32, b: i32) -> i32"), "Signature: {signature}");
    }

    #[test]
    fn test_fn_signature_with_generics_where_clause() {
        let code = r#"
            pub fn generic_fn<T: Clone>(x: T) -> T where T: std::fmt::Debug {
                x
            }
        "#;
        let fn_node: ast::Fn = parse_first_node_of_type(code);

        let signature = fn_node.generate_signature(None);
        assert!(
            signature.contains("pub fn generic_fn<T: Clone>(x: T) -> T where T: std::fmt::Debug"),
            "Signature: {signature}"
        );
    }

    #[test]
    fn test_fn_signature_with_docs() {
        let code = r#"
            /// This function does something.
            /// Another line of docs.
            pub fn documented() {}
        "#;
        let fn_node: ast::Fn = parse_first_node_of_type(code);

        let doc_text = docs_from_lines(&[
            "This function does something.",
            "Another line of docs."
        ]);
        let signature = fn_node.generate_signature(Some(&doc_text));
        assert!(signature.contains("/// This function does something."));
        assert!(signature.contains("/// Another line of docs."));
        assert!(signature.contains("pub fn documented()"));
    }

    // -------------------------------- Struct Tests --------------------------------

    #[test]
    fn test_struct_signature_no_generics() {
        let code = r#"
            pub struct MyStruct { x: i32 }
        "#;
        let st_node: ast::Struct = parse_first_node_of_type(code);

        let signature = st_node.generate_signature(None);
        assert!(signature.contains("pub struct MyStruct"), "Signature: {signature}");
        assert!(signature.contains("{ /* fields omitted */ }"), "Signature: {signature}");
    }

    #[test]
    fn test_struct_signature_with_generics_and_docs() {
        let code = r#"
            /// A generic struct
            pub struct Container<T> where T: Clone {
                value: T
            }
        "#;
        let st_node: ast::Struct = parse_first_node_of_type(code);
        let doc_text = docs_from_lines(&["A generic struct"]);

        let signature = st_node.generate_signature(Some(&doc_text));
        assert!(signature.contains("/// A generic struct"));
        assert!(signature.contains("pub struct Container<T> where T: Clone"));
    }

    // -------------------------------- Enum Tests --------------------------------

    #[test]
    fn test_enum_signature_with_generics_where_clause() {
        let code = r#"
            pub enum MyEnum<T> where T: Copy {
                A(T),
                B
            }
        "#;
        let enum_node: ast::Enum = parse_first_node_of_type(code);

        let signature = enum_node.generate_signature(None);
        assert!(signature.contains("pub enum MyEnum<T> where T: Copy"), "Signature: {signature}");
    }

    // -------------------------------- Trait Tests --------------------------------

    #[test]
    fn test_trait_signature() {
        let code = r#"
            pub trait MyTrait {
                fn required_method(&self);
            }
        "#;
        let trait_node: ast::Trait = parse_first_node_of_type(code);

        let signature = trait_node.generate_signature(None);
        assert!(signature.contains("pub trait MyTrait"));
        assert!(signature.contains("{ /* items omitted */ }"));
    }

    #[test]
    fn test_trait_signature_with_generics_where_clause_and_docs() {
        let code = r#"
            /// This trait does stuff
            pub trait DoStuff<T> where T: Clone {
                fn do_it(&self, x: T);
            }
        "#;
        let trait_node: ast::Trait = parse_first_node_of_type(code);
        let doc_text = docs_from_lines(&["This trait does stuff"]);

        let signature = trait_node.generate_signature(Some(&doc_text));
        assert!(signature.contains("/// This trait does stuff"));
        assert!(signature.contains("pub trait DoStuff<T> where T: Clone"));
    }

    // -------------------------------- TypeAlias Tests --------------------------------

    #[test]
    fn test_type_alias_signature() {
        let code = r#"
            pub type MyAlias = i32;
        "#;
        let type_node: ast::TypeAlias = parse_first_node_of_type(code);

        let signature = type_node.generate_signature(None);
        assert!(signature.contains("pub type MyAlias"));
        assert!(signature.contains("= /* aliased type omitted */;"));
    }

    #[test]
    fn test_type_alias_signature_with_generics_where() {
        let code = r#"
            pub type MyGenericAlias<T> where T: Default = Vec<T>;
        "#;
        let type_node: ast::TypeAlias = parse_first_node_of_type(code);

        let signature = type_node.generate_signature(None);
        assert!(
            signature.contains("pub type MyGenericAlias<T> where T: Default = /* aliased type omitted */;"),
            "Signature: {signature}"
        );
    }

    // -------------------------------- MacroRules Tests --------------------------------

    #[test]
    fn test_macro_rules_signature() {
        let code = r#"
            #[macro_export]
            macro_rules! my_macro {
                () => {};
            }
        "#;
        let mac_node: ast::MacroRules = parse_first_node_of_type(code);

        let signature = mac_node.generate_signature(None);
        assert!(signature.contains("macro_rules! my_macro"));
        assert!(signature.contains("{ /* macro body omitted */ }"));
    }

    #[test]
    fn test_macro_rules_signature_with_docs() {
        let code = r#"
            /// A fancy macro
            #[macro_export]
            macro_rules! fancy_macro {
                () => {};
            }
        "#;
        let mac_node: ast::MacroRules = parse_first_node_of_type(code);
        let doc_text = docs_from_lines(&["A fancy macro"]);

        let signature = mac_node.generate_signature(Some(&doc_text));
        assert!(signature.contains("/// A fancy macro"));
        assert!(signature.contains("macro_rules! fancy_macro"));
    }
}
