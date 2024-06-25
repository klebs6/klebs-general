crate::ix!();

#[derive(Debug,Clone)]
pub struct ResumeCertifications(Vec<ResumeCertification>);

impl From<Vec<ResumeCertification>> for ResumeCertifications {
    fn from(x: Vec<ResumeCertification>) -> Self {
        Self(x)
    }
}

impl ResumeCertifications {
    delegate!{
        to self.0 {
            pub fn is_empty(&self) -> bool;
            pub fn len(&self) -> usize;
        }
    }
}

impl LatexSectionItem for ResumeCertifications {

    fn render_latex_snippet(&self) -> String {

        let mut result = String::new();

        if !self.0.is_empty() {

            result.push_str(r#"\section*{Certifications}\begin{itemize}[leftmargin=*, label=-]"#);

            for cert in &self.0 {
                result.push_str(&cert.render_latex_snippet());
            }

            result.push_str(r#"\end{itemize}\vspace{2pt}"#);
        }

        result
    }
}

#[derive(Debug,Clone)]
pub struct ResumeCertification {
    name:                 String,
    issuing_organization: String,
    date:                 NaiveDate,
}

impl LatexSectionItem for ResumeCertification {
    fn render_latex_snippet(&self) -> String {
        format!(r#"    \item {}, {} \hfill \textit{{{}}} \\"#, self.name, self.issuing_organization, self.date)
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
