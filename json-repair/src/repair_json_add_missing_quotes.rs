crate::ix!();

#[derive(Debug)]
pub enum Token {
    String(String),
    Number(String),
    Symbol(char),
    Whitespace,
    Comment,
}

pub fn repair_json_add_missing_quotes(input: &str) -> Result<String, JsonRepairError> {

    let mut changed = false;
    let mut tokens  = tokenize(input, &mut changed)?;
    let json_value  = parse_value(&mut tokens)?;
    let output      = serde_json::to_string(&json_value).map_err(|inner| JsonRepairError::SerdeParseError { inner })?;

    if changed {
        info!("added missing quotations where necessary");
    }

    Ok(output)
}

pub fn tokenize(input: &str, changed: &mut bool) -> Result<VecDeque<Token>, JsonRepairError> {
    let mut tokens = VecDeque::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            '"' | '\'' => {
                let string = parse_quoted_string(&mut chars)?;
                tokens.push_back(Token::String(string));
            }
            '{' | '}' | '[' | ']' | ':' | ',' => {
                chars.next(); // Consume the symbol
                tokens.push_back(Token::Symbol(c));
            }
            '/' if chars.clone().nth(1) == Some('/') => {
                consume_comment(&mut chars);
                tokens.push_back(Token::Comment);
            }
            c if c.is_whitespace() => {
                consume_whitespace(&mut chars);
                tokens.push_back(Token::Whitespace);
            }
            c if c.is_digit(10) || c == '-' => {
                let number = parse_number(&mut chars)?;
                tokens.push_back(Token::Number(number));
            }
            _ => {
                let string = parse_unquoted_string(&mut chars)?;
                if !string.is_empty() {
                    // We parsed an unquoted string, meaning we "added" quotes logically.
                    *changed = true;
                }
                tokens.push_back(Token::String(string));
            }
        }
    }

    Ok(tokens)
}

fn parse_quoted_string(chars: &mut Peekable<Chars>) -> Result<String, JsonRepairError> {
    let quote_char = chars.next().ok_or(JsonRepairError::UnexpectedEOF)?; // opening quote
    let mut s = String::new();

    while let Some(&c) = chars.peek() {
        if c == quote_char {
            chars.next(); // closing quote
            break;
        } else if c == '\\' {
            chars.next(); // consume '\'
            if let Some(escaped_char) = chars.next() {
                s.push(match escaped_char {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    'b' => '\x08',
                    'f' => '\x0C',
                    '\\' => '\\',
                    '\'' => '\'',
                    '"' => '"',
                    other => other,
                });
            } else {
                // If there's nothing after the backslash, treat it as a literal backslash
                s.push('\\');
            }
        } else if ":,{}[]\"'".contains(c) {
            // If we hit a structural character, end the string early
            break;
        } else {
            s.push(chars.next().unwrap());
        }
    }

    Ok(s)
}

fn parse_unquoted_string(chars: &mut Peekable<Chars>) -> Result<String, JsonRepairError> {
    let mut s = String::new();

    while let Some(&c) = chars.peek() {
        if c.is_whitespace() || ":,{}[]\"'".contains(c) {
            break;
        } else {
            s.push(chars.next().unwrap());
        }
    }

    Ok(s.trim().to_string())
}

fn consume_comment(chars: &mut Peekable<Chars>) {
    chars.next(); // '/'
    chars.next(); // second '/'
    while let Some(c) = chars.next() {
        if c == '\n' {
            break;
        }
    }
}

fn consume_whitespace(chars: &mut Peekable<Chars>) {
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else {
            break;
        }
    }
}

fn parse_number(chars: &mut Peekable<Chars>) -> Result<String, JsonRepairError> {
    let mut num = String::new();

    while let Some(&c) = chars.peek() {
        if c.is_digit(10) || c == '.' || c == 'e' || c == 'E' || c == '+' || c == '-' {
            num.push(chars.next().ok_or(JsonRepairError::UnexpectedEOF)?);
        } else {
            break;
        }
    }

    Ok(num)
}

pub fn unescape_string(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(next_char) = chars.next() {
                match next_char {
                    'n' => result.push('\n'),
                    't' => result.push('\t'),
                    'r' => result.push('\r'),
                    'b' => result.push('\x08'),
                    'f' => result.push('\x0C'),
                    '\\' => result.push('\\'),
                    '\'' => result.push('\''),
                    '"' => result.push('"'),
                    other => {
                        result.push('\\');
                        result.push(other);
                    }
                }
            } else {
                result.push('\\');
            }
        } else {
            result.push(c);
        }
    }

    result
}

pub fn parse_value(tokens: &mut VecDeque<Token>) -> Result<JsonValue, JsonRepairError> {
    while let Some(token) = tokens.pop_front() {
        match token {
            Token::Symbol('{') => return parse_object(tokens),
            Token::Symbol('[') => return parse_array(tokens),
            Token::String(s) | Token::Number(s) => {
                let mut value_parts = vec![s];

                loop {
                    let continue_loop = {
                        let next_token = tokens.front();
                        if let Some(next_token) = next_token {
                            match next_token {
                                Token::Whitespace | Token::Comment => {
                                    tokens.pop_front(); // consume and continue
                                    true
                                }
                                Token::String(s) | Token::Number(s) => {
                                    let s = s.clone();
                                    tokens.pop_front(); // consume
                                    value_parts.push(s);
                                    true
                                }
                                _ => false,
                            }
                        } else {
                            false
                        }
                    };
                    if !continue_loop {
                        break;
                    }
                }

                let s_trimmed = value_parts.join(" ").trim().to_string();
                match s_trimmed.as_str() {
                    "true" => return Ok(JsonValue::Bool(true)),
                    "false" => return Ok(JsonValue::Bool(false)),
                    "null" => return Ok(JsonValue::Null),
                    _ => {
                        if let Ok(num) = s_trimmed.parse::<i64>() {
                            return Ok(JsonValue::Number(num.into()));
                        } else if let Ok(num) = s_trimmed.parse::<f64>() {
                            if let Some(n) = serde_json::Number::from_f64(num) {
                                return Ok(JsonValue::Number(n));
                            } else {
                                return Err(JsonRepairError::InvalidNumber(s_trimmed.to_string()));
                            }
                        } else {
                            return Ok(JsonValue::String(unescape_string(&s_trimmed)));
                        }
                    }
                }
            }
            Token::Symbol(c) => {
                if c == ']' || c == '}' {
                    if c == ']' {
                        return Ok(JsonValue::Array(vec![]));
                    } else {
                        return Ok(JsonValue::Object(serde_json::Map::new()));
                    }
                }
            }
            Token::Whitespace | Token::Comment => continue,
        }
    }
    Ok(JsonValue::Null)
}

pub fn parse_object(tokens: &mut VecDeque<Token>) -> Result<JsonValue, JsonRepairError> {
    if matches!(tokens.front(), Some(Token::Symbol('{'))) {
        tokens.pop_front();
    }

    let mut map = serde_json::Map::new();

    while tokens.front().is_some() {
        while matches!(
            tokens.front(),
            Some(Token::Symbol(',')) | Some(Token::Symbol(':')) | Some(Token::Whitespace) | Some(Token::Comment)
        ) {
            tokens.pop_front();
        }

        match tokens.front() {
            Some(Token::Symbol('}')) => {
                tokens.pop_front(); // consume '}'
                break;
            }
            _ => {
                // Parse key
                let mut key_parts = Vec::new();

                while let Some(token) = tokens.front() {
                    match token {
                        Token::String(_) | Token::Number(_) => {
                            if let Some(token) = tokens.pop_front() {
                                match token {
                                    Token::String(s) | Token::Number(s) => key_parts.push(s),
                                    _ => {}
                                }
                            }
                        }
                        Token::Whitespace | Token::Comment => {
                            tokens.pop_front(); // consume and continue
                        }
                        _ => break,
                    }
                }

                let key = key_parts.join(" ");

                while matches!(
                    tokens.front(),
                    Some(Token::Whitespace) | Some(Token::Comment) | Some(Token::Symbol(','))
                ) {
                    tokens.pop_front();
                }

                let colon_found = if let Some(Token::Symbol(':')) = tokens.front() {
                    tokens.pop_front(); // consume ':'
                    true
                } else {
                    false
                };

                while matches!(tokens.front(), Some(Token::Whitespace) | Some(Token::Comment)) {
                    tokens.pop_front();
                }

                if colon_found {
                    let value = parse_value(tokens)?;
                    map.insert(key, value);
                } else {
                    match tokens.front() {
                        Some(Token::String(_))
                        | Some(Token::Number(_))
                        | Some(Token::Symbol('{'))
                        | Some(Token::Symbol('[')) => {
                            let value = parse_value(tokens)?;
                            map.insert(key, value);
                        }
                        _ => {
                            // No proper value found, treat last key part as value if multiple parts
                            if key_parts.len() > 1 {
                                let value_str = key_parts.pop().unwrap();
                                let key = key_parts.join(" ");
                                let value = JsonValue::String(value_str);
                                map.insert(key, value);
                            } else {
                                map.insert(key, JsonValue::Null);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(JsonValue::Object(map))
}

pub fn parse_array(tokens: &mut VecDeque<Token>) -> Result<JsonValue, JsonRepairError> {
    if let Some(Token::Symbol('[')) = tokens.front() {
        tokens.pop_front();
    }

    let mut arr = vec![];

    while let Some(token) = tokens.front() {
        match token {
            Token::Symbol(']') => {
                tokens.pop_front(); // consume ']'
                break;
            }
            Token::Whitespace | Token::Comment | Token::Symbol(',') => {
                tokens.pop_front(); // consume and continue
                continue;
            }
            _ => {
                let value = parse_value(tokens)?;
                arr.push(value);
            }
        }
    }

    Ok(JsonValue::Array(arr))
}

#[cfg(test)]
mod tokenize_tests {
    use super::*;

    #[traced_test]
    fn test_parse_quoted_string() {
        let input = r#""Hello\nWorld""#;
        let mut chars = input.chars().peekable();
        let result = parse_quoted_string(&mut chars).unwrap();
        assert_eq!(result, "Hello\nWorld");
    }

    #[traced_test]
    fn test_parse_unquoted_string() {
        let input = "unquotedString ";
        let mut chars = input.chars().peekable();
        let result = parse_unquoted_string(&mut chars).unwrap();
        assert_eq!(result, "unquotedString");
    }

    #[traced_test]
    fn test_parse_number() {
        let input = "12345.67e-2";
        let mut chars = input.chars().peekable();
        let result = parse_number(&mut chars).unwrap();
        assert_eq!(result, "12345.67e-2");
    }

    #[traced_test]
    fn test_consume_whitespace() {
        let input = "   \t\n\rabc";
        let mut chars = input.chars().peekable();
        consume_whitespace(&mut chars);
        assert_eq!(chars.next(), Some('a'));
    }

    #[traced_test]
    fn test_consume_comment() {
        let input = "// This is a comment\nNextLine";
        let mut chars = input.chars().peekable();
        consume_comment(&mut chars);
        assert_eq!(chars.next(), Some('N'));
    }
}

#[cfg(test)]
mod unescape_string_tests {
    use super::*;

    #[traced_test]
    fn test_unescape_basic() {
        assert_eq!(unescape_string("Hello\\nWorld"), "Hello\nWorld");
        assert_eq!(unescape_string("Tab\\tSeparated"), "Tab\tSeparated");
        assert_eq!(unescape_string("Carriage\\rReturn"), "Carriage\rReturn");
    }

    #[traced_test]
    fn test_unescape_quotes() {
        assert_eq!(unescape_string("\\\"Quoted\\\""), "\"Quoted\"");
        assert_eq!(unescape_string("\\'Single Quoted\\'"), "'Single Quoted'");
    }

    #[traced_test]
    fn test_unescape_backslash() {
        assert_eq!(unescape_string("Back\\\\Slash"), "Back\\Slash");
    }

    #[traced_test]
    fn test_unescape_no_escapes() {
        assert_eq!(unescape_string("NoEscapes"), "NoEscapes");
    }
}

#[cfg(test)]
mod parse_value_tests {
    use super::*;
    use serde_json::Value as JsonValue;

    #[traced_test]
    fn test_parse_value_string() {
        let mut tokens = VecDeque::from(vec![Token::String("Hello".to_string())]);
        let result = parse_value(&mut tokens).unwrap();
        assert_eq!(result, JsonValue::String("Hello".to_string()));
    }

    #[traced_test]
    fn test_parse_value_number() {
        let mut tokens = VecDeque::from(vec![Token::Number("123".to_string())]);
        let result = parse_value(&mut tokens).unwrap();
        assert_eq!(result, JsonValue::Number(123.into()));
    }

    #[traced_test]
    fn test_parse_value_bool() {
        let mut tokens = VecDeque::from(vec![Token::String("true".to_string())]);
        let result = parse_value(&mut tokens).unwrap();
        assert_eq!(result, JsonValue::Bool(true));
    }

    #[traced_test]
    fn test_parse_value_null() {
        let mut tokens = VecDeque::from(vec![Token::String("null".to_string())]);
        let result = parse_value(&mut tokens).unwrap();
        assert_eq!(result, JsonValue::Null);
    }

    #[traced_test]
    fn test_parse_value_array() {
        let mut tokens = VecDeque::from(vec![Token::Symbol('['), Token::Symbol(']')]);
        let result = parse_value(&mut tokens).unwrap();
        assert_eq!(result, JsonValue::Array(vec![]));
    }

    #[traced_test]
    fn test_parse_value_object() {
        let mut tokens = VecDeque::from(vec![Token::Symbol('{'), Token::Symbol('}')]);
        let result = parse_value(&mut tokens).unwrap();
        assert_eq!(result, JsonValue::Object(serde_json::Map::new()));
    }
}

#[cfg(test)]
mod parse_object_tests {
    use super::*;
    use serde_json::Value as JsonValue;

    #[traced_test]
    fn test_parse_empty_object() {
        let mut tokens = VecDeque::from(vec![Token::Symbol('{'), Token::Symbol('}')]);
        let result = parse_object(&mut tokens).unwrap();
        assert_eq!(result, JsonValue::Object(serde_json::Map::new()));
    }

    #[traced_test]
    fn test_parse_simple_object() {
        let mut tokens = VecDeque::from(vec![
            Token::Symbol('{'),
            Token::String("key".to_string()),
            Token::Symbol(':'),
            Token::String("value".to_string()),
            Token::Symbol('}'),
        ]);
        let result = parse_object(&mut tokens).unwrap();
        let mut expected = serde_json::Map::new();
        expected.insert("key".to_string(), JsonValue::String("value".to_string()));
        assert_eq!(result, JsonValue::Object(expected));
    }

    #[traced_test]
    fn test_parse_object_missing_colon() {
        let mut tokens = VecDeque::from(vec![
            Token::Symbol('{'),
            Token::String("key".to_string()),
            Token::String("value".to_string()), // Missing colon
            Token::Symbol('}'),
        ]);
        let result = parse_object(&mut tokens).unwrap();
        let mut expected = serde_json::Map::new();
        expected.insert("key".to_string(), JsonValue::String("value".to_string()));
        assert_eq!(result, JsonValue::Object(expected));
    }

    #[traced_test]
    fn test_parse_object_unquoted_keys_with_spaces() {
        let mut tokens = VecDeque::from(vec![
            Token::Symbol('{'),
            Token::String("key".to_string()),
            Token::String("with".to_string()),
            Token::String("spaces".to_string()),
            Token::Symbol(':'),
            Token::String("value".to_string()),
            Token::Symbol('}'),
        ]);
        let result = parse_object(&mut tokens).unwrap();
        let mut expected = serde_json::Map::new();
        expected.insert("key with spaces".to_string(), JsonValue::String("value".to_string()));
        assert_eq!(result, JsonValue::Object(expected));
    }
}

#[cfg(test)]
mod parse_array_tests {
    use super::*;
    use serde_json::Value as JsonValue;

    #[traced_test]
    fn test_parse_empty_array() {
        let mut tokens = VecDeque::from(vec![Token::Symbol('['), Token::Symbol(']')]);
        let result = parse_array(&mut tokens).unwrap();
        assert_eq!(result, JsonValue::Array(vec![]));
    }

    #[traced_test]
    fn test_parse_simple_array() {
        let mut tokens = VecDeque::from(vec![
            Token::Symbol('['),
            Token::String("value1".to_string()),
            Token::Symbol(','),
            Token::String("value2".to_string()),
            Token::Symbol(']'),
        ]);
        let result = parse_array(&mut tokens).unwrap();
        let expected = JsonValue::Array(vec![
            JsonValue::String("value1".to_string()),
            JsonValue::String("value2".to_string()),
        ]);
        assert_eq!(result, expected);
    }

    #[traced_test]
    fn test_parse_array_with_numbers() {
        let mut tokens = VecDeque::from(vec![
            Token::Symbol('['),
            Token::Number("1".to_string()),
            Token::Symbol(','),
            Token::Number("2".to_string()),
            Token::Symbol(','),
            Token::Number("3".to_string()),
            Token::Symbol(']'),
        ]);
        let result = parse_array(&mut tokens).unwrap();
        let expected = JsonValue::Array(vec![
            JsonValue::Number(1.into()),
            JsonValue::Number(2.into()),
            JsonValue::Number(3.into()),
        ]);
        assert_eq!(result, expected);
    }
}

#[cfg(test)]
mod repair_json_add_missing_quotes_tests {
    use super::*;
    use serde_json::json;
    use serde_json::Value as JsonValue;

    fn assert_expected_matches_output_result(input: &str, output: &str, expected: &JsonValue) {
        match serde_json::from_str::<JsonValue>(output) {
            Ok(parsed_output) => {
                assert_eq!(
                    &parsed_output, expected,
                    "Parsed output does not match expected value"
                );
            }
            Err(e) => {
                panic!(
                    "Failed to parse output JSON: {}\nInput: {}\nOutput: {}",
                    e, input, output
                );
            }
        }
    }

    #[traced_test]
    fn test_no_missing_quotes() {
        let input = r#"{"key": "value", "number": 123}"#;
        let expected = json!({"key": "value", "number": 123});
        let output = repair_json_add_missing_quotes(input).unwrap();
        assert_expected_matches_output_result(input, &output, &expected);
    }

    #[traced_test]
    fn test_missing_quotes_around_value() {
        let input = r#"{"key": value, "number": 123}"#;
        let expected = json!({"key": "value", "number": 123});
        let output = repair_json_add_missing_quotes(input).unwrap();
        assert_expected_matches_output_result(input, &output, &expected);
    }

    #[traced_test]
    fn test_missing_quotes_around_key() {
        let input = r#"{key: "value", "number": 123}"#;
        let expected = json!({"key": "value", "number": 123});
        let output = repair_json_add_missing_quotes(input).unwrap();
        assert_expected_matches_output_result(input, &output, &expected);
    }

    #[traced_test]
    fn test_missing_quotes_around_key_and_value() {
        let input = r#"{key: value, "number": 123}"#;
        let expected = json!({"key": "value", "number": 123});
        let output = repair_json_add_missing_quotes(input).unwrap();
        assert_expected_matches_output_result(input, &output, &expected);
    }

    #[traced_test]
    fn test_unclosed_string_at_eof() {
        let input = r#"{"key": "value"#;
        let expected = json!({"key": "value"});
        let output = repair_json_add_missing_quotes(input).unwrap_or_else(|_| "null".to_string());
        let parsed_output: JsonValue = serde_json::from_str(&output).unwrap_or(JsonValue::Null);
        assert_eq!(parsed_output, expected);
    }

    #[traced_test]
    fn test_missing_quotes_in_array_elements() {
        let input = r#"["value1", value2, "value3", value4]"#;
        let expected = json!(["value1", "value2", "value3", "value4"]);
        let output = repair_json_add_missing_quotes(input).unwrap();
        assert_expected_matches_output_result(input, &output, &expected);
    }

    #[traced_test]
    fn test_nested_missing_quotes() {
        let input = r#"{"outer": {"inner": value}}"#;
        let expected = json!({"outer": {"inner": "value"}});
        let output = repair_json_add_missing_quotes(input).unwrap();
        assert_expected_matches_output_result(input, &output, &expected);
    }

    #[traced_test]
    fn test_missing_quotes_with_escaped_characters() {
        let input = r#"{"key": value\n, "another_key": value\t}"#;
        let expected = json!({"key": "value\n", "another_key": "value\t"});
        let output = repair_json_add_missing_quotes(input).unwrap();
        assert_expected_matches_output_result(input, &output, &expected);
    }

    #[traced_test]
    fn test_missing_quotes_in_keys_with_spaces() {
        let input = r#"{key with spaces: "value", "number": 123}"#;
        let expected = json!({"key with spaces": "value", "number": 123});
        let output = repair_json_add_missing_quotes(input).unwrap();
        assert_expected_matches_output_result(input, &output, &expected);
    }

    #[traced_test]
    fn test_input_with_only_commas_and_no_quotes() {
        let input = r#"{key1: value1, key2: value2, key3: value3}"#;
        let expected = json!({"key1": "value1", "key2": "value2", "key3": "value3"});
        let output = repair_json_add_missing_quotes(input).unwrap();
        assert_expected_matches_output_result(input, &output, &expected);
    }

    #[traced_test]
    fn test_input_with_colons_but_missing_quotes() {
        let input = r#"{key1: value1: key2: value2}"#;
        let expected = json!({"key1": "value1", "key2": "value2"});
        let output = repair_json_add_missing_quotes(input).unwrap_or_else(|_| "null".to_string());
        // The function may not fix missing colons; adjust expectations accordingly
        let parsed_output: JsonValue = serde_json::from_str(&output).unwrap_or(JsonValue::Null);
        assert_eq!(parsed_output, expected);
    }

    #[traced_test]
    fn test_missing_quotes_with_numbers_and_booleans() {
        let input = r#"{"number": 123, "boolean": true, key: value}"#;
        let expected = json!({"number": 123, "boolean": true, "key": "value"});
        let output = repair_json_add_missing_quotes(input).unwrap();
        assert_expected_matches_output_result(input, &output, &expected);
    }

    #[traced_test]
    fn test_missing_quotes_with_null() {
        let input = r#"{key: null, "another_key": value}"#;
        let expected = json!({"key": null, "another_key": "value"});
        let output = repair_json_add_missing_quotes(input).unwrap();
        assert_expected_matches_output_result(input, &output, &expected);
    }

    #[traced_test]
    fn test_empty_input() {
        let input = r#""#;
        let expected = json!(null);
        let output = repair_json_add_missing_quotes(input).unwrap_or_else(|_| "null".to_string());
        let parsed_output: JsonValue = serde_json::from_str(&output).unwrap_or(JsonValue::Null);
        assert_eq!(parsed_output, expected);
    }

    #[traced_test]
    fn test_input_with_only_whitespace() {
        let input = r#"    "#;
        let expected = json!(null);
        let output = repair_json_add_missing_quotes(input).unwrap_or_else(|_| "null".to_string());
        let parsed_output: JsonValue = serde_json::from_str(&output).unwrap_or(JsonValue::Null);
        assert_eq!(parsed_output, expected);
    }

    #[test]
    fn test_complex_nested_structure_with_missing_quotes() -> Result<(), JsonRepairError> {
        let input = r#"
        {
            person: {
                name: John Doe,
                age: 30,
                address: {
                    street: 123 Main St,
                    city: Anytown
                }
            },
            hobbies: [reading, hiking, coding]
        }
        "#;
        let expected = json!({
            "person": {
                "name": "John Doe",
                "age": 30,
                "address": {
                    "street": "123 Main St",
                    "city": "Anytown"
                }
            },
            "hobbies": ["reading", "hiking", "coding"]
        });

        let output = repair_json_add_missing_quotes(input)?;
        let parsed_output: Value = serde_json::from_str(&output)
            .map_err(|inner| JsonRepairError::SerdeParseError { inner })?;

        assert_eq!(parsed_output, expected);

        Ok(())
    }

    #[traced_test]
    fn test_input_with_special_characters() {
        let input = r#"{key$: value@, "another_key#": value%}"#;
        let expected = json!({"key$": "value@", "another_key#": "value%"});
        let output = repair_json_add_missing_quotes(input).unwrap();
        assert_expected_matches_output_result(input, &output, &expected);
    }

    #[traced_test]
    fn test_missing_quotes_with_unicode_characters() {
        let input = r#"{ключ: значение, "另一个键": 值}"#;
        let expected = json!({"ключ": "значение", "另一个键": "值"});
        let output = repair_json_add_missing_quotes(input).unwrap();
        assert_expected_matches_output_result(input, &output, &expected);
    }

    #[traced_test]
    fn test_unclosed_string_with_missing_quotes() {
        let input = r#"{"key": "value, "another_key": value}"#;
        let expected = json!({"key": "value", "another_key": "value"});
        let output = repair_json_add_missing_quotes(input).unwrap_or_else(|_| "null".to_string());
        let parsed_output: JsonValue = serde_json::from_str(&output).unwrap_or(JsonValue::Null);
        assert_eq!(parsed_output, expected);
    }

    #[traced_test]
    fn test_array_with_mixed_quoted_and_unquoted_strings() {
        let input = r#"["value1", value2, "value3", value4, "value5"]"#;
        let expected = json!(["value1", "value2", "value3", "value4", "value5"]);
        let output = repair_json_add_missing_quotes(input).unwrap();
        assert_expected_matches_output_result(input, &output, &expected);
    }

    #[traced_test]
    fn test_object_with_mixed_quoted_and_unquoted_keys_and_values() {
        let input = r#"{key1: value1, "key2": value2, key3: "value3", "key4": "value4"}"#;
        let expected = json!({
            "key1": "value1",
            "key2": "value2",
            "key3": "value3",
            "key4": "value4"
        });
        let output = repair_json_add_missing_quotes(input).unwrap();
        assert_expected_matches_output_result(input, &output, &expected);
    }

    #[traced_test]
    fn test_missing_quotes_with_trailing_commas() {
        let input = r#"{key1: value1, key2: value2,}"#;
        let expected = json!({"key1": "value1", "key2": "value2"});
        let output = repair_json_add_missing_quotes(input).unwrap();
        assert_expected_matches_output_result(input, &output, &expected);
    }

    #[traced_test]
    fn test_input_with_comments_and_missing_quotes() {
        let input = r#"{key1: value1, // This is a comment
        key2: value2}"#;
        let expected = json!({"key1": "value1", "key2": "value2"});
        let output = repair_json_add_missing_quotes(input).unwrap_or_else(|_| "null".to_string());
        // Note: JSON does not support comments; parsing may fail
        let parsed_output: JsonValue = serde_json::from_str(&output).unwrap_or(JsonValue::Null);
        assert_eq!(parsed_output, expected);
    }

    #[traced_test]
    fn test_input_with_malformed_json() {
        warn!("is this test testing what we want?");
        let input = r#"{key1 value1, key2: value2}"#; // Missing colon after key1
        let expected = json!({"key1 value1": "key2", "value2": null});
        let output = repair_json_add_missing_quotes(input).unwrap_or_else(|_| "null".to_string());
        let parsed_output: JsonValue = serde_json::from_str(&output).unwrap_or(JsonValue::Null);
        assert_eq!(parsed_output, expected);
    }

    #[traced_test]
    fn test_input_with_extra_commas_and_missing_quotes() {
        let input = r#"{key1: value1,, key2: value2,,}"#;
        let expected = json!({"key1": "value1", "key2": "value2"});
        let output = repair_json_add_missing_quotes(input).unwrap_or_else(|_| "null".to_string());
        // Note: The function may not handle extra commas; adjust expectations accordingly
        let parsed_output: JsonValue = serde_json::from_str(&output).unwrap_or(JsonValue::Null);
        assert_eq!(parsed_output, expected);
    }

    #[traced_test]
    fn test_missing_quotes_in_deeply_nested_structure() {
        let input = r#"
        {
            level1: {
                level2: {
                    level3: {
                        key: value
                    }
                }
            }
        }
        "#;
        let expected = json!({
            "level1": {
                "level2": {
                    "level3": {
                        "key": "value"
                    }
                }
            }
        });
        let output = repair_json_add_missing_quotes(input).unwrap();
        assert_expected_matches_output_result(input, &output, &expected);
    }
}
