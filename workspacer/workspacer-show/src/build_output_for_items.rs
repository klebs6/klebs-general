crate::ix!();

/// Renders a list of `T: std::fmt::Display` items into a single string separated by blank lines.
pub fn build_output_for_items<T>(items: &[T], options: &ShowFlags) -> String
where
    T: std::fmt::Display,
{
    if items.is_empty() && options.show_items_with_no_data() {
        return "<no-data-for-crate>\n".to_string();
    }
    let mut lines = Vec::new();
    for item in items {
        lines.push(format!("{}", item));
    }
    join_with_blank_line(lines)
}
