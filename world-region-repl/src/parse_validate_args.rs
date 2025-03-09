// ---------------- [ File: src/parse_validate_args.rs ]
crate::ix!();

/* ------------------------------------------------------------------------
   1) Parsing tokens
   ------------------------------------------------------------------------ */
pub fn parse_validate_args<'a,I:StorageInterface>(
    line:  &str,
    lines: &mut Vec<String>,
    st:    &mut ReplState<I>,

) -> Option<ValidateParseResult> {

    let tokens: Vec<&str> = line.split_whitespace().collect();
    if tokens.len() < 2 || !tokens[0].eq_ignore_ascii_case("validate") {
        lines.push("Usage: validate <zip> <city> [house_num] <street...>".to_string());
        return None;
    }

    let ends_with_space = line
        .chars()
        .last()
        .map(|ch| ch.is_whitespace())
        .unwrap_or(false);

    match parse_validate_tokens_longest_city(
        &tokens,
        ends_with_space,
        st.db_access(),
        st.current_region(),
    ) {
        Ok(p) => Some(p),
        Err(e) => {
            lines.push(format!("Validate parse error: {:?}", e));
            None
        }
    }
}
