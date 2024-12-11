crate::ix!();

/// Extracts attributes from an iterator of `ast::Attr`. Returns `(attributes, is_test)`.
/// `is_test` is true if any attribute is `#[test]`.
pub fn extract_attributes(attrs: impl Iterator<Item=ast::Attr>) -> (Vec<String>, bool) {
    let mut attributes = Vec::new();
    let mut is_test = false;
    for attr in attrs {
        let txt = attr.syntax().text().to_string();
        attributes.push(txt.trim().to_string());
        if txt.contains("#[test]") {
            is_test = true;
        }
    }
    (attributes, is_test)
}

#[cfg(test)]
mod extract_attributes_tests {
    use super::*;

    #[test]
    fn test_extract_attributes() {
        let code = r#"
#[inline]
#[test]
#[some_attr]
fn myfunc() {}
"#;
        let syntax = parse_source(code);
        let fn_node = syntax.descendants().find_map(ast::Fn::cast).unwrap();
        let (attrs, is_test) = extract_attributes(fn_node.attrs());
        assert_eq!(attrs.len(), 3);
        assert!(attrs.iter().any(|a| a.contains("#[test]")));
        assert!(is_test);
    }
}
