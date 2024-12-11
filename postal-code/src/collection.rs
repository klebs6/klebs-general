crate::ix!();

#[derive(Builder, Debug)]
#[builder(setter(into))]
pub struct PostalCodeCollection {
    // A collection of postal codes
    #[builder(setter(each(name = "code")))]
    codes: Vec<PostalCode>,
}

impl PostalCodeCollection {
    pub fn len(&self) -> usize {
        self.codes.len()
    }
}

#[cfg(test)]
mod postal_code_collection_tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_postal_code_collection() {
        // Test building a collection using `#[builder(each = "code")]`
        let pc1 = PostalCode::new(Country::USA, "12345").unwrap();
        let pc2 = PostalCode::new(Country::France, "75001").unwrap();
        let pc3 = PostalCode::new(Country::Canada, "K1A0B1").unwrap();

        let collection = PostalCodeCollectionBuilder::default()
            .code(pc1.clone())
            .code(pc2.clone())
            .code(pc3.clone())
            .build()
            .expect("All required fields set");

        assert_eq!(collection.len(), 3);
        let set: HashSet<_> = collection.codes.iter().collect();
        assert!(set.contains(&pc1));
        assert!(set.contains(&pc2));
        assert!(set.contains(&pc3));
    }

    #[test]
    fn test_postal_code_collection_empty() {
        // Test an empty collection
        let collection = PostalCodeCollectionBuilder::default().build();
        // Missing required field (codes) triggers UninitializedFieldError converted to InvalidFormat.
        assert!(collection.is_err());
    }
}
