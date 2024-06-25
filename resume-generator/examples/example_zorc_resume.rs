use storefront_resume_builder::*;
use lazy_static::*;
use indoc::*;
use structopt::StructOpt;

lazy_static! {
    pub static ref BETHESDA:       String = "Bethesda, MD".to_string();
    pub static ref CHEVY_CHASE:    String = "Chevy Chase, MD".to_string();
    pub static ref WASHINGTON_DC:  String = "Washington, DC".to_string();
    pub static ref SALT_LAKE_CITY: String = "Salt Lake City, UT".to_string();

    pub static ref CHRIS_ABSTRACT: String = indoc!{"
        Accomplished professional with a diverse background spanning business
        administration, marketing, and social media management. 

        Proven track record in project management, system
        implementation, and strategic operational efficiency upgrades.

        Adept at integrating AI technologies into business processes. 

        Solid foundation in Business Administration and Marketing.
        "
    }.to_string();

    pub static ref WASHINGTON_INSTITUTE_OF_SURGERY: ResumeWorkExperience = ResumeWorkExperience::builder()
        .company("Washington Institute of Surgery".to_string())
        .location(CHEVY_CHASE.clone())
        .role("Billing Manager".to_string())
        .dates(date_range!(start => (2019, 1, 1), end => (2024, 5, 31)))
        .responsibilities(vec![
            "Managed a team of five billing associates. Handled billing cycles for 4 doctors, 3 physician assistants, 1 nurse practitioner, and 3 nurses.".to_string(),
            "Directly responsible for incoming payments, patient statements, and office-patient communications.".to_string(),
            "Oversaw website, software, and hardware upgrades. Served as super-user for all system upgrades".to_string(),
        ])
        .build();

    pub static ref BETHESDA_BOARDS: ResumeWorkExperience = ResumeWorkExperience::builder()
        .company("Bethesda Boards".to_string())
        .location(BETHESDA.clone())
        .role("General Manager, Social Media Manager".to_string())
        .dates(date_range!(start => (2021, 1, 1), end => (2022, 12, 31)))
        .responsibilities(vec![
            "Led the opening of a new location through buildout, website creation, and inventory digitalization.".to_string(),
            "Created and managed content for TikTok, Instagram, and Facebook.".to_string(),
            "Doubled several metrics including engagement and views.".to_string(),
            "Brought in purchases from the West Coast within 6 months.".to_string(),
        ])
        .build();

    pub static ref ZORC_MEDIA_LLC: ResumeWorkExperience = ResumeWorkExperience::builder()
        .company("Zorc Media LLC".to_string())
        .location(BETHESDA.clone())
        .role("Owner, Founder, CEO".to_string())
        .dates(date_range!(start => (2017, 1, 1), end => (2023, 12, 31)))
        .responsibilities(vec![
            "Developed and managed social media profiles including branding for artists, businesses, nonprofits, and personal pages.".to_string(),
            "Implemented AI strategies to optimize social media campaigns and enhance client visibility and engagement.".to_string(),
        ])
        .build();

    pub static ref RAYMOND_JAMES: ResumeWorkExperience = ResumeWorkExperience::builder()
        .company("Raymond James".to_string())
        .location(WASHINGTON_DC.clone())
        .role("Branch Operations Specialist".to_string())
        .dates(date_range!(start => (2017, 1, 1), end => (2017, 12, 31)))
        .responsibilities(vec![
            "Supported daily operations for the Complex Operations Manager and Complex Manager.".to_string(),
            "Utilized LinkedIn and print marketing to promote financial services and events.".to_string(),
        ])
        .build();

    pub static ref DICKS_SPORTING_GOODS: ResumeWorkExperience = ResumeWorkExperience::builder()
        .company("Dick’s Sporting Goods".to_string())
        .location(SALT_LAKE_CITY.clone())
        .role("Customer Specialist".to_string())
        .dates(date_range!(start => (2015, 1, 1), end => (2016, 12, 31)))
        .responsibilities(vec![
            "Managed store cash office procedures and provided high-level customer service.".to_string(),
        ])
        .build();

    pub static ref TOMMY_BAHAMA: ResumeWorkExperience = ResumeWorkExperience::builder()
        .company("Tommy Bahama".to_string())
        .location(BETHESDA.clone())
        .role("Floor Supervisor".to_string())
        .dates(date_range!(start => (2013, 1, 1), end => (2014, 12, 31)))
        .responsibilities(vec![
            "Supervised sales activities, inventory management, and cash reconciliation.".to_string(),
            "Trained staff on effective sales techniques and customer service strategies.".to_string(),
        ])
        .build();

    pub static ref MUSSEL_BAR: ResumeWorkExperience = ResumeWorkExperience::builder()
        .company("Mussel Bar".to_string())
        .location(BETHESDA.clone())
        .role("Host, Food Runner, Barback, Dishwasher, Customer Service".to_string())
        .dates(date_range!(start => (2011, 1, 1), end => (2013, 12, 31)))
        .responsibilities(vec![
            "Worked my way through several front-of-house positions which I enjoyed, especially backing up the bartenders and servers with my hosting, food running, and customer service skills.".to_string(),
        ])
        .build();

    pub static ref KENWOOD_COUNTRY_CLUB: ResumeWorkExperience = ResumeWorkExperience::builder()
        .company("Kenwood Country Club".to_string())
        .location(BETHESDA.clone())
        .role("Indoor and Outdoor Lifeguard".to_string())
        .dates(date_range!(start => (2007, 1, 1), end => (2009, 12, 31)))
        .responsibilities(vec![
            "Began as an indoor lifeguard during the winter season and assimilated well into working as an outdoor lifeguard on a rotating 3 pool schedule.".to_string(),
        ])
        .build();

    pub static ref MIT: ResumeEducationInfo = ResumeEducationInfo::builder()
        .institution("M.I.T.".to_string())
        .location("Online".to_string())
        .degree(r#"Generative Artificial Intelligence \& the Digital Transformation"#.to_string())
        .dates(date_range!(start => (2024, 3, 1)))
        .status(EducationStatus::Student)
        .build();

    pub static ref FORDHAM_UNIVERSITY: ResumeEducationInfo = ResumeEducationInfo::builder()
        .institution("Fordham University".to_string())
        .location("Bronx, NY".to_string())
        .degree("Business Management".to_string())
        .dates(date_range!(start => (2015, 1, 1), end => (2016, 12, 31)))
        .status(EducationStatus::Student)
        .build();

    pub static ref MONTGOMERY_COLLEGE: ResumeEducationInfo = ResumeEducationInfo::builder()
        .institution("Montgomery College".to_string())
        .location("Rockville, MD".to_string())
        .degree(r#"Business Administration \& Marketing"#.to_string())
        .dates(date_range!(start => (2012, 1, 1), end => (2015, 12, 31)))
        .status(EducationStatus::Graduate)
        .build();

    pub static ref TULANE_UNIVERSITY: ResumeEducationInfo = ResumeEducationInfo::builder()
        .institution("Tulane University".to_string())
        .location("New Orleans, LA".to_string())
        .degree("Architecture".to_string())
        .dates(date_range!(start => (2011, 1, 1), end => (2011, 12, 31)))
        .status(EducationStatus::Student)
        .build();

    pub static ref GEORGETOWN_PREP: ResumeEducationInfo = ResumeEducationInfo::builder()
        .institution("Georgetown Prep".to_string())
        .location("Rockville, MD".to_string())
        .degree("".to_string())
        .dates(date_range!(start => (2007, 1, 1), end => (2011, 12, 31)))
        .build();

     pub static ref CHRIS_SKILLS: Vec<ResumeSkill> = vec![
         skill!("Social Media Management"),
         skill!("Content Creation (TikTok, Instagram, Facebook)"),
         skill!("Brand Development"),
         skill!("Project Management"),
         skill!("Billing and Accounts Receivable"),
         skill!("Team Leadership and Training"),
         skill!("AI Integration in Business Processes"),
         skill!("Digital Marketing Strategies"),
         skill!("Customer Service Excellence"),
         skill!("Inventory Management"),
     ];
}

fn main() -> Result<(), ResumeBuilderError> {
    let opt = Cli::from_args();
    generate_resume(opt.output_filename(), opt.output_directory())
}

fn generate_resume(output_filename: &str, output_directory: &str) -> Result<(), ResumeBuilderError> {

    let contact_info = ContactInfo::builder()
        .name("Christopher Joseph Zorc".to_string())
        .email(Email::new("ChristopherZorc@gmail.com".to_string()))
        .phone(PhoneNumber::new("(301) 215-0406".to_string()))
        .location("Bethesda, MD".to_string())
        .build()
        .expect("Failed to build ContactInfo");

    let resume = Resume::builder()
        .contact_info(contact_info)
        .abstract_text(CHRIS_ABSTRACT.clone())
        .work_experience(vec![
            WASHINGTON_INSTITUTE_OF_SURGERY.clone(),
            BETHESDA_BOARDS.clone(),
            ZORC_MEDIA_LLC.clone(),
            RAYMOND_JAMES.clone(),
            DICKS_SPORTING_GOODS.clone(),
            TOMMY_BAHAMA.clone(),
            MUSSEL_BAR.clone(),
            KENWOOD_COUNTRY_CLUB.clone(),
        ])
        .education(vec![
            MIT.clone(),
            FORDHAM_UNIVERSITY.clone(),
            MONTGOMERY_COLLEGE.clone(),
            TULANE_UNIVERSITY.clone(),
            GEORGETOWN_PREP.clone(),
        ])
        .skills(CHRIS_SKILLS.clone())
        .projects(vec![])
        .certifications(vec![])
        .languages(vec![])
        .interests(vec![])
        .build()
        .expect("Failed to build resume");

    let latex_content = resume.latex();
    let output_path = format!("{}/{}", output_directory, output_filename);

    write_content(&latex_content, &output_path)?;
    pdflatex(&output_directory, &output_path)?;

    println!("Chris's Resume LaTeX file generated and compiled successfully.");
    Ok(())
}
