crate::ix!();

pub fn repair_json_mismatched_brackets(input: &str) -> Result<String,JsonRepairError> {

    info!("fixing any mismatched brackets");

    let mut repaired    = String::new();
    let mut stack       = Vec::new();
    let mut in_string   = false;
    let mut escape_next = false;
    let mut chars       = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '"' && !escape_next {
            in_string = !in_string;
        }

        if !in_string {
            match c {
                '{' | '[' => stack.push(c),
                '}' | ']' => {
                    if let Some(open) = stack.pop() {
                        if (open == '{' && c == ']') || (open == '[' && c == '}') {
                            // Replace mismatched closing bracket
                            repaired.push(if open == '{' { '}' } else { ']' });
                            continue;
                        }
                    } else {
                        // Extra closing bracket/brace; skip it
                        continue;
                    }
                }
                _ => {}
            }
        } else if c == '\\' && !escape_next {
            escape_next = true;
        } else {
            escape_next = false;
        }

        repaired.push(c);
    }

    // Close any unclosed brackets/braces
    while let Some(open) = stack.pop() {
        repaired.push(if open == '{' { '}' } else { ']' });
    }

    Ok(repaired)
}

