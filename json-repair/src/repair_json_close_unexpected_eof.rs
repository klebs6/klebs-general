crate::ix!();

pub fn repair_json_close_unexpected_eof(input: &str) -> String {

    let mut repaired             = input.to_owned();
    let mut open_brackets: isize = 0;
    let mut open_braces: isize   = 0;
    let mut in_string            = false;
    let mut escape_next          = false;

    for c in input.chars() {
        if in_string {
            if escape_next {
                escape_next = false;
            } else if c == '\\' {
                escape_next = true;
            } else if c == '"' {
                in_string = false;
            }
        } else {
            match c {
                '"' => in_string = true,
                '{' => open_braces += 1,
                '}' => {
                    if open_braces > 0 {
                        open_braces -= 1;
                    }
                },
                '[' => open_brackets += 1,
                ']' => {
                    if open_brackets > 0 {
                        open_brackets -= 1;
                    }
                },
                _ => {}
            }
        }
    }

    // Close any unclosed strings
    if in_string {
        repaired.push('"');
    }

    // Close any unclosed structures
    for _ in 0..open_brackets {
        repaired.push(']');
    }
    for _ in 0..open_braces {
        repaired.push('}');
    }

    repaired
}
