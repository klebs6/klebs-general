crate::ix!();

#[derive(Debug)]
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
