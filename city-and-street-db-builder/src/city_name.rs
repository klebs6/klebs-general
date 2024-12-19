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
        write!(x,"{}",self.name)
    }
}

impl CityName {
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

    #[test]
    fn city_name_construction_valid() {
        let city = CityName::new("Baltimore");
        assert!(city.is_ok());
        assert_eq!(city.unwrap().name(), "Baltimore");
    }

    #[test]
    fn city_name_construction_empty() {
        let city = CityName::new("   ");
        match city {
            Err(CityNameConstructionError::InvalidName { attempted_name }) => {
                assert_eq!(attempted_name.trim(), "");
            },
            _ => panic!("Expected InvalidName error"),
        }
    }
}
