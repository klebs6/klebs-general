crate::ix!();

use std::collections::HashMap;

/// Represents a hierarchical clustering dendrogram node.
#[derive(Debug)]
pub enum Dendrogram {
    /// A leaf node representing a single crate.
    Leaf {
        /// Name of the crate represented by this leaf.
        crate_name: String,
    },
    /// An internal node representing a merge of two clusters.
    Internal {
        /// Left child cluster.
        left: Box<Dendrogram>,
        /// Right child cluster.
        right: Box<Dendrogram>,
        /// The distance at which the two child clusters were merged.
        distance: f64,
    },
}

/// Errors that can occur during hierarchical clustering.
#[derive(Debug)]
pub enum HierarchicalClusteringError {
    /// No crates provided for clustering.
    NoCrates,
    /// Inconsistent or incomplete data caused shape issues.
    DataShapeError,
    /// Insufficient correlation data.
    IncompleteCorrelationData,
    /// Other I/O or data-related issues.
    IoError(std::io::Error),
}

/// Perform hierarchical clustering using single-linkage based on crate correlations.
///
/// # Arguments
///
/// * `correlations` - A vector of (crate_a, crate_b, correlation) tuples from `compute_pairwise_correlations`.
///
/// # Returns
///
/// A `Dendrogram` representing the hierarchical clustering structure.
pub fn perform_hierarchical_clustering(
    correlations: &[(String, String, f64)]
) -> Result<Dendrogram, HierarchicalClusteringError> {

    // Extract unique crate names from the correlation tuples.
    let mut crate_set = HashMap::new();
    for (a, b, _) in correlations {
        crate_set.entry(a.clone()).or_insert(true);
        crate_set.entry(b.clone()).or_insert(true);
    }

    let mut crate_names: Vec<String> = crate_set.keys().cloned().collect();
    if crate_names.is_empty() {
        // No crates at all means we cannot cluster.
        return Err(HierarchicalClusteringError::NoCrates);
    }
    crate_names.sort(); // Ensure stable ordering of crates.

    // Map crate names to indices
    let index_map: HashMap<String, usize> = crate_names
        .iter()
        .enumerate()
        .map(|(i, name)| (name.clone(), i))
        .collect();

    let n = crate_names.len();

    // If there's only one crate, hierarchical clustering is trivial.
    // Just return a single leaf node.
    if n == 1 {
        // Single crate scenario: just return a leaf, ignoring correlations.
        return Ok(Dendrogram::Leaf {
            crate_name: crate_names[0].clone(),
        });
    }

    // Initialize a distance matrix.
    // Default distance = 1.0 for missing pairs.
    // Distance = 1 - correlation.
    let mut distance_matrix = vec![1.0; n * n];

    // Distance to self is zero.
    for i in 0..n {
        distance_matrix[i * n + i] = 0.0;
    }

    // Fill in distance matrix from correlations
    // If not present, distance remains 1.0 (implying no correlation).
    for (a, b, corr) in correlations {
        if let (Some(&i), Some(&j)) = (index_map.get(a), index_map.get(b)) {
            let dist = 1.0 - corr;
            let idx1 = i * n + j;
            let idx2 = j * n + i;
            if idx1 < distance_matrix.len() && idx2 < distance_matrix.len() {
                distance_matrix[idx1] = dist;
                distance_matrix[idx2] = dist;
            } else {
                return Err(HierarchicalClusteringError::DataShapeError);
            }
        }
    }

    #[derive(Clone)]
    struct Cluster {
        indices: Vec<usize>,
    }

    // Each crate starts as its own cluster
    let mut clusters: Vec<Cluster> = (0..n).map(|i| Cluster { indices: vec![i] }).collect();
    let mut active = vec![true; n]; // which cluster IDs are active

    // Each leaf node initially points to a leaf dendrogram
    let mut dendrogram_nodes: Vec<Option<Dendrogram>> = crate_names
        .iter()
        .map(|name| Some(Dendrogram::Leaf { crate_name: name.clone() }))
        .collect();

    // Perform (n-1) merges
    for _step in 0..(n-1) {
        // Find the two closest distinct active clusters
        let mut min_dist = f64::MAX;
        let mut closest_pair = (0, 0);

        for i in 0..n {
            if !active[i] { continue; }
            for j in (i+1)..n {
                if !active[j] { continue; }

                let dist = cluster_distance(&clusters[i].indices, &clusters[j].indices, &distance_matrix, n)?;
                if dist < min_dist {
                    min_dist = dist;
                    closest_pair = (i, j);
                }
            }
        }

        let (c1, c2) = closest_pair;
        let mut new_indices = Vec::new();
        new_indices.extend_from_slice(&clusters[c1].indices);
        new_indices.extend_from_slice(&clusters[c2].indices);

        // Create a new dendrogram node from merging c1 and c2
        let left_node = dendrogram_nodes[c1].take().ok_or(HierarchicalClusteringError::DataShapeError)?;
        let right_node = dendrogram_nodes[c2].take().ok_or(HierarchicalClusteringError::DataShapeError)?;

        let new_node = Dendrogram::Internal {
            left: Box::new(left_node),
            right: Box::new(right_node),
            distance: min_dist,
        };

        // Merge c2 into c1 and deactivate c2
        clusters[c1] = Cluster { indices: new_indices };
        dendrogram_nodes[c1] = Some(new_node);
        active[c2] = false;
    }

    // The final active cluster is our root
    let final_node = dendrogram_nodes
        .into_iter()
        .enumerate()
        .filter(|(i, _)| active[*i])
        .map(|(_, node)| node)
        .find(|n| n.is_some())
        .ok_or(HierarchicalClusteringError::DataShapeError)?;

    final_node.ok_or(HierarchicalClusteringError::DataShapeError)
}

/// Compute single-linkage distance between two clusters.
fn cluster_distance(
    c1: &impl AsRef<[usize]>,
    c2: &impl AsRef<[usize]>,
    distance_matrix: &[f64],
    n: usize,
) -> Result<f64, HierarchicalClusteringError> {
    let mut min_dist = f64::MAX;
    for &i in c1.as_ref() {
        for &j in c2.as_ref() {
            let idx = i*n + j;
            if idx >= distance_matrix.len() {
                return Err(HierarchicalClusteringError::DataShapeError);
            }
            let d = distance_matrix[idx];
            if d < min_dist {
                min_dist = d;
            }
        }
    }
    Ok(min_dist)
}

#[cfg(test)]
mod hierarchical_clustering_tests {
    use super::*;

    fn correlation_tuple(a: &str, b: &str, corr: f64) -> (String, String, f64) {
        (a.to_string(), b.to_string(), corr)
    }

    #[test]
    fn test_no_crates() {
        let correlations: Vec<(String, String, f64)> = Vec::new();
        let result = perform_hierarchical_clustering(&correlations);
        match result {
            Err(HierarchicalClusteringError::NoCrates) => (),
            _ => panic!("Expected NoCrates error for empty input."),
        }
    }

    #[test]
    fn test_single_crate() {
        // Single crate means no pairwise correlations.
        let correlations: Vec<(String, String, f64)> = Vec::new();
        // Since no correlations provided, we cannot infer a second crate.
        // So let's consider that no second crate was given at all.
        // Actually, if we have only one crate, it must appear in correlations. Let's simulate that:
        // If we only have one crate, we can't have correlations. We must handle this scenario
        // by providing at least a mention of a crate (but there's no pair). The code
        // currently extracts crates from correlation tuples only.
        // To handle a single crate scenario properly, we need at least one correlation line referencing it.
        // But with one crate, we can't form a pair. For now, let's assume this scenario is not
        // possible unless we modify the code to accept crates separately from correlations.

        // As a workaround, let's test one crate scenario by forcibly adding a self-pair with correlation=0.
        let single_crate_corr = vec![
            correlation_tuple("only_crate", "only_crate", 1.0) // This is artificial, but let's assume it.
        ];

        let result = perform_hierarchical_clustering(&single_crate_corr);
        if let Ok(dendrogram) = result {
            match dendrogram {
                Dendrogram::Leaf { crate_name } => {
                    assert_eq!(crate_name, "only_crate");
                },
                _ => panic!("Expected a leaf for a single crate."),
            }
        } else {
            panic!("Expected success for single crate scenario.");
        }
    }

    #[test]
    fn test_two_crates_no_correlation() {
        // Two distinct crates with zero correlation (distance=1.0)
        let correlations = vec![correlation_tuple("crateA", "crateB", 0.0)];
        let result = perform_hierarchical_clustering(&correlations);
        if let Ok(dendrogram) = result {
            // Expect a single internal node with two leaves
            match dendrogram {
                Dendrogram::Internal { left, right, distance } => {
                    // Distance should be 1 - 0 = 1
                    assert!((distance - 1.0).abs() < 1e-9);
                    match (*left, *right) {
                        (Dendrogram::Leaf { crate_name: ref c1 }, Dendrogram::Leaf { crate_name: ref c2 }) => {
                            let mut crates = vec![c1.as_str(), c2.as_str()];
                            crates.sort();
                            assert_eq!(crates, vec!["crateA", "crateB"]);
                        },
                        _ => panic!("Expected two leaf nodes."),
                    }
                },
                _ => panic!("Expected an Internal node for two crates."),
            }
        } else {
            panic!("Expected success for two crates no correlation.");
        }
    }

    #[test]
    fn test_perfect_correlation() {
        // Two identical crates with correlation=1.0
        let correlations = vec![correlation_tuple("crateA", "crateB", 1.0)];
        let result = perform_hierarchical_clustering(&correlations);
        if let Ok(dendrogram) = result {
            match dendrogram {
                Dendrogram::Internal { distance, .. } => {
                    // distance = 1 - corr = 0.0 since corr=1.0
                    assert!((distance - 0.0).abs() < 1e-9);
                },
                _ => panic!("Expected Internal node for two crates."),
            }
        } else {
            panic!("Expected success for perfect correlation.");
        }
    }

    #[test]
    fn test_three_crates_mixed_correlations() {
        // crateA and crateB correlate 0.8 -> distance=0.2
        // crateB and crateC correlate 0.3 -> distance=0.7
        // crateA and crateC no entry => distance=1.0 by default
        let correlations = vec![
            correlation_tuple("crateA", "crateB", 0.8),
            correlation_tuple("crateB", "crateC", 0.3),
        ];
        let result = perform_hierarchical_clustering(&correlations);
        if let Ok(dendrogram) = result {
            // We expect that the first merge will be between crateA and crateB (closest pair),
            // then that cluster merges with crateC.
            // The first merge distance: 1 - 0.8 = 0.2 (A-B)
            // Then merging (A,B) cluster with C: min distance to C is via crateB (distance=0.7).
            match dendrogram {
                Dendrogram::Internal { left, right, distance: top_dist } => {
                    // The top-level merge should be at distance=0.7
                    assert!((top_dist - 0.7).abs() < 1e-9);

                    // One side should be crateC leaf, the other side the A-B cluster
                    let mut leaves = Vec::new();
                    fn collect_leaves(d: &Dendrogram, leaves: &mut Vec<String>) {
                        match d {
                            Dendrogram::Leaf { crate_name } => leaves.push(crate_name.clone()),
                            Dendrogram::Internal { left, right, .. } => {
                                collect_leaves(left, leaves);
                                collect_leaves(right, leaves);
                            }
                        }
                    }

                    collect_leaves(&*left, &mut leaves);
                    collect_leaves(&*right, &mut leaves);

                    leaves.sort();
                    assert_eq!(leaves, vec!["crateA", "crateB", "crateC"]);
                },
                _ => panic!("Expected internal node at top."),
            }
        } else {
            panic!("Expected success for three crates mixed correlations.");
        }
    }

    #[test]
    fn test_incomplete_correlation_data() {
        // Suppose we have three crates, but only one correlation.
        // This means some pairs are missing. Our code treats missing as distance=1.0.
        // This should still be fine, not produce an error, just larger distances.
        let correlations = vec![
            correlation_tuple("crateX", "crateY", 0.5),
        ];
        // Should cluster all three crates (X, Y, and maybe a crateZ if we define one)
        // Wait, we only have two crates defined above. For three crates test, define three in correlation.

        // Actually, to simulate incomplete data: 
        // Let's say we have crates: crateX, crateY, crateZ
        // Provide correlation only for X-Y. Z is never mentioned.
        let correlations = vec![
            correlation_tuple("crateX", "crateY", 0.5),
        ];
        // Here crateZ is not in correlations at all, so no mention. We must provide it somehow.
        // The code currently extracts crates only from correlation tuples. If we don't mention crateZ, it doesn't exist.
        // To test incomplete correlation data meaningfully, we need at least mention crateZ with another crate.
        // Let's do:
        let correlations = vec![
            correlation_tuple("crateX", "crateY", 0.5),
            correlation_tuple("crateX", "crateZ", 0.0),  // X-Z defined, Y-Z missing
        ];

        // Now Y-Z is missing, so Y-Z distance = 1.0, X-Z distance=1.0, X-Y distance=0.5 => dist=0.5
        let result = perform_hierarchical_clustering(&correlations);
        if let Ok(dendrogram) = result {
            // Just ensure it doesn't fail. Check we have three leaves total.
            let mut leaves = Vec::new();
            fn collect_leaves(d: &Dendrogram, leaves: &mut Vec<String>) {
                match d {
                    Dendrogram::Leaf { crate_name } => leaves.push(crate_name.clone()),
                    Dendrogram::Internal { left, right, .. } => {
                        collect_leaves(left, leaves);
                        collect_leaves(right, leaves);
                    }
                }
            }

            collect_leaves(&dendrogram, &mut leaves);
            leaves.sort();
            assert_eq!(leaves, vec!["crateX", "crateY", "crateZ"]);
        } else {
            panic!("Expected success even with incomplete data (missing pairs).");
        }
    }

    #[test]
    fn test_many_crates_low_correlation() {
        // Several crates, all with zero correlation => large distances.
        // Just test performance & correctness, ensure no panic.
        let crates = &["a", "b", "c", "d", "e"];
        let mut correlations = Vec::new();
        // minimal set of correlations with zero correlation
        correlations.push(correlation_tuple("a", "b", 0.0));
        correlations.push(correlation_tuple("b", "c", 0.0));
        correlations.push(correlation_tuple("c", "d", 0.0));
        correlations.push(correlation_tuple("d", "e", 0.0));
        // Missing pairs means distance=1.0 anyway.

        let result = perform_hierarchical_clustering(&correlations);
        if let Ok(dendrogram) = result {
            // Collect leaves
            let mut leaves = Vec::new();
            fn collect_leaves(d: &Dendrogram, leaves: &mut Vec<String>) {
                match d {
                    Dendrogram::Leaf { crate_name } => leaves.push(crate_name.clone()),
                    Dendrogram::Internal { left, right, .. } => {
                        collect_leaves(left, leaves);
                        collect_leaves(right, leaves);
                    }
                }
            }

            collect_leaves(&dendrogram, &mut leaves);
            leaves.sort();
            assert_eq!(leaves, vec!["a", "b", "c", "d", "e"]);
        } else {
            panic!("Expected success with many crates low correlation.");
        }
    }

    // Additional tests could simulate data shape errors by mocking functions or passing
    // invalid states, but that requires controlling internal states which may not be trivial.
    // The hierarchical clustering code is structured in a way that errors mainly occur on
    // empty datasets or indexing issues. We've tested empty (no crates) scenario already.
}

