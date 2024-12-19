crate::ix!();

/// A simple structure to hold address info extracted from OSM
/// Not all OSM ways/nodes have addresses, we only store those that do.
#[derive(Clone,Default,Builder,Getters,Setters,Debug)]
#[getset(get="pub",set="pub")]
#[builder(default,setter(into))]
pub struct AddressRecord {
    city:     Option<CityName>,
    street:   Option<StreetName>,
    postcode: Option<PostalCode>,
}

/// Helper to create an AddressRecord easily
#[macro_export] macro_rules! address_record {
    ($city:ident, $street:ident, $postcode:ident) => {
        AddressRecord::new($city.clone(),$street.clone(),$postcode.clone())
    }
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

/// Collect tags into a HashMap for easier access
fn collect_tags<'a>(tags: impl Iterator<Item=(&'a str, &'a str)>) -> HashMap<&'a str, &'a str> {
    tags.collect()
}

impl<'a> TryFrom<osmpbf::Element<'a>> for AddressRecord {

    type Error = IncompatibleOsmPbfElement;

    fn try_from(element: osmpbf::Element<'a>) 
        -> Result<Self,Self::Error> 
    {
        // dense nodes can be handled if needed
        const PARSE_DENSE_NODES: bool = true;

        // relations can be handled if needed
        const PARSE_RELATIONS:   bool = true;

        match element {
            Element::Node(node)         => Ok(AddressRecord::try_from(&node)?),
            Element::Way(way)           => Ok(AddressRecord::try_from(&way)?),
            Element::Relation(relation) => match PARSE_RELATIONS {
                true  => Ok(AddressRecord::try_from(&relation)?),
                false => Err(IncompatibleOsmPbfElement::MaybeTodoUnhandledOsmPbfRelationElement),
            },
            Element::DenseNode(node)    => match PARSE_DENSE_NODES {
                true  => Ok(AddressRecord::try_from(&node)?),
                false => Err(IncompatibleOsmPbfElement::MaybeTodoUnhandledOsmPbfDenseNode),
            }
        }
    }
}

macro_rules! impl_from_osmpbf_element {
    ($elem:ty,$err:ty) => {

        impl<'a> TryFrom<&$elem> for AddressRecord {

            type Error = $err;

            fn try_from(x: &$elem) 
                -> Result<Self,Self::Error> 
            {
                let tags     = collect_tags(x.tags());
                let city     = tags.get("addr:city");
                let street   = tags.get("addr:street");
                let postcode = tags.get("addr:postcode");

                if city.is_some() || street.is_some() || postcode.is_some() {

                    let city = match city {
                        Some(city) => Some(CityName::new(&city)?),
                        None       => None,
                    };

                    let street = match street {
                        Some(street) => Some(StreetName::new(&street)?),
                        None       => None,
                    };

                    let postcode = match postcode {
                        Some(postcode) => Some(PostalCode::new(Country::USA, &postcode)?),
                        None       => None,
                    };

                    Ok(AddressRecord { 
                        city, 
                        street, 
                        postcode
                    })

                } else {
                    let id = x.id();
                    Err(<$err>::Incompatible {
                        id
                    })
                }
            }
        }
    }
}

impl_from_osmpbf_element!{osmpbf::Way<'a>,       IncompatibleOsmPbfWay}
impl_from_osmpbf_element!{osmpbf::Node<'a>,      IncompatibleOsmPbfNode}
impl_from_osmpbf_element!{osmpbf::Relation<'a>,  IncompatibleOsmPbfRelation}
impl_from_osmpbf_element!{osmpbf::DenseNode<'a>, IncompatibleOsmPbfDenseNode}

/// Tests for AddressRecord conversions from OSM Elements
#[cfg(test)]
mod address_record_tests {
    use super::*;

    pub fn mock_tag_iter<'a>(tags: Vec<(&'a str,&'a str)>) -> impl Iterator<Item=(&'a str,&'a str)> {
        tags.into_iter()
    }

    #[test]
    fn test_collect_tags() {
        let tags = vec![
            ("addr:city", "Baltimore"),
            ("addr:street", "North Avenue"),
            ("addr:postcode", "21201"),
        ];
        let map = collect_tags(mock_tag_iter(tags));
        assert_eq!(map.get("addr:city").unwrap(), "Baltimore");
    }

    #[test]
    fn address_record_is_empty() {
        let empty = AddressRecordBuilder::default().build().unwrap();
        assert!(empty.is_empty());

        let non_empty = AddressRecordBuilder::default()
            .city(Some(CityName::new("Baltimore").unwrap()))
            .build()
            .unwrap();
        assert!(!non_empty.is_empty());
    }
}
