// ---------------- [ File: src/address_record_from_element_and_country.rs ]
crate::ix!();

/// Implements conversion from an OSM PBF element and a `Country` into an `AddressRecord`.
/// Dispatches to one of the above converter functions based on the element variant.
impl<'a> TryFrom<(&osmpbf::Element<'a>, &Country)> for AddressRecord {
    type Error = IncompatibleOsmPbfElement;

    fn try_from((element, country): (&osmpbf::Element<'a>, &Country)) -> Result<Self, Self::Error> {
        match element {
            osmpbf::Element::Node(node)       => convert_osm_node_to_address_record(node, *country),
            osmpbf::Element::Way(way)         => convert_osm_way_to_address_record(way, *country),
            osmpbf::Element::Relation(rel)    => convert_osm_relation_to_address_record(rel, *country),
            osmpbf::Element::DenseNode(dense) => convert_osm_dense_node_to_address_record(dense, *country),
        }
    }
}

/// A macro to generate multiple OSM-element-to-AddressRecord converter functions
/// with minimal boilerplate. Each generated function:
///   1. Extracts an identifier (`id`).
///   2. Logs the element processing progress.
///   3. Invokes a shared `address_record_from_tags(...)`.
///   4. Transforms `IncompatibleOsmPbfElement::IncompatibleOsmPbfNode` into the
///      appropriate variant if needed.
///
/// The macro accepts a list of converters to generate. Each converter
/// is declared in the form:
///
/// ```text
///   fn_name, ElementType, "DescriptiveLabel", error_mapping
/// ```
///
/// - `fn_name` is the desired function identifier.
/// - `ElementType` is any type offering:
///      - `.id()` -> i64
///      - `.tags()` -> Iterator<(K, V)>
/// - `"DescriptiveLabel"` is a textual label included in log messages.
/// - `error_mapping` is a closure of the form:
///      `|id, node_err| { <transform> }`
///   which transforms `IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(...)` into the desired variant.
///   If no transformation is needed (i.e. for Node), simply pass an identity closure.
///
/// Example usage for Node (identity):
/// ```text
/// |_, node_err| IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(node_err)
/// ```
///
/// Example usage for Way (transform to IncompatibleOsmPbfWay):
/// ```text
/// |id, _| IncompatibleOsmPbfElement::IncompatibleOsmPbfWay(
///     IncompatibleOsmPbfWay::Incompatible { id }
/// )
/// ```
macro_rules! generate_osm_address_record_converters {
    ( $($fn_name:ident, $elem_ty:ty, $label:expr, $err_mapping:expr);+ $(;)?) => {
        $(
            #[doc = concat!("Converts an OSM ", $label, " element into an AddressRecord.")]
            ///
            /// # Arguments
            ///
            /// * `elem`    - Reference to the OSM element.
            /// * `country` - The associated `Country`.
            ///
            /// # Returns
            ///
            /// * `Ok(AddressRecord)` if successful.
            /// * `Err(IncompatibleOsmPbfElement)` if conversion fails.
            fn $fn_name(
                elem: &$elem_ty,
                country: Country
            ) -> Result<AddressRecord, IncompatibleOsmPbfElement> {
                let id = elem.id();
                trace!("{}: converting {} with id {}", stringify!($fn_name), $label, id);
                let tag_iter = elem.tags().map(|(k, v)| (k, v));

                match try_build_address_record_from_tags(tag_iter, country, id) {
                    Ok(record) => {
                        trace!("Successfully converted {} (id={}) into AddressRecord", $label, id);
                        Ok(record)
                    }
                    Err(e) => {
                        error!("Failed to convert {} (id={}): {:?}", $label, id, e);
                        Err(match e {
                            IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(node_err) => {
                                $err_mapping(id, node_err)
                            }
                            other => other,
                        })
                    }
                }
            }
        )+
    };
}

// Generate all the converters in a single macro invocation.
generate_osm_address_record_converters!(
    convert_osm_node_to_address_record,
    osmpbf::Node,
    "Node",
    // Node just passes the node-specific error along (identity).
    |_, node_err| IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(node_err);

    convert_osm_way_to_address_record,
    osmpbf::Way,
    "Way",
    // Convert IncompatibleOsmPbfNode(...) to IncompatibleOsmPbfWay(...).
    |id, _| IncompatibleOsmPbfElement::IncompatibleOsmPbfWay(
        IncompatibleOsmPbfWay::Incompatible { id }
    );

    convert_osm_relation_to_address_record,
    osmpbf::Relation,
    "Relation",
    // Convert IncompatibleOsmPbfNode(...) to IncompatibleOsmPbfRelation(...).
    |id, _| IncompatibleOsmPbfElement::IncompatibleOsmPbfRelation(
        IncompatibleOsmPbfRelation::Incompatible { id }
    );

    convert_osm_dense_node_to_address_record,
    osmpbf::DenseNode,
    "DenseNode",
    // Convert IncompatibleOsmPbfNode(...) to IncompatibleOsmPbfDenseNode(...).
    |id, _| IncompatibleOsmPbfElement::IncompatibleOsmPbfDenseNode(
        IncompatibleOsmPbfDenseNode::Incompatible { id }
    );
);
