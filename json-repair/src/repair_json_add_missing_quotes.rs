crate::ix!();

pub fn repair_json_add_missing_quotes(input: &str) -> String {
    let mut repaired = String::new();
    let mut in_string = false;
    let mut escape_next = false;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '"' && !escape_next {
            in_string = !in_string;
        }

        if !in_string && c == ',' {
            // Check if the previous characters indicate a missing closing quote
            let mut idx = repaired.len().saturating_sub(1);
            while idx > 0 && repaired.chars().nth(idx - 1).unwrap().is_whitespace() {
                idx -= 1;
            }
            if repaired.chars().nth(idx - 1).unwrap() != '"' {
                // Insert missing closing quote
                repaired.insert(idx, '"');
            }
        }

        if c == '\\' && !escape_next {
            escape_next = true;
        } else {
            escape_next = false;
        }

        repaired.push(c);
    }

    if in_string {
        // Close any unclosed strings at EOF
        repaired.push('"');
    }

    repaired
}

