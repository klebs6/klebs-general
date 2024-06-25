crate::ix!();

pub struct Resume {
    contact_info:     ContactInfo,
    abstract_text:    String,
    work_experience:  Vec<ResumeWorkExperience>,
    education:        Vec<ResumeEducationInfo>,
    skills:           Option<ResumeSkills>,
    projects:         Vec<ResumeProject>,
    certifications:   Vec<ResumeCertification>,
    languages:        Vec<Language>,
    interests:        Vec<ResumeInterest>,
}

impl RenderLatex for Resume {
    fn latex(&self) -> String {
        let contact_info = self.contact_info().render_latex_snippet();
        let abstract_section = format!(indoc! {r#"
            \section*{{Abstract}}
            {}
        "#}, self.abstract_text());

        let mut sections = vec![self.begin_document(), contact_info, abstract_section];

        if let Some(work_section) = render_latex_section(&self.work_experience, "Work Experience") {
            sections.push(work_section);
        }

        if let Some(education_section) = render_latex_section(&self.education, "Education") {
            sections.push(education_section);
        }

        if let Some(skills_section) = self.skills.as_ref().map(|skills| skills.render_latex_snippet()) {
            sections.push(skills_section);
        }

        if let Some(projects_section) = render_latex_section(&self.projects, "Projects") {
            sections.push(projects_section);
        }

        if let Some(certifications_section) = render_latex_section(&self.certifications, "Certifications") {
            sections.push(certifications_section);
        }

        if let Some(languages_section) = render_latex_section(&self.languages, "Languages") {
            sections.push(languages_section);
        }

        if let Some(interests_section) = render_latex_section(&self.interests, "Interests") {
            sections.push(interests_section);
        }

        sections.push(self.end_document());
        sections.join("\n")
    }
}

impl Resume {

    pub fn builder() -> ResumeBuilder {
        ResumeBuilder::default()
    }

    pub fn contact_info(&self) -> &ContactInfo {
        &self.contact_info
    }

    pub fn abstract_text(&self) -> &str {
        &self.abstract_text
    }

    pub fn work_experience(&self) -> &[ResumeWorkExperience] {
        &self.work_experience
    }

    pub fn education(&self) -> &[ResumeEducationInfo] {
        &self.education
    }

    pub fn skills(&self) -> &Option<ResumeSkills> {
        &self.skills
    }

    pub fn projects(&self) -> &[ResumeProject] {
        &self.projects
    }

    pub fn certifications(&self) -> &[ResumeCertification] {
        &self.certifications
    }

    pub fn languages(&self) -> &[Language] {
        &self.languages
    }

    pub fn interests(&self) -> &[ResumeInterest] {
        &self.interests
    }

    pub fn has_work_experience(&self) -> bool {
        !self.work_experience.is_empty()
    }

    pub fn has_education(&self) -> bool {
        !self.education.is_empty()
    }

    pub fn has_skills(&self) -> bool {
        self.skills.is_some() && !self.skills.as_ref().unwrap().is_empty()
    }

    pub fn has_projects(&self) -> bool {
        !self.projects.is_empty()
    }

    pub fn has_certifications(&self) -> bool {
        !self.certifications.is_empty()
    }

    pub fn has_languages(&self) -> bool {
        !self.languages.is_empty()
    }

    pub fn has_interests(&self) -> bool {
        !self.interests.is_empty()
    }
}

#[derive(Default)]
pub struct ResumeBuilder {
    contact_info:     Option<ContactInfo>,
    abstract_text:    Option<String>,
    work_experience:  Vec<ResumeWorkExperience>,
    education:        Vec<ResumeEducationInfo>,
    skills:           Option<ResumeSkills>,
    projects:         Vec<ResumeProject>,
    certifications:   Vec<ResumeCertification>,
    languages:        Vec<Language>,
    interests:        Vec<ResumeInterest>,
}

impl ResumeBuilder {

    pub fn new() -> Self {
        Self::default()
    }

    pub fn contact_info(mut self, contact_info: ContactInfo) -> Self {
        self.contact_info = Some(contact_info);
        self
    }

    pub fn abstract_text(mut self, abstract_text: String) -> Self {
        self.abstract_text = Some(abstract_text);
        self
    }

    pub fn work_experience(mut self, work_experience: Vec<ResumeWorkExperience>) -> Self {
        self.work_experience = work_experience;
        self
    }

    pub fn education(mut self, education: Vec<ResumeEducationInfo>) -> Self {
        self.education = education;
        self
    }

    pub fn skills(mut self, skills: Vec<ResumeSkill>) -> Self {

        if skills.is_empty() {
            self.skills = None;
            return self;
        }

        self.skills = Some(ResumeSkills::from(skills));
        self
    }

    pub fn projects(mut self, projects: Vec<ResumeProject>) -> Self {
        self.projects = projects;
        self
    }

    pub fn certifications(mut self, certifications: Vec<ResumeCertification>) -> Self {
        self.certifications = certifications;
        self
    }

    pub fn languages(mut self, languages: Vec<Language>) -> Self {
        self.languages = languages;
        self
    }

    pub fn interests(mut self, interests: Vec<ResumeInterest>) -> Self {
        self.interests = interests;
        self
    }

    pub fn build(self) -> Result<Resume, &'static str> {
        Ok(Resume {
            contact_info:  self.contact_info.ok_or("ContactInfo is required")?,
            abstract_text: self.abstract_text.ok_or("Abstract text is required")?,
            work_experience: self.work_experience,
            education: self.education,
            skills: self.skills,
            projects: self.projects,
            certifications: self.certifications,
            languages: self.languages,
            interests: self.interests,
        })
    }
}
