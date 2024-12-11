crate::ix!();

/// Represents a person's name as a string.
///
/// Encapsulates the name as a string type for semantic clarity.
#[derive(Builder,Getters,Setters,Debug,PartialEq,Eq,Clone,Serialize,Deserialize)]
#[builder(setter(into, strip_option))]
#[getset(get="pub",set="pub")]
pub struct PersonName {

    #[builder(default)]
    title:  Option<PersonTitle>,

    first:  FirstName,

    #[builder(default)]
    middle: Option<Middle>,

    last:   LastName,
}

impl Default for PersonName {

    fn default() -> Self {
        Self {
            title:  Some(PersonTitle::Mr),
            first:  "John".into(),
            middle: Some('G'.into()),
            last:   "Doe".into(),
        }
    }

}

impl PersonName {

    /// Return a formatted full name.
    /// Includes the title if present, followed by first, middle (if any), and last name.
    pub fn full_name(&self) -> String {
        let mut parts = Vec::new();

        if let Some(t) = &self.title {
            parts.push(t.to_string());
        }

        parts.push(self.first().to_string());

        if let Some(m) = &self.middle {
            parts.push(m.to_string());
        }

        parts.push(self.last().to_string());

        parts.join(" ")
    }
}

// Implement Display for PersonName for convenience.
impl fmt::Display for PersonName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.full_name())
    }
}

#[cfg(test)]
mod person_name_tests {
    use super::*;

    #[test]
    fn test_default_person_name() {
        let default_name = PersonName::default();
        assert_eq!(*default_name.first(), "John".into());
        assert_eq!(*default_name.last(), "Doe".into());
        assert_eq!(default_name.middle.as_ref().unwrap().to_string(), "G");
        assert_eq!(default_name.title.unwrap(), PersonTitle::Mr);
    }

    #[test]
    fn test_full_name() {
        let name = PersonNameBuilder::default()
            .title(PersonTitle::Dr)
            .first("Alice")
            .middle("Marie")
            .last(last!("Smith"))
            .build()
            .expect("Failed to build PersonName");

        assert_eq!(name.full_name(), "Dr. Alice Marie Smith");
    }

    #[test]
    fn test_no_middle_or_title() {
        let name = PersonNameBuilder::default()
            .first("Bob")
            .last(last!("Johnson"))
            .build()
            .expect("Failed to build PersonName");
        
        // Should just be "Bob Johnson"
        assert_eq!(name.full_name(), "Bob Johnson");
    }

    #[test]
    fn test_display_person_name() {
        let name = PersonNameBuilder::default()
            .title(PersonTitle::Mx)
            .first("Taylor")
            .middle('J')
            .last(last!("Adams"))
            .build()
            .unwrap();

        assert_eq!(format!("{}", name), "Mx. Taylor J Adams");
    }

    #[test]
    fn test_serde_round_trip() {
        let name = PersonNameBuilder::default()
            .title(PersonTitle::Eng)
            .first("Linus")
            .middle("Torvalds")
            .last(last!("Smith"))
            .build()
            .unwrap();

        let json = serde_json::to_string(&name).expect("Failed to serialize");
        let deserialized: PersonName = serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(name, deserialized);
    }

    #[test]
    fn test_edge_cases_middle_initial() {
        let name = PersonNameBuilder::default()
            .first("Anne")
            .middle('Z')
            .last(last!("Clark"))
            .build()
            .unwrap();

        assert_eq!(name.full_name(), "Anne Z Clark");
    }
}
