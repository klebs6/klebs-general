crate::ix!();

/// Retrieves the element ID (Node, Way, Relation, or DenseNode), or returns "?" if unknown.
/// Primarily used for logging.
pub fn get_element_id(element: &osmpbf::Element) -> String {
    match element {
        osmpbf::Element::Node(n) => format!("{}", n.id()),
        osmpbf::Element::Way(w) => format!("{}", w.id()),
        osmpbf::Element::Relation(r) => format!("{}", r.id()),
        osmpbf::Element::DenseNode(dn) => format!("{}", dn.id()),
    }
}
