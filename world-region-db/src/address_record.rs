// ---------------- [ File: src/address_record.rs ]
crate::ix!();

/// A simple structure to hold address info extracted from OSM.
/// Not all OSM ways/nodes have addresses, we only store those that do.
#[derive(PartialEq,Eq,Clone, Default, Builder, Getters, Setters, Debug)]
#[getset(get = "pub", set = "pub")]
#[builder(default, setter(into))]
pub struct AddressRecord {
    city:     Option<CityName>,
    street:   Option<StreetName>,
    postcode: Option<PostalCode>,
}

/// Helper to create an AddressRecord easily
#[macro_export]
macro_rules! address_record {
    ($city:ident, $street:ident, $postcode:ident) => {
        AddressRecord::new($city.clone(), $street.clone(), $postcode.clone())
    };
}

impl AddressRecord {
    pub fn new(city: CityName, street: StreetName, postcode: PostalCode) -> AddressRecord {
        AddressRecordBuilder::default()
            .city(Some(city))
            .street(Some(street))
            .postcode(Some(postcode))
            .build()
            .unwrap()
    }

    pub fn is_empty(&self) -> bool {
        self.city.is_none() && self.street.is_none() && self.postcode.is_none()
    }
}

// If you also want to define separate impls for &Node, &Way, etc., do so here or remove them.
// This example shows a single approach using address_record_from_tags for all element types.
#[cfg(test)]
mod address_record_tests {
    use super::*;

    #[traced_test]
    fn test_collect_tags() {
        let pairs = vec![
            ("addr:city", "Baltimore"),
            ("addr:street", "North Avenue"),
            ("addr:postcode", "21201"),
        ];
        let map = collect_tags(mock_tag_iter(pairs));
        assert_eq!(map.get("addr:city").map(|x| x.as_str()), Some("Baltimore"));
        assert_eq!(map.get("addr:street").map(|x| x.as_str()), Some("North Avenue"));
        assert_eq!(map.get("addr:postcode").map(|x| x.as_str()), Some("21201"));
    }

    #[traced_test]
    fn test_address_record_new_valid() {
        let city = CityName::new("Baltimore").unwrap();
        let street = StreetName::new("North Avenue").unwrap();
        let pc = PostalCode::new(Country::USA, "21201").unwrap();
        let record = AddressRecord::new(city.clone(), street.clone(), pc.clone());
        assert!(!record.is_empty());
        assert_eq!(record.city().as_ref().unwrap().name(), city.name());
        assert_eq!(record.street().as_ref().unwrap().name(), street.name());
        assert_eq!(record.postcode().as_ref().unwrap().code(), pc.code());
    }

    #[traced_test]
    fn test_address_record_macro() {
        let city = CityName::new("Rockville").unwrap();
        let street = StreetName::new("Veirs Mill Rd").unwrap();
        let pc = PostalCode::new(Country::USA, "20850").unwrap();

        let record = address_record!(city, street, pc);
        assert!(!record.is_empty());
        assert_eq!(record.city().as_ref().unwrap().name(), "rockville");
        assert_eq!(record.street().as_ref().unwrap().name(), "veirs mill rd");
        assert_eq!(record.postcode().as_ref().unwrap().code(), "20850");
    }

    #[traced_test]
    fn test_address_record_is_empty() {
        let empty = AddressRecordBuilder::default().build().unwrap();
        assert!(empty.is_empty());

        let partial_city = AddressRecordBuilder::default()
            .city(Some(CityName::new("Alexandria").unwrap()))
            .build()
            .unwrap();
        assert!(!partial_city.is_empty());
        assert!(partial_city.street().is_none());
        assert!(partial_city.postcode().is_none());
    }

    #[traced_test]
    fn test_address_record_from_tags_all_fields() {
        let tags = vec![
            ("addr:city", "Fairfax"),
            ("addr:street", "Main Street"),
            ("addr:postcode", "22030"),
        ];
        let result = try_build_address_record_from_tags(tags.into_iter(), Country::USA, 999);
        assert!(result.is_ok());
        let record = result.unwrap();
        assert_eq!(record.city().as_ref().unwrap().name(), "fairfax");
        assert_eq!(record.street().as_ref().unwrap().name(), "main street");
        assert_eq!(record.postcode().as_ref().unwrap().code(), "22030");
    }

    #[traced_test]
    fn test_try_build_address_record_from_tags_no_fields() {
        let tags = vec![];
        let result = try_build_address_record_from_tags(tags.into_iter(), Country::USA, 1001);
        assert!(result.is_err());
        match result.err().unwrap() {
            IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(IncompatibleOsmPbfNode::Incompatible { id }) => {
                assert_eq!(id, 1001);
            }
            other => panic!("Expected IncompatibleOsmPbfNode, got: {:?}", other),
        }
    }

    #[traced_test]
    fn test_try_build_address_record_from_tags_invalid_street() {
        // Suppose street is whitespace => StreetName returns error
        let tags = vec![
            ("addr:city", "Reston"),
            ("addr:street", "   "),
        ];
        let result = try_build_address_record_from_tags(
            tags.into_iter(),
            Country::USA,
            123
        );
        assert!(result.is_err());

        match result.err().unwrap() {
            IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
                IncompatibleOsmPbfNode::StreetNameConstructionError(e)
            ) => {
                // Great! Exactly the variant that indicates
                // a street parse error. If needed, you can also
                // inspect `e` for more detail.
                println!("Got StreetNameConstructionError: {:?}", e);
            }
            other => {
                panic!("Expected StreetNameConstructionError, got: {:?}", other);
            }
        }
    }
}
