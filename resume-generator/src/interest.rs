crate::ix!();

#[derive(Debug,Clone)]
pub struct ResumeInterests(Vec<ResumeInterest>);

impl From<Vec<ResumeInterest>> for ResumeInterests {
    fn from(x: Vec<ResumeInterest>) -> Self {
        Self(x)
    }
}

impl ResumeInterests {
    delegate!{
        to self.0 {
            pub fn is_empty(&self) -> bool;
            pub fn len(&self) -> usize;
        }
    }
}

impl LatexSectionItem for ResumeInterests {

    fn render_latex_snippet(&self) -> String {

        let mut result = String::new();

        if !self.0.is_empty() {

            result.push_str(r#"\section*{Interests}\begin{itemize}[leftmargin=*, label=-]"#);

            for interest in &self.0 {
                let text = interest.render_latex_snippet();
                result.push_str(&text);
            }

            result.push_str(r#"\end{itemize}\vspace{2pt}"#);
        }

        result
    }
}

#[derive(Debug,Clone)]
pub struct ResumeInterest {
    name: String,
}

impl LatexSectionItem for ResumeInterest {
    fn render_latex_snippet(&self) -> String {
        format!("    \\item {}\n", self.name)
    }
}

impl ResumeInterest {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
