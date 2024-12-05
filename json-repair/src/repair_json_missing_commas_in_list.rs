crate::ix!();

pub fn repair_json_missing_commas_in_list(input: &str) -> Result<String, JsonRepairError> {

    info!("repairing json missing commas in list");

    let mut repaired = String::new();
    let mut chars = input.chars().peekable();

    // States
    let mut in_string = false;
    let mut escape = false;
    let mut last_was_value = false;
    let mut made_correction = false;

    while let Some(c) = chars.next() {
        if in_string {
            // Handle escape sequences in strings
            repaired.push(c);
            if escape {
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        } else {
            match c {
                '"' => {
                    // Start of a string
                    if last_was_value {
                        repaired.push(',');
                        made_correction = true;
                    }
                    in_string = true;
                    repaired.push(c);
                    last_was_value = true;
                }
                '[' | '{' => {
                    // Start of array or object
                    if last_was_value {
                        repaired.push(',');
                        made_correction = true;
                    }
                    repaired.push(c);
                    last_was_value = false;
                }
                ']' | '}' => {
                    // End of array or object
                    repaired.push(c);
                    last_was_value = true;
                }
                ',' => {
                    // Comma between values
                    repaired.push(c);
                    last_was_value = false;
                }
                ':' => {
                    // Key-value separator in objects
                    repaired.push(c);
                    last_was_value = false;
                }
                c if c.is_whitespace() => {
                    if in_string {
                        // Include whitespace inside strings
                        repaired.push(c);
                    } else {
                        // Skip whitespace outside strings
                        continue;
                    }
                }
                c if c.is_digit(10) || c == '-' => {
                    // Start of a number
                    if last_was_value {
                        repaired.push(',');
                        made_correction = true;
                    }
                    repaired.push(c);
                    // Consume the rest of the number
                    while let Some(&next_c) = chars.peek() {
                        if next_c.is_digit(10)
                            || next_c == '.'
                            || next_c == 'e'
                            || next_c == 'E'
                            || next_c == '+'
                            || next_c == '-'
                        {
                            repaired.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    last_was_value = true;
                }
                c if c.is_alphabetic() => {
                    // Start of a literal: true, false, null
                    if last_was_value {
                        repaired.push(',');
                        made_correction = true;
                    }
                    repaired.push(c);
                    // Consume the rest of the literal
                    while let Some(&next_c) = chars.peek() {
                        if next_c.is_alphabetic() {
                            repaired.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    last_was_value = true;
                }
                _ => {
                    // Unknown character
                    repaired.push(c);
                }
            }
        }
    }

    // Log only if a correction was made
    if made_correction {
        info!("repairing json missing commas in list");
    }

    Ok(repaired)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_commas_between_strings() {
        let input = r#"["apple""banana""cherry"]"#;
        let expected = r#"["apple","banana","cherry"]"#;
        let result = repair_json_missing_commas_in_list(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_no_missing_commas() {
        let input = r#"["apple","banana","cherry"]"#;
        let expected = input.to_string();
        let result = repair_json_missing_commas_in_list(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_missing_commas_with_numbers() {
        let input = r#"[1 2 3]"#;
        let expected = r#"[1,2,3]"#;
        let result = repair_json_missing_commas_in_list(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_missing_commas_between_numbers_and_strings() {
        let input = r#"["apple"1"banana"2]"#;
        let expected = r#"["apple",1,"banana",2]"#;
        let result = repair_json_missing_commas_in_list(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_escaped_quotes_inside_strings() {
        let input = r#"["He said \"Hello\"""She replied \"Hi\""]"#;
        let expected = r#"["He said \"Hello\"","She replied \"Hi\""]"#;
        let result = repair_json_missing_commas_in_list(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_strings_with_commas_inside() {
        let input = r#"["apple, banana""cherry"]"#;
        let expected = r#"["apple, banana","cherry"]"#;
        let result = repair_json_missing_commas_in_list(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_nested_arrays() {
        let input = r#"[["apple""banana"]["cherry""date"]]"#;
        let expected = r#"[["apple","banana"],["cherry","date"]]"#;
        let result = repair_json_missing_commas_in_list(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_empty_array() {
        let input = r#"[]"#;
        let expected = r#"[]"#;
        let result = repair_json_missing_commas_in_list(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_single_element_array() {
        let input = r#"["apple"]"#;
        let expected = r#"["apple"]"#;
        let result = repair_json_missing_commas_in_list(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_missing_commas_between_objects() {
        let input = r#"[{"key1": "value1"}{"key2": "value2"}]"#;
        let expected = r#"[{"key1":"value1"},{"key2":"value2"}]"#;
        let result = repair_json_missing_commas_in_list(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_mixed_types_missing_commas() {
        let input = r#"["apple" true null 42 {"key": "value"} [1,2]]"#;
        let expected = r#"["apple",true,null,42,{"key":"value"},[1,2]]"#;
        let result = repair_json_missing_commas_in_list(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_string_with_escaped_backslash() {
        let input = r#"["c:\\path\\to\\file""another string"]"#;
        let expected = r#"["c:\\path\\to\\file","another string"]"#;
        let result = repair_json_missing_commas_in_list(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_unicode_characters_in_strings() {
        let input = r#"["emoji: \uD83D\uDE00""another string"]"#;
        let expected = r#"["emoji: \uD83D\uDE00","another string"]"#;
        let result = repair_json_missing_commas_in_list(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_string_with_inner_quotes() {
        let input = r#"["He said, \"Hello, world!\"""She replied, \"Hi!\""]"#;
        let expected = r#"["He said, \"Hello, world!\"","She replied, \"Hi!\""]"#;
        let result = repair_json_missing_commas_in_list(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_big_one() {
        let input = r#"{
  "xxxxx_xxxx": "xxxxxxxxxxxxxxxxxxxxxxxxxx",
  "xxxx_xxxxxxx_xxx_xxxxxxxxx": [
    "xxx xxxxx xx xxxxxx xxxxxxxx xxxx xxxxxx xxxxx.",
    "xxxxxxx xx xxxxxxx xxx xxxxxx xxxxxx.",
    "xxx xxxxxxxxxxx xx xxx xxxxxxx xxx xxxxxxx xxxxxxxx.",
    "xxxxxxxxx xxx xxxxxxxxxxxxx xxxxxxxxxx xxxx xxxxxxxx xxxxxxxx.",
    "xxx xxxxxxxxx xx xxxxxxxxx xxx xxxxxxxxxxxx xxxxxxxxxxxxx.",
    "xxx xxxxxx xx xxxxxxxxxxxxxx xxxxxxxx xxxxxxxxxxx.",
    "xxxxxxxx xxxxxxx xxxxxxx xxxxxxxxxxx xxx xxxxxxxxxx.",
    "xx xxxxxxxxxxxx xx xxxxx xxxxxx xx xxxxxxxxx xxxxxx.",
    "xxx xxxxx'x xxxxxxx xxxxx xxxxxxxxxxxx xxx xxxxxxxxxxxx.",
    "xxx xxxxxxxxxx xxxxx xx xxxxxxxxxxxxx xxx xxxxxx.",
    "x xxxxxx xx xxxxxxxxxxxx xxxxxx xx xxxxxxxx.",
    "xxx xxxxxxxxx xx xxxxxxxxx xxxxxx xx xxxxxxxxx.",
    "xxx xxxxxxx xxxxx xx xxxxxxxxxxx xxxxx xxx xxxxxxxxx xxxxxxxxx.",
    "xxxxxxxxx xx xxxxx xxxxxxxxx xx xxxxx xxxxxxxxxxx.",
    "xxx xxxxxxxxxxxxxxx xx xxxxxxxx xxxxxxx xxxx xxxxxxx xxxxxx.",
    "xxxxxx xx xxxxxx xxxxxxxx xxxxxx xxxxxxxx xxxxxx.",
    "xxx xxxxx xxx xxxxx xx xxxxxx xxx xxxxxxxx.",
    "xxxxx'x xxxxx xx xxxxxxx xxx xxxxxx xxxxxxx.",
    "xxxx xxxxxx xxxx xxxxxxxx xxxxxxx xxxx'x xxxx.",
    "x xxxxxxx xxxx xxx xxxxxx xx xxxxxxx xxxxxx xxxxxxx."
  ],
  "xxxxxxx_xxx_xxxxxxxx_xxxxxxxxxxx": [
    "xxxxxxxxxx xxxxxxx xx xxxxxx xxx xxx xxxxxxx xxxxxxx xxxxxxx.",
    "xxxxx xxxxxxx xxxxx, xxxx xx xxx xxxxx xxx xxxxxx xx xxxxxxx xx xxx xxxxx.",
    "xxx xxxxx xx xxxxxx xxxxxxxx xxxxxxx xxxx xxxxxxxx xxxxxxx xxxx.",
    "x xxxxx, xxxxxxxxx xxxx xxxxxxxxxx xxx xxx xxxxxx'x xxxxxxxx.",
    "xxxxxxx xxxxxxxxxx xxxxx, xxxxxxxxxx xxxxxx xxxx xxxxx xxxxxx.",
    "xxxx xxxxxxxxx xxxxxx xx xxx xxxx xxxx xxxxxxxxx xx xxx xxxxx.",
    "xxxxx xxx xxxxxxx xxxxxxxx xxxxxxx xx xxxx xxxxxxx.",
    "xxx xxx xxxxxx xx xxxxxxx xxxxxxxxxxx xxxxxx.",
    "xxx xxxxx xxxxxxxx xx xxxxxxxx xxxxxx xxx xxxx'x xxxxx.",
    "xxxxxxxxxx xxxxxx xx xxxxxxxx xx xxx xxxx'x xxxxxxx xxxxxxxxxx xxxxx.",
    "xxx xxxxx xx xxxxxxxxxx xxxxxx xx xxxxxx xxxxxx xxx xxxxx.",
    "x xxxxxxxx xxxxxxx xx xxxxxxxxx, xxxx xxxxx xxx xxxxxxxx.",
    "xxxxxx xxxxxx xxx xxxxxxx xxxxxxx xxxx xxxxx xxxxxxxxx.",
    "x xxxxxxx xxxxx xx xxxxx xxx xxxxxx xx xxxx xxxxxxxx.",
    "xxx xxxxx'x xxxxxxxx xxxxxxx xxxxxxx xxxxxxx xxxxxxxxxxx xxxxxx xxxxx.",
    "xxx xxxx xxxxxxx xx xxx xxxxxx xx xxxxx xxxxxxx xxxx xxxx.",
    "xxx xxxxx xxxxx xx xxxxxxx xxxxxx xxxx, xxxxxxx xxx.",
    "xxxxxxxxxxx xxx xxxxxxxxxxx xxxxxxxx xxx xxxxx'x xxxxxxx.",
    "xxx xxxxxxx xxxxxxx xx xxxxxx, xxxxxx xxx xxxx xxxx xxxx.",
    "x xxxxxxxxxx xxxxxxx xxxxxx xx xxxx xxxxxxxxx xxx xxxxx.",
    "xxxxx'x xxxxxxxx xxxxxxxxxx xx xxxx xxxxxx, xxxxxxxxx xxxxx.",
    "xxx xxxx'x xxxxxxxx xxxxxx xx xxx xxxxxx xx xxx xxxxxx.",
    "xxxxxxxx xxxxxxxx xx xxxx xx xxxxxxx, xxxxxxx xx xxxxxx xxxxxxx.",
    "xxx xxxxxxx xxxxxxx xx x xxxxxx xxxxxx xxxxxxxxxx xx.",
    "xxx xxxxxxxx, xxxxxx xxxxxxxx xx xxxxxxx xxxx xxxxx.",
    "xxxxxx xxxxxxxx xxxxxxx xxx xxx xxxx xxxxxx xxxxxxxxx.",
    "xxxx xxxx xxxxxxxxx, xxxxxxxx xx xxxx xxxxxxx xxxxxx.",
    "xxxxxxxxxxx xxxxxxx xx xxxx xxxxx xx xxxxx xxxxxxxxxx.",
    "xxxxxxxxx xxxxxxxxxx xx xxxxxxxxx xxxxxx xxxxxx’x xxxxxxx.",
    "xxx xxxxxx xxxxxxx xxxxxxx xxxxxxx xx xxx xxxxx’x xxxxxx."
  ],
  "xxxxxxxxxx_xxxxxxx": [
    "xxx xxxx xx xxx xx xxxxxxxx'x xxxxxxxx xxxxxxxx.",
    "xxxxxxx xxxxxxx xxxxxxxxx xxxx xxxx xx xxxxxxxxxxx xxxx xxxxxxx.",
    "xxxxx xx xxxxxx xxxxx xx xxxxxxxxxxx xxx xxx xxxxxxxxxx.",
    "xxx xx xxx xxxxx xxx xxxxxxxx xx xxx xxxx xx xxxxxxxx.",
    "xxx xxxxxxxxxx xxxxxxxxx xx xxxx xxxxxxxxxxxx xxxxxx xxx xxxxxxxxxxx xxxxxx xxxxxxxx.",
    "xxxxxx xxxxxx xxxxxxx xx xxxx xxxxxxxxx xxx xxxxxxxxxx xxx xxxxxxxx xx xxxxxxx xxxxx.",
    "xxxxxxxxxxxx, xxxxx xxxxxx xx xxxxxx xxxxxx xxxxx xx xxxxxxx xxxxxxxx.",
    "xxx xxxxxxxxx xx xxxx-xxxxxx xxxxxxxxxx xxxxxxx xxxxx xxxxx xxxx.",
    "xxxxxxx xx xxxxxxxxxx, xxx xxxxxxx xxxx xx xxxxxxx, xxxxxxx xxxxxxx xxxxxxxx.",
    "xxxxxx xxxx xx xxxxxxx xxxxx xx xxxxxxx xxxxxxxxx.",
    "xxxxxxxx xxxxx xxxxx xxxxx-xxxxxxxx xxxxxx xxxxx xx xxxxxxxxxxxx xxxxxxxx.",
    "xxxxxxxxxxx xxxx xxxxxx xxxxxxxxx xxx xxxxxxxxxxx xx xxx, xxxxxx, xxx xxxxxxxxx xxxxxx.",
    "xxxx xxxxxxxxxxx xx xxxxxxxxx xxxxxxxxx xxxxxxxxxx xxxx xxxxxx.",
    "xxxx'x xxxxxxxxxx xxxx xx xxxxxxx xxxxx xxx xxxxxxx xxxxxxxxx.",
    "xxxxx xxx xxxxxxxx xx xxxxx xxxxxxx xxx xxxx xxxxxxx.",
    "xxxxx xx xxxxxxxxxx xxxxxxxxxx xx xxxxx xxx xxxxxxx xxx xxxxxxxx.",
    "xxxxxxxxxx xxxxxxxx xx xxxxxx xxxxxxx xxxxxxx xxxxxx xxxxx.",
    "xxxxxx xxxxx xxxxxxx xx xxxxxxx xx xxxx xxx xxxxxxxxxx xxxxxx xxxxxx.",
    "xxxxxxxxxxxxxx xxxxxxxx xx xxxxxx xxxxxxx xx xxxxxxx xxxxx.",
    "xxxxx xxxxxxxxxx xx xxxxx xxxxxx xxx xxxxxxxxx xxxxxxxxx xxx xxxxxxxxx."
  ],
  "xxxxxxxxxxxx_xxxxxxx": [
    "xxx xxxx xx xxxxxxx xxxxxxxxxx xx xxx xxxxxxxxxx, xxxxxx xx xxxxxxxxxx.",
    "xxxxxxx xx xxx xxxxx xxxxx xxxx xxxxxxxxx xxxxxx xx xxxxxx xxxxx.",
    "xxx xxxxx xxxx xx xxxxx xxx xxxxxxxxxx xxxxxxx xx xxxxxxxxxxxx xxxxxxxxxxx.",
    "xx xxxxxxxxxxxxxx, xxxxxxx xxxxxx xxxxxxxxxxxx xxxxxxxx xxxxxxx xx xxxxxxx.",
    "xxxxx xxxxx xx xxx xxx xxx xxxxxxxx xxxx xxxxx xx xxxxx xxxxx.",
    "xxxxxxxx xx xxxxx'x xxxx, xxxx xx xxx xxxxx, xxxxxxxx xxxxxxxxx xxxxxxx xxxxxxxxxx xxxxxx.",
    "xxxxxx xxxxxxx xx xxx xxxxxx xé xxxxxx, xxxxxx xx xxxxxx xxxxx.",
    "xxxxxxxx xxxxxxxxx xxxxxxxxx xxxxxxxxx xxxxxx xx x xxxx, xxxxxxxx xxxxxxxx xxxxx xxxxx xxxxx xxx xxx.",
    "xxx xxxxxx xx xxxxxxxx xxxxxxxxxxx xxxx-xxxxxxxx xxxxx xxx xx xxxxxxxxx xxxxxx.",
    "xxxxxxxx xxxxxxxxxx xx xxx xxxxxxxxxx xxxxxxxx xxxxxxxxxxx xx xx'x xxxxxxx.",
    "xxxxxxxx xxxxxxx xx xxx-xxxxxx xxxxxx xxxxx xxxxxx xxxxxx xxxxx.",
    "xxxxxxxx’ xxxxx xxxxxxxxx xxx xxxxx-xxxxxxxxx xxxx xx xxxxxxxx.",
    "xxxxxxxx xxxxxxxx xx xxxxxxxx xxxx xxx xx xxxxxxxxxxx xxxxx.",
    "xxxx'x xxxxx xxxxxx xxxxxxxx xxxxx xx xxxxxxx xxxxxxxx xxxxxx.",
    "xxxxxxxxx xxxxxxx xxxxx xxx xxxx xxxxx xxxxx xxxxxx xxxxx xx xxxxxx xxxxxxx."
  ],
  "xxxxxxxx_xxxxxxx": [
    "xxx xxx xx xxxxx xxx xxxxxxxxxx xxx xxxxxxx xx xxxxxxxx xxxxxxxx.",
    "xxxxxxxxxxx xxxxxxxxxxxx xxxxxx xxxxxxxx xxxxx xx xxxxxx xxxxx.",
    "xxxx xxxxxxxxx xxxxxxxxx xxxxxxx xxxxxx xxxxxxx xxxx xxxxxxxxxx.",
    "xxxxx xxxxxxx xxxxxxxxxxxxx xxxx xx x xxxxx xx xxxxxxxxx xxxx xxxxxxxxx xxxxxx xxxxxx xxxxxxxx.",
    "xxxxx xxxxxxxxxx xx xxxxxxxx xxxx xxxxxx xxxxxxx xxxxx xxxxxx xxx xxxxx.",
    "xāxxx xxxxxxxx xxxxxxxxxx xxxxxxx xx xxxxxxxx xx xxxx-xxx xxxxx.",
    "xxxxxx xxxxxxxxx xxx xxx xxxxx xx x xxxxxx xxxxxxxxxx xxxxxx xxxxxxx.",
    "xxxxxx xxxxxxxx xxxxxxxx xxxxxxx xxxxxxxxx xxxxxxxxxx xx xxxx xxxxxxxxxxx.",
    "xxxxxxx xxxx xxxxx xxxxxxxxx xx xxxxxxxx xxxxxxxx xxxxx.",
    "xxxxx xxxxxxxxxx xx xxxxx xxxxx xx xxxxxx xxx xxxxxxxxxx xxxxxxxxxx.",
    "xxxxxxxx xxxxx xxxxxxxxx xx xxx xxxx xxxx, xxxxxxxxx xx xxxxxxx.",
    "xxxxx xxxxxxxxxxxx xxxxx xxxx xx xxxxxxxxx xxxxxx xxxxxxxxxxx xx xxxxxxx."
  ],
  "xxxxxxxxx_xxx_xxxxxxxxxxxxx_xxxxxx_xxxxxxxxx": [
    "x xxxxx xx xxxxxxxx xxxxxxxx xxx xxxxxxxxx.",
    "xxxxxxxx xx xxxxxxxxx xxx xxxxxxx xxx xxxxxx xxxxx.",
    "xxxxxxxxx xx xxxxxxxxx xxx xxx xxxxxx xx xxxxxxxxx.",
    "x xxxxxxxx xxxxx xx xxxxxxxxxx xxxx xxx xxxxx xxxxx.",
    "xxxxx xxx xxxxxxxxxxx xxxxx xx xxxxxx xxxx xxxxxxx xxxxxxxxxx.",
    "xxxx xxxxxxxx xxxxxxxxx xxxx xxxxxxxxxx xxx xxxxxxxxxxxxx.",
    "xxxxxxxx xx xxxxxxx, xxxxxxx xx xxxxxxxxxx xxxxx.",
    "xxx xx xxx xxxxxxxx xxxxxx xx x xxxxxx xxxxx.",
    "xxxxxxxx xxxxxxxxxxxxx xxxxxxxxxx xx xxxxxx xxx xxxxx xxxxxxxxx.",
    "x xxxxxxxxx xxxxx xx xxxx xxx xxxxxxxxxxxx xxxx xxx xxxxxxx.",
    "xxxxxxxxx xx xxxxxxxxxx, xxxxxxxx xx xxxxxx xxxxxxxxxx.",
    "xxxxxxxxxx xxxxxxxxx xx xxxxxxxxx xx xxxxxx xxxxxxxxx."
  ],
  "xxxxxxxx_xxx_xxxxxxxxxxxxxx_xxxxxxxx": [
    "xxxxxxxx xxxxx xxx xxxxxx xxxx xxxx xxxxxxxxxxxx xxxx'x xxxxxxxxx xxxxxx.",
    "xxxx xx x xxxxxxxxxxxx, xxxxxxxxx xxx xxxxxxxxxxxx xxx xxxxxxxxxxx.",
    "xxx xxxxxxx xx xxxxx xxxxxx xx xxx xxxxxxx xxxxxxx xx xxxxx.",
    "xxx xxxxxxxxxxxxxx xxxx xxxx xx xxxxxxx xxxxxx xxxxxxxx xxxxxx.",
    "xxxxxxx xxxxxxxxx xx xxxxxxxxxx xx xxx xxxxxxxxxxx xxxxxx xxx xxxx.",
    "xxxxxx xxxxxx xx xxxxxxxxxx xx xxx xxxxxxxxxxx xx xxxxx.",
    "xxxxxxxxxxxx xxxxxx xx xxxxxxx xxx xxxxxxx xx xxx xxxx'x xxxxxxxx.",
    "xxx xxxxxxxx xxxxxx xx xxxxx xxxxxxxxxx xxxx xxxxxxxxx xxxxx.",
    "xxxxxxxxxx xxxxxx xx xxxxxxxxx xxxxxxxxxxx xxx xxxx xxxx.",
    "xxxxx xx xxxxxxxxxxx xxxxxx xxxxxxxxx xxxxxxxxxxx xx xxx xxxx'x xxxxx.",
    "xxxxxxxx xxxxxxxxxxxxxx xx xxx xxxxxxx xxxx xxx xxxxxxxx xxxx xxxxx.",
    "xxx xxxxxxx xxxxxxx xx xxxx xxx xxx xxxxxxxxx xxxx'x xxxxxxx."
  ],
  "xxxxxx_xxx_xxxxxxxxxx_xxxxxxxx": [
    "xxxxxx xxxxx xxxxxx xx xxxxxx xxxxxx xxxxxxxxxxx.",
    "xxxxxxxxxxxxx xxxxxxxxx xx xxxxxxxxxxxxxx xxxx xxxx xxxxxxxxxxxx.",
    "xxxxxx xxxxxxx xxx xxxxxx xxxxxxxxxx xxxxxx xxxxxx xxxxxx.",
    "xxxxxxxxxx xx xxxxx xxxxxxxxx xxxxxx xx xxxxxxxx xxxxxx.",
    "xxxxxxxxxxx xxxxxx xx xxx xxxxxxxx xx xxxxxx xxxxxxxx.",
    "xxxxxxxx xxxxxxx xxxxxxxxxx xx xxxxxxxx xxxxxxxxxxxx.",
    "xxxxx'x xxxxxxxx xxxxxxx xx xxxx xxxx xxxx xxxxxxxx.",
    "xxx xxxxxxxxxxx xx xxxxxx xxxxxxxxx xxxxx xxxxxx xxxxxxxxx xxxxx.",
    "xxxxxxxx xxxxxxxx xx xxxxx xx xxxxxx xxxxx'x xxxxxxxxx.",
    "xxxxxx xxxxxxxxx xx xxxxx xxxxx xxxxxxx xxxxxxxxx xx xxx xxxxxxx xxxxx.",
    "xxxxxx xx xxxxxxxxxx xxx xxxxxxxxxxxx xxxxxx xxxxxxxxx.",
    "xxxxxxxx xxxxxxxx xxxxxxxxxxx xxxxxxxxxxx xxx xxxxxxxxxxx."
  ],
  "xxxxxxxxx_xxx_xxxxxxxxx_xxxxxxxxx": [
    "xxxx xxxxxxxxxxx xxxx xxxx, xxxxxxxxxx xxxxx xxx xxx xxxxxxxx.",
    "xxx xxxxx'x xxxxx xxxxxxxxxx xxxxx'x xxxx xxx xxxxxxxxxx.",
    "xxxxxxx xxx xxxxx, xxx xxxxx xxxxxxxxxx xxxxxx xxxxxx xxxxxxxx.",
    "xxxxx xxxxxxx xxxxxx xxxxxxxx xxxxxxxxxxx xxx'x xxxxxxxxxxx xxxxxxx.",
    "xxxxx'x xxxxxxxxx xxxxxx xxxxxxxxxx xx xxxxx xxxxxx xxxxx.",
    "xxxxxx xx xxxxxxx xxxxxx xxxxxx xx xxxxxxxx xxxxxxxxxxx.",
    "xxxxx’x xxxxx xxxxxxxx xxxxxxxxx xx xxxxxxxxxx xxxxxxxxxx.",
    "xxxxxxxxx xx xxxx’x xxxxxxxx xxxxxx xxxxxxxxxxx xxxxxxxxxxxx xxxxx.",
    "xxxxxx xxxxxx xx xxxx xxxxxxxxx xxxxxxxx xxxxxxxx xxxxxxx xxxxx.",
    "xxx xxxxxxxxx xx xxxxx'x xxxxxxxxxxxx xxxx xxx xxxxx'x xxxxxxxx.",
    "xxxxxxxxxxxxx xxxxxx xx xxxxxxxxx xxxxxx xxxxxx xxxxx.",
    "x xxxxxxx xxxxxxxx xx xxxxxxxx xxx xxxxxxxx xxxxxx."
  ],
  "xxxxxxxxx_xxx_xxxxxxxxx_xxx_xxxxxxxxxx": [
    "x xxxxx xxxxxxx xxxxxx x xxxxxxxxxxx xxxxxxx xxxx xxxxxx xxx xxxxx.",
    "xxx xxxx xxxxxxxxxxxx xxxxxxxx xx xxx xxxxxxx xx xxxxx'x xxxxxxxxx.",
    "xxxxxxx xxx xxx xxxxxxx xxxxxxxxxxxx xxx xxxxxxxxx xxxxxxxxxx.",
    "x xxxxxxxxx'x xxxxxxxxxxxxxx xxxxxxxxx xx xxxxxxxx xxxxxx xx xxx xxxxx-xxx xxxx.",
    "x xxxxxxxx xxxxxx xxxxxxxx xxxxxxx xxxxxxxxxx xxxxxxx."
    "xxxxxx xxxxxxx xxxxxxxxx xxxx xx xxxxxxxxxx xx xxxx’x xxxxxx.",
    "xxx xxxxxxxxxx xx xxxxxx xxxxxxxxx xxxxxxxxxxx xx xxxxx.",
    "xxxxxx xxxxxxxxx xxxx xxxxxx xxxx xxxxxxxx, xxxxx xxxx xxx xxxxx'x xxxxxxx.",
    "xxxxxxx xxxxx xxxx xxx xxxx'x xxxxxxxx xxxxxx xxx xxxx'x xxxxxxx.",
    "xxx xxxxxxx xx xxxxxxxxxx xxxxx xxxxxx xxxxxxxxx xxxxxxxx.",
    "xxxxx xxx xxxxxxxxxxxxx xxxxxxxxxxxx xxxxxxx xxxx xxxxx xxxxxxx.",
    "xxx xxxx'x xxxxxxxxxxxxx xxxxxxx xxxxxxxxxx xxxxxxx xxxxxxxx xxxxx."
  ],
  "xxxxxxxxxx_xxx_xxxxxxxx_xxxxxxxxx": [
    "xxxxxxxxx xxxxxxxxxxx xx xxxxxxxxx xxxxxxxxx xxxxxxx xxxxx xxx xxxx.",
    "xxxxxxxx xxxxxxx xxxxxx xx xxx xxxxxx xx xxxx xxx xxxx.",
    "xxxxxxxxx xxxxxxxx xxxxxxx xxxxxxxx xxxx xx xxxxxx xx xxxxx.",
    "xxxxxxxxxx xxxx xxxxxxx xx xxxxxxxx xxxxxxxxxx xxx xxxxxxxxxxx.",
    "xxxxxxxx xxxxxxx xxxxxxx xxx xxxxxxx xxxxx xx xxxxxxx xxxxxx.",
    "xxxxxxxxxxx xxxxxxxxx xxxxxxxxxxxx xxxxx xxx xxxxxxxxx xx xxxxxxxxx.",
    "xxxxxxx xxxxxxxxx xxxx xxxxxxx xxxxxxx xxxxxxxx xx xxxxxx xxxxxxxx.",
    "xxxxxxxxxxx xxxxxxxxx xx xxx xxxxx xxxx xxxxxxxxxx' xxxxx xxxxxxxx.",
    "xxxxxxxxxxx xxxxxxxx xxxxxxx xxxxxxx xxxxx xxxxxxxxxxxx.",
    "xxxxx xxxxxxx xxxxx xx xxxxxx xxx xxxxxxx xx xxxxx xxx xxxxxxx.",
    "xxxxxxxx xxxxxxx xx xxxxxxxxx xxx xxxxxxx xx xxxx xxx xxxxx xxx xxxxxx xxxxxxxxx.",
    "xxxxxxxxx xxxxxxxxxx’ xxxxxxxx xxxxxxxxxx xx xxxxxxxxxxx xx xxxxx xxxxxxxxxxx."
  ],
  "xxxxxxxx_xxxxx_xx_xxxxxx_xx_xxx_xxxxxxxx": [
    "xxxxxx x xxxxxxxxx xxxxxx xxxx xxxxxx xxx xxxxxxx xxxxxxx xxxxxxxxxxxx.",
    "xxxxxx xxxxxxxx, xxxxxxxx xxxx xxx xx xxxxxxxx xxx xxxxxxxx xxxxx.",
    "xxxxxx xxxxxxxxx xxxxxxxx xxx xxxx xxx xxxx xxxxxxxxxxx xx xxxxx.",
    "xxxxxxx xxx xxxxxxxx xxxx, xxxxxxxx xxxxxx xxxxx xxxxxxxxxx xxxxxxx xxxxxxxxx.",
    "xxxxxxxx xxx xxxxxxx xxxxxx xxxx xxxxxx xx xxxxxxxxx xxx xxxxx.",
    "xxxxxxx xxxx xxx xxxxxx xxx xxxxxxxxx xxx xxxxxxxxxxx xxxxx.",
    "xxxxxxx xxxxxxx xxxxxxxxxx xxx xxxxxxxx xxxxxxxxxx xxx xxxxxxxxx.",
    "xxxxxxx xxxxxxxx xxxxx xxxxxxx xx xxxxxxxxxx xxx xxxx'x xxxxxxx xxxx.",
    "xxxxxxxxx xxxxxx xxxxxxxx, xxxxxxxxxx xxxxxxxxxxxx xxxxx xxxxxxxx xxxxxx.",
    "xxxxxxxxx xxxxxxx xxx xxxxxxxx xxxxxxx xxx xxxxxxxxxxxxx xxxxx.",
    "xxxxxxxx x xxxxxxx xxx xxxxxxxxxxx xxx xxxxxxxxxx xxx xxxxx.",
    "xxxxxx xxxxxxxx xxxxxx xxx xxxxx xx xxxxxxx xxx xxxxxx xxxxxxxx."
  ],
  "xxxxxxxxx_xxx_xxxxxxxxxx_xxxxxxxxx": [
    "xxxxxxxxxx xxxxxxx xxxxxxxx xxxxxxx xxxxxxx xxxxxx xxxx.",
    "xxxxxxxxxx xxxxxxxxxxxxx xxx x xxxxxxx, xxxxxxxx xxxxx.",
    "xxxxxxx xxxxxxxxxxx xxxxxxx xxxxxxxxxxx xxxxx xxx xxxx xxxxxxxxx.",
    "xxxxxxx xxxxxx-xxxx xxxxxxxxxx xxxxxx xxx xxxxxxxxxx xxxxxxxx xxxxx.",
    "xxxxxxx xxxxxx xxxx-xxxxxxxxx xxxxxxx xxxxxxx xxxxxxxxxxx xx xxxxx.",
    "xxxxx xxxxxxxxx xxxxxxxxxx xxxxxxxx xxxxxx xx xxxxxxx xx xxxx.",
    "xxxxx xxxxxxxxx xx xxxxxx xxxxxxxxx xxxxxxx xx xxxxxx xxxxxxxxxxx."
    "xxxxxxxx xxxxxxxxxxx xxxxxxxx xxxxxxx xxxxxxxxx xxxxxxxxxx.",
    "xxxxxxxxx xxxxxxx xxxxxx xxxxxxx xxxxxxx xxx xxxxxxxxx xxxx.",
    "xxxxxxxx xxxxxxxxxxxxx xxxx xxxxxx, xxxxxx xxxxxxxxx xxxxxx."
    "xxxxxxx xxxxxxxxxxxx xxxxxxx xxxx xxxxxxxxxx xxxxxxx xxxxxxxx.",
    "xxxxxxxxx xxxxxxxxxx xx xxxxxxxx xxxxxx xxxx xxx xxxxxxxxxx xxxxxx."
  ],
  "xxxxxxx_xxx_xxxxxxx": [
    "xxxxxxxxxxxxx xx xxxxxxxxxxx xxx xxxxxxxxxx xx xxxx xxx xxxxx.",
    "xxxxxxxxxxxx xxxxxxxxxx xx xxxxxxxxx xxx xxxxx xxxxxx xxxxxxxx xxxxx.",
    "xxxxxxxxxxxx xxxxxxx xxxx xxx xxxxxxxxxxx xx xxx xxxxxxx xxxx.",
    "xxxxx’x xxxxxxxx xxx xxxxxxxxxxx xxxxxxxxxxx xxxx’x xxxxxxxx xxxxxxx.",
    "xxxxx xxxxxxx xxxx xxxxxxxxxx xx xxxxxxxxxx xxxx'x xxxxxxxxxxxxxxxx.",
    "xxx xxxx xx xxxxxxxxxxx xxxxxxxxxxx xxxxxxxxx xxxxxxxx.",
    "xxxxxxxx xxxxxxxxx x xxxxxxx xxx xxxxxxxx xxxxxxxxxxxx.",
    "xxxxxxxx xx xxxxxx xxxxxxx xxxxxx xx xxxxxxxxxxx xx xxxx."
    "xxxxxxxx xxxxxxxx xxxx xxxxxxxx xxxxxxxxxxxx xxx xxxxx.",
    "xxxx xxxxxxxxxxx xxxxxx xxxxxxxxxx xxxxxxxx xx xxxx xxx xxxxxxxx.",
    "xxxxxxxx xxxxxxx xxxxxxxxxxxx xxxx xxxxxxx xxxxxx xxx xxxxxxx.",
    "xxx xxxxxxxx xx xxxxxxxxx xxxxxxxxxxxx xx xxxxxx xx xxx xxxx."
  ],
  "xxxxxxxxx_xxxxxxx": [
    "xxxxxx xx xxxxx xxxxxxxx xxxxxxxx xxxxxx xxxxxx xxxxxxxx.",
    "xxxxxxxxxxx xxxxxxxxxx xx xxxxxxxxxx xxxxxx xx xxxxxxx xxx xxxxx'x xxxxxxxxxx.",
    "xxxxxxxx xxxx xx xxx xxxxxxxx xxxxxxxxxx xxxxxxx xxxxxxx xxxxxxx xxxxxxxxxxx.",
    "xxxxxxx xxxxxxx xxxxxx xxxxxx xxxxxxx xxxxxxx xxxxx xxx xxxxxxxxx.",
    "xxxxxxxxxx xxx xx xxxxxx xxxxxxxx xx xxxxxxx xxxxxxx xxxxxxxxx.",
    "xxxxxx xx xxxxx xxxxxx xx xxxxxxxxxxx xxxxxxxxx xxxxxxx.",
    "xxxxxxxxxxx xxxxxxxxx xx xxxxx xxx xxxxxxxxxxxxxx xx xxxxx.",
    "xxxxxxxxx xxxxxxxxx xxxxxxxx xxxx xxxxx xxxxxxxxx xxxx xxxxxxxxx xxxxxxxxx.",
    "xxxxxxxxxx xxxxxxxxx xxx xxxxx xxxxx xx xxxxx xxxxxxxx xx xxxxx xxxxxxxxxxx.",
    "xxxx xxxxxxxxx xx xxxxxxxx xx xxxxxx xxxx xxxxxxxxxx xxxxxxxx.",
    "xxxxxxxxxx xxx xx xxx xxxxxxx xx xxxxxxx xxxxxxxx xxxxx xxxxxxxxx.",
    "xxxx xxxxxxxxx xx xxxxxxxxxxxx xxxx xxxxxxxxxxxx xxxxxxxxxx xxxxxxxxxxxx."
  ]
}"#;
        let expected = r#"{
  "xxxxx_xxxx": "xxxxxxxxxxxxxxxxxxxxxxxxxx",
  "xxxx_xxxxxxx_xxx_xxxxxxxxx": [
    "xxx xxxxx xx xxxxxx xxxxxxxx xxxx xxxxxx xxxxx.",
    "xxxxxxx xx xxxxxxx xxx xxxxxx xxxxxx.",
    "xxx xxxxxxxxxxx xx xxx xxxxxxx xxx xxxxxxx xxxxxxxx.",
    "xxxxxxxxx xxx xxxxxxxxxxxxx xxxxxxxxxx xxxx xxxxxxxx xxxxxxxx.",
    "xxx xxxxxxxxx xx xxxxxxxxx xxx xxxxxxxxxxxx xxxxxxxxxxxxx.",
    "xxx xxxxxx xx xxxxxxxxxxxxxx xxxxxxxx xxxxxxxxxxx.",
    "xxxxxxxx xxxxxxx xxxxxxx xxxxxxxxxxx xxx xxxxxxxxxx.",
    "xx xxxxxxxxxxxx xx xxxxx xxxxxx xx xxxxxxxxx xxxxxx.",
    "xxx xxxxx'x xxxxxxx xxxxx xxxxxxxxxxxx xxx xxxxxxxxxxxx.",
    "xxx xxxxxxxxxx xxxxx xx xxxxxxxxxxxxx xxx xxxxxx.",
    "x xxxxxx xx xxxxxxxxxxxx xxxxxx xx xxxxxxxx.",
    "xxx xxxxxxxxx xx xxxxxxxxx xxxxxx xx xxxxxxxxx.",
    "xxx xxxxxxx xxxxx xx xxxxxxxxxxx xxxxx xxx xxxxxxxxx xxxxxxxxx.",
    "xxxxxxxxx xx xxxxx xxxxxxxxx xx xxxxx xxxxxxxxxxx.",
    "xxx xxxxxxxxxxxxxxx xx xxxxxxxx xxxxxxx xxxx xxxxxxx xxxxxx.",
    "xxxxxx xx xxxxxx xxxxxxxx xxxxxx xxxxxxxx xxxxxx.",
    "xxx xxxxx xxx xxxxx xx xxxxxx xxx xxxxxxxx.",
    "xxxxx'x xxxxx xx xxxxxxx xxx xxxxxx xxxxxxx.",
    "xxxx xxxxxx xxxx xxxxxxxx xxxxxxx xxxx'x xxxx.",
    "x xxxxxxx xxxx xxx xxxxxx xx xxxxxxx xxxxxx xxxxxxx."
  ],
  "xxxxxxx_xxx_xxxxxxxx_xxxxxxxxxxx": [
    "xxxxxxxxxx xxxxxxx xx xxxxxx xxx xxx xxxxxxx xxxxxxx xxxxxxx.",
    "xxxxx xxxxxxx xxxxx, xxxx xx xxx xxxxx xxx xxxxxx xx xxxxxxx xx xxx xxxxx.",
    "xxx xxxxx xx xxxxxx xxxxxxxx xxxxxxx xxxx xxxxxxxx xxxxxxx xxxx.",
    "x xxxxx, xxxxxxxxx xxxx xxxxxxxxxx xxx xxx xxxxxx'x xxxxxxxx.",
    "xxxxxxx xxxxxxxxxx xxxxx, xxxxxxxxxx xxxxxx xxxx xxxxx xxxxxx.",
    "xxxx xxxxxxxxx xxxxxx xx xxx xxxx xxxx xxxxxxxxx xx xxx xxxxx.",
    "xxxxx xxx xxxxxxx xxxxxxxx xxxxxxx xx xxxx xxxxxxx.",
    "xxx xxx xxxxxx xx xxxxxxx xxxxxxxxxxx xxxxxx.",
    "xxx xxxxx xxxxxxxx xx xxxxxxxx xxxxxx xxx xxxx'x xxxxx.",
    "xxxxxxxxxx xxxxxx xx xxxxxxxx xx xxx xxxx'x xxxxxxx xxxxxxxxxx xxxxx.",
    "xxx xxxxx xx xxxxxxxxxx xxxxxx xx xxxxxx xxxxxx xxx xxxxx.",
    "x xxxxxxxx xxxxxxx xx xxxxxxxxx, xxxx xxxxx xxx xxxxxxxx.",
    "xxxxxx xxxxxx xxx xxxxxxx xxxxxxx xxxx xxxxx xxxxxxxxx.",
    "x xxxxxxx xxxxx xx xxxxx xxx xxxxxx xx xxxx xxxxxxxx.",
    "xxx xxxxx'x xxxxxxxx xxxxxxx xxxxxxx xxxxxxx xxxxxxxxxxx xxxxxx xxxxx.",
    "xxx xxxx xxxxxxx xx xxx xxxxxx xx xxxxx xxxxxxx xxxx xxxx.",
    "xxx xxxxx xxxxx xx xxxxxxx xxxxxx xxxx, xxxxxxx xxx.",
    "xxxxxxxxxxx xxx xxxxxxxxxxx xxxxxxxx xxx xxxxx'x xxxxxxx.",
    "xxx xxxxxxx xxxxxxx xx xxxxxx, xxxxxx xxx xxxx xxxx xxxx.",
    "x xxxxxxxxxx xxxxxxx xxxxxx xx xxxx xxxxxxxxx xxx xxxxx.",
    "xxxxx'x xxxxxxxx xxxxxxxxxx xx xxxx xxxxxx, xxxxxxxxx xxxxx.",
    "xxx xxxx'x xxxxxxxx xxxxxx xx xxx xxxxxx xx xxx xxxxxx.",
    "xxxxxxxx xxxxxxxx xx xxxx xx xxxxxxx, xxxxxxx xx xxxxxx xxxxxxx.",
    "xxx xxxxxxx xxxxxxx xx x xxxxxx xxxxxx xxxxxxxxxx xx.",
    "xxx xxxxxxxx, xxxxxx xxxxxxxx xx xxxxxxx xxxx xxxxx.",
    "xxxxxx xxxxxxxx xxxxxxx xxx xxx xxxx xxxxxx xxxxxxxxx.",
    "xxxx xxxx xxxxxxxxx, xxxxxxxx xx xxxx xxxxxxx xxxxxx.",
    "xxxxxxxxxxx xxxxxxx xx xxxx xxxxx xx xxxxx xxxxxxxxxx.",
    "xxxxxxxxx xxxxxxxxxx xx xxxxxxxxx xxxxxx xxxxxx’x xxxxxxx.",
    "xxx xxxxxx xxxxxxx xxxxxxx xxxxxxx xx xxx xxxxx’x xxxxxx."
  ],
  "xxxxxxxxxx_xxxxxxx": [
    "xxx xxxx xx xxx xx xxxxxxxx'x xxxxxxxx xxxxxxxx.",
    "xxxxxxx xxxxxxx xxxxxxxxx xxxx xxxx xx xxxxxxxxxxx xxxx xxxxxxx.",
    "xxxxx xx xxxxxx xxxxx xx xxxxxxxxxxx xxx xxx xxxxxxxxxx.",
    "xxx xx xxx xxxxx xxx xxxxxxxx xx xxx xxxx xx xxxxxxxx.",
    "xxx xxxxxxxxxx xxxxxxxxx xx xxxx xxxxxxxxxxxx xxxxxx xxx xxxxxxxxxxx xxxxxx xxxxxxxx.",
    "xxxxxx xxxxxx xxxxxxx xx xxxx xxxxxxxxx xxx xxxxxxxxxx xxx xxxxxxxx xx xxxxxxx xxxxx.",
    "xxxxxxxxxxxx, xxxxx xxxxxx xx xxxxxx xxxxxx xxxxx xx xxxxxxx xxxxxxxx.",
    "xxx xxxxxxxxx xx xxxx-xxxxxx xxxxxxxxxx xxxxxxx xxxxx xxxxx xxxx.",
    "xxxxxxx xx xxxxxxxxxx, xxx xxxxxxx xxxx xx xxxxxxx, xxxxxxx xxxxxxx xxxxxxxx.",
    "xxxxxx xxxx xx xxxxxxx xxxxx xx xxxxxxx xxxxxxxxx.",
    "xxxxxxxx xxxxx xxxxx xxxxx-xxxxxxxx xxxxxx xxxxx xx xxxxxxxxxxxx xxxxxxxx.",
    "xxxxxxxxxxx xxxx xxxxxx xxxxxxxxx xxx xxxxxxxxxxx xx xxx, xxxxxx, xxx xxxxxxxxx xxxxxx.",
    "xxxx xxxxxxxxxxx xx xxxxxxxxx xxxxxxxxx xxxxxxxxxx xxxx xxxxxx.",
    "xxxx'x xxxxxxxxxx xxxx xx xxxxxxx xxxxx xxx xxxxxxx xxxxxxxxx.",
    "xxxxx xxx xxxxxxxx xx xxxxx xxxxxxx xxx xxxx xxxxxxx.",
    "xxxxx xx xxxxxxxxxx xxxxxxxxxx xx xxxxx xxx xxxxxxx xxx xxxxxxxx.",
    "xxxxxxxxxx xxxxxxxx xx xxxxxx xxxxxxx xxxxxxx xxxxxx xxxxx.",
    "xxxxxx xxxxx xxxxxxx xx xxxxxxx xx xxxx xxx xxxxxxxxxx xxxxxx xxxxxx.",
    "xxxxxxxxxxxxxx xxxxxxxx xx xxxxxx xxxxxxx xx xxxxxxx xxxxx.",
    "xxxxx xxxxxxxxxx xx xxxxx xxxxxx xxx xxxxxxxxx xxxxxxxxx xxx xxxxxxxxx."
  ],
  "xxxxxxxxxxxx_xxxxxxx": [
    "xxx xxxx xx xxxxxxx xxxxxxxxxx xx xxx xxxxxxxxxx, xxxxxx xx xxxxxxxxxx.",
    "xxxxxxx xx xxx xxxxx xxxxx xxxx xxxxxxxxx xxxxxx xx xxxxxx xxxxx.",
    "xxx xxxxx xxxx xx xxxxx xxx xxxxxxxxxx xxxxxxx xx xxxxxxxxxxxx xxxxxxxxxxx.",
    "xx xxxxxxxxxxxxxx, xxxxxxx xxxxxx xxxxxxxxxxxx xxxxxxxx xxxxxxx xx xxxxxxx.",
    "xxxxx xxxxx xx xxx xxx xxx xxxxxxxx xxxx xxxxx xx xxxxx xxxxx.",
    "xxxxxxxx xx xxxxx'x xxxx, xxxx xx xxx xxxxx, xxxxxxxx xxxxxxxxx xxxxxxx xxxxxxxxxx xxxxxx.",
    "xxxxxx xxxxxxx xx xxx xxxxxx xé xxxxxx, xxxxxx xx xxxxxx xxxxx.",
    "xxxxxxxx xxxxxxxxx xxxxxxxxx xxxxxxxxx xxxxxx xx x xxxx, xxxxxxxx xxxxxxxx xxxxx xxxxx xxxxx xxx xxx.",
    "xxx xxxxxx xx xxxxxxxx xxxxxxxxxxx xxxx-xxxxxxxx xxxxx xxx xx xxxxxxxxx xxxxxx.",
    "xxxxxxxx xxxxxxxxxx xx xxx xxxxxxxxxx xxxxxxxx xxxxxxxxxxx xx xx'x xxxxxxx.",
    "xxxxxxxx xxxxxxx xx xxx-xxxxxx xxxxxx xxxxx xxxxxx xxxxxx xxxxx.",
    "xxxxxxxx’ xxxxx xxxxxxxxx xxx xxxxx-xxxxxxxxx xxxx xx xxxxxxxx.",
    "xxxxxxxx xxxxxxxx xx xxxxxxxx xxxx xxx xx xxxxxxxxxxx xxxxx.",
    "xxxx'x xxxxx xxxxxx xxxxxxxx xxxxx xx xxxxxxx xxxxxxxx xxxxxx.",
    "xxxxxxxxx xxxxxxx xxxxx xxx xxxx xxxxx xxxxx xxxxxx xxxxx xx xxxxxx xxxxxxx."
  ],
  "xxxxxxxx_xxxxxxx": [
    "xxx xxx xx xxxxx xxx xxxxxxxxxx xxx xxxxxxx xx xxxxxxxx xxxxxxxx.",
    "xxxxxxxxxxx xxxxxxxxxxxx xxxxxx xxxxxxxx xxxxx xx xxxxxx xxxxx.",
    "xxxx xxxxxxxxx xxxxxxxxx xxxxxxx xxxxxx xxxxxxx xxxx xxxxxxxxxx.",
    "xxxxx xxxxxxx xxxxxxxxxxxxx xxxx xx x xxxxx xx xxxxxxxxx xxxx xxxxxxxxx xxxxxx xxxxxx xxxxxxxx.",
    "xxxxx xxxxxxxxxx xx xxxxxxxx xxxx xxxxxx xxxxxxx xxxxx xxxxxx xxx xxxxx.",
    "xāxxx xxxxxxxx xxxxxxxxxx xxxxxxx xx xxxxxxxx xx xxxx-xxx xxxxx.",
    "xxxxxx xxxxxxxxx xxx xxx xxxxx xx x xxxxxx xxxxxxxxxx xxxxxx xxxxxxx.",
    "xxxxxx xxxxxxxx xxxxxxxx xxxxxxx xxxxxxxxx xxxxxxxxxx xx xxxx xxxxxxxxxxx.",
    "xxxxxxx xxxx xxxxx xxxxxxxxx xx xxxxxxxx xxxxxxxx xxxxx.",
    "xxxxx xxxxxxxxxx xx xxxxx xxxxx xx xxxxxx xxx xxxxxxxxxx xxxxxxxxxx.",
    "xxxxxxxx xxxxx xxxxxxxxx xx xxx xxxx xxxx, xxxxxxxxx xx xxxxxxx.",
    "xxxxx xxxxxxxxxxxx xxxxx xxxx xx xxxxxxxxx xxxxxx xxxxxxxxxxx xx xxxxxxx."
  ],
  "xxxxxxxxx_xxx_xxxxxxxxxxxxx_xxxxxx_xxxxxxxxx": [
    "x xxxxx xx xxxxxxxx xxxxxxxx xxx xxxxxxxxx.",
    "xxxxxxxx xx xxxxxxxxx xxx xxxxxxx xxx xxxxxx xxxxx.",
    "xxxxxxxxx xx xxxxxxxxx xxx xxx xxxxxx xx xxxxxxxxx.",
    "x xxxxxxxx xxxxx xx xxxxxxxxxx xxxx xxx xxxxx xxxxx.",
    "xxxxx xxx xxxxxxxxxxx xxxxx xx xxxxxx xxxx xxxxxxx xxxxxxxxxx.",
    "xxxx xxxxxxxx xxxxxxxxx xxxx xxxxxxxxxx xxx xxxxxxxxxxxxx.",
    "xxxxxxxx xx xxxxxxx, xxxxxxx xx xxxxxxxxxx xxxxx.",
    "xxx xx xxx xxxxxxxx xxxxxx xx x xxxxxx xxxxx.",
    "xxxxxxxx xxxxxxxxxxxxx xxxxxxxxxx xx xxxxxx xxx xxxxx xxxxxxxxx.",
    "x xxxxxxxxx xxxxx xx xxxx xxx xxxxxxxxxxxx xxxx xxx xxxxxxx.",
    "xxxxxxxxx xx xxxxxxxxxx, xxxxxxxx xx xxxxxx xxxxxxxxxx.",
    "xxxxxxxxxx xxxxxxxxx xx xxxxxxxxx xx xxxxxx xxxxxxxxx."
  ],
  "xxxxxxxx_xxx_xxxxxxxxxxxxxx_xxxxxxxx": [
    "xxxxxxxx xxxxx xxx xxxxxx xxxx xxxx xxxxxxxxxxxx xxxx'x xxxxxxxxx xxxxxx.",
    "xxxx xx x xxxxxxxxxxxx, xxxxxxxxx xxx xxxxxxxxxxxx xxx xxxxxxxxxxx.",
    "xxx xxxxxxx xx xxxxx xxxxxx xx xxx xxxxxxx xxxxxxx xx xxxxx.",
    "xxx xxxxxxxxxxxxxx xxxx xxxx xx xxxxxxx xxxxxx xxxxxxxx xxxxxx.",
    "xxxxxxx xxxxxxxxx xx xxxxxxxxxx xx xxx xxxxxxxxxxx xxxxxx xxx xxxx.",
    "xxxxxx xxxxxx xx xxxxxxxxxx xx xxx xxxxxxxxxxx xx xxxxx.",
    "xxxxxxxxxxxx xxxxxx xx xxxxxxx xxx xxxxxxx xx xxx xxxx'x xxxxxxxx.",
    "xxx xxxxxxxx xxxxxx xx xxxxx xxxxxxxxxx xxxx xxxxxxxxx xxxxx.",
    "xxxxxxxxxx xxxxxx xx xxxxxxxxx xxxxxxxxxxx xxx xxxx xxxx.",
    "xxxxx xx xxxxxxxxxxx xxxxxx xxxxxxxxx xxxxxxxxxxx xx xxx xxxx'x xxxxx.",
    "xxxxxxxx xxxxxxxxxxxxxx xx xxx xxxxxxx xxxx xxx xxxxxxxx xxxx xxxxx.",
    "xxx xxxxxxx xxxxxxx xx xxxx xxx xxx xxxxxxxxx xxxx'x xxxxxxx."
  ],
  "xxxxxx_xxx_xxxxxxxxxx_xxxxxxxx": [
    "xxxxxx xxxxx xxxxxx xx xxxxxx xxxxxx xxxxxxxxxxx.",
    "xxxxxxxxxxxxx xxxxxxxxx xx xxxxxxxxxxxxxx xxxx xxxx xxxxxxxxxxxx.",
    "xxxxxx xxxxxxx xxx xxxxxx xxxxxxxxxx xxxxxx xxxxxx xxxxxx.",
    "xxxxxxxxxx xx xxxxx xxxxxxxxx xxxxxx xx xxxxxxxx xxxxxx.",
    "xxxxxxxxxxx xxxxxx xx xxx xxxxxxxx xx xxxxxx xxxxxxxx.",
    "xxxxxxxx xxxxxxx xxxxxxxxxx xx xxxxxxxx xxxxxxxxxxxx.",
    "xxxxx'x xxxxxxxx xxxxxxx xx xxxx xxxx xxxx xxxxxxxx.",
    "xxx xxxxxxxxxxx xx xxxxxx xxxxxxxxx xxxxx xxxxxx xxxxxxxxx xxxxx.",
    "xxxxxxxx xxxxxxxx xx xxxxx xx xxxxxx xxxxx'x xxxxxxxxx.",
    "xxxxxx xxxxxxxxx xx xxxxx xxxxx xxxxxxx xxxxxxxxx xx xxx xxxxxxx xxxxx.",
    "xxxxxx xx xxxxxxxxxx xxx xxxxxxxxxxxx xxxxxx xxxxxxxxx.",
    "xxxxxxxx xxxxxxxx xxxxxxxxxxx xxxxxxxxxxx xxx xxxxxxxxxxx."
  ],
  "xxxxxxxxx_xxx_xxxxxxxxx_xxxxxxxxx": [
    "xxxx xxxxxxxxxxx xxxx xxxx, xxxxxxxxxx xxxxx xxx xxx xxxxxxxx.",
    "xxx xxxxx'x xxxxx xxxxxxxxxx xxxxx'x xxxx xxx xxxxxxxxxx.",
    "xxxxxxx xxx xxxxx, xxx xxxxx xxxxxxxxxx xxxxxx xxxxxx xxxxxxxx.",
    "xxxxx xxxxxxx xxxxxx xxxxxxxx xxxxxxxxxxx xxx'x xxxxxxxxxxx xxxxxxx.",
    "xxxxx'x xxxxxxxxx xxxxxx xxxxxxxxxx xx xxxxx xxxxxx xxxxx.",
    "xxxxxx xx xxxxxxx xxxxxx xxxxxx xx xxxxxxxx xxxxxxxxxxx.",
    "xxxxx’x xxxxx xxxxxxxx xxxxxxxxx xx xxxxxxxxxx xxxxxxxxxx.",
    "xxxxxxxxx xx xxxx’x xxxxxxxx xxxxxx xxxxxxxxxxx xxxxxxxxxxxx xxxxx.",
    "xxxxxx xxxxxx xx xxxx xxxxxxxxx xxxxxxxx xxxxxxxx xxxxxxx xxxxx.",
    "xxx xxxxxxxxx xx xxxxx'x xxxxxxxxxxxx xxxx xxx xxxxx'x xxxxxxxx.",
    "xxxxxxxxxxxxx xxxxxx xx xxxxxxxxx xxxxxx xxxxxx xxxxx.",
    "x xxxxxxx xxxxxxxx xx xxxxxxxx xxx xxxxxxxx xxxxxx."
  ],
  "xxxxxxxxx_xxx_xxxxxxxxx_xxx_xxxxxxxxxx": [
    "x xxxxx xxxxxxx xxxxxx x xxxxxxxxxxx xxxxxxx xxxx xxxxxx xxx xxxxx.",
    "xxx xxxx xxxxxxxxxxxx xxxxxxxx xx xxx xxxxxxx xx xxxxx'x xxxxxxxxx.",
    "xxxxxxx xxx xxx xxxxxxx xxxxxxxxxxxx xxx xxxxxxxxx xxxxxxxxxx.",
    "x xxxxxxxxx'x xxxxxxxxxxxxxx xxxxxxxxx xx xxxxxxxx xxxxxx xx xxx xxxxx-xxx xxxx.",
    "x xxxxxxxx xxxxxx xxxxxxxx xxxxxxx xxxxxxxxxx xxxxxxx.",
    "xxxxxx xxxxxxx xxxxxxxxx xxxx xx xxxxxxxxxx xx xxxx’x xxxxxx.",
    "xxx xxxxxxxxxx xx xxxxxx xxxxxxxxx xxxxxxxxxxx xx xxxxx.",
    "xxxxxx xxxxxxxxx xxxx xxxxxx xxxx xxxxxxxx, xxxxx xxxx xxx xxxxx'x xxxxxxx.",
    "xxxxxxx xxxxx xxxx xxx xxxx'x xxxxxxxx xxxxxx xxx xxxx'x xxxxxxx.",
    "xxx xxxxxxx xx xxxxxxxxxx xxxxx xxxxxx xxxxxxxxx xxxxxxxx.",
    "xxxxx xxx xxxxxxxxxxxxx xxxxxxxxxxxx xxxxxxx xxxx xxxxx xxxxxxx.",
    "xxx xxxx'x xxxxxxxxxxxxx xxxxxxx xxxxxxxxxx xxxxxxx xxxxxxxx xxxxx."
  ],
  "xxxxxxxxxx_xxx_xxxxxxxx_xxxxxxxxx": [
    "xxxxxxxxx xxxxxxxxxxx xx xxxxxxxxx xxxxxxxxx xxxxxxx xxxxx xxx xxxx.",
    "xxxxxxxx xxxxxxx xxxxxx xx xxx xxxxxx xx xxxx xxx xxxx.",
    "xxxxxxxxx xxxxxxxx xxxxxxx xxxxxxxx xxxx xx xxxxxx xx xxxxx.",
    "xxxxxxxxxx xxxx xxxxxxx xx xxxxxxxx xxxxxxxxxx xxx xxxxxxxxxxx.",
    "xxxxxxxx xxxxxxx xxxxxxx xxx xxxxxxx xxxxx xx xxxxxxx xxxxxx.",
    "xxxxxxxxxxx xxxxxxxxx xxxxxxxxxxxx xxxxx xxx xxxxxxxxx xx xxxxxxxxx.",
    "xxxxxxx xxxxxxxxx xxxx xxxxxxx xxxxxxx xxxxxxxx xx xxxxxx xxxxxxxx.",
    "xxxxxxxxxxx xxxxxxxxx xx xxx xxxxx xxxx xxxxxxxxxx' xxxxx xxxxxxxx.",
    "xxxxxxxxxxx xxxxxxxx xxxxxxx xxxxxxx xxxxx xxxxxxxxxxxx.",
    "xxxxx xxxxxxx xxxxx xx xxxxxx xxx xxxxxxx xx xxxxx xxx xxxxxxx.",
    "xxxxxxxx xxxxxxx xx xxxxxxxxx xxx xxxxxxx xx xxxx xxx xxxxx xxx xxxxxx xxxxxxxxx.",
    "xxxxxxxxx xxxxxxxxxx’ xxxxxxxx xxxxxxxxxx xx xxxxxxxxxxx xx xxxxx xxxxxxxxxxx."
  ],
  "xxxxxxxx_xxxxx_xx_xxxxxx_xx_xxx_xxxxxxxx": [
    "xxxxxx x xxxxxxxxx xxxxxx xxxx xxxxxx xxx xxxxxxx xxxxxxx xxxxxxxxxxxx.",
    "xxxxxx xxxxxxxx, xxxxxxxx xxxx xxx xx xxxxxxxx xxx xxxxxxxx xxxxx.",
    "xxxxxx xxxxxxxxx xxxxxxxx xxx xxxx xxx xxxx xxxxxxxxxxx xx xxxxx.",
    "xxxxxxx xxx xxxxxxxx xxxx, xxxxxxxx xxxxxx xxxxx xxxxxxxxxx xxxxxxx xxxxxxxxx.",
    "xxxxxxxx xxx xxxxxxx xxxxxx xxxx xxxxxx xx xxxxxxxxx xxx xxxxx.",
    "xxxxxxx xxxx xxx xxxxxx xxx xxxxxxxxx xxx xxxxxxxxxxx xxxxx.",
    "xxxxxxx xxxxxxx xxxxxxxxxx xxx xxxxxxxx xxxxxxxxxx xxx xxxxxxxxx.",
    "xxxxxxx xxxxxxxx xxxxx xxxxxxx xx xxxxxxxxxx xxx xxxx'x xxxxxxx xxxx.",
    "xxxxxxxxx xxxxxx xxxxxxxx, xxxxxxxxxx xxxxxxxxxxxx xxxxx xxxxxxxx xxxxxx.",
    "xxxxxxxxx xxxxxxx xxx xxxxxxxx xxxxxxx xxx xxxxxxxxxxxxx xxxxx.",
    "xxxxxxxx x xxxxxxx xxx xxxxxxxxxxx xxx xxxxxxxxxx xxx xxxxx.",
    "xxxxxx xxxxxxxx xxxxxx xxx xxxxx xx xxxxxxx xxx xxxxxx xxxxxxxx."
  ],
  "xxxxxxxxx_xxx_xxxxxxxxxx_xxxxxxxxx": [
    "xxxxxxxxxx xxxxxxx xxxxxxxx xxxxxxx xxxxxxx xxxxxx xxxx.",
    "xxxxxxxxxx xxxxxxxxxxxxx xxx x xxxxxxx, xxxxxxxx xxxxx.",
    "xxxxxxx xxxxxxxxxxx xxxxxxx xxxxxxxxxxx xxxxx xxx xxxx xxxxxxxxx.",
    "xxxxxxx xxxxxx-xxxx xxxxxxxxxx xxxxxx xxx xxxxxxxxxx xxxxxxxx xxxxx.",
    "xxxxxxx xxxxxx xxxx-xxxxxxxxx xxxxxxx xxxxxxx xxxxxxxxxxx xx xxxxx.",
    "xxxxx xxxxxxxxx xxxxxxxxxx xxxxxxxx xxxxxx xx xxxxxxx xx xxxx.",
    "xxxxx xxxxxxxxx xx xxxxxx xxxxxxxxx xxxxxxx xx xxxxxx xxxxxxxxxxx.",
    "xxxxxxxx xxxxxxxxxxx xxxxxxxx xxxxxxx xxxxxxxxx xxxxxxxxxx.",
    "xxxxxxxxx xxxxxxx xxxxxx xxxxxxx xxxxxxx xxx xxxxxxxxx xxxx.",
    "xxxxxxxx xxxxxxxxxxxxx xxxx xxxxxx, xxxxxx xxxxxxxxx xxxxxx.",
    "xxxxxxx xxxxxxxxxxxx xxxxxxx xxxx xxxxxxxxxx xxxxxxx xxxxxxxx.",
    "xxxxxxxxx xxxxxxxxxx xx xxxxxxxx xxxxxx xxxx xxx xxxxxxxxxx xxxxxx."
  ],
  "xxxxxxx_xxx_xxxxxxx": [
    "xxxxxxxxxxxxx xx xxxxxxxxxxx xxx xxxxxxxxxx xx xxxx xxx xxxxx.",
    "xxxxxxxxxxxx xxxxxxxxxx xx xxxxxxxxx xxx xxxxx xxxxxx xxxxxxxx xxxxx.",
    "xxxxxxxxxxxx xxxxxxx xxxx xxx xxxxxxxxxxx xx xxx xxxxxxx xxxx.",
    "xxxxx’x xxxxxxxx xxx xxxxxxxxxxx xxxxxxxxxxx xxxx’x xxxxxxxx xxxxxxx.",
    "xxxxx xxxxxxx xxxx xxxxxxxxxx xx xxxxxxxxxx xxxx'x xxxxxxxxxxxxxxxx.",
    "xxx xxxx xx xxxxxxxxxxx xxxxxxxxxxx xxxxxxxxx xxxxxxxx.",
    "xxxxxxxx xxxxxxxxx x xxxxxxx xxx xxxxxxxx xxxxxxxxxxxx.",
    "xxxxxxxx xx xxxxxx xxxxxxx xxxxxx xx xxxxxxxxxxx xx xxxx.",
    "xxxxxxxx xxxxxxxx xxxx xxxxxxxx xxxxxxxxxxxx xxx xxxxx.",
    "xxxx xxxxxxxxxxx xxxxxx xxxxxxxxxx xxxxxxxx xx xxxx xxx xxxxxxxx.",
    "xxxxxxxx xxxxxxx xxxxxxxxxxxx xxxx xxxxxxx xxxxxx xxx xxxxxxx.",
    "xxx xxxxxxxx xx xxxxxxxxx xxxxxxxxxxxx xx xxxxxx xx xxx xxxx."
  ],
  "xxxxxxxxx_xxxxxxx": [
    "xxxxxx xx xxxxx xxxxxxxx xxxxxxxx xxxxxx xxxxxx xxxxxxxx.",
    "xxxxxxxxxxx xxxxxxxxxx xx xxxxxxxxxx xxxxxx xx xxxxxxx xxx xxxxx'x xxxxxxxxxx.",
    "xxxxxxxx xxxx xx xxx xxxxxxxx xxxxxxxxxx xxxxxxx xxxxxxx xxxxxxx xxxxxxxxxxx.",
    "xxxxxxx xxxxxxx xxxxxx xxxxxx xxxxxxx xxxxxxx xxxxx xxx xxxxxxxxx.",
    "xxxxxxxxxx xxx xx xxxxxx xxxxxxxx xx xxxxxxx xxxxxxx xxxxxxxxx.",
    "xxxxxx xx xxxxx xxxxxx xx xxxxxxxxxxx xxxxxxxxx xxxxxxx.",
    "xxxxxxxxxxx xxxxxxxxx xx xxxxx xxx xxxxxxxxxxxxxx xx xxxxx.",
    "xxxxxxxxx xxxxxxxxx xxxxxxxx xxxx xxxxx xxxxxxxxx xxxx xxxxxxxxx xxxxxxxxx.",
    "xxxxxxxxxx xxxxxxxxx xxx xxxxx xxxxx xx xxxxx xxxxxxxx xx xxxxx xxxxxxxxxxx.",
    "xxxx xxxxxxxxx xx xxxxxxxx xx xxxxxx xxxx xxxxxxxxxx xxxxxxxx.",
    "xxxxxxxxxx xxx xx xxx xxxxxxx xx xxxxxxx xxxxxxxx xxxxx xxxxxxxxx.",
    "xxxx xxxxxxxxx xx xxxxxxxxxxxx xxxx xxxxxxxxxxxx xxxxxxxxxx xxxxxxxxxxxx."
  ]
}"#;
        let result = repair_json_missing_commas_in_list(input).unwrap();

        // Parse both the result and the expected output as JSON values
        let result_json: serde_json::Value = serde_json::from_str(&result).expect("Failed to parse result JSON");
        let expected_json: serde_json::Value = serde_json::from_str(expected).expect("Failed to parse expected JSON");

        // Compare the parsed JSON values
        assert_eq!(result_json, expected_json);
    }

}

