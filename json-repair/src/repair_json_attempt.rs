crate::ix!();

pub fn attempt_repair_json_string(input: &str) -> Result<Value, JsonRepairError> {
    let mut repaired = String::new();
    let mut stack: VecDeque<char> = VecDeque::new();
    let mut in_string = false;
    let mut escape_next = false;
    let mut last_char = '\0';
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if in_string {
            if escape_next {
                escape_next = false;
                repaired.push(c);
                continue;
            }
            match c {
                '\\' => {
                    escape_next = true;
                    repaired.push(c);
                }
                '"' => {
                    in_string = false;
                    repaired.push(c);
                }
                '\'' => {
                    // Replace single quotes inside strings with double quotes
                    repaired.push('"');
                }
                '\n' | '\r' => {
                    // Unescaped newline in string; close the string
                    in_string = false;
                    repaired.push('"');
                    // Do not include the newline character
                }
                _ => {
                    repaired.push(c);
                }
            }
        } else {
            match c {
                '{' | '[' => {
                    stack.push_back(c);
                    repaired.push(c);
                }
                '}' => {
                    if stack.back() == Some(&'{') {
                        stack.pop_back();
                        repaired.push(c);
                    } else {
                        // Mismatched closing brace
                        return Err(JsonRepairError::Unrepairable(input.to_string()));
                    }
                }
                ']' => {
                    if stack.back() == Some(&'[') {
                        stack.pop_back();
                        repaired.push(c);
                    } else {
                        // Mismatched closing bracket
                        return Err(JsonRepairError::Unrepairable(input.to_string()));
                    }
                }
                '"' => {
                    in_string = true;
                    repaired.push(c);
                }
                '\'' => {
                    // Start of a string with single quote; replace with double quote
                    in_string = true;
                    repaired.push('"');
                }
                ':' | ',' => {
                    repaired.push(c);
                }
                c if c.is_whitespace() => {
                    repaired.push(c);
                }
                c => {
                    // Check if a comma is missing between values
                    if needs_comma(last_char, c) {
                        repaired.push(',');
                    }
                    repaired.push(c);
                }
            }
        }
        if !c.is_whitespace() {
            last_char = c;
        }
    }

    // Close unclosed strings
    if in_string {
        repaired.push('"');
        in_string = false;
    }

    assert!(in_string == false);

    // Remove trailing commas before closing braces/brackets
    let mut repaired = remove_trailing_commas(&repaired);

    // Close unclosed structures
    while let Some(open) = stack.pop_back() {
        match open {
            '{' => repaired.push('}'),
            '[' => repaired.push(']'),
            _ => {}
        }
    }

    // Try to parse the repaired string
    let mut value = json5::from_str(&repaired).map_err(|_| JsonRepairError::Unrepairable(repaired))?;

    // Remove control characters from string values
    remove_control_characters_in_value(&mut value);

    Ok(value)
}

fn needs_comma(last_char: char, current_char: char) -> bool {
    match (last_char, current_char) {
        // Insert comma between a closing quote/bracket/brace and an opening quote/bracket/brace if necessary
        ('"', '"') | ('"', '{') | ('"', '[') | ('}', '"') | (']', '"') | ('0'..='9', '"') => true,
        // Insert comma between numbers and strings
        ('0'..='9', '{') | ('0'..='9', '[') | ('"', '0'..='9') => true,
        // Insert comma between closing brackets/braces and numbers
        ('}', '0'..='9') | (']', '0'..='9') => true,
        // Default case
        _ => false,
    }
}

fn remove_trailing_commas(input: &str) -> String {
    let mut output = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == ',' {
            // Peek ahead to see if the next significant character is a closing brace/bracket
            let mut iter = chars.clone();
            while let Some(&nc) = iter.peek() {
                if nc.is_whitespace() {
                    iter.next();
                    continue;
                }
                if nc == '}' || nc == ']' {
                    // Skip this comma
                    break;
                } else {
                    output.push(c);
                    break;
                }
            }
        } else {
            output.push(c);
        }
    }
    output
}
