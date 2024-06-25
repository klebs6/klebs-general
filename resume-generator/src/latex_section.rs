crate::ix!();

pub trait LatexSectionItem {
    fn render_latex_snippet(&self) -> String;
}

pub fn render_latex_section<T: LatexSectionItem>(items: &[T], title: &str) -> Option<String> {
    if items.is_empty() {
        return None;
    }

    let mut section = format!(r#"\needspace{{1\baselineskip}}\section*{{{}}}"#, title);

    for item in items {
        section.push_str(r#"\needspace{1\baselineskip}"#); // Adjust as needed based on the item size
        section.push_str(&item.render_latex_snippet());
    }

    Some(section)
}
