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

/// Tests for StreetName construction and validation
#[cfg(test)]
mod street_name_tests {
    use super::*;

    #[test]
    fn street_name_construction_valid() {
        let st = StreetName::new("North Avenue");
        assert!(st.is_ok());
        let st = st.unwrap();
        assert_eq!(st.name(), "North Avenue");
    }

    #[test]
    fn street_name_construction_empty() {
        let st = StreetName::new("   ");
        match st {
            Err(StreetNameConstructionError::InvalidName { attempted_name }) => {
                assert_eq!(attempted_name.trim(), "");
            },
            _ => panic!("Expected InvalidName error"),
        }
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
