crate::ix!();

#[derive(Debug,Clone)]
pub struct ResumeLanguages(Vec<Language>);

impl From<Vec<Language>> for ResumeLanguages {
    fn from(x: Vec<Language>) -> Self {
        Self(x)
    }
}

impl ResumeLanguages {
    delegate!{
        to self.0 {
            pub fn is_empty(&self) -> bool;
            pub fn len(&self) -> usize;
        }
    }
}

impl LatexSectionItem for ResumeLanguages {

    fn render_latex_snippet(&self) -> String {

        let mut result = String::new();

        if !self.0.is_empty() {

            result.push_str(r#"\section*{Languages}\begin{itemize}[leftmargin=*, label=-]"#);

            for lang in &self.0 {
                let text = lang.render_latex_snippet();
                result.push_str(&text);
            }

            result.push_str(r#"\end{itemize}\vspace{2pt}"#);
        }

        result
    }
}

#[derive(Debug,Clone)]
pub struct Language {
    name:        LanguageName,
    proficiency: ProficiencyLevel,
}

impl LatexSectionItem for Language {
    fn render_latex_snippet(&self) -> String {
        format!("    \\item {} - {}\n", self.name, self.proficiency)
    }
}

impl Language {
    pub fn new(name: LanguageName, proficiency: ProficiencyLevel) -> Self {
        Self { name, proficiency }
    }

    pub fn name(&self) -> &LanguageName {
        &self.name
    }

    pub fn proficiency(&self) -> &ProficiencyLevel {
        &self.proficiency
    }
}


#[derive(Debug,Clone)]
pub enum LanguageName {
    English,
    Spanish,
    French,
    German,
    Chinese,
    Other(String),
}

impl fmt::Display for LanguageName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LanguageName::English => write!(f, "English"),
            LanguageName::Spanish => write!(f, "Spanish"),
            LanguageName::French => write!(f, "French"),
            LanguageName::German => write!(f, "German"),
            LanguageName::Chinese => write!(f, "Chinese"),
            LanguageName::Other(name) => write!(f, "{}", name),
        }
    }
}

#[derive(Debug,Clone)]
pub enum ProficiencyLevel {
    Native,
    Fluent,
    Professional,
    Intermediate,
    Basic,
}

impl fmt::Display for ProficiencyLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProficiencyLevel::Native => write!(f, "Native"),
            ProficiencyLevel::Fluent => write!(f, "Fluent"),
            ProficiencyLevel::Professional => write!(f, "Professional"),
            ProficiencyLevel::Intermediate => write!(f, "Intermediate"),
            ProficiencyLevel::Basic => write!(f, "Basic"),
        }
    }
}
