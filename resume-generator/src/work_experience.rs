crate::ix!();

#[derive(Debug, Clone)]
pub struct ResumeWorkExperience {
    company:          String,
    location:         String,
    role:             String,
    dates:            DateRange,
    responsibilities: Vec<String>,
}

impl LatexSectionItem for ResumeWorkExperience {

    fn render_latex_snippet(&self) -> String {
        let mut result = String::new();

        // Company and location
        if !self.company.is_empty() && !self.location.is_empty() {
            result.push_str(&format!(
                r#"\textbf{{{}, {}}} \\"#,
                self.company, self.location
            ));
        }

        // Role
        if !self.role.is_empty() {
            result.push_str(&format!(
                r#"\textit{{{}}} \\"#,
                self.role
            ));
        }

        // Dates
        let date_range = format_date_range(&self.dates);
        if !date_range.is_empty() {
            result.push_str(&format!(
                r#"{}"#,
                date_range
            ));
        }

        // Responsibilities
        if !self.responsibilities.is_empty() {
            result.push_str(r#"\begin{itemize}[leftmargin=*, label=-]"#);
            for responsibility in &self.responsibilities {
                result.push_str(&format!("    \\item {}\n", responsibility));
            }
            result.push_str(r#"\end{itemize}"#);
        }

        result
    }
}

impl ResumeWorkExperience {
    pub fn builder() -> ResumeWorkExperienceBuilder {
        ResumeWorkExperienceBuilder::default()
    }

    // Accessor methods
    pub fn company(&self) -> &str { &self.company }
    pub fn location(&self) -> &str { &self.location }
    pub fn role(&self) -> &str { &self.role }
    pub fn dates(&self) -> &DateRange { &self.dates }
    pub fn responsibilities(&self) -> &[String] { &self.responsibilities }
}

#[derive(Default)]
pub struct ResumeWorkExperienceBuilder {
    company:          Option<String>,
    location:         Option<String>,
    role:             Option<String>,
    dates:            Option<DateRange>,
    responsibilities: Vec<String>,
}

impl ResumeWorkExperienceBuilder {
    pub fn company(mut self, company: String) -> Self {
        self.company = Some(company);
        self
    }

    pub fn location(mut self, location: String) -> Self {
        self.location = Some(location);
        self
    }

    pub fn role(mut self, role: String) -> Self {
        self.role = Some(role);
        self
    }

    pub fn dates(mut self, dates: DateRange) -> Self {
        self.dates = Some(dates);
        self
    }

    pub fn responsibilities(mut self, responsibilities: Vec<String>) -> Self {
        self.responsibilities = responsibilities;
        self
    }

    pub fn build(self) -> ResumeWorkExperience {
        ResumeWorkExperience {
            company: self.company.expect("Company is required"),
            location: self.location.expect("Location is required"),
            role: self.role.expect("Role is required"),
            dates: self.dates.expect("Dates are required"),
            responsibilities: self.responsibilities,
        }
    }
}
