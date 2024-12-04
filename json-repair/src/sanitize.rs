crate::ix!();

pub fn sanitize_json_str(s: &str) -> String {
    s.chars()
        .filter(|c| {
            // Allow standard whitespace characters and visible characters
            !c.is_control() || *c == '\n' || *c == '\r' || *c == '\t'
        })
        .collect()
}

pub fn assert_expected_matches_output_result(input: &str, output: &Result<Value,JsonRepairError>, expected: &Value) {
      if output != &Ok(expected.clone()) {
        println!("input: {:#?}", input);
        println!("output: {:#?}", output);
        println!("expected: {:#?}", expected);
        assert_eq!(output,&Ok(expected.clone()));
    }
}

pub fn skip_whitespace(
    chars:    &mut Peekable<Chars>, 
    repaired: &mut String
) {
    while let Some(&nc) = chars.peek() {
        if nc.is_whitespace() {
            repaired.push(chars.next().unwrap());
        } else {
            break;
        }
    }
}

pub fn is_valid_json_value_start(c: char) -> bool {
    c == '"' || c == '\'' || c == '{' || c == '[' || c.is_digit(10) || c == '-' || matches!(c, 't' | 'f' | 'n')
}
