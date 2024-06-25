crate::ix!();

#[derive(Debug,Clone)]
pub struct ResumeProjects(Vec<ResumeProject>);

impl From<Vec<ResumeProject>> for ResumeProjects {
    fn from(x: Vec<ResumeProject>) -> Self {
        Self(x)
    }
}

impl ResumeProjects {
    delegate!{
        to self.0 {
            pub fn is_empty(&self) -> bool;
            pub fn len(&self) -> usize;
        }
    }
}

impl LatexSectionItem for ResumeProjects {

    fn render_latex_snippet(&self) -> String {

        let mut result = String::new();

        if !self.0.is_empty() {

            result.push_str(r#"\section*{Projects}\begin{itemize}[leftmargin=*, label=-]"#);

            for project in &self.0 {
                result.push_str(&project.render_latex_snippet());
            }

            result.push_str(r#"\end{itemize}\vspace{2pt}"#);
        }

        result
    }
}

#[derive(Debug,Clone)]
pub struct ResumeProject {
    title:       String,
    dates:       DateRange,
    description: Vec<String>,
}

impl LatexSectionItem for ResumeProject {

    fn render_latex_snippet(&self) -> String {
        let mut result = String::new();
        result.push_str(&format!(
            indoc! {r#"
            \item \textbf{{{}}} \hfill \textit{{{}}} \\
            "#},
            self.title, format_date_range(&self.dates)
        ));
        if !self.description.is_empty() {
            result.push_str(r#"\begin{itemize}[leftmargin=*, label=-]"#);
            for desc in &self.description {
                result.push_str(&format!("    \\item {}\n", desc));
            }
            result.push_str(r#"\end{itemize}"#);
        }
        result
    }
}

impl ResumeProject {

    pub fn builder() -> ResumeProjectBuilder {
        ResumeProjectBuilder::default()
    }

    pub fn new(title: String, dates: DateRange, description: Vec<String>) -> Self {
        Self { title, dates, description }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn dates(&self) -> &DateRange {
        &self.dates
    }

    pub fn description(&self) -> &[String] {
        &self.description
    }
}

#[derive(Default)]
pub struct ResumeProjectBuilder {
    title:       Option<String>,
    dates:       Option<DateRange>,
    description: Vec<String>,
}

impl ResumeProjectBuilder {
    pub fn title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    pub fn dates(mut self, dates: DateRange) -> Self {
        self.dates = Some(dates);
        self
    }

    pub fn description(mut self, description: Vec<String>) -> Self {
        self.description = description;
        self
    }

    pub fn build(self) -> ResumeProject {
        ResumeProject {
            title: self.title.expect("Title is required"),
            dates: self.dates.expect("Dates are required"),
            description: self.description,
        }
    }
}
