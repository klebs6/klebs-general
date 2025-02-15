crate::ix!();

/// Print lines normally if they're short, otherwise invoke scrolling_pager.
pub fn print_or_page(lines: &[String]) {
    if lines.len() > 20 {
        scrolling_pager(lines);
    } else {
        for line in lines {
            println!("{}", line);
        }
    }
}
