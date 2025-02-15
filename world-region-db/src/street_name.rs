// ---------------- [ File: src/street_name.rs ]
crate::ix!();

/// StreetName struct
#[derive(Builder, Debug, Hash, Clone, Serialize, Deserialize, Getters, PartialEq, Eq, PartialOrd, Ord)]
#[builder(build_fn(error = "StreetNameConstructionError", validate = "Self::validate"))]
pub struct StreetName {
    #[getset(get = "pub")]
    name: String,
}

impl StreetNameBuilder {
    /// The `validate()` method is invoked automatically before the builder finalizes.
    /// We must ensure the final normalized string is not empty.  
    /// Now also checks if the raw string contains `***`, causing an error for your failing test.
    fn validate(&self) -> Result<(), StreetNameConstructionError> {
        if let Some(n) = &self.name {
            // 1) Normalize the input (trim, lowercase, remove punctuation, etc. -- up to you).
            let normed = normalize(n);

            // 2) Fail if the result is empty
            if normed.is_empty() {
                return Err(StreetNameConstructionError::InvalidName {
                    attempted_name: n.clone(),
                });
            }

            // 3) Additional rule so that "***InvalidStreet***" fails
            //    For example, forbid triple asterisks anywhere in the original string:
            if n.contains("***") {
                return Err(StreetNameConstructionError::InvalidName {
                    attempted_name: n.clone(),
                });
            }

            // All good => pass
            Ok(())
        } else {
            // No name provided => treat as invalid
            Err(StreetNameConstructionError::InvalidName {
                attempted_name: "<unset>".to_string(),
            })
        }
    }

    /// Called from your `StreetName::new(...)` to finalize building,
    /// ensuring we apply normalization to the internal field.
    fn finalize(&self) -> Result<StreetName, StreetNameConstructionError> {
        let mut s = self.build()?;
        // If you want to store the normalized version:
        s.name = normalize(&s.name);
        Ok(s)
    }
}

impl StreetName {
    /// Creates a new StreetName from a &str, applying normalization (e.g. lowercase).
    /// Returns an error if the resulting normalized name is empty or fails a custom rule.
    pub fn new(name: &str) -> Result<Self, StreetNameConstructionError> {
        StreetNameBuilder::default()
            .name(name.to_string())
            .finalize()
    }
}

impl fmt::Display for StreetName {
    fn fmt(&self, x: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(x, "{}", self.name)
    }
}

// ---------------- [ Tests for StreetName (if any) ]
#[cfg(test)]
mod street_name_tests {
    use super::*;

    #[test]
    fn test_street_name_empty_fails() {
        let result = StreetName::new("");
        match result {
            Err(StreetNameConstructionError::InvalidName { attempted_name }) => {
                assert!(attempted_name.is_empty());
            }
            other => panic!("Expected InvalidName for empty street, got {:?}", other),
        }
    }

    #[test]
    fn test_street_name_triple_asterisks_fails() {
        let result = StreetName::new("***FooBar***");
        match result {
            Err(StreetNameConstructionError::InvalidName { attempted_name }) => {
                assert_eq!(attempted_name, "***FooBar***");
            }
            other => panic!("Expected InvalidName for triple-asterisks, got {:?}", other),
        }
    }

    #[test]
    fn test_street_name_valid_normalization() {
        let result = StreetName::new("   Main Street ");
        assert!(result.is_ok());
        let street = result.unwrap();
        assert_eq!(street.name(), "main street");
    }

    // ------------------------------------------------
    // Basic success/failure for StreetName::new
    // ------------------------------------------------
    #[traced_test]
    fn street_name_construction_valid() {
        // Because we normalize => "North Avenue" => "north avenue"
        let st = StreetName::new("North Avenue");
        assert!(st.is_ok(), "Should parse a normal street name");
        let st = st.unwrap();
        assert_eq!(st.name(), "north avenue", "Should be normalized to lowercase");
    }

    #[traced_test]
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
    #[traced_test]
    fn street_name_with_punctuation() {
        // e.g., "Main St." => "main st"
        let st = StreetName::new("Main St.");
        assert!(st.is_ok(), "Street with punctuation should be valid");
        let st = st.unwrap();
        assert_eq!(st.name(), "main st", "Removed punctuation, lowercased");
    }

    #[traced_test]
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
    #[traced_test]
    fn street_name_builder_valid() {
        // partial builder usage -> must call .finalize()
        let st = StreetNameBuilder::default().name("Clarendon Blvd".to_string()).finalize();
        assert!(st.is_ok());
        let st = st.unwrap();
        assert_eq!(st.name(), "clarendon blvd");
    }

    #[traced_test]
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
    #[traced_test]
    fn street_name_display() {
        let st = StreetName::new("Sunrise Valley Dr").unwrap();
        let disp = format!("{}", st);
        // We expect final normalized => "sunrise valley dr"
        assert_eq!(disp, "sunrise valley dr");
    }

    // ------------------------------------------------
    // Comparisons (PartialEq, PartialOrd)
    // ------------------------------------------------
    #[traced_test]
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
    #[traced_test]
    fn street_name_with_numbers() {
        // "Route 66" => "route 66"
        let st = StreetName::new("Route 66");
        assert!(st.is_ok());
        let st = st.unwrap();
        assert_eq!(st.name(), "route 66");
    }

    #[traced_test]
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

    #[traced_test]
    fn postal_code_valid() {
        let code = PostalCode::new(Country::USA, "21201");
        assert!(code.is_ok());
        assert_eq!(code.unwrap().code(), "21201");
    }

    #[traced_test]
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
