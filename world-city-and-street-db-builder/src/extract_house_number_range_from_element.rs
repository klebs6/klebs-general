// ---------------- [ File: src/extract_house_number_range_from_element.rs ]
crate::ix!();

/// A macro that generates specialized extraction functions for each OSM element variant
/// (Node, Way, Relation, DenseNode) plus a unifying
/// `extract_house_number_range_from_element(...)` top‐level function.
///
/// For each variant, you supply:
/// 1) A name for the generated extraction function.
/// 2) The concrete element type (e.g. `osmpbf::Node`).
/// 3) The variant name (e.g. `Node`, `Way`, `Relation`, `DenseNode`).
/// 4) A closure that transforms a node‐specific error into the variant’s corresponding error.
///
/// Usage example:
/// ```rust
/// generate_house_number_extractors!(
///     extract_house_number_range_from_node,
///     osmpbf::Node,
///     Node,
///     |id, node_err| IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(node_err);
///
///     extract_house_number_range_from_way,
///     osmpbf::Way,
///     Way,
///     |id, _node_err| IncompatibleOsmPbfElement::IncompatibleOsmPbfWay(
///         IncompatibleOsmPbfWay::Incompatible { id }
///     );
///
///     // etc. for Relation, DenseNode...
/// );
/// ```
#[macro_export]
macro_rules! generate_house_number_extractors {
    (
        $(
            $fn_name:ident,
            $elem_ty:ty,
            $elem_variant:ident,
            $error_transform:expr
        );+ $(;)?
    ) => {
        $(
            /// Generated function to extract a [`HouseNumberRange`] (if any) from an `$elem_variant`.
            /// Converts node‐specific errors using the supplied closure.
            fn $fn_name(elem: &$elem_ty) -> Result<Option<HouseNumberRange>, IncompatibleOsmPbfElement> {
                let id = elem.id();
                trace!(
                    "{}: extracting house number range for {} with id={}",
                    stringify!($fn_name),
                    stringify!($elem_variant),
                    id
                );

                let tag_iter = elem.tags().map(|(k, v)| (k, v));
                let result = extract_house_number_range_from_tags(tag_iter, id)
                    .map_err(|err| match err {
                        IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(node_err) => {
                            // Apply the variant‐specific transformation closure
                            let new_err = $error_transform(id, node_err);
                            error!(
                                "{}: converting node‐specific error into {} error for id={}",
                                stringify!($fn_name),
                                stringify!($elem_variant),
                                id
                            );
                            new_err
                        }
                        other => other,
                    });

                match &result {
                    Ok(Some(_)) => {
                        debug!(
                            "{}: found house‐number range for {} with id={}",
                            stringify!($fn_name),
                            stringify!($elem_variant),
                            id
                        );
                    }
                    Ok(None) => {
                        debug!(
                            "{}: no house‐number range for {} with id={}",
                            stringify!($fn_name),
                            stringify!($elem_variant),
                            id
                        );
                    }
                    Err(e) => {
                        error!(
                            "{}: error extracting range for {} with id={}: {:?}",
                            stringify!($fn_name),
                            stringify!($elem_variant),
                            id,
                            e
                        );
                    }
                }

                result
            }
        )+

        /// A unified function that dispatches to the appropriate
        /// extractor depending on the element variant.
        pub fn extract_house_number_range_from_element<'a>(
            element: &'a osmpbf::Element<'a>
        ) -> Result<Option<HouseNumberRange>, IncompatibleOsmPbfElement> {
            match element {
                $(
                    osmpbf::Element::$elem_variant(e) => $fn_name(e),
                )+
            }
        }
    }
}

/// Example usage of the macro. Here we define four specialized
/// extraction functions plus the unified `extract_house_number_range_from_element`.
generate_house_number_extractors!(
    extract_house_number_range_from_node,
    osmpbf::Node,
    Node,
    // Node has no special error mapping; we pass the node error through unchanged.
    |_, node_err| IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(node_err);

    extract_house_number_range_from_way,
    osmpbf::Way,
    Way,
    // Convert node‐specific error => way‐specific error
    |id, _| IncompatibleOsmPbfElement::IncompatibleOsmPbfWay(
        IncompatibleOsmPbfWay::Incompatible { id }
    );

    extract_house_number_range_from_relation,
    osmpbf::Relation,
    Relation,
    // Convert node‐specific error => relation‐specific error
    |id, _| IncompatibleOsmPbfElement::IncompatibleOsmPbfRelation(
        IncompatibleOsmPbfRelation::Incompatible { id }
    );

    extract_house_number_range_from_dense_node,
    osmpbf::DenseNode,
    DenseNode,
    // Convert node‐specific error => dense‐node‐specific error
    |id, _| IncompatibleOsmPbfElement::IncompatibleOsmPbfDenseNode(
        IncompatibleOsmPbfDenseNode::Incompatible { id }
    );
);
