crate::ix!();

#[derive(Debug,Clone)]
pub struct ResumeSkills(Vec<ResumeSkill>);

impl From<Vec<ResumeSkill>> for ResumeSkills {
    fn from(x: Vec<ResumeSkill>) -> Self {
        Self(x)
    }
}

impl ResumeSkills {
    delegate!{
        to self.0 {
            pub fn is_empty(&self) -> bool;
            pub fn len(&self) -> usize;
        }
    }
}

impl LatexSectionItem for ResumeSkills {
    fn render_latex_snippet(&self) -> String {
        let mut result = String::new();
        if !self.0.is_empty() {
            result.push_str(r#"\section*{Skills}\begin{itemize}[leftmargin=*, label=-]"#);
            for skill in &self.0 {
                result.push_str(&format!("    \\item {}\n", skill.name()));
            }
            result.push_str(r#"\end{itemize}\vspace{2pt}"#);
        }
        result
    }
}


#[derive(Debug,Clone)]
pub struct ResumeSkill {
    name: String,
}

impl ResumeSkill {
    pub fn builder() -> ResumeSkillBuilder {
        ResumeSkillBuilder::default()
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Default)]
pub struct ResumeSkillBuilder {
    name: Option<String>,
}

impl ResumeSkillBuilder {
    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn build(self) -> ResumeSkill {
        ResumeSkill {
            name: self.name.expect("Name is required"),
        }
    }
}

#[macro_export]
macro_rules! skill {
    ($name:expr) => {
        ResumeSkill::builder().name($name.to_string()).build()
    };
}
