crate::ix!();

pub fn repair_json_handle_eof_between_lists(input: &str) -> String {
    let mut repaired = input.to_owned();

    // Initialize variables
    let mut in_string = false;
    let mut escape = false;

    // Process the string to handle unclosed strings and collect braces/brackets
    let mut open_braces = 0;
    let mut open_brackets = 0;
    let mut chars_iter = repaired.chars().peekable();
    let mut last_quote_pos = None;
    let mut last_colon_pos = None;

    while let Some(c) = chars_iter.next() {
        match c {
            '\\' => {
                escape = !escape;
            }
            '"' if !escape => {
                in_string = !in_string;
                if in_string {
                    last_quote_pos = Some(repaired.len() - chars_iter.clone().count() - 1);
                }
            }
            '{' if !in_string => {
                open_braces += 1;
            }
            '}' if !in_string => {
                open_braces -= 1;
            }
            '[' if !in_string => {
                open_brackets += 1;
            }
            ']' if !in_string => {
                open_brackets -= 1;
            }
            ':' if !in_string => {
                last_colon_pos = Some(repaired.len() - chars_iter.clone().count() - 1);
            }
            _ => {
                escape = false;
            }
        }
    }

    // Close unclosed string
    if in_string {
        repaired.push('"');
    }

    // Add missing colon and null value if necessary
    if last_quote_pos.is_some() && (last_colon_pos.is_none() || last_colon_pos.unwrap() < last_quote_pos.unwrap()) {
        repaired.push_str(": null");
    }

    // Close any open braces
    for _ in 0..open_braces {
        repaired.push('}');
    }

    // Close any open brackets
    for _ in 0..open_brackets {
        repaired.push(']');
    }

    repaired
}

