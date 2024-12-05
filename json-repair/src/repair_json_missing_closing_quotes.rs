crate::ix!();

#[allow(unused_assignments)]
pub fn repair_json_missing_closing_quotes(input: &str) -> Result<String, JsonRepairError> {
    let mut output = String::new();
    let mut chars = input.chars().peekable();
    let mut inside_string = false;
    let mut string_start_index = 0;
    let mut index = 0;
    let mut escape = false;

    while let Some(c) = chars.next() {
        output.push(c);

        if escape {
            escape = false;
            index += 1;
            continue;
        }

        if c == '\\' {
            escape = true;
            index += 1;
            continue;
        }

        if c == '"' {
            if !inside_string {
                inside_string = true;
                string_start_index = index;
            } else {
                inside_string = false;
            }
        }

        if c == ',' || c == '\n' || c == '\r' || c == ']' || c == '}' {
            if inside_string {
                // Missing closing quote detected before a comma, newline, closing bracket, or brace.
                // Insert a closing double quote before the comma or newline.
                let insert_position = output.len() - c.len_utf8();
                output.insert(insert_position, '"');
                inside_string = false;
            }
        }

        index += 1;
    }

    Ok(output)
}

