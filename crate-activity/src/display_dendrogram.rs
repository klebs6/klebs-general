crate::ix!();

pub fn display_dendrogram(dendrogram: &Dendrogram) {
    println!("----------------[hierarchical-clustering-dendrogram]----------------");
    fn print_node(node: &Dendrogram, indent: usize) {
        let prefix = " ".repeat(indent);
        match node {
            Dendrogram::Leaf { crate_name } => {
                println!("{}- {}", prefix, crate_name);
            }
            Dendrogram::Internal { left, right, distance } => {
                println!("{}(distance: {:.2})", prefix, distance);
                print_node(left, indent + 2);
                print_node(right, indent + 2);
            }
        }
    }
    print_node(dendrogram, 0);
}
