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
            pub fn $fn_name(elem: &$elem_ty) -> Result<Option<HouseNumberRange>, IncompatibleOsmPbfElement> {
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

#[cfg(test)]
mod house_number_extractors_tests {
    use super::*;

    // --------------------------------------------------------------
    // 1) Minimal mocks for Node, Way, Relation, DenseNode
    // --------------------------------------------------------------
    // They provide:
    //   fn id(&self) -> i64
    //   fn tags(&self) -> impl Iterator<Item = (&str, &str)>

    #[derive(Clone)]
    struct MockNode {
        id: i64,
        tags_map: HashMap<String, String>,
    }
    impl MockNode {
        fn new(id: i64) -> Self {
            Self { id, tags_map: HashMap::new() }
        }
        fn with_tag(mut self, k: &str, v: &str) -> Self {
            self.tags_map.insert(k.to_string(), v.to_string());
            self
        }
    }
    impl MockNode {
        fn id(&self) -> i64 { self.id }
        fn tags(&self) -> impl Iterator<Item = (&str, &str)> {
            self.tags_map.iter().map(|(k,v)| (k.as_str(), v.as_str()))
        }
    }

    #[derive(Clone)]
    struct MockWay {
        id: i64,
        tags_map: HashMap<String, String>,
    }
    impl MockWay {
        fn new(id: i64) -> Self {
            Self { id, tags_map: HashMap::new() }
        }
        fn with_tag(mut self, k: &str, v: &str) -> Self {
            self.tags_map.insert(k.to_string(), v.to_string());
            self
        }
    }
    impl MockWay {
        fn id(&self) -> i64 { self.id }
        fn tags(&self) -> impl Iterator<Item = (&str, &str)> {
            self.tags_map.iter().map(|(k,v)| (k.as_str(), v.as_str()))
        }
    }

    #[derive(Clone)]
    struct MockRelation {
        id: i64,
        tags_map: HashMap<String, String>,
    }
    impl MockRelation {
        fn new(id: i64) -> Self {
            Self { id, tags_map: HashMap::new() }
        }
        fn with_tag(mut self, k: &str, v: &str) -> Self {
            self.tags_map.insert(k.to_string(), v.to_string());
            self
        }
    }
    impl MockRelation {
        fn id(&self) -> i64 { self.id }
        fn tags(&self) -> impl Iterator<Item = (&str, &str)> {
            self.tags_map.iter().map(|(k,v)| (k.as_str(), v.as_str()))
        }
    }

    #[derive(Clone)]
    struct MockDenseNode {
        id: i64,
        tags_map: HashMap<String, String>,
    }
    impl MockDenseNode {
        fn new(id: i64) -> Self {
            Self { id, tags_map: HashMap::new() }
        }
        fn with_tag(mut self, k: &str, v: &str) -> Self {
            self.tags_map.insert(k.to_string(), v.to_string());
            self
        }
    }
    impl MockDenseNode {
        fn id(&self) -> i64 { self.id }
        fn tags(&self) -> impl Iterator<Item = (&str, &str)> {
            self.tags_map.iter().map(|(k,v)| (k.as_str(), v.as_str()))
        }
    }

    // --------------------------------------------------------------
    // 2) Minimal "osmpbf::Element" wrapper that can hold our mocks
    // --------------------------------------------------------------
    // We'll define a local enum that exactly mirrors "osmpbf::Element" variants,
    // but storing references to the mocks. Then we call "extract_house_number_range_from_element"
    // on it. The macro expects "osmpbf::Element::Node(n)", etc.
    // We'll define a local or test-only version of "osmpbf::Element".
    mod test_osmpbf {
        use super::{MockNode, MockWay, MockRelation, MockDenseNode};

        pub enum Element<'a> {
            Node(&'a MockNode),
            Way(&'a MockWay),
            Relation(&'a MockRelation),
            DenseNode(&'a MockDenseNode),
        }
    }
    use test_osmpbf::Element; // local test version

    // We'll re-import the macro's generated "extract_house_number_range_from_element" but we'd
    // need to rename it or re-implement. Instead, we can rename ours:
    use crate::extract_house_number_range_from_element as real_unified;
    use crate::extract_house_number_range_from_node as real_node;
    use crate::extract_house_number_range_from_way as real_way;
    use crate::extract_house_number_range_from_relation as real_rel;
    use crate::extract_house_number_range_from_dense_node as real_dense;

    // We'll define local fns that match the signature of the generated ones but take our test Element.
    // Actually, the macro is "match element { osmpbf::Element::Node(e) => ... }".
    // We replicate that logic for our "test_osmpbf::Element".
    pub fn test_extract_house_number_range_from_element<'a>(
        element: &'a Element<'a>
    ) -> Result<Option<HouseNumberRange>, IncompatibleOsmPbfElement> {
        match element {
            Element::Node(n) => real_node(n),
            Element::Way(w) => real_way(w),
            Element::Relation(r) => real_rel(r),
            Element::DenseNode(d) => real_dense(d),
        }
    }

    // --------------------------------------------------------------
    // 3) Now the actual tests
    // --------------------------------------------------------------
    #[traced_test]
    fn test_node_no_housenumber() {
        let node = MockNode::new(101)
            .with_tag("addr:city", "Baltimore")
            .with_tag("some_other_tag", "anything");
        let res = real_node(&node);
        assert!(res.is_ok());
        let opt = res.unwrap();
        assert!(opt.is_none(), "No housenumber => None");
        assert!(logs_contain("no house‐number range for Node with id=101"));
    }

    #[traced_test]
    fn test_node_valid_housenumber() {
        // e.g. "100-110"
        let node = MockNode::new(202)
            .with_tag("addr:housenumber", "100-110")
            .with_tag("addr:city", "Baltimore");
        let res = real_node(&node);
        assert!(res.is_ok());
        let opt = res.unwrap();
        assert!(opt.is_some());
        let rng = opt.unwrap();
        assert_eq!(rng.start(), 100);
        assert_eq!(rng.end(), 110);
        assert!(logs_contain("found house‐number range for Node with id=202"));
    }

    #[traced_test]
    fn test_node_invalid_housenumber() {
        let node = MockNode::new(303)
            .with_tag("addr:housenumber", "ABC not valid");
        let res = real_node(&node);
        assert!(res.is_err());
        match res.err().unwrap() {
            IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(IncompatibleOsmPbfNode::Incompatible { id }) => {
                assert_eq!(id, 303);
            }
            other => panic!("Expected IncompatibleOsmPbfNode::Incompatible, got {:?}", other),
        }
        assert!(logs_contain("error extracting range for Node with id=303:"));
    }

    #[traced_test]
    fn test_way_no_housenumber() {
        let way = MockWay::new(404).with_tag("addr:city", "Rockville");
        let res = real_way(&way);
        assert!(res.is_ok());
        let opt = res.unwrap();
        assert!(opt.is_none());
    }

    #[traced_test]
    fn test_way_valid_housenumber() {
        let way = MockWay::new(505).with_tag("addr:housenumber", "123");
        let res = real_way(&way);
        assert!(res.is_ok());
        let rng_opt = res.unwrap();
        assert!(rng_opt.is_some());
        assert_eq!(rng_opt.unwrap().start(), 123);
    }

    #[traced_test]
    fn test_way_invalid_housenumber() {
        let way = MockWay::new(606).with_tag("addr:housenumber", "-not a number-");
        let res = real_way(&way);
        assert!(res.is_err());
        match res.err().unwrap() {
            // The macro transforms Node error => Way error => IncompatibleOsmPbfWay::Incompatible{id=606}
            IncompatibleOsmPbfElement::IncompatibleOsmPbfWay(IncompatibleOsmPbfWay::Incompatible { id }) => {
                assert_eq!(id, 606);
            }
            other => panic!("Expected IncompatibleOsmPbfWay, got {:?}", other),
        }
    }

    #[traced_test]
    fn test_relation_ok_none() {
        let rel = MockRelation::new(707);
        let res = real_rel(&rel);
        assert!(res.is_ok());
        assert!(res.unwrap().is_none());
    }

    #[traced_test]
    fn test_relation_ok_some() {
        let rel = MockRelation::new(808)
            .with_tag("addr:housenumber", "200-300");
        let res = real_rel(&rel);
        let rng_opt = res.unwrap();
        let rng = rng_opt.unwrap();
        assert_eq!((rng.start(), rng.end()), (200, 300));
    }

    #[traced_test]
    fn test_relation_invalid_housenumber() {
        let rel = MockRelation::new(909)
            .with_tag("addr:housenumber", "?? unparseable");
        let res = real_rel(&rel);
        assert!(res.is_err());
        match res.err().unwrap() {
            IncompatibleOsmPbfElement::IncompatibleOsmPbfRelation(IncompatibleOsmPbfRelation::Incompatible { id }) => {
                assert_eq!(id, 909);
            }
            other => panic!("Expected IncompatibleOsmPbfRelation, got {:?}", other),
        }
    }

    #[traced_test]
    fn test_dense_node_ok_none() {
        let dn = MockDenseNode::new(1001);
        let res = real_dense(&dn);
        assert!(res.is_ok());
        assert!(res.unwrap().is_none());
    }

    #[traced_test]
    fn test_dense_node_ok_some() {
        let dn = MockDenseNode::new(2002)
            .with_tag("addr:housenumber", "999");
        let res = real_dense(&dn);
        let rng_opt = res.unwrap();
        assert!(rng_opt.is_some());
        assert_eq!(rng_opt.unwrap().start(), 999);
    }

    #[traced_test]
    fn test_dense_node_invalid() {
        let dn = MockDenseNode::new(3003)
            .with_tag("addr:housenumber", "N/A");
        let res = real_dense(&dn);
        assert!(res.is_err());
        match res.err().unwrap() {
            IncompatibleOsmPbfElement::IncompatibleOsmPbfDenseNode(IncompatibleOsmPbfDenseNode::Incompatible { id }) => {
                assert_eq!(id, 3003);
            }
            other => panic!("Expected IncompatibleOsmPbfDenseNode, got {:?}", other),
        }
    }

    // --------------------------------------------------------------
    // 4) Unified function test => test_extract_house_number_range_from_element(...)
    // --------------------------------------------------------------

    #[traced_test]
    fn test_unified_node_ok() {
        let node = MockNode::new(111).with_tag("addr:housenumber", "45-50");
        let elem = Element::Node(&node);
        let res = test_extract_house_number_range_from_element(&elem);
        assert!(res.is_ok());
        let rng = res.unwrap().unwrap();
        assert_eq!((rng.start(), rng.end()), (45, 50));
    }

    #[traced_test]
    fn test_unified_way_err() {
        let way = MockWay::new(222).with_tag("addr:housenumber", "invalid##");
        let elem = Element::Way(&way);
        let res = test_extract_house_number_range_from_element(&elem);
        assert!(res.is_err());
        match res.err().unwrap() {
            IncompatibleOsmPbfElement::IncompatibleOsmPbfWay(IncompatibleOsmPbfWay::Incompatible { id }) => {
                assert_eq!(id, 222);
            }
            other => panic!("Expected IncompatibleOsmPbfWay, got {:?}", other),
        }
    }

    #[traced_test]
    fn test_unified_relation_none() {
        let rel = MockRelation::new(333)
            .with_tag("some_tag", "no housenumber");
        let elem = Element::Relation(&rel);
        let res = test_extract_house_number_range_from_element(&elem);
        assert!(res.is_ok());
        assert!(res.unwrap().is_none());
    }

    #[traced_test]
    fn test_unified_dense_node_ok() {
        let dn = MockDenseNode::new(444)
            .with_tag("addr:housenumber", "1234");
        let elem = Element::DenseNode(&dn);
        let res = test_extract_house_number_range_from_element(&elem);
        assert!(res.is_ok());
        let rng = res.unwrap().unwrap();
        assert_eq!(rng.start(), 1234);
        assert_eq!(rng.end(), 1234);
    }
}
