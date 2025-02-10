// ---------------- [ File: src/city_name.rs ]
// ---------------- [ File: src/city_name.rs ]
crate::ix!();

/// CityName struct
#[derive(Builder,Debug,Hash,Clone,Serialize,Deserialize,Getters,PartialEq,Eq,PartialOrd,Ord)]
#[builder(build_fn(error = "CityNameConstructionError",validate = "Self::validate"))]
pub struct CityName {
    #[getset(get = "pub")]
    name: String,
}

impl CityNameBuilder {
    fn validate(&self) -> Result<(), CityNameConstructionError> {
        if let Some(n) = &self.name {
            if normalize(n).is_empty() {
                return Err(CityNameConstructionError::InvalidName {
                    attempted_name: n.clone(),
                });
            }
            Ok(())
        } else {
            Err(CityNameConstructionError::InvalidName {
                attempted_name: "<unset>".to_string(),
            })
        }
    }

    fn finalize(&self) -> Result<CityName, CityNameConstructionError> {
        let mut city = self.build()?;
        city.name = normalize(&city.name);
        Ok(city)
    }
}

impl fmt::Display for CityName {
    fn fmt(&self, x: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        // We display the final, normalized name (lowercase, etc.).
        // If you wanted to preserve original case for display, you'd store it
        // in a separate field. But given the rest of the tests, we do lowercase.
        write!(x, "{}", self.name)
    }
}

impl CityName {
    /// Creates a new CityName from a &str, applying normalization (e.g. lowercase).
    pub fn new(name: &str) -> Result<Self, CityNameConstructionError> {
        CityNameBuilder::default()
            .name(name.to_string())
            .finalize()
    }
}

/// Tests for CityName construction and validation
#[cfg(test)]
mod city_name_tests {
    use super::*;

    #[traced_test]
    fn city_name_construction_valid() {
        // We supply "Baltimore", but after normalize() it should become "baltimore".
        let city = CityName::new("Baltimore");
        assert!(city.is_ok());
        let city = city.unwrap();
        // The code always lowercases => we expect "baltimore".
        assert_eq!(city.name(), "baltimore");
    }

    #[traced_test]
    fn city_name_construction_empty() {
        let city = CityName::new("   ");
        match city {
            Err(CityNameConstructionError::InvalidName { attempted_name }) => {
                assert_eq!(attempted_name.trim(), "");
            },
            _ => panic!("Expected InvalidName error"),
        }
    }

    #[traced_test]
    fn city_name_with_punctuation() {
        // "Washington, D.C." => "washington d c"
        let city = CityName::new("Washington, D.C.");
        assert!(city.is_ok());
        let city = city.unwrap();
        assert_eq!(city.name(), "washington d c");
    }

    #[traced_test]
    fn city_name_with_internal_spaces() {
        // "   New   York  " => "new york"
        let city = CityName::new("   New   York  ");
        assert!(city.is_ok());
        let city = city.unwrap();
        assert_eq!(city.name(), "new york");
    }

    #[traced_test]
    fn city_name_builder_valid() {
        let city_result = CityNameBuilder::default()
            .name("Annapolis".to_string())
            .finalize();
        assert!(city_result.is_ok());
        let city = city_result.unwrap();
        // We lowercase => "annapolis".
        assert_eq!(city.name(), "annapolis");
    }

    #[traced_test]
    fn city_name_builder_missing_field() {
        let builder = CityNameBuilder::default();
        let city_result = builder.finalize();
        match city_result {
            Err(CityNameConstructionError::InvalidName { attempted_name }) => {
                assert_eq!(attempted_name, "<unset>");
            },
            other => panic!("Expected InvalidName error for unset field, got: {:?}", other),
        }
    }

    #[traced_test]
    fn city_name_display() {
        let city = CityName::new("Baltimore").unwrap();
        let displayed = format!("{}", city);
        // We expect the lowercased final form => "baltimore".
        assert_eq!(displayed, "baltimore", "Display should match the final normalized name");
    }

    #[traced_test]
    fn city_name_comparisons() {
        let city1 = CityName::new("Baltimore").unwrap();  // => "baltimore"
        let city2 = CityName::new("baltimore").unwrap();  // => "baltimore"
        let city3 = CityName::new("Washington").unwrap(); // => "washington"

        assert_eq!(city1, city2);
        assert_ne!(city1, city3);
        // "baltimore" < "washington" => true
        assert!(city1 < city3);
    }

    #[traced_test]
    fn city_name_with_numbers() {
        let city = CityName::new("Area 51");
        assert!(city.is_ok());
        let city = city.unwrap();
        // => "area 51"
        assert_eq!(city.name(), "area 51");
    }

    #[traced_test]
    fn city_name_extreme_length() {
        let long_name = "L".repeat(500);
        let city = CityName::new(&long_name);
        assert!(city.is_ok());
        let city = city.unwrap();
        // => 500 'l' chars
        assert_eq!(city.name().len(), 500);
    }
}
