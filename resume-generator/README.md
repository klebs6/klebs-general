# Resume Builder Library in Rust

This project is a Rust-based library for generating LaTeX-formatted resumes. 

The resume information is structured using various Rust structs, and the final output is a LaTeX file that can be compiled to produce a professional resume.

## Features

- Struct-based approach for organizing resume data
- Builder pattern for constructing resume sections
- Generates LaTeX formatted resume
- Supports nested itemize environments
- Ensures sections do not split across page breaks

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
resume-builder = "*"
```

## Example

```rust
use structopt::*;
use resume_generator::*;

fn main() -> Result<(), ResumeBuilderError> {

    let opt = Cli::from_args();

    generate_resume(opt.output_filename(), opt.output_directory())
}

fn generate_resume(output_filename: &str, output_directory: &str) -> Result<(), ResumeBuilderError> {
    let contact_info = ContactInfo::builder()
        .name("Your Name".to_string())
        .email(Email::new("youremail@example.com".to_string()))
        .linkedin(LinkedInInfo::new("https://www.linkedin.com/in/yourprofile".to_string()))
        .github(GitHubInfo::new("https://github.com/yourprofile".to_string()))
        .phone(PhoneNumber::new("Your Phone Number".to_string()))
        .location("City, State, ZIP Code".to_string())
        .build()
        .expect("Failed to build ContactInfo");

    let resume = ResumeBuilder::new()
        .contact_info(contact_info)
        .abstract_text("A brief summary about yourself, your skills, and your career goals.".to_string())
        .work_experience(vec![
            ResumeWorkExperience::builder()
                .company("Company Name".to_string())
                .location("Location".to_string())
                .role("Role".to_string())
                .dates(date_range!(start => (2020, 1, 1), end => (2021, 12, 31)))
                .responsibilities(vec![
                    "Responsibility or achievement 1".to_string(),
                    "Responsibility or achievement 2".to_string(),
                    "Responsibility or achievement 3".to_string(),
                ])
                .build(),
            // Add more work experience as needed
        ])
        .education(vec![
            ResumeEducationInfo::builder()
                .institution("Institution Name".to_string())
                .location("Location".to_string())
                .degree("Degree Title".to_string())
                .dates(date_range!(start => (2016, 9, 1), end => (2020, 6, 30)))
                .build(),
            // Add more education as needed
        ])
        .skills(vec![
            skill!("Skill 1"),
            skill!("Skill 2"),
            skill!("Skill 3"),
            skill!("Skill 4"),
        ])
        .projects(vec![
            ResumeProject::builder()
                .title("Project Title".to_string())
                .dates(date_range!(start => (2019, 1, 1), end => (2019, 12, 31)))
                .description(vec![
                    "Description of the project and your role in it".to_string(),
                ])
                .build(),
            ResumeProject::builder()
                .title("Project2 Title".to_string())
                .dates(date_range!(start => (2024, 1, 1), end => (2024, 12, 31)))
                .description(vec![
                    "Description of the second project and your role in it".to_string(),
                ])
                .build(),

            ResumeProject::builder()
                .title("Project3 Title".to_string())
                .dates(date_range!(start => (2024, 3, 1), end => (2025, 12, 31)))
                .description(vec![
                    "Description of the third project and your role in it".to_string(),
                ])
                .build(),
            // Add more projects as needed
        ])
        .certifications(vec![
            ResumeCertification::builder()
                .name("Certification Name".to_string())
                .issuing_organization("First Issuing Organization".to_string())
                .date(date!(2020, 6, 1))
                .build(),
            ResumeCertification::builder()
                .name("Certification2 Name".to_string())
                .issuing_organization("Second Issuing Organization".to_string())
                .date(date!(2021, 8, 10))
                .build(),
            ResumeCertification::builder()
                .name("Certification3 Name".to_string())
                .issuing_organization("Third Issuing Organization".to_string())
                .date(date!(2023, 2, 14))
                .build(),
            // Add more certifications as needed
        ])
        .languages(vec![
            Language::new(LanguageName::English, ProficiencyLevel::Native),
            // Add more languages as needed
        ])
        .interests(vec![
            ResumeInterest::new("Interest 1".to_string()),
            ResumeInterest::new("Interest 2".to_string()),
            ResumeInterest::new("Interest 3".to_string()),
            ResumeInterest::new("Interest 4".to_string()),
        ])
        .build()
        .expect("Failed to build resume");

    let latex_content = resume.latex();

    let output_path = format!("{}/{}", output_directory, output_filename);

    write_content(&latex_content, &output_path)?;

    pdflatex(output_directory, &output_path)?;

    println!("Resume LaTeX file generated and compiled successfully.");

    Ok(())
}
```

See the examples directory for more usage examples.

## Contributing

Contributions are welcome! Please submit a pull request or open an issue to discuss your ideas.

## License

This project is licensed under the MIT License. See the LICENSE file for details.
