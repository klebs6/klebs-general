crate::ix!();

pub struct ContactInfo {
    name:     String,
    email:    Email,
    linkedin: Option<LinkedInInfo>,
    github:   Option<GitHubInfo>,
    phone:    PhoneNumber,
    location: String,
}

impl ContactInfo {
    pub fn new(
        name: String,
        email: Email,
        linkedin: Option<LinkedInInfo>,
        github: Option<GitHubInfo>,
        phone: PhoneNumber,
        location: String,
    ) -> Self {
        Self {
            name,
            email,
            linkedin,
            github,
            phone,
            location,
        }
    }

    pub fn builder() -> ContactInfoBuilder {
        ContactInfoBuilder::default()
    }
}

#[derive(Default)]
pub struct ContactInfoBuilder {
    name:     Option<String>,
    email:    Option<Email>,
    linkedin: Option<LinkedInInfo>,
    github:   Option<GitHubInfo>,
    phone:    Option<PhoneNumber>,
    location: Option<String>,
}

impl ContactInfoBuilder {
    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn email(mut self, email: Email) -> Self {
        self.email = Some(email);
        self
    }

    pub fn linkedin(mut self, linkedin: LinkedInInfo) -> Self {
        self.linkedin = Some(linkedin);
        self
    }

    pub fn github(mut self, github: GitHubInfo) -> Self {
        self.github = Some(github);
        self
    }

    pub fn phone(mut self, phone: PhoneNumber) -> Self {
        self.phone = Some(phone);
        self
    }

    pub fn location(mut self, location: String) -> Self {
        self.location = Some(location);
        self
    }

    pub fn build(self) -> Result<ContactInfo, &'static str> {
        Ok(ContactInfo {
            name: self.name.ok_or("Name is required")?,
            email: self.email.ok_or("Email is required")?,
            linkedin: self.linkedin,
            github: self.github,
            phone: self.phone.ok_or("PhoneNumber is required")?,
            location: self.location.ok_or("Location is required")?,
        })
    }
}

impl LatexSectionItem for ContactInfo {
    fn render_latex_snippet(&self) -> String {
        let mut contact_info = indoc! {r#"
            % Contact Information
            \begin{center}
                {\LARGE NAME_PLACEHOLDER} \\
                \vspace{2pt}
                \href{mailto:EMAIL_PLACEHOLDER}{EMAIL_PLACEHOLDER} \\
        "#}
        .replace("NAME_PLACEHOLDER", &self.name)
        .replace("EMAIL_PLACEHOLDER", &self.email.to_string());

        if let Some(linkedin) = &self.linkedin {
            contact_info.push_str(&format!(r"\href{{{}}}{{{}}} \\", linkedin.url(), linkedin.url()));
        }

        if let Some(github) = &self.github {
            contact_info.push_str(&format!(r"\href{{{}}}{{{}}} \\", github.url(), github.url()));
        }

        contact_info.push_str(&indoc!(r#"
                PHONE_PLACEHOLDER \\
                LOCATION_PLACEHOLDER \\
            \end{center}
        "#)
        .replace("PHONE_PLACEHOLDER", &self.phone.to_string())
        .replace("LOCATION_PLACEHOLDER", &self.location));

        contact_info
    }
}
