crate::ix!();

pub fn repair_json_missing_commas_in_list(input: &str) -> String {

    let mut repaired  = String::new();
    let mut in_string = false;
    let mut last_char = '\0';
    let mut chars     = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '"' {
            in_string = !in_string;
        }

        if !in_string && last_char == '"' && c == '"' {
            repaired.push(',');
        }

        repaired.push(c);
        last_char = c;
    }

    repaired
}
