// ---------------- [ File: src/print_help.rs ]
crate::ix!();

pub fn print_help() {
    let mut lines = Vec::new();
    lines.push("Commands:".to_string());
    lines.push("  help                  => this help".to_string());
    lines.push("  exit                  => quit the REPL".to_string());
    lines.push("  region <abbr>         => switch to that region (must be in the 'DONE' set)".to_string());
    lines.push("  mode <city|street>    => toggle fuzzy completion mode".to_string());
    lines.push("  validate <zip> <city> [<house_num>] <street...>".to_string());
    lines.push("  byzip <zip>           => list city/street sets for that zip".to_string());
    lines.push("  range <street> [num]  => show house number ranges (and optionally check a house_num)".to_string());
    lines.push("Typing a recognized city or street shows DB info.\n".to_string());

    print_or_page(&lines);
}
