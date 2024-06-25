crate::ix!();

#[derive(Debug, Clone)]
pub enum EducationStatus {
    Student,
    Graduate,
}

impl fmt::Display for EducationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EducationStatus::Student => write!(f, "Student"),
            EducationStatus::Graduate => write!(f, "Graduate"),
        }
    }
}

#[derive(Debug,Clone)]
pub struct ResumeEducationInfo {
    institution:     String,
    location:        String,
    degree:          String,
    dates:           DateRange,
    status:          Option<EducationStatus>,
}

impl LatexSectionItem for ResumeEducationInfo {
    fn render_latex_snippet(&self) -> String {
        let mut result = String::new();

        // Institution and location
        if !self.institution.is_empty() && !self.location.is_empty() {
            result.push_str(&format!(
                r#"\textbf{{{}, {}}} \\"#,
                self.institution, self.location
            ));
        }

        // Degree
        if !self.degree.is_empty() {
            result.push_str(&format!(
                r#"\textit{{{}}} \\"#,
                self.degree
            ));
        }

        // Dates
        let date_range = format_date_range(&self.dates);
        if !date_range.is_empty() {
            result.push_str(&format!(
                r#"{} \\"#,
                date_range
            ));
        }

        // Status
        if let Some(status) = &self.status {
            result.push_str(&format!(
                r#"\textit{{{}}} "#,
                match status {
                    EducationStatus::Student => "Student",
                    EducationStatus::Graduate => "Graduate",
                }
            ));
        }

        result
    }
}


impl ResumeEducationInfo {

    pub fn builder() -> ResumeEducationInfoBuilder {
        ResumeEducationInfoBuilder::default()
    }

    pub fn institution(&self) -> &str {
        &self.institution
    }

    pub fn location(&self) -> &str {
        &self.location
    }

    pub fn degree(&self) -> &str {
        &self.degree
    }

    pub fn dates(&self) -> &DateRange {
        &self.dates
    }

    pub fn status(&self) -> &Option<EducationStatus> {
        &self.status
    }
}

#[derive(Default)]
pub struct ResumeEducationInfoBuilder {
    institution:     Option<String>,
    location:        Option<String>,
    degree:          Option<String>,
    dates:           Option<DateRange>,
    status:          Option<EducationStatus>,
}

impl ResumeEducationInfoBuilder {
    pub fn institution(mut self, institution: String) -> Self {
        self.institution = Some(institution);
        self
    }

    pub fn location(mut self, location: String) -> Self {
        self.location = Some(location);
        self
    }

    pub fn degree(mut self, degree: String) -> Self {
        self.degree = Some(degree);
        self
    }

    pub fn dates(mut self, dates: DateRange) -> Self {
        self.dates = Some(dates);
        self
    }

    pub fn status(mut self, status: EducationStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn build(self) -> ResumeEducationInfo {
        ResumeEducationInfo {
            institution: self.institution.expect("Institution is required"),
            location: self.location.expect("Location is required"),
            degree: self.degree.expect("Degree is required"),
            dates: self.dates.expect("Dates are required"),
            status: self.status,
        }
    }
}
