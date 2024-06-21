crate::ix!();

#[derive(Clone,Debug)]
pub struct ErrorTree {
    pub(crate) enums: Vec<ErrorEnum>,
}

impl ToTokens for ErrorTree {

    fn to_tokens(&self, tokens: &mut TokenStream2) {

        self.enums.iter().for_each(|x| x.to_tokens(tokens));

        let from_impls: Vec<FromImplGenerationConfig> = self.into();

        from_impls.iter().for_each(|x| x.to_tokens(tokens));
    }
}

impl From<Vec<ErrorEnum>> for ErrorTree {

    fn from(enums: Vec<ErrorEnum>) -> Self {
        Self { enums }
    }
}

impl Validate for ErrorTree {

    fn validate(&self) -> bool {

        // Check for duplicate enum names
        let mut enum_names = HashSet::new();

        for error_enum in &self.enums {

            if !enum_names.insert(&error_enum.ident) {
                return false; // Duplicate enum name found
            }

            // Additional checks for each enum
            if !error_enum.validate() {
                return false;
            }
        }

        // Other global checks can be added here

        true // Passes all checks
    }
}
