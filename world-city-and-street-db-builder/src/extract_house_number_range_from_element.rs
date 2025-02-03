// ---------------- [ File: src/extract_house_number_range_from_element.rs ]
crate::ix!();

/// Extracts a house‐number range (if any) from one OSM element, by matching on its variant
/// (Node, Way, Relation, DenseNode) and then using the same tag-based logic as
/// `extract_house_number_range_from_tags(...)`. This avoids calling `element.tags()` directly,
/// since `Element` is an enum with no `.tags()` method.
pub fn extract_house_number_range_from_element<'a>(
    element: &'a osmpbf::Element<'a>
) -> Result<Option<HouseNumberRange>, IncompatibleOsmPbfElement> 
{
    match element {
        Element::Node(node) => {
            let id = node.id();
            let tag_iter = node.tags().map(|(k,v)| (k,v));
            extract_house_number_range_from_tags(tag_iter, id)
        }
        Element::Way(way) => {
            let id = way.id();
            let tag_iter = way.tags().map(|(k,v)| (k,v));
            extract_house_number_range_from_tags(tag_iter, id)
                .map_err(|err| match err {
                    IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(_) => {
                        IncompatibleOsmPbfElement::IncompatibleOsmPbfWay(
                            IncompatibleOsmPbfWay::Incompatible { id }
                        )
                    }
                    other => other,
                })
        }
        Element::Relation(rel) => {
            let id = rel.id();
            let tag_iter = rel.tags().map(|(k,v)| (k,v));
            extract_house_number_range_from_tags(tag_iter, id)
                .map_err(|err| match err {
                    IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(_) => {
                        IncompatibleOsmPbfElement::IncompatibleOsmPbfRelation(
                            IncompatibleOsmPbfRelation::Incompatible { id }
                        )
                    }
                    other => other,
                })
        }
        Element::DenseNode(dn) => {
            let id = dn.id();
            let tag_iter = dn.tags().map(|(k,v)| (k,v));
            extract_house_number_range_from_tags(tag_iter, id)
                .map_err(|err| match err {
                    IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(_) => {
                        IncompatibleOsmPbfElement::IncompatibleOsmPbfDenseNode(
                            IncompatibleOsmPbfDenseNode::Incompatible { id }
                        )
                    }
                    other => other,
                })
        }
    }
}
