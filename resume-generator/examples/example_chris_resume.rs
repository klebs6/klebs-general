use resume_generator::*;
use std::fs::File;
use std::io::Write;

fn main() -> Result<(), ResumeBuilderError> {

    let output_filename = get_output_filename()?;

    let resume = ResumeBuilder::new()
        .name("Chris Doe".to_string())
        .email(Email::new("chris.doe@example.com".to_string()))
        .linkedin(LinkedInInfo::new("https://www.linkedin.com/in/chrisdoe".to_string()))
        .github(GitHubInfo::new("https://github.com/chrisdoe".to_string()))
        .phone(PhoneNumber::new("123-456-7890".to_string()))
        .location("San Francisco, CA".to_string())
        .abstract_text("Experienced software engineer with a passion for developing innovative programs that expedite the efficiency and effectiveness of organizational success.".to_string())
        .work_experience(vec![
            ResumeWorkExperience::new(
                "Tech Company".to_string(),
                "San Francisco, CA".to_string(),
                "Senior Software Engineer".to_string(),
                DateRange::new(date(2018, 5, 1)?, None),
                vec![
                    "Led a team of 10 software engineers to develop scalable applications".to_string(),
                    "Improved system performance by 20%".to_string(),
                    "Implemented CI/CD pipelines".to_string(),
                ],
            ),
            ResumeWorkExperience::new(
                "Another Tech Company".to_string(),
                "San Jose, CA".to_string(),
                "Software Engineer".to_string(),
                DateRange::new(date(2015, 6, 1)?, Some(date(2018, 4, 30)?)),
                vec![
                    "Developed web applications using Rust and JavaScript".to_string(),
                    "Collaborated with cross-functional teams to define project requirements".to_string(),
                    "Conducted code reviews and provided mentorship to junior engineers".to_string(),
                ],
            ),
        ])
        .education(vec![
            ResumeEducationInfo::new(
                "University of California, Berkeley".to_string(),
                "Berkeley, CA".to_string(),
                "Bachelor of Science in Computer Science".to_string(),
                DateRange::new(date(2011, 9, 1)?, Some(date(2015, 5, 31)?)),
                vec![
                    "Graduated with honors".to_string(),
                    "Member of the Computer Science Club".to_string(),
                ],
            ),
        ])
        .skills(vec![
            ResumeSkill::new("Rust".to_string()),
            ResumeSkill::new("JavaScript".to_string()),
            ResumeSkill::new("CI/CD".to_string()),
            ResumeSkill::new("Team Leadership".to_string()),
        ])
        .projects(vec![
            ResumeProject::new(
                "Open Source Contribution".to_string(),
                DateRange::new(date(2019, 1, 1)?, None),
                vec![
                    "Contributed to open source projects on GitHub".to_string(),
                    "Fixed bugs and implemented new features".to_string(),
                ],
            ),
        ])
        .certifications(vec![
            ResumeCertification::new(
                "Certified Kubernetes Administrator".to_string(),
                "The Linux Foundation".to_string(),
                date(2020, 8, 1)?,
            ),
        ])
        .languages(vec![
            Language::new(LanguageName::English, ProficiencyLevel::Native),
            Language::new(LanguageName::Spanish, ProficiencyLevel::Intermediate),
        ])
        .interests(vec![
            ResumeInterest::new("Hiking".to_string()),
            ResumeInterest::new("Photography".to_string()),
            ResumeInterest::new("Traveling".to_string()),
            ResumeInterest::new("Open Source Contribution".to_string()),
        ])
        .build()
        .expect("Failed to build resume");

    let latex_content = generate_latex(&resume);

    let mut file = File::create(output_filename).expect("Could not create file");
    file.write_all(latex_content.as_bytes()).expect("Could not write to file");

    println!("Chris's Resume LaTeX file generated successfully.");

    Ok(())
}
