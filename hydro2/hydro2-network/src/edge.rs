// ---------------- [ File: src/edge.rs ]
crate::ix!();

/// Represents an edge in the DAG, connecting one node's output
/// to another node's input.
#[derive(Builder,MutGetters,Setters,Getters,Debug,Clone)]
#[getset(get="pub",set = "pub", get_mut = "pub")]
#[builder(setter(into))]
pub struct NetworkEdge {
    /// The source node index.
    source_index: usize,
    /// The output buffer index in the source node.
    source_output_idx: usize,
    /// The destination node index.
    dest_index: usize,
    /// The input buffer index in the destination node.
    dest_input_idx: usize,
}

#[macro_export]
macro_rules! edge {

    ($src:literal : $src_out:literal -> $dest:literal : $dest_in:literal) => {{
        assert!($src_out < 4, "src output port must be usize less than 4");
        assert!($dest_in < 4,  "dst input port must be usize less than 4");
        NetworkEdgeBuilder::default()
            .source_index($src as usize)
            .source_output_idx($src_out as usize)
            .dest_index($dest as usize)
            .dest_input_idx($dest_in as usize)
            .build()
            .unwrap()
    }};

    (($src:expr, $src_out:expr) -> ($dest:expr, $dest_in:expr)) => {{
        assert!($src_out < 4, "src output port must be usize less than 4");
        assert!($dest_in < 4,  "dst input port must be usize less than 4");
        NetworkEdgeBuilder::default()
            .source_index(($src) as usize)
            .source_output_idx(($src_out) as usize)
            .dest_index(($dest) as usize)
            .dest_input_idx(($dest_in) as usize)
            .build()
            .unwrap()
    }};
}

#[cfg(test)]
mod edge_macro_tests {
    use super::*;

    #[test]
    fn test_edge_macro_literal() {
        // edge!(0:0 -> 1:0)
        let e = edge!(0:0 -> 1:0);
        assert_eq!(*e.source_index(), 0);
        assert_eq!(*e.source_output_idx(), 0);
        assert_eq!(*e.dest_index(), 1);
        assert_eq!(*e.dest_input_idx(), 0);
    }

    #[test]
    fn test_edge_macro_tuple() {
        // edge!((5,1)->(10,2))
        let e = edge!((5,1) -> (10,2));
        assert_eq!(*e.source_index(), 5);
        assert_eq!(*e.source_output_idx(), 1);
        assert_eq!(*e.dest_index(), 10);
        assert_eq!(*e.dest_input_idx(), 2);
    }

    #[test]
    #[should_panic(expected = "src output port must be usize less than 4")]
    fn test_edge_macro_panic_src_too_large() {
        // we pass an out port >=4 => triggers the assert!
        let _ = edge!(0:5 -> 1:0);
    }

    #[test]
    #[should_panic(expected = "dst input port must be usize less than 4")]
    fn test_edge_macro_panic_dst_too_large() {
        let _ = edge!((0,1)->(1,4));
    }
}
