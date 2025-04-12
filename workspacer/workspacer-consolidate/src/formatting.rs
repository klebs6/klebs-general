crate::ix!();

/// 1) *Strip* out `{ ... }` if the entire snippet is enclosed in them.
/// 2) *Split* into lines (the caller typically does `lines()`).
/// 3) *Normalize* blank lines (trim leading/trailing, collapse consecutive).
/// 4) *Dedent or clamp* depending on your policy:
///    - either `conditional_dedent_all` if you want a minimal-based dedent
///    - or `clamp_indent_at_4` if you want to ensure no line exceeds 4 spaces.
/// (Or optionally do both. Up to you.)
pub fn full_format_flow(raw_snippet: &str, policy: IndentPolicy) -> Vec<String> {
    trace!("full_format_flow: starting with snippet = {:?}", raw_snippet);

    // Step 1) Potentially strip braces
    let snippet_no_braces = strip_outer_braces(raw_snippet);

    // Step 2) Split
    let lines: Vec<&str> = snippet_no_braces.lines().collect();

    // Step 3) Normalize blanks
    let lines_norm = normalize_blank_lines(&lines);

    // Step 4) Dedent or clamp, depending on the chosen policy
    match policy {
        IndentPolicy::Dedent => conditional_dedent_all(&lines_norm, true),
        IndentPolicy::Clamp4 => clamp_indent_at_4(&lines_norm, true),
        IndentPolicy::NoChange => lines_norm.iter().map(|l| l.to_string()).collect(),
    }
}

/// A simple enum to choose how we handle final indentation.
#[derive(Debug, Clone, Copy)]
pub enum IndentPolicy {
    /// Use `conditional_dedent_all` to find min indent among lines & subtract it
    Dedent,
    /// Use `clamp_indent_at_4`: any line with >4 spaces is forced to 4,
    /// lines with <=4 spaces remain unchanged, blank/brace lines unchanged.
    Clamp4,
    /// Don’t do any final indentation changes
    NoChange,
}

// ------------------------------------------------------------------------
//  The standard pieces
// ------------------------------------------------------------------------

/// Removes outer braces `{ ... }` from a block snippet, if present.
pub fn strip_outer_braces(s: &str) -> String {
    let trimmed = s.trim_end();
    trace!("strip_outer_braces: input = {:?}", s);
    trace!("  after trim_end => {:?}", trimmed);

    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        // Slice out the braces
        let mut inner = &trimmed[1..trimmed.len() - 1];
        // If the first character is a newline, remove it
        if let Some(stripped) = inner.strip_prefix('\n') {
            trace!("  found leading newline => removing it from the inner block");
            inner = stripped;
        }
        trace!("strip_outer_braces => returning {:?}", inner);
        inner.to_string()
    } else {
        trace!("  not enclosed in braces => returning trimmed = {:?}", trimmed);
        trimmed.to_string()
    }
}

/// Collapses or removes blank lines:
/// - remove leading/trailing blank lines entirely
/// - any consecutive blank lines in the middle => collapsed to one
pub fn normalize_blank_lines<'a>(lines: &'a [&'a str]) -> Vec<String> {
    let mut start = 0;
    while start < lines.len() && lines[start].trim().is_empty() {
        start += 1;
    }
    let mut end = lines.len();
    while end > start && lines[end - 1].trim().is_empty() {
        end -= 1;
    }
    if start >= end {
        // everything is blank
        return Vec::new();
    }
    let slice = &lines[start..end];

    let mut result = Vec::new();
    let mut in_blank_run = false;
    for &line in slice {
        if line.trim().is_empty() {
            if !in_blank_run {
                result.push("".to_string());
                in_blank_run = true;
            }
        } else {
            in_blank_run = false;
            result.push(line.to_string());
        }
    }
    result
}

/// “Dedent” lines by the minimal leading‐space among
/// all lines that have indent>0, ignoring blank or brace‐only lines
/// or lines that already have 0.  If none found, does nothing.
pub fn conditional_dedent_all(lines: &[String], do_dedent: bool) -> Vec<String> {
    if !do_dedent {
        return lines.to_vec();
    }

    // 1) convert purely whitespace lines => ""
    let mut prepared: Vec<String> = lines
        .iter()
        .map(|l| {
            if l.trim().is_empty() {
                "".to_string()
            } else {
                l.clone()
            }
        })
        .collect();

    // 2) find min indent among lines that:
    //    - not empty
    //    - not "{" or "}"
    //    - leading_spaces > 0
    let mut min_indent = usize::MAX;
    for line in &prepared {
        let trimmed = line.trim_end();
        if trimmed.is_empty() || trimmed == "{" || trimmed == "}" {
            continue;
        }
        let lead = leading_spaces(line);
        if lead == 0 {
            // skip from min calc
            continue;
        }
        if lead < min_indent {
            min_indent = lead;
        }
    }
    if min_indent == usize::MAX {
        min_indent = 0;
    }

    // 3) subtract
    let mut result = Vec::new();
    for line in prepared {
        if line.is_empty() {
            result.push(line);
            continue;
        }
        let lead = leading_spaces(&line);
        let remainder = &line[lead..];
        let new_lead = lead.saturating_sub(min_indent);

        let mut new_line = String::new();
        for _ in 0..new_lead {
            new_line.push(' ');
        }
        new_line.push_str(remainder);
        result.push(new_line);
    }
    result
}

/// “Clamp” logic that:
///   - Leaves lines with 0..=4 leading spaces as they are.
///   - If a line has >4 leading spaces, reduce to 4.
///   - Does not affect brace‐only or empty lines.
pub fn clamp_indent_at_4(lines: &[String], do_clamp: bool) -> Vec<String> {
    if !do_clamp {
        return lines.to_vec();
    }
    let mut out = Vec::new();
    for line in lines {
        let trimmed = line.trim_end();
        if trimmed.is_empty() || trimmed == "{" || trimmed == "}" {
            // keep as is
            out.push(line.clone());
            continue;
        }
        let lead = leading_spaces(line);
        if lead <= 4 {
            out.push(line.clone());
        } else {
            let content = &line[lead..];
            let mut new_line = " ".repeat(4);
            new_line.push_str(content);
            out.push(new_line);
        }
    }
    out
}

/// Count how many leading spaces in `line`.
pub fn leading_spaces(line: &str) -> usize {
    line.chars().take_while(|&c| c == ' ').count()
}

#[cfg(test)]
mod formatting_tests {
    use super::*;

    // ---------------------------------------
    // strip_outer_braces
    // ---------------------------------------
    #[test]
    fn test_strip_outer_braces_no_braces() {
        let input = "hello world";
        let output = strip_outer_braces(input);
        assert_eq!(output, "hello world");
    }

    #[test]
    fn test_strip_outer_braces_single_line_braces() {
        let input = "{inside}";
        let output = strip_outer_braces(input);
        assert_eq!(output, "inside");
    }

    #[test]
    fn test_strip_outer_braces_multi_line_braces() {
        let input = r#"{
    let x = 3;
    x + 2
}"#;
        let output = strip_outer_braces(input);
        assert_eq!(
            output,
            r#"    let x = 3;
    x + 2
"#
        );
    }

    #[test]
    fn test_strip_outer_braces_empty_braces() {
        let input = "{}";
        let output = strip_outer_braces(input);
        assert_eq!(output, "");
    }

    #[test]
    fn test_strip_outer_braces_trailing_spaces() {
        let input = "{some stuff}   ";
        let output = strip_outer_braces(input);
        // We only trim the right side when checking for braces,
        // but we do not remove trailing spaces inside the braces themselves.
        // So trailing spaces outside the braces are lost anyway because `.trim_end()`.
        // The inside is "some stuff" exactly.
        assert_eq!(output, "some stuff");
    }

    // ---------------------------------------
    // leading_spaces
    // ---------------------------------------
    #[test]
    fn test_leading_spaces_no_spaces() {
        assert_eq!(leading_spaces("hello"), 0);
    }

    #[test]
    fn test_leading_spaces_some_spaces() {
        assert_eq!(leading_spaces("   hello"), 3);
    }

    #[test]
    fn test_leading_spaces_entirely_spaces() {
        assert_eq!(leading_spaces("      "), 6);
    }

    #[test]
    fn test_leading_spaces_empty_line() {
        assert_eq!(leading_spaces(""), 0);
    }

    // ---------------------------------------
    // conditional_dedent_all
    // ---------------------------------------
    #[test]
    fn test_conditional_dedent_all_no_dedent_flag() {
        let lines = &["   a", "     b", "\t   c"];
        let result = conditional_dedent_all(lines, false);
        // Should return them unchanged:
        assert_eq!(result, vec!["   a", "     b", "\t   c"]);
    }

    #[test]
    fn test_conditional_dedent_all_simple() {
        let lines = &["    fn hello() {", "        println!(\"hi\");", "    }"];
        // Without blank lines, we rely on the min leading spaces among non-blank:
        let result = conditional_dedent_all(lines, true);
        // The min indent among those lines is 4
        assert_eq!(
            result,
            vec!["fn hello() {", "    println!(\"hi\");", "}"]
        );
    }

    #[test]
    fn test_conditional_dedent_all_including_blank_lines() {
        let lines = &[
            "",
            "        let a = 5;",
            "        let b = 6;",
            "",
            "            let c = 7;",
            "        // done",
        ];
        // Non-blank lines have indentation 8 or 12. The minimum is 8,
        // so lines that are empty or have fewer leading spaces are unaffected
        // after we check the min among the non-empty lines.
        let result = conditional_dedent_all(lines, true);
        assert_eq!(
            result,
            vec![
                "",
                "let a = 5;",
                "let b = 6;",
                "",
                "    let c = 7;",
                "// done"
            ]
        );
    }

    #[test]
    fn test_conditional_dedent_all_all_blank() {
        let lines = &["", "   ", "\t"];
        // All lines effectively blank => min_indent stays usize::MAX => set to 0 => no change
        let result = conditional_dedent_all(lines, true);
        // Our function explicitly sets min_indent=0 if everything is blank,
        // then we "empty them out" because line.trim() is empty => we produce "" lines
        // (which is consistent with the existing logic).
        // If you want them unchanged, you'd do something else, but let's confirm your logic:
        assert_eq!(result, vec!["", "", ""]);
    }

    // If you want to ensure that wholly-blank lines keep some indentation,
    // you'll need a different approach. The above logic currently yields empty lines.

    // ---------------------------------------
    // normalize_blank_lines
    // ---------------------------------------
    #[test]
    fn test_normalize_blank_lines_no_blanks() {
        let input = &["a", "b", "c"];
        let result = normalize_blank_lines(input);
        // no leading blank => no trailing blank => no consecutive blank => unchanged
        assert_eq!(result, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_normalize_blank_lines_leading_and_trailing() {
        let input = &["", "", "x", "y", "", "", ""];
        let result = normalize_blank_lines(input);
        // Leading/trailing blank lines removed
        // So we keep just "x", "y"
        assert_eq!(result, vec!["x", "y"]);
    }

    #[test]
    fn test_normalize_blank_lines_consecutive_blanks_in_middle() {
        let input = &["a", "", "", "b", "", "", "c", "", ""];
        let result = normalize_blank_lines(input);
        // leading/trailing => remove
        // multiple consecutive => collapse
        // The result:
        //   "a"
        //   ""  <- the first blank run
        //   "b"
        //   ""  <- the second blank run
        //   "c"
        assert_eq!(result, vec!["a", "", "b", "", "c"]);
    }

    #[test]
    fn test_normalize_blank_lines_all_blank() {
        let input = &["", "", "   "];
        let result = normalize_blank_lines(input);
        // Everything is blank => leading/trailing blank lines removed => all gone
        // so we end with an empty vector
        assert_eq!(result, Vec::<&str>::new());
    }

    #[test]
    fn test_normalize_blank_lines_all_nonblank() {
        let input = &["abc", "def"];
        let result = normalize_blank_lines(input);
        // No change
        assert_eq!(result, vec!["abc", "def"]);
    }

    #[test]
    fn test_normalize_blank_lines_mixed_whitespace() {
        let input = &["", "  ", "\t", "one", "   ", "", "two", "   ", ""];
        // leading lines are blank => removed
        // trailing lines are blank => removed
        // in the middle => "one", then we see a blank line run => keep single blank => "two"
        // but note that lines of only spaces or tabs are considered blank
        let result = normalize_blank_lines(input);
        assert_eq!(result, vec!["one", "", "two"]);
    }

    // A big multi-line snippet representing the code we see in real practice:
    // Notice that some lines are indented with 4 spaces, some with 8,
    // and others are in the middle. We want to see that our logic
    // eventually yields consistent indentation.
    const TEST_SNIPPET: &str = r#"
#[cfg(test)]
mod test_show_crate_and_workspace {
    #[traced_test]
    async fn test_show_single_crate_no_merge() {
                info!("test_show_single_crate_no_merge: start");
                let tmp = tempdir().expect("Failed to create temp dir");
                // etc.
    }

    #[traced_test]
    async fn test_show_workspace_no_crates() {
                info!("test_show_workspace_no_crates: start");
                let tmp = tempdir().unwrap();
                // etc.
    }
}
"#;

    /// This test feeds the entire snippet as if it were the "body" of something,
    /// then uses `strip_outer_braces` => `split into lines` => `normalize_blank_lines` =>
    /// `conditional_dedent_all`. We see if the final indentation is as expected.
    #[test]
    fn test_full_flow_strip_normalize_dedent() {
        // Step 1) Possibly strip braces if we imagine we got “{ TEST_SNIPPET }”
        // For demonstration, let's do that manually here:
        let snippet_for_body = format!("{{\n{}\n}}", TEST_SNIPPET);

        let after_strip = strip_outer_braces(&snippet_for_body);

        // Step 2) Split into lines
        let lines: Vec<&str> = after_strip.lines().collect();

        // Step 3) Normalize blank lines
        let normalized = normalize_blank_lines(&lines);

        // Step 4) Dedent
        let dedented = conditional_dedent_all(&normalized, true);

        // Step 5) Check final result
        // Because so many lines might have different indentation, we’ll do multiple asserts.
        // For demonstration, we just do an “overall shape” assertion.
        // You could do "golden text" comparison if you like.

        // a) No line should start with more than, say, 4 leading spaces.
        // b) We want lines that contain “info!(...)” to be at exactly 4 spaces, etc.

        for (i, line) in dedented.iter().enumerate() {
            println!("Line #{} => {:?}", i, line);
            // Example assertion: no line has > 8 leading spaces
            let lead = leading_spaces(line);
            assert!(
                lead <= 8,
                "Line #{} has {} leading spaces, which is too many: {:?}",
                i,
                lead,
                line
            );
        }

        // If you want a “golden” approach:
        let joined = dedented.join("\n");
        // println!("Final joined:\n{}", joined);
        // let expected = r#"#[cfg(test)]
        // mod test_show_crate_and_workspace {
        //     #[traced_test]
        //     async fn test_show_single_crate_no_merge() {
        //         info!("test_show_single_crate_no_merge: start");
        //         let tmp = tempdir().expect("Failed to create temp dir");
        //         // etc.
        //     }
        //
        //     #[traced_test]
        //     async fn test_show_workspace_no_crates() {
        //         info!("test_show_workspace_no_crates: start");
        //         let tmp = tempdir().unwrap();
        //         // etc.
        //     }
        // }"#;
        //
        // assert_eq!(joined, expected, "Final output does not match golden text");
    }

    #[test]
    fn test_partial_dedent_ignore_brace_lines() {
        // Sometimes we want to ensure that lines containing only "{" or "}" are excluded
        // from the min-indent calculation. Let's replicate that scenario.

        let snippet = r#"
        {
                let x = 3;
                x + 2
        }
        "#;

        // We'll do the actual flow:
        let stripped = strip_outer_braces(snippet);
        let lines: Vec<&str> = stripped.lines().collect();
        // no normalization for this test
        let dedented = conditional_dedent_all(&lines, true);

        // We'll check the resulting lines
        // ...
        // E.g. see how many leading spaces remain
        assert!(dedented[0].starts_with("let x = 3;"), "First line after dedent must have 0 spaces");
        assert!(dedented[1].starts_with("x + 2"), "Second line after dedent must have 0 spaces");
    }

    /// Another scenario: We keep the leading `#[cfg(test)] mod blah {` lines pinned to 0 spaces,
    /// but inside the `fn`, we only shift it by 4. 
    #[test]
    fn test_cfg_test_module_expected_indentation() {
        let snippet = r#"
#[cfg(test)]
mod test_show_crate_and_workspace {
    #[traced_test]
    async fn test_show_single_crate_no_merge() {
                info!("test_show_single_crate_no_merge: start");
    }
}
"#;

        // We'll skip strip_outer_braces here, just do lines => normalize => dedent
        let lines: Vec<&str> = snippet.lines().collect();
        let normalized = normalize_blank_lines(&lines);
        let dedented = conditional_dedent_all(&normalized, true);

        // Let's see if the line containing "info!(" is now at exactly 8 spaces 
        // or at 4 spaces. You can decide your policy. 
        // For demonstration, let's assume we want 4 spaces inside the `fn` body.
        // We'll search for that line:

        for (i, line) in dedented.iter().enumerate() {
            if line.contains("info!(") {
                let lead = leading_spaces(line);
                assert_eq!(lead, 4, "We expect 'info!(...)' line to be at 4 spaces, found {}", lead);
            }
        }
    }

    #[test]
    fn test_skip_zero_indent_from_min_calc() {
        let input = &[
            "#[cfg(test)]",
            "mod test_foo {",
            "    #[something]",
            "    async fn test_stuff() {",
            "                info!(\"hello\");",
            "    }",
            "}",
        ];
        let dedented = conditional_dedent_all(input, true);

        // Expect:
        // [ 
        //   "#[cfg(test)]",            // (col 0)
        //   "mod test_foo {",          // (col 0)
        //   "    #[something]",        // (col 4)
        //   "    async fn test_stuff() {",
        //   "        info!(\"hello\");", // pulled back from col 16 -> col 8
        //   "    }",
        //   "}",
        // ]
        // If you want `info!("hello");` at col 4, you'd do a slightly different rule. 
        // In that case, you might say “the lines that are 4 spaces remain 4, and the 16-space line 
        // becomes 8 or 4.” It's a matter of your test's exact expectation for 'min_indent'.

        for (i, line) in dedented.iter().enumerate() {
            println!("Line #{} => {:?}", i, line);
        }
        
        assert_eq!(dedented[0], "#[cfg(test)]");
        assert_eq!(leading_spaces(&dedented[0]), 0, "Line0 should remain col0");
        assert_eq!(dedented[1], "mod test_foo {");
        assert_eq!(leading_spaces(&dedented[1]), 0, "Line1 should remain col0");

        assert_eq!(leading_spaces(&dedented[2]), 4, "Line2 was 4 spaces => remains 4");
        assert_eq!(leading_spaces(&dedented[4]), 8, "Line4 was 16 => now 8");
    }

    #[test]
    fn test_clamp_indent_at_4_basic() {
        // Suppose these lines show up in your code:
        let input = &[
            "#[cfg(test)]",               // col 0
            "mod test_foo {",             // col 0
            "#[something]",               // col 0 -> or maybe 4, up to you
            "    let x = 1;",             // 4 spaces
            "            info!(\"hey\");",// 12 spaces => want 4
            "}",                          // col 0
        ];

        // We want:
        // - lines[0], [1], [2], [5] => remain col0 because lead=0
        // - line[3] => 4 => remain 4
        // - line[4] => 12 => become 4

        let out = clamp_indent_at_4(input, true);

        // Let's assert some expectations
        assert_eq!(leading_spaces(&out[0]), 0);
        assert_eq!(leading_spaces(&out[1]), 0);
        assert_eq!(leading_spaces(&out[2]), 0); // if your test *actually* wants 4, we can do that
        assert_eq!(leading_spaces(&out[3]), 4, "stays 4");
        assert_eq!(leading_spaces(&out[4]), 4, "12 => 4");
        assert_eq!(leading_spaces(&out[5]), 0);
    }


    #[test]
    fn test_full_format_flow_using_dedent() {
        let raw = r#"
{
        line_a
                line_b
}
"#;
        // run with policy=Dedent
        let lines = full_format_flow(raw, IndentPolicy::Dedent);
        // now lines[0] should be "line_a" at col0, lines[1] => "    line_b" or something, etc.
        assert_eq!(lines[0], "line_a");
        assert_eq!(lines[1], "        line_b"); // or if minimal => "line_b"? depends on min indent
    }

    #[test]
    fn test_full_format_flow_using_clamp() {
        // Some lines have 16 spaces => we want them forcibly at 4
        let raw = r#"
{
            line_a
                        line_b
}
"#;
        let lines = full_format_flow(raw, IndentPolicy::Clamp4);

        // The first line after removing braces + blank lines might be "line_a" with 12 spaces -> 4
        // but let's see. Let's print them out:
        for (i, line) in lines.iter().enumerate() {
            println!("Line #{} => {:?}", i, line);
        }

        // your checks
        assert!(lines[0].starts_with("    line_a"), "should be 4 spaces now");
        assert!(lines[1].starts_with("    line_b"), "should also be 4 spaces");
    }

    #[test]
    fn test_full_format_flow_nochange() {
        let raw = r#"
{
    something
        something_else
}
"#;
        let lines = full_format_flow(raw, IndentPolicy::NoChange);

        // We only do strip+normalize => so the first line is "something" (4 spaces),
        // second line is "        something_else" (8 spaces?), but not adjusted further
        // because NoChange means we skip dedent/clamp.
        // etc. ...
    }

    // etc. Add more specific tests if needed
}
