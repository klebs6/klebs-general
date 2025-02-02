// ---------------- [ File: src/street_name.rs ]
crate::ix!();

/// StreetName struct
#[derive(Builder,Debug,Hash,Clone,Serialize,Deserialize,Getters,PartialEq,Eq,PartialOrd,Ord)]
#[builder(build_fn(error = "StreetNameConstructionError",validate = "Self::validate"))]
pub struct StreetName {
    #[getset(get = "pub")]
    name: String,
}

impl StreetNameBuilder {
    fn validate(&self) -> Result<(), StreetNameConstructionError> {
        if let Some(n) = &self.name {
            if normalize(n).is_empty() {
                return Err(StreetNameConstructionError::InvalidName {
                    attempted_name: n.clone(),
                });
            }
            Ok(())
        } else {
            Err(StreetNameConstructionError::InvalidName {
                attempted_name: "<unset>".to_string(),
            })
        }
    }

    fn finalize(&self) -> Result<StreetName, StreetNameConstructionError> {
        let mut city = self.build()?;
        city.name = normalize(&city.name);
        Ok(city)
    }
}

impl fmt::Display for StreetName {

    fn fmt(&self, x: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(x,"{}",self.name)
    }
}

impl StreetName {

    pub fn new(name: &str) -> Result<Self, StreetNameConstructionError> {
        StreetNameBuilder::default()
            .name(name.to_string())
            .finalize()
    }
}

#[cfg(test)]
mod street_name_tests {
    use super::*;

    // ------------------------------------------------
    // Basic success/failure for StreetName::new
    // ------------------------------------------------
    #[test]
    fn street_name_construction_valid() {
        // Because we normalize => "North Avenue" => "north avenue"
        let st = StreetName::new("North Avenue");
        assert!(st.is_ok(), "Should parse a normal street name");
        let st = st.unwrap();
        assert_eq!(st.name(), "north avenue", "Should be normalized to lowercase");
    }

    #[test]
    fn street_name_construction_empty() {
        let st = StreetName::new("   ");
        match st {
            Err(StreetNameConstructionError::InvalidName { attempted_name }) => {
                assert_eq!(attempted_name.trim(), "", "Name is effectively empty");
            },
            _ => panic!("Expected InvalidName error for empty string"),
        }
    }

    // ------------------------------------------------
    // Testing punctuation & spacing
    // ------------------------------------------------
    #[test]
    fn street_name_with_punctuation() {
        // e.g., "Main St." => "main st"
        let st = StreetName::new("Main St.");
        assert!(st.is_ok(), "Street with punctuation should be valid");
        let st = st.unwrap();
        assert_eq!(st.name(), "main st", "Removed punctuation, lowercased");
    }

    #[test]
    fn street_name_with_internal_spaces() {
        // "  Redwood   Rd  " => "redwood rd"
        let st = StreetName::new("  Redwood   Rd  ");
        assert!(st.is_ok());
        let st = st.unwrap();
        assert_eq!(st.name(), "redwood rd", "Collapses spaces, lowercases");
    }

    // ------------------------------------------------
    // Builder usage: partial + finalize
    // ------------------------------------------------
    #[test]
    fn street_name_builder_valid() {
        // partial builder usage -> must call .finalize()
        let st = StreetNameBuilder::default().name("Clarendon Blvd".to_string()).finalize();
        assert!(st.is_ok());
        let st = st.unwrap();
        assert_eq!(st.name(), "clarendon blvd");
    }

    #[test]
    fn street_name_builder_missing_field() {
        // If we never call .name(...), finalize should yield an error
        let builder = StreetNameBuilder::default();
        let st = builder.finalize();
        match st {
            Err(StreetNameConstructionError::InvalidName { attempted_name }) => {
                assert_eq!(attempted_name, "<unset>");
            }
            other => panic!("Expected InvalidName error for unset field, got: {:?}", other),
        }
    }

    // ------------------------------------------------
    // Test Display
    // ------------------------------------------------
    #[test]
    fn street_name_display() {
        let st = StreetName::new("Sunrise Valley Dr").unwrap();
        let disp = format!("{}", st);
        // We expect final normalized => "sunrise valley dr"
        assert_eq!(disp, "sunrise valley dr");
    }

    // ------------------------------------------------
    // Comparisons (PartialEq, PartialOrd)
    // ------------------------------------------------
    #[test]
    fn street_name_comparisons() {
        let st1 = StreetName::new("Wilson Blvd").unwrap();   // => "wilson blvd"
        let st2 = StreetName::new("wilson blvd").unwrap();   // => "wilson blvd"
        let st3 = StreetName::new("Main Street").unwrap();   // => "main street"

        assert_eq!(st1, st2, "Case differences vanish => equal after normalize");
        assert_ne!(st1, st3);

        // In ASCII, "main street" < "wilson blvd"
        assert!(st3 < st1, "Expected alphabetical ordering");
    }

    // ------------------------------------------------
    // Edge cases: numeric or symbolic
    // ------------------------------------------------
    #[test]
    fn street_name_with_numbers() {
        // "Route 66" => "route 66"
        let st = StreetName::new("Route 66");
        assert!(st.is_ok());
        let st = st.unwrap();
        assert_eq!(st.name(), "route 66");
    }

    #[test]
    fn street_name_extreme_length() {
        // e.g. 500 chars of "A"
        let long_name = "A".repeat(500);
        let st = StreetName::new(&long_name);
        assert!(st.is_ok());
        let st = st.unwrap();
        // => 500 chars, all lower => "a"
        assert_eq!(st.name().len(), 500);
    }
}

/// Tests for PostalCode construction
#[cfg(test)]
mod postal_code_tests {
    use super::*;

    #[test]
    fn postal_code_valid() {
        let code = PostalCode::new(Country::USA, "21201");
        assert!(code.is_ok());
        assert_eq!(code.unwrap().code(), "21201");
    }

    #[test]
    fn postal_code_empty() {
        let code = PostalCode::new(Country::USA, "  ");
        match code {
            Err(PostalCodeConstructionError::InvalidFormat { 
                attempted_code, 
                attempted_country: Some(Country::USA) 
            }) => {
                assert_eq!(attempted_code.trim(), "");
            },
            _ => panic!("Expected InvalidFormat"),
        }
    }
}
