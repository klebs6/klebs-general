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

/// A small helper function that collects key-value pairs into a HashMap.
/// Typically used after calling `element.tags()`.
fn collect_tags<'a>(tags: impl Iterator<Item = (&'a str, &'a str)>) -> HashMap<&'a str, &'a str> {
    tags.collect()
}

/// This function factors out the logic of extracting (city, street, postcode)
/// from an iterator of (&str, &str) tags. We return an `AddressRecord` if
/// at least one of them is present, or an error if no relevant tags exist.
pub fn address_record_from_tags<'a>(
    tags_iter: impl Iterator<Item = (&'a str, &'a str)>,
    country:   Country,
    element_id: i64,
) -> Result<AddressRecord, IncompatibleOsmPbfElement>
{
    let tags = collect_tags(tags_iter);

    let city     = tags.get("addr:city");
    let street   = tags.get("addr:street");
    let postcode = tags.get("addr:postcode");

    if city.is_none() && street.is_none() && postcode.is_none() {
        // No relevant tags => treat as incompatible element
        return Err(IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
            IncompatibleOsmPbfNode::Incompatible { id: element_id }
        ));
    }

    let city_obj = match city {
        Some(c) => {
            match CityName::new(c) {
                Ok(city_name) => Some(city_name),
                Err(e) => {
                    return Err(IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
                        IncompatibleOsmPbfNode::CityNameConstructionError(e),
                    ));
                }
            }
        }
        None => None,
    };

    let street_obj = match street {
        Some(s) => {
            match StreetName::new(s) {
                Ok(street_name) => Some(street_name),
                Err(e) => {
                    return Err(IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
                        IncompatibleOsmPbfNode::StreetNameConstructionError(e),
                    ));
                }
            }
        }
        None => None,
    };

    let postcode_obj = match postcode {
        Some(pc) => {
            match PostalCode::new(country, pc) {
                Ok(postal) => Some(postal),
                Err(e) => {
                    return Err(IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
                        IncompatibleOsmPbfNode::PostalCodeConstructionError(e),
                    ));
                }
            }
        }
        None => None,
    };

    let record = AddressRecord {
        city: city_obj,
        street: street_obj,
        postcode: postcode_obj,
    };
    Ok(record)
}

/// Now the TryFrom implementation can simply call the factored-out function.
/// If you also want to parse ways/relations, do the same pattern by calling
/// `elem.tags()` and `address_record_from_tags(..., elem.id())`.
impl<'a> TryFrom<(&osmpbf::Element<'a>, &Country)> for AddressRecord {
    type Error = IncompatibleOsmPbfElement;

    fn try_from(x: (&osmpbf::Element<'a>, &Country)) -> Result<Self, Self::Error> {
        let (element, country) = x;

        match element {
            osmpbf::Element::Node(node) => {
                let id = node.id();
                let tag_iter = node.tags().map(|(k, v)| (k, v));
                address_record_from_tags(tag_iter, *country, id)
                    .map_err(|e| match e {
                        IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(n) => {
                            IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(n)
                        }
                        other => other,
                    })
            }
            osmpbf::Element::Way(way) => {
                let id = way.id();
                let tag_iter = way.tags().map(|(k, v)| (k, v));
                address_record_from_tags(tag_iter, *country, id)
                    .map_err(|e| match e {
                        IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(n) => {
                            // Convert to IncompatibleOsmPbfWay if you want distinct error
                            IncompatibleOsmPbfElement::IncompatibleOsmPbfWay(
                                IncompatibleOsmPbfWay::Incompatible { id }
                            )
                        }
                        other => other,
                    })
            }
            osmpbf::Element::Relation(rel) => {
                let id = rel.id();
                let tag_iter = rel.tags().map(|(k, v)| (k, v));
                address_record_from_tags(tag_iter, *country, id)
                    .map_err(|e| match e {
                        IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(n) => {
                            IncompatibleOsmPbfElement::IncompatibleOsmPbfRelation(
                                IncompatibleOsmPbfRelation::Incompatible { id }
                            )
                        }
                        other => other,
                    })
            }
            osmpbf::Element::DenseNode(dense) => {
                let id = dense.id();
                let tag_iter = dense.tags().map(|(k, v)| (k, v));
                address_record_from_tags(tag_iter, *country, id)
                    .map_err(|e| match e {
                        IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(n) => {
                            IncompatibleOsmPbfElement::IncompatibleOsmPbfDenseNode(
                                IncompatibleOsmPbfDenseNode::Incompatible { id }
                            )
                        }
                        other => other,
                    })
            }
        }
    }
}

// If you also want to define separate impls for &Node, &Way, etc., do so here or remove them.
// This example shows a single approach using address_record_from_tags for all element types.
#[cfg(test)]
mod address_record_tests {
    use super::*;
    use crate::normalize;

    // A quick helper for mocking an iterator over (&str, &str).
    pub fn mock_tag_iter<'a>(pairs: Vec<(&'a str, &'a str)>) -> impl Iterator<Item = (&'a str, &'a str)> {
        pairs.into_iter()
    }

    #[test]
    fn test_collect_tags() {
        let pairs = vec![
            ("addr:city", "Baltimore"),
            ("addr:street", "North Avenue"),
            ("addr:postcode", "21201"),
        ];
        let map = super::collect_tags(mock_tag_iter(pairs));
        assert_eq!(map.get("addr:city"), Some(&"Baltimore"));
        assert_eq!(map.get("addr:street"), Some(&"North Avenue"));
        assert_eq!(map.get("addr:postcode"), Some(&"21201"));
    }

    #[test]
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

    #[test]
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

    #[test]
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

    #[test]
    fn test_address_record_from_tags_all_fields() {
        let tags = vec![
            ("addr:city", "Fairfax"),
            ("addr:street", "Main Street"),
            ("addr:postcode", "22030"),
        ];
        let result = address_record_from_tags(tags.into_iter(), Country::USA, 999);
        assert!(result.is_ok());
        let record = result.unwrap();
        assert_eq!(record.city().as_ref().unwrap().name(), "fairfax");
        assert_eq!(record.street().as_ref().unwrap().name(), "main street");
        assert_eq!(record.postcode().as_ref().unwrap().code(), "22030");
    }

    #[test]
    fn test_address_record_from_tags_no_fields() {
        let tags = vec![];
        let result = address_record_from_tags(tags.into_iter(), Country::USA, 1001);
        assert!(result.is_err());
        match result.err().unwrap() {
            IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(IncompatibleOsmPbfNode::Incompatible { id }) => {
                assert_eq!(id, 1001);
            }
            other => panic!("Expected IncompatibleOsmPbfNode, got: {:?}", other),
        }
    }

    #[test]
    fn test_address_record_from_tags_invalid_street() {
        // Suppose street is whitespace => StreetName returns error
        let tags = vec![
            ("addr:city", "Reston"),
            ("addr:street", "   "),
        ];
        let result = address_record_from_tags(tags.into_iter(), Country::USA, 123);
        assert!(result.is_err());
        match result.err().unwrap() {
            IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
                IncompatibleOsmPbfNode::StreetNameConstructionError(
                    StreetNameConstructionError::InvalidName { attempted_name }
                )
            ) => {
                assert_eq!(attempted_name.trim(), "");
            }
            other => panic!("Expected StreetNameConstructionError, got: {:?}", other),
        }
    }
}
