crate::ix!();

/// A validated postal code with country context.
#[derive(Builder,Debug,Hash,Clone,PartialEq,Eq,Serialize,Deserialize,Getters,Ord,PartialOrd)]
#[builder(build_fn(error = "PostalCodeConstructionError",validate = "Self::validate"))]
pub struct PostalCode {

    /// The country associated with this postal code.
    #[getset(get = "pub")]
    country: Country,

    /// The validated postal code string.
    #[getset(get = "pub")]
    code: String,
}

impl PostalCode {

    pub fn new(country: Country, code: &str) -> Result<Self,PostalCodeConstructionError> {
        PostalCodeBuilder::default()
            .country(country)
            .code(code.to_string())
            .build()
    }
}

impl PostalCodeBuilder {
    pub fn validate(&self) -> Result<(), PostalCodeConstructionError> {
        if self.country.is_none() || self.code.is_none() {
            return Err(PostalCodeConstructionError::InvalidFormat {
                attempted_code: "<unset>".to_string(),
                attempted_country: None,
            });
        }
        let country = self.country.as_ref().unwrap();
        let code = self.code.as_ref().unwrap();

        if let Some(validator) = country.get_postal_code_validator() {
            if validator.validate(code) {
                Ok(())
            } else {
                Err(PostalCodeConstructionError::InvalidFormat {
                    attempted_code: code.clone(),
                    attempted_country: Some(country.clone()),
                })
            }
        } else {
            Err(PostalCodeConstructionError::UnsupportedCountry {
                attempted_country: country.clone(),
            })
        }
    }
}

#[cfg(test)]
mod postal_code_tests {
    use super::*;
    use country::Country;
    use rand::Rng; // If you add rand to dev-dependencies for randomness tests

    #[test]
    fn test_us_valid() {
        let pc = PostalCode::new(Country::USA, "12345");
        assert!(pc.is_ok());
        assert_eq!(pc.unwrap().code(), "12345");
    }

    #[test]
    fn test_us_valid_zip_plus4() {
        let pc = PostalCode::new(Country::USA, "12345-6789");
        assert!(pc.is_ok());
    }

    #[test]
    fn test_us_invalid_alphabetic() {
        let pc = PostalCode::new(Country::USA, "ABCDE");
        assert!(pc.is_err());
        if let Err(PostalCodeConstructionError::InvalidFormat { attempted_code, attempted_country }) = pc {
            assert_eq!(attempted_code, "ABCDE");
            assert_eq!(attempted_country, Some(Country::USA));
        } else {
            panic!("Unexpected error type");
        }
    }

    #[test]
    fn test_us_invalid_length() {
        let pc = PostalCode::new(Country::USA, "1234");
        assert!(pc.is_err());
    }

    #[test]
    fn test_ca_valid() {
        let pc = PostalCode::new(Country::Canada, "K1A0B1");
        assert!(pc.is_ok());
    }

    #[test]
    fn test_ca_valid_with_space() {
        // Common Canadian formatting includes a space after the first three characters.
        let pc = PostalCode::new(Country::Canada, "K1A 0B1");
        assert!(pc.is_ok());
    }

    #[test]
    fn test_ca_invalid() {
        let pc = PostalCode::new(Country::Canada, "123456");
        assert!(pc.is_err());
    }

    #[test]
    fn test_ca_invalid_non_alphanumeric() {
        // Contains a symbol that's not allowed
        let pc = PostalCode::new(Country::Canada, "K1A!0B1");
        assert!(pc.is_err());
    }

    #[test]
    fn test_uk_valid() {
        // Buckingham Palace code: "SW1A 1AA"
        let pc = PostalCode::new(Country::UK, "SW1A 1AA");
        assert!(pc.is_ok());
    }

    #[test]
    fn test_uk_invalid_no_space() {
        // UK codes typically have a space; this might fail validation
        let pc = PostalCode::new(Country::UK, "SW1A1AA");
        assert!(pc.is_err());
    }

    #[test]
    fn test_uk_invalid_too_long() {
        let pc = PostalCode::new(Country::UK, "SW1A 1AAA");
        assert!(pc.is_err());
    }

    #[test]
    fn test_fr_valid() {
        let pc = PostalCode::new(Country::France, "75001");
        assert!(pc.is_ok());
    }

    #[test]
    fn test_fr_invalid_short() {
        let pc = PostalCode::new(Country::France, "7500");
        assert!(pc.is_err());
    }

    #[test]
    fn test_fr_invalid_alpha() {
        let pc = PostalCode::new(Country::France, "75A01");
        assert!(pc.is_err());
    }

    #[test]
    fn test_de_valid() {
        let pc = PostalCode::new(Country::Germany, "10115");
        assert!(pc.is_ok());
    }

    #[test]
    fn test_de_invalid_short() {
        let pc = PostalCode::new(Country::Germany, "101");
        assert!(pc.is_err());
    }

    #[test]
    fn test_de_invalid_alpha() {
        let pc = PostalCode::new(Country::Germany, "10A15");
        assert!(pc.is_err());
    }

    #[test]
    fn test_it_valid() {
        let pc = PostalCode::new(Country::Italy, "00144");
        assert!(pc.is_ok());
    }

    #[test]
    fn test_it_invalid_short() {
        let pc = PostalCode::new(Country::Italy, "0144");
        assert!(pc.is_err());
    }

    #[test]
    fn test_it_invalid_alpha() {
        let pc = PostalCode::new(Country::Italy, "00A44");
        assert!(pc.is_err());
    }

    #[test]
    fn test_unsupported_country() {
        let pc = PostalCode::new(Country::Uzbekistan, "12345");
        assert!(pc.is_err());
        if let Err(PostalCodeConstructionError::UnsupportedCountry { attempted_country }) = pc {
            assert_eq!(attempted_country, Country::Uzbekistan);
        } else {
            panic!("Expected UnsupportedCountry error");
        }
    }

    #[test]
    fn test_missing_fields_via_builder() {
        // If we try to build without setting required fields, we should get a suitable error.
        let pc = PostalCodeBuilder::default().build(); 
        assert!(pc.is_err());
        if let Err(PostalCodeConstructionError::InvalidFormat { attempted_code, attempted_country }) = pc {
            assert_eq!(attempted_code, "<unset>");
            assert_eq!(attempted_country, None);
        } else {
            panic!("Expected InvalidFormat error due to missing fields");
        }
    }

    #[test]
    fn test_multiple_random_us_codes() {
        // Test random US ZIP codes
        // Just ensure they pass the regex if they follow correct format
        let mut rng = rand::thread_rng();
        for _ in 0..10 {
            let base: u32 = rng.gen_range(0..100000);
            let code = format!("{:05}", base);
            let pc = PostalCode::new(Country::USA, &code);
            assert!(pc.is_ok());
        }
        // Test some random invalid US codes with letters
        for _ in 0..5 {
            let code = format!("{}ABCD", rng.gen_range(0..10000));
            let pc = PostalCode::new(Country::USA, &code);
            assert!(pc.is_err());
        }
    }

    #[test]
    fn test_space_and_hyphen_tolerance_in_ca_codes() {
        // Canadian codes can have an optional space. Check that a hyphen fails.
        let pc = PostalCode::new(Country::Canada, "K1A-0B1");
        assert!(pc.is_err());
    }

    #[test]
    fn test_uk_gir_special_case() {
        // G.IR 0AA is a special National Girobank code. Check that it is valid.
        let pc = PostalCode::new(Country::UK, "GIR 0AA");
        assert!(pc.is_ok());
    }
}
