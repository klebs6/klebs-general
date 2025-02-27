// ---------------- [ File: src/extract_non_macro_lines.rs ]
crate::ix!();

pub fn extract_non_macro_lines(new_top_block: &str) -> Vec<String> {
    trace!("Entering extract_non_macro_lines");
    debug!("new_top_block length={}", new_top_block.len());

    let mut lines = Vec::new();
    for line in new_top_block.split('\n') {
        if line.contains("x!{") {
            trace!("Skipping line containing x!{{...}}: '{}'", line);
        } else {
            trace!("Keeping line: '{}'", line);
            lines.push(line.to_string());
        }
    }

    debug!("Resulting lines: {:?}", lines);
    trace!("Exiting extract_non_macro_lines");
    lines
}

#[cfg(test)]
mod test_extract_non_macro_lines {
    use super::*;

    /// 1) Empty string => we get one line (which is ""), and since it doesn't contain `x!{`,
    ///    it should be retained as a single empty line in the result.
    #[traced_test]
    fn test_empty_string() {
        let input = "";
        let result = extract_non_macro_lines(input);
        assert_eq!(result.len(), 1, "We expect a single empty line from splitting an empty string by newline");
        assert_eq!(result[0], "", "Should be an empty string line");
    }

    /// 3) If a line contains "x!{", that entire line is skipped
    #[traced_test]
    fn test_skip_lines_with_x_macro() {
        let input = r#"
some line
x!{this_line}
another line
"#;
        // Splitting on '\n' => 
        // 1: ""
        // 2: "some line"
        // 3: "x!{this_line}"
        // 4: "another line"
        // 5: ""
        let result = extract_non_macro_lines(input);

        // We skip lines 3, because it contains "x!{".
        // So we keep lines 1 (empty), 2, 4, 5 (empty).
        assert_eq!(result.len(), 4, "We skip exactly the line with x!{{ in it");
        assert_eq!(result[0], "", "Leading empty line");
        assert_eq!(result[1], "some line");
        assert_eq!(result[2], "another line");
        assert_eq!(result[3], "", "Trailing empty line");
    }

    /// 4) Multiple lines each containing "x!{", skip them all
    #[traced_test]
    fn test_multiple_macro_lines_skipped() {
        let input = r#"
x!{alpha}
blah
x!{beta}
x!{gamma}
done
"#;
        let result = extract_non_macro_lines(input);

        // lines containing x!{alpha}, x!{beta}, x!{gamma} get skipped
        // we keep the blank line at start, "blah", and "done", plus trailing blank
        // Actually let's see how many lines after splitting:
        //   [ "", "x!{alpha}", "blah", "x!{beta}", "x!{gamma}", "done", "" ]
        // skipping 1,3,4 => we keep [0:"", 2:"blah", 5:"done", 6:""]
        assert_eq!(result.len(), 4);
        assert_eq!(result[0], "");
        assert_eq!(result[1], "blah");
        assert_eq!(result[2], "done");
        assert_eq!(result[3], "");
    }

    /// 5) If a line has other text plus "x!{" inside, we skip that entire line
    #[traced_test]
    fn test_line_with_inline_x_macro_skipped() {
        let input = "some prefix x!{inline} some suffix\nno macro here\nx!{again}";
        let result = extract_non_macro_lines(input);

        // splitted lines => 
        //   1: "some prefix x!{inline} some suffix"
        //   2: "no macro here"
        //   3: "x!{again}"
        // we skip lines 1 and 3, keep just line 2
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "no macro here");
    }

    /// 7) We preserve spacing and indentation in lines we keep
    #[traced_test]
    fn test_preserve_whitespace_in_kept_lines() {
        let input = r#"
    some indent
        // a comment
x!{macro_here}
    keep me
"#;
        // splitted => 
        // [ "", "    some indent", "        // a comment", "x!{macro_here}", "    keep me", "" ]
        let result = extract_non_macro_lines(input);

        // skip line 3 => "x!{macro_here}"
        // keep everything else
        assert_eq!(result.len(), 5);
        assert_eq!(result[0], "");
        assert_eq!(result[1], "    some indent");
        assert_eq!(result[2], "        // a comment");
        assert_eq!(result[3], "    keep me");
        assert_eq!(result[4], "");
    }

    /// 8) If *every* line contains "x!{", we end up with all lines skipped => empty.
    #[traced_test]
    fn test_all_lines_with_x_macro_skipped() {
        let input = "x!{one}\nx!{two}\nx!{three}";
        let result = extract_non_macro_lines(input);

        assert!(
            result.is_empty(),
            "All lines contain x!{{ => we skip them all"
        );
    }

    /// 9) If the new_top_block has trailing empty lines, they are included if they don't have x!{
    #[traced_test]
    fn test_trailing_empty_lines_kept() {
        let input = "line1\nline2\n\n\n";
        // splitted => ["line1", "line2", "", "", ""]
        // none contain x!{ => keep them all
        let result = extract_non_macro_lines(input);

        assert_eq!(result.len(), 5);
        assert_eq!(result[0], "line1");
        assert_eq!(result[1], "line2");
        assert_eq!(result[2], "");
        assert_eq!(result[3], "");
        assert_eq!(result[4], "");
    }

    /// 10) Integration test with random lines, some with x!{, some not
    #[traced_test]
    fn test_integration_random() {
        let input = r#"
abc
x!{def}
ghi x!{jkl} mno
??? 
"#;
        // splitted => 
        // [ "", "abc", "x!{def}", "ghi x!{jkl} mno", "??? ", "" ]
        // skip lines 2,3 => we keep 1: "", 2:"abc", 5:"??? ", 6:""
        let result = extract_non_macro_lines(input);
        assert_eq!(result.len(), 4);
        assert_eq!(result[0], "");
        assert_eq!(result[1], "abc");
        assert_eq!(result[2], "??? ");
        assert_eq!(result[3], "");
    }
    
    /// 2) If the `new_top_block` has no lines that contain "x!{",
    ///    then we keep them all verbatim.
    #[traced_test]
    fn test_no_macro_lines_at_all() {
        // NOTE: Removed the extra newline right after r#", so the first line is *not* empty.
        let input = r#"// This is a comment
fn something() {}
// Another line
"#;

        // Splitting on '\n' => 4 lines:
        // [ "// This is a comment", "fn something() {}", "// Another line", "" ]
        trace!("About to call extract_non_macro_lines");
        let result = extract_non_macro_lines(input);
        debug!("Returned result: {:?}", result);

        assert_eq!(
            result.len(),
            4,
            "All lines should be retained if none has x!{{"
        );
        assert_eq!(result[0], "// This is a comment");
        assert_eq!(result[1], "fn something() {}");
        assert_eq!(result[2], "// Another line");
        assert_eq!(result[3], "", "Trailing empty line is preserved");
    }

    /// 6) Case sensitivity: "x!{" is not the same as "X!{" => lines with "X!{" are not skipped
    #[traced_test]
    fn test_case_sensitivity() {
        // NOTE: Removed the trailing newline before the end of the raw string, so there's *no* final empty line.
        let input = r#"
X!{not_lowercase}
x!{yes_this_one}"#;

        // Splitting on '\n' now yields:
        //   line 0: ""
        //   line 1: "X!{not_lowercase}"
        //   line 2: "x!{yes_this_one}"
        // We'll skip line 2 because it literally has "x!{", but keep lines 0 and 1.
        // That should give us exactly 2 lines in the final result.
        trace!("About to call extract_non_macro_lines");
        let result = extract_non_macro_lines(input);
        debug!("Returned result: {:?}", result);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "");
        assert_eq!(result[1], "X!{not_lowercase}");
    }
}
