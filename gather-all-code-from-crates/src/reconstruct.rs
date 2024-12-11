crate::ix!();

pub fn reconstruct_code_from_filtered_items(items: &[ItemInfo], omit_bodies: bool) -> String {

    //println!("items: {:#?}", items);

    let mut output = String::new();

    for item in items {
        match item {
            ItemInfo::Function(f) => {
                reconstruct_function(f, omit_bodies, &mut output, 0);
            }
            ItemInfo::Struct { attributes, is_public, signature, .. } => {
                for attr in attributes {
                    output.push_str(attr);
                    output.push('\n');
                }
                if *is_public && !signature.starts_with("pub ") {
                    //output.push_str("pub ");
                }
                output.push_str(signature);
                output.push('\n');
                output.push('\n');
            }
            ItemInfo::Enum { attributes, is_public, signature, .. } => {
                for attr in attributes {
                    output.push_str(attr);
                    output.push('\n');
                }
                if *is_public && !signature.starts_with("pub ") {
                    //output.push_str("pub ");
                }
                output.push_str(signature);
                output.push('\n');
                output.push('\n');
            }
            ItemInfo::TypeAlias { attributes, is_public, signature, .. } => {
                for attr in attributes {
                    output.push_str(attr);
                    output.push('\n');
                }
                if *is_public && !signature.starts_with("pub ") {
                    //output.push_str("pub ");
                }
                output.push_str(signature);
                output.push('\n');
                output.push('\n');
            }
            ItemInfo::ImplBlock { attributes, signature, methods, .. } => {
                for attr in attributes {
                    output.push_str(attr);
                    output.push('\n');
                }
                // impl blocks do not have a pub keyword. Just print the cleaned signature.
                output.push_str(signature);
                output.push_str(" {\n");
                for m in methods {
                    reconstruct_function(m, omit_bodies, &mut output,4);
                }
                output.push_str("}\n\n");
            }
        }
    }

    output
}


pub fn reconstruct_function(f: &FunctionInfo, omit_bodies: bool, output: &mut String, indent_level: usize) {
    let indent = " ".repeat(indent_level);

    // Add attributes
    for attr in f.attributes() {
        output.push_str(&format!("{}{}\n", indent, attr.trim()));
    }

    // Add the function signature
    let mut sig_line = f.signature().trim().to_string();

    if omit_bodies {
        if !sig_line.ends_with(';') {
            sig_line.push(';');
        }
        output.push_str(&format!("{}{}\n", indent, sig_line.trim()));
    } else {
        if let Some(body) = f.body() {
            let sig_line = sig_line.trim_end().trim_end_matches(';').trim_end();
            output.push_str(&format!("{}{} ", indent, sig_line));

            // Normalize and reindent the body
            let normalized_body = normalize_and_reindent_body(body, indent_level + 4);
            output.push_str("{\n");
            output.push_str(&normalized_body);
            output.push_str(&format!("\n{}}}\n", indent)); // Only one closing brace
        } else {
            if !sig_line.ends_with(';') {
                sig_line.push(';');
            }
            output.push_str(&format!("{}{}\n", indent, sig_line.trim()));
        }
    }

    // Ensure a blank line after each function
    output.push('\n');
}


fn normalize_and_reindent_body(body: &str, target_indent: usize) -> String {
    let lines: Vec<&str> = body.lines().collect();

    // If the first line is just "{", and the last line is just "}", skip them.
    let (start, end) = if !lines.is_empty()
        && lines.first().map(|l| l.trim()) == Some("{")
        && lines.last().map(|l| l.trim()) == Some("}")
    {
        (1, lines.len() - 1)
    } else {
        (0, lines.len())
    };

    let relevant_lines = &lines[start..end];

    // Determine min indentation of non-empty lines
    let min_indent = relevant_lines.iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.chars().take_while(|c| c.is_whitespace()).count())
        .min()
        .unwrap_or(0);

    let mut final_body = String::new();

    for line in relevant_lines {
        let trimmed_line = if line.trim().is_empty() {
            "".to_string()  // preserve empty lines as empty
        } else {
            let start_idx = min_indent.min(line.len());
            line[start_idx..].to_string()
        };

        let indented_line = format!("{}{}", " ".repeat(target_indent), trimmed_line);
        final_body.push_str(&indented_line);
        final_body.push('\n');
    }

    final_body
}


#[cfg(test)]
mod reconstruct_code_from_filtered_fns_tests {
    use super::*;

    #[test]
    fn test_reconstruct_code_from_filtered_fns_with_bodies() {

        let fns = vec![
            ItemInfo::Function(FunctionInfoBuilder::default()
                .name("foo".to_string())
                .is_public(false)
                .is_test(false)
                .attributes(vec!["#[inline]".to_string()])
                .signature("fn foo(x: i32) -> i32".to_string())
                .body(Some("{ x + 1 }".to_string()))
                .build()
                .unwrap()),
                ItemInfo::Function(FunctionInfoBuilder::default()
                    .name("bar".to_string())
                    .is_public(true)
                    .is_test(true)
                    .attributes(vec!["#[test]".to_string()])
                    .signature("pub fn bar()".to_string())
                    .body(Some("{ assert_eq!(2,2); }".to_string()))
                    .build()
                    .unwrap()),
        ];

        let code = reconstruct_code_from_filtered_items(&fns, false);
        assert!(code.contains("#[inline]\nfn foo(x: i32) -> i32 { x + 1 }"));
        assert!(code.contains("#[test]\npub fn bar() { assert_eq!(2,2); }"));
    }

    #[test]
    fn test_reconstruct_code_from_filtered_fns_omit_bodies() {

        let fns = vec![
            ItemInfo::Function(FunctionInfoBuilder::default()
                .name("foo".to_string())
                .is_public(false)
                .is_test(false)
                .attributes(vec!["#[inline]".to_string()])
                .signature("fn foo(x: i32) -> i32".to_string())
                .body(Some("{ x + 1 }".to_string()))
                .build()
                .unwrap()),
        ];

        let code = reconstruct_code_from_filtered_items(&fns, true);
        // Body should be omitted and replaced with a semicolon
        assert!(code.contains("#[inline]\nfn foo(x: i32) -> i32;"));
        assert!(!code.contains("{ x + 1 }"));
    }
}
