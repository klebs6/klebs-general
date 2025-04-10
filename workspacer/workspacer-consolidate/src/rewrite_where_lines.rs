crate::ix!();

#[tracing::instrument(level = "trace", skip(signature))]
pub fn rewrite_where_lines(signature: &str) -> String {
    trace!("rewrite_where_lines: START => signature:\n{:?}", signature);

    if !signature.contains(" where") {
        return signature.to_string();
    }

    let mut out_lines = Vec::new();

    // For each line of input:
    for line in signature.lines() {
        // If no " where", keep line as-is
        if !line.contains(" where") {
            out_lines.push(line.to_string());
            continue;
        }

        // We found " where", so split the portion before " where" to out_lines,
        // then put "where" on its own line, parse remainder for constraints.
        let (before, after) = split_at_first_occurrence(line, " where");
        let trimmed_before = before.trim_end();
        if !trimmed_before.is_empty() {
            out_lines.push(trimmed_before.to_string());
        }

        out_lines.push("where".to_string());

        let remainder = after.trim_start();
        if remainder.is_empty() {
            continue;
        }

        // Split the remainder on commas & comments
        let mut splitted = split_where_remainder(remainder);

        // If splitted isn't empty, the last piece might end with '{', so we pop & reinsert:
        if let Some(last_line) = splitted.pop() {
            let mut new_lines = try_extract_brace_onto_new_line(last_line);
            splitted.append(&mut new_lines);
        }

        // We now print each piece as a new line, indented 4 spaces,
        // but if splitted is empty or if we see no constraints, we won't insert blank lines.
        for piece in splitted {
            out_lines.push(format!("    {}", piece));
        }
    }

    // Then we ensure any lines that still end with '{' get that brace on its own line.
    pull_brace_onto_own_line(&mut out_lines);

    // Additional fix: if we have a line that’s purely blank **right** before the brace, we remove it.
    // This helps remove the extra blank line if the constraints are done. 
    remove_extraneous_blank_line_before_brace(&mut out_lines);

    let final_str = out_lines.join("\n");
    trace!("rewrite_where_lines: END => final_str:\n{:?}", final_str);
    final_str
}

/// Removes a blank line immediately before a line that is just `{`.
/// This ensures something like:
///     P: Debug
///
/// {
/// becomes:
///     P: Debug
/// {
#[tracing::instrument(level = "trace", skip(lines))]
fn remove_extraneous_blank_line_before_brace(lines: &mut Vec<String>) {
    // We look for consecutive lines: `i` is blank, and `i+1` == "{" => remove line `i`.
    let mut i = 0;
    while i + 1 < lines.len() {
        if lines[i].trim().is_empty() && lines[i+1].trim() == "{" {
            // Remove lines[i]
            lines.remove(i);
        } else {
            i += 1;
        }
    }
}

/// Splits the portion after "where" into lines by commas, then handles `//` and `/* ... */` comments carefully.
/// We do **not** forcibly split leftover code from a line-comment. The entire `// ...` line remains one piece.
#[tracing::instrument(level = "trace", skip(s))]
fn split_where_remainder(s: &str) -> Vec<String> {
    // 1) Split on commas, keeping the comma attached to the preceding text.
    let comma_chunks = break_on_commas(s);

    let mut results = Vec::new();
    for chunk in comma_chunks {
        let sub = separate_comments_and_code(&chunk);
        for piece in sub {
            let trimmed = piece.trim_end();
            if !trimmed.is_empty() {
                results.push(trimmed.to_string());
            }
        }
    }
    results
}

/// Splits input on commas, but keeps the comma with the preceding text.
/// E.g. "A: Copy /* cmt */, B: Clone {" => [ "A: Copy /* cmt */,", " B: Clone {" ]
#[tracing::instrument(level = "trace", skip(input))]
fn break_on_commas(input: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();

    let chars: Vec<_> = input.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        if ch == ',' {
            current.push(ch); // attach the comma
            out.push(current.trim_end().to_string());
            current.clear();
            i += 1;
        } else {
            current.push(ch);
            i += 1;
        }
    }
    if !current.trim().is_empty() {
        out.push(current.trim_end().to_string());
    }
    out
}

#[tracing::instrument(level = "trace", skip(chunk))]
fn separate_comments_and_code(chunk: &str) -> Vec<String> {
    let mut results = Vec::new();
    let chars: Vec<_> = chunk.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut current = String::new();

    while i < len {
        // 1) line comment `//`: keep everything from `//` to the end in one piece
        if i + 1 < len && chars[i] == '/' && chars[i + 1] == '/' {
            if !current.trim().is_empty() {
                results.push(current.trim_end().to_string());
                current.clear();
            }
            let remainder = &chunk[i..];
            results.push(remainder.trim_end().to_string());
            break;
        }

        // 2) block comment `/* ... */`
        if i + 1 < len && chars[i] == '/' && chars[i + 1] == '*' {
            // flush code so far
            if !current.trim().is_empty() {
                results.push(current.trim_end().to_string());
                current.clear();
            }
            // gather block comment
            let mut bc = String::from("/*");
            i += 2;
            while i < len {
                // look for `*/`
                if i + 1 < len && chars[i] == '*' && chars[i + 1] == '/' {
                    bc.push_str("*/");
                    i += 2;
                    break;
                } else {
                    bc.push(chars[i]);
                    i += 1;
                }
            }
            // see if there's a trailing comma
            if i < len && chars[i] == ',' {
                bc.push(',');
                i += 1;
            }
            results.push(bc.trim_end().to_string());
            continue;
        }

        // Normal char => accumulate
        current.push(chars[i]);
        i += 1;
    }

    if !current.trim().is_empty() {
        results.push(current.trim_end().to_string());
    }

    results
}

/// If the line ends with `{`, we split it into two lines: the line without `{` and a new line with just `{`.
#[tracing::instrument(level = "trace", skip(last_line))]
fn try_extract_brace_onto_new_line(last_line: String) -> Vec<String> {
    let mut result = Vec::new();
    let mut line = last_line.trim_end().to_string();
    while line.ends_with(' ') {
        line.pop();
    }
    if let Some(pos) = line.rfind('{') {
        if pos == line.len() - 1 {
            line.truncate(pos);
            while line.ends_with(' ') {
                line.pop();
            }
            if !line.is_empty() {
                result.push(line);
            }
            result.push("{".to_string());
            return result;
        }
    }
    result.push(line);
    result
}

/// Checks for lines that end with `{` and puts that brace on a new line, from bottom to top.
#[tracing::instrument(level = "trace", skip(lines))]
fn pull_brace_onto_own_line(lines: &mut Vec<String>) {
    if lines.is_empty() {
        return;
    }
    for i in (0..lines.len()).rev() {
        let mut ln = lines[i].clone();
        while ln.ends_with(' ') {
            ln.pop();
        }
        if ln.ends_with('{') {
            ln.pop(); // remove '{'
            while ln.ends_with(' ') {
                ln.pop();
            }
            lines[i] = ln;
            lines.insert(i+1, "{".to_string());
        }
    }
}

/// Splits at the first occurrence of `needle`, returning `(left, right_after_needle)`.
#[tracing::instrument(level="trace", skip(s, needle))]
fn split_at_first_occurrence(s: &str, needle: &str) -> (String, String) {
    if let Some(pos) = s.find(needle) {
        let (left, right_with_needle) = s.split_at(pos);
        let right = &right_with_needle[needle.len()..];
        (left.to_string(), right.to_string())
    } else {
        (s.to_string(), "".to_string())
    }
}

#[cfg(test)]
mod test_exhaustive {
    use super::*;

    /// A helper function that just wraps `rewrite_where_lines`, logs input and output,
    /// and returns the final result so we can reuse it in multiple tests.
    fn rewrite_where_lines_test_helper(input: &str) -> String {
        trace!("rewrite_where_lines_test_helper: input = {:?}", input);
        let output = rewrite_where_lines(input);
        info!("rewrite_where_lines_test_helper: output = {:?}", output);
        output
    }

    /// A helper function to test the field alignment logic on a record struct.
    /// We build an `ast::Struct` from a snippet, then apply `generate_signature_with_opts`
    /// with `fully_expand=true` to confirm alignment.
    fn generate_record_struct_signature_test_helper(snippet: &str) -> String {
        trace!("generate_record_struct_signature_test_helper: snippet = {:?}", snippet);

        // Parse the snippet as a SourceFile and find the first Struct node:
        let parse = SourceFile::parse(snippet, Edition::Edition2021);
        let file_syntax = parse.tree().syntax().clone();

        let mut found_struct: Option<ast::Struct> = None;
        for node in file_syntax.descendants() {
            if let Some(s) = ast::Struct::cast(node.clone()) {
                found_struct = Some(s);
                break;
            }
        }

        let st = found_struct.expect("Expected to find a struct in snippet");
        let opts = SignatureOptionsBuilder::default()
            .include_docs(true)
            .fully_expand(true)
            .build()
            .unwrap();
        let sig = st.generate_signature_with_opts(&opts);
        info!("generate_record_struct_signature_test_helper: output = {:?}", sig);
        sig
    }

    /// A helper function for a tuple struct test.
    fn generate_tuple_struct_signature_test_helper(snippet: &str) -> String {
        trace!("generate_tuple_struct_signature_test_helper: snippet = {:?}", snippet);

        let parse = SourceFile::parse(snippet, Edition::Edition2021);
        let file_syntax = parse.tree().syntax().clone();

        let mut found_struct: Option<ast::Struct> = None;
        for node in file_syntax.descendants() {
            if let Some(s) = ast::Struct::cast(node.clone()) {
                found_struct = Some(s);
                break;
            }
        }

        let st = found_struct.expect("Expected to find a tuple struct in snippet");
        let opts = SignatureOptionsBuilder::default()
            .include_docs(true)
            .fully_expand(true)
            .build()
            .unwrap();
        let sig = st.generate_signature_with_opts(&opts);
        info!("generate_tuple_struct_signature_test_helper: output = {:?}", sig);
        sig
    }

    // ------------------------------------------------------------------------
    //  Tests for rewrite_where_lines (handling line/block comments, leftover code)
    // ------------------------------------------------------------------------
    #[traced_test]
    fn test_no_where() {
        trace!("test_no_where start");
        let input = "#[async_trait]\nimpl Foo for Bar {\n    fn x() {}\n}";
        let output = rewrite_where_lines_test_helper(input);
        // We expect no changes because there's no " where" substring
        assert_eq!(output, input, "Expected no changes with no ' where' present");
    }

    #[traced_test]
    fn test_simple_where_no_comments() {
        trace!("test_simple_where_no_comments start");
        let input = "#[async_trait]\nimpl<P> TraitName for Something where P: Debug {\n    fn x() {}\n}";
        let expected = r#"#[async_trait]
impl<P> TraitName for Something
where
    P: Debug
{
    fn x() {}
}"#;
        let output = rewrite_where_lines_test_helper(input);
        assert_eq!(output, expected);
    }

    #[traced_test]
    fn test_line_comment_in_where_clause() {
        trace!("test_line_comment_in_where_clause start");
        let input = "#[async_trait]\nimpl<P> ValidateIntegrity for Workspace<P> where P: SomeBound, // line comment\n    P: AnotherBound {\n}";
        let expected = r#"#[async_trait]
impl<P> ValidateIntegrity for Workspace<P>
where
    P: SomeBound,
    // line comment
    P: AnotherBound
{
}"#;
        let output = rewrite_where_lines_test_helper(input);
        assert_eq!(output, expected);
    }

    #[traced_test]
    fn test_block_comment_in_where_clause() {
        trace!("test_block_comment_in_where_clause start");
        let input = "#[async_trait]\nimpl<A, B> Foo for Bar where A: Copy /* block comment */ , B: Clone {\n}";
        let expected = r#"#[async_trait]
impl<A, B> Foo for Bar
where
    A: Copy
    /* block comment */,
    B: Clone
{
}"#;
        let output = rewrite_where_lines_test_helper(input);
        assert_eq!(output, expected);
    }

    #[traced_test]
    fn test_leftover_code_after_line_comment() {
        trace!("test_leftover_code_after_line_comment start");
        let input = "#[async_trait]\nimpl<P> Something for Stuff where P: Bound, // comment leftover 'async_trait plus more\n    for<'async_trait> P: Another {\n}";
        // We expect the leftover code `'async_trait plus more` on its own line, then the next constraint
        let expected = r#"#[async_trait]
impl<P> Something for Stuff
where
    P: Bound,
    // comment leftover 'async_trait plus more
    for<'async_trait> P: Another
{
}"#;
        let output = rewrite_where_lines_test_helper(input);
        assert_eq!(output, expected, "line comment leftover code scenario failed");
    }

    #[traced_test]
    fn test_line_comment_with_no_leftover_code() {
        trace!("test_line_comment_with_no_leftover_code start");
        let input = "#[async_trait]\nimpl Foo for Bar where // entire line comment\n    Foo: Bar {\n}";
        let expected = r#"#[async_trait]
impl Foo for Bar
where
    // entire line comment
    Foo: Bar
{
}"#;
        let output = rewrite_where_lines_test_helper(input);
        assert_eq!(output, expected);
    }

    // ------------------------------------------------------------------------
    //  Tests for struct field alignment (record + tuple)
    // ------------------------------------------------------------------------
    #[traced_test]
    fn test_record_struct_alignment_basic() {
        trace!("test_record_struct_alignment_basic start");
        let snippet = r#"
            /// Docs
            pub struct MyStruct {
                alpha: i32,
                beta_longer_name: String,
                z: bool,
            }
        "#;
        let out = generate_record_struct_signature_test_helper(snippet);
        info!("Received signature:\n{}", out);

        // Check we see alignment:
        // Something like:
        //  pub struct MyStruct {
        //      alpha:            i32,
        //      beta_longer_name: String,
        //      z:                bool,
        //  }
        assert!(out.contains("alpha:            i32,"));
        assert!(out.contains("beta_longer_name: String,"));
        assert!(out.contains("z:                bool,"));
    }

    #[traced_test]
    fn test_record_struct_alignment_no_fields() {
        trace!("test_record_struct_alignment_no_fields start");
        let snippet = r#"
            /// Docs
            pub struct Empty {}
        "#;
        let out = generate_record_struct_signature_test_helper(snippet);
        info!("Received signature:\n{}", out);
        // We expect something like:
        //  pub struct Empty {
        // }
        // or just { }
        assert!(out.contains("pub struct Empty"));
        assert!(out.contains("{"));
        assert!(out.contains("}"));
    }

    #[traced_test]
    fn test_tuple_struct_alignment_basic() {
        trace!("test_tuple_struct_alignment_basic start");
        let snippet = r#"
            /// A tuple struct
            pub struct MyTuple(pub i32, String, Option<bool>);
        "#;

        let out = generate_tuple_struct_signature_test_helper(snippet);
        info!("Received signature:\n{}", out);
        // We expect alignment like:
        //   pub struct MyTuple(
        //       pub i32     ,
        //       String      ,
        //       Option<bool>,
        //   );
        assert!(out.contains("pub i32     ,"));
        assert!(out.contains("String      ,"));
        assert!(out.contains("Option<bool>,"));
    }

    #[traced_test]
    fn test_tuple_struct_alignment_with_vis() {
        trace!("test_tuple_struct_alignment_with_vis start");
        let snippet = r#"
            struct ComplexTuple(pub(in crate::something) bool, i32);
        "#;
        let out = generate_tuple_struct_signature_test_helper(snippet);
        info!("Received signature:\n{}", out);
        // We expect alignment like:
        //  struct ComplexTuple(
        //      pub(in crate::something) bool,
        //      i32,
        //  );
        assert!(out.contains("pub(in crate::something) bool,"));
        assert!(out.contains("i32                          ,"));
    }

    #[traced_test]
    fn test_comment_with_async_trait_in_where_clause() {
        let input = r#"
    #[async_trait]
    impl<P,H> ValidateIntegrity for Workspace<P,H>
    where
        H: CrateHandleInterface<P>, // leftover 'async_trait plus more
        for<'async_trait> P: SomeBound,
    {
    }
    "#;
        let expected = r#"
    #[async_trait]
    impl<P,H> ValidateIntegrity for Workspace<P,H>
    where
        H: CrateHandleInterface<P>, // leftover 'async_trait plus more
        for<'async_trait> P: SomeBound
    {
    }
    "#.trim_start();

        let output = rewrite_where_lines_test_helper(input.trim_start());
        assert_eq!(output.trim_end(), expected.trim_end());
    }
}
