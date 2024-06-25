crate::ix!();

#[derive(Debug)]
pub struct ResumeCertification {
    name:                 String,
    issuing_organization: String,
    date:                 NaiveDate,
}

impl LatexSectionItem for ResumeCertification {
    fn render_latex_snippet(&self) -> String {
        format!(r#"    \\item {}, {} \hfill \textit{{{}}}\n"#, self.name, self.issuing_organization, self.date)
    }
}

impl ResumeCertification {

    pub fn builder() -> ResumeCertificationBuilder {
        ResumeCertificationBuilder::default()
    }

    pub fn new(name: String, issuing_organization: String, date: NaiveDate) -> Self {
        Self { name, issuing_organization, date }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn issuing_organization(&self) -> &str {
        &self.issuing_organization
    }

    pub fn date(&self) -> NaiveDate {
        self.date
    }
}

#[derive(Default)]
pub struct ResumeCertificationBuilder {
    name:                 Option<String>,
    issuing_organization: Option<String>,
    date:                 Option<NaiveDate>,
}

impl ResumeCertificationBuilder {

    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn issuing_organization(mut self, issuing_organization: String) -> Self {
        self.issuing_organization = Some(issuing_organization);
        self
    }

    pub fn date(mut self, date: NaiveDate) -> Self {
        self.date = Some(date);
        self
    }

    pub fn build(self) -> ResumeCertification {
        ResumeCertification {
            name: self.name.expect("Name is required"),
            issuing_organization: self.issuing_organization.expect("Issuing organization is required"),
            date: self.date.expect("Date is required"),
        }
    }
}
