crate::ix!();

#[derive(Debug)]
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


#[derive(Debug)]
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

#[derive(Debug)]
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
