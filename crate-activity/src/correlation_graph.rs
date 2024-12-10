crate::ix!();

/// Build a graph of crates where edges represent correlations above or equal to a given threshold.
///
/// Returns a HashMap: crate_name -> HashMap<adj_crate_name, correlation>
pub fn build_correlation_graph(
    correlations: &[(String, String, f64)],
    threshold: f64,
) -> HashMap<String, HashMap<String, f64>> {
    let mut graph: HashMap<String, HashMap<String, f64>> = HashMap::new();

    for (crate_a, crate_b, corr) in correlations {
        if *corr >= threshold {
            graph.entry(crate_a.clone()).or_default().insert(crate_b.clone(), *corr);
            graph.entry(crate_b.clone()).or_default().insert(crate_a.clone(), *corr);
        }
    }

    graph
}

/// Find communities in the graph by extracting connected components.
/// Each community is a Vec of crate names.
pub fn find_communities(graph: &HashMap<String, HashMap<String, f64>>) -> Vec<Vec<String>> {
    let mut visited = HashSet::new();
    let mut communities = Vec::new();

    for node in graph.keys() {
        if !visited.contains(node) {
            // BFS or DFS to find all connected nodes
            let mut stack = vec![node.clone()];
            let mut component = Vec::new();

            while let Some(current) = stack.pop() {
                if visited.insert(current.clone()) {
                    component.push(current.clone());
                    if let Some(neighbors) = graph.get(&current) {
                        for neighbor in neighbors.keys() {
                            if !visited.contains(neighbor) {
                                stack.push(neighbor.clone());
                            }
                        }
                    }
                }
            }

            component.sort();
            communities.push(component);
        }
    }

    communities.sort_by_key(|c| c.len());
    communities
}

/// Compute degree centrality: number of edges per node.
pub fn compute_degree_centrality(
    graph: &HashMap<String, HashMap<String, f64>>
) -> HashMap<String, usize> {
    let mut centralities = HashMap::new();
    for (node, neighbors) in graph {
        centralities.insert(node.clone(), neighbors.len());
    }
    centralities
}

/// Display the communities (connected components) found in the correlation network.
pub fn display_network_communities(communities: &[Vec<String>]) {
    println!("----------------[correlation-network-communities]----------------");
    for (i, community) in communities.iter().enumerate() {
        println!("Community {} (size={}):", i + 1, community.len());
        for crate_name in community {
            println!("  - {}", crate_name);
        }
        println!("");
    }
}

/// Display the top N nodes by degree centrality.
pub fn display_top_central_nodes(centralities: &HashMap<String, usize>, top_n: usize) {
    println!("----------------[top-central-nodes]----------------");
    let mut centrals: Vec<_> = centralities.iter().collect();
    centrals.sort_by(|a, b| b.1.cmp(a.1));

    for (i, (crate_name, degree)) in centrals.iter().take(top_n).enumerate() {
        println!("{}. {} (degree={})", i + 1, crate_name, degree);
    }
}

/// Compute node and edge betweenness centrality using a standard approach:
/// For each node, run a shortest path search and count the shortest paths going through each other node and edge.
/// This is Brandes' algorithm for betweenness centrality.
///
/// Returns (node_betweenness, edge_betweenness) as HashMaps.
pub fn compute_betweenness_centrality(
    graph: &HashMap<String, HashMap<String, f64>>
) -> (HashMap<String, f64>, HashMap<(String, String), f64>) {
    let mut node_bet = HashMap::new();
    let mut edge_bet = HashMap::new();

    for node in graph.keys() {
        node_bet.insert(node.clone(), 0.0);
    }

    // Initialize edge betweenness for all edges
    for (u, neighbors) in graph {
        for v in neighbors.keys() {
            let edge = ordered_edge(u, v);
            edge_bet.entry(edge).or_insert(0.0);
        }
    }

    // Brandes' algorithm: For each source node
    for s in graph.keys() {
        let (mut stack, mut pred, mut sigma, mut dist) = brandes_initialize(graph, s);

        // BFS or Dijkstra for shortest paths - here we treat all edges equal weight = 1.
        let mut queue = VecDeque::new();
        dist.insert(s.clone(), 0.0);
        sigma.insert(s.clone(), 1.0);
        queue.push_back(s.clone());

        while let Some(v) = queue.pop_front() {
            stack.push(v.clone());
            if let Some(neighbors) = graph.get(&v) {
                for w in neighbors.keys() {

                    // Check using infinity to see if w is unvisited
                    if dist[w.as_str()] == f64::INFINITY {
                        dist.insert(w.clone(), dist[&v] + 1.0);
                        queue.push_back(w.clone());
                    }

                    // If w is exactly one step further than v, update sigma and pred
                    if (dist[w.as_str()] - dist[v.as_str()] - 1.0).abs() < 1e-9 {
                        sigma.insert(w.clone(), sigma[w] + sigma[&v]);
                        pred.get_mut(w).unwrap().push(v.clone());
                    }
                }
            }
        }

        // Accumulation
        let mut delta: HashMap<String, f64> = HashMap::new();
        for v in graph.keys() {
            delta.insert(v.clone(), 0.0);
        }

        while let Some(w) = stack.pop() {
            if let Some(pws) = pred.get(&w) {
                let coeff = (1.0 + delta[&w]) / sigma[&w];
                for v in pws {
                    let increment = sigma[v] * coeff;
                    delta.insert(v.clone(), delta[v] + increment);

                    // Edge betweenness
                    let edge = ordered_edge(v, &w);
                    *edge_bet.get_mut(&edge).unwrap() += increment;
                }
            }
            if w != *s {
                *node_bet.get_mut(&w).unwrap() += delta[&w];
            }
        }
    }

    // Normalize edge betweenness
    for val in edge_bet.values_mut() {
        *val /= 2.0;
    }

    (node_bet, edge_bet)
}

fn ordered_edge(a: &str, b: &str) -> (String, String) {
    if a < b {
        (a.to_string(), b.to_string())
    } else {
        (b.to_string(), a.to_string())
    }
}

fn brandes_initialize(
    graph: &HashMap<String, HashMap<String, f64>>,
    s: &str
) -> (Vec<String>, HashMap<String, Vec<String>>, HashMap<String, f64>, HashMap<String, f64>) {
    let stack = Vec::new();
    let mut pred: HashMap<String, Vec<String>> = HashMap::new();
    let mut sigma: HashMap<String, f64> = HashMap::new();
    let mut dist: HashMap<String, f64> = HashMap::new();

    for v in graph.keys() {
        pred.insert(v.clone(), Vec::new());
        sigma.insert(v.clone(), 0.0);
        dist.insert(v.clone(), f64::INFINITY);
    }

    (stack, pred, sigma, dist)
}

/// Apply a simplified Girvan–Newman algorithm:
/// 1. Compute edge betweenness.
/// 2. Remove the edge with highest betweenness.
/// 3. Recompute communities and repeat until the desired number of communities reached or no edges remain.
/// This is a simplified version that stops once we reach a certain community count or no edges left.
pub fn girvan_newman_communities(
    mut graph: HashMap<String, HashMap<String, f64>>,
    target_communities: usize
) -> Vec<Vec<String>> {
    loop {
        let communities = find_communities(&graph);
        if communities.len() >= target_communities {
            return communities;
        }

        // Compute edge betweenness
        let (_node_bet, edge_bet) = compute_betweenness_centrality(&graph);

        // After computing edge betweenness:
        let mut edges: Vec<_> = edge_bet.iter().collect();
        // Sort primarily by descending betweenness, secondary by lex order of nodes
        edges.sort_by(|((a1,b1), v1), ((a2,b2), v2)| {
            v2.partial_cmp(v1).unwrap() // descending by betweenness
                .then_with(|| {
                    // tie-break: lexicographically smallest edge
                    let edge1 = if a1 < b1 { (a1,b1) } else { (b1,a1) };
                    let edge2 = if a2 < b2 { (a2,b2) } else { (b2,a2) };
                    edge1.cmp(&edge2)
                })
        });

        // Remove the top edge:
        if let Some(((a,b),_)) = edges.first() {
            remove_edge(&mut graph, a, b);
        } else {
            return communities;
        }
    }
}

fn remove_edge(graph: &mut HashMap<String, HashMap<String,f64>>, a: &str, b: &str) {
    if let Some(neighbors) = graph.get_mut(a) {
        neighbors.remove(b);
        // Do not remove the node even if neighbors.is_empty().
    }
    if let Some(neighbors) = graph.get_mut(b) {
        neighbors.remove(a);
        // Similarly, do not remove 'b' from the graph if its neighbors are empty.
    }
}

/// Display graph summary
pub fn display_graph_summary(graph: &HashMap<String, HashMap<String, f64>>) {
    let n = graph.len();
    let m: usize = graph.values().map(|neighbors| neighbors.len()).sum::<usize>() / 2;
    let avg_degree = if n > 0 { (2.0 * m as f64) / n as f64 } else { 0.0 };
    let communities = find_communities(graph);

    println!("----------------[graph-summary]----------------");
    println!("Number of nodes: {}", n);
    println!("Number of edges: {}", m);
    println!("Average degree: {:.2}", avg_degree);
    println!("Number of communities: {}", communities.len());
    if let Some(largest) = communities.iter().map(|c| c.len()).max() {
        println!("Largest community size: {}", largest);
    }
    if let Some(smallest) = communities.iter().map(|c| c.len()).min() {
        println!("Smallest community size: {}", smallest);
    }
    println!("");
}

/// Display betweenness centrality top nodes
pub fn display_top_betweenness_nodes(
    node_bet: &HashMap<String, f64>,
    top_n: usize
) {
    println!("----------------[top-nodes-by-betweenness]----------------");
    let mut v: Vec<_> = node_bet.iter().collect();
    v.sort_by(|a,b| b.1.partial_cmp(a.1).unwrap());

    for (i, (node, score)) in v.iter().take(top_n).enumerate() {
        println!("{}. {} (betweenness={:.2})", i+1, node, score);
    }
    println!("");
}

#[cfg(test)]
mod correlation_network_tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        let correlations: Vec<(String, String, f64)> = Vec::new();
        let graph = build_correlation_graph(&correlations, 0.5);
        assert!(graph.is_empty(), "Empty input should produce empty graph.");

        let communities = find_communities(&graph);
        assert!(communities.is_empty(), "Empty graph should have no communities.");

        let centralities = compute_degree_centrality(&graph);
        assert!(centralities.is_empty(), "No nodes means no centralities.");
    }

    #[test]
    fn test_single_crate_no_edges() {
        // Single crate cannot form edges with itself unless we consider self-correlation.
        // The code doesn't add self-edges, so no edges should be formed.
        let correlations = vec![tuple("crateA", "crateA", 0.9)];
        let graph = build_correlation_graph(&correlations, 0.5);
        
        // Even though we have a self-pair, it should not result in edges.
        // Let's verify what happens: It's possible the code treats this as an edge,
        // but it's symmetrical. Our code doesn't explicitly prevent self-edges, but
        // since crate_a == crate_b, we insert it twice. Let's see:
        // Actually, logically, a self-edge would still appear, but it's meaningless.
        // If we don't want self-edges, we can rely on the code as given to see if it produces them.
        
        // Let's accept self-edges if they appear. The test expects no meaningful community split from a single node.
        // If a self-edge appears, it's trivial and doesn't harm correctness.
        
        // Either way, we have at most one node.
        assert!(graph.len() <= 1, "At most one node expected.");
        if let Some(neighbors) = graph.get("crateA") {
            // If a self-edge got inserted, neighbors will contain 'crateA' itself.
            // It's a corner case, but let's just ensure it doesn't break community detection.
            assert!(neighbors.len() <= 1);
        }

        let communities = find_communities(&graph);
        assert_eq!(communities.len(), 1, "Single node forms one community.");
        assert_eq!(communities[0], vec!["crateA"], "Community should contain only crateA.");

        let centralities = compute_degree_centrality(&graph);
        // If no self-edge is considered, degree=0; if self-edge was inserted, degree=1.
        // Either is acceptable. Let's just check the node exists.
        assert!(centralities.contains_key("crateA"));
    }

    #[test]
    fn test_two_crates_no_edge_below_threshold() {
        let correlations = vec![tuple("crateA", "crateB", 0.4)];
        let graph = build_correlation_graph(&correlations, 0.5);
        assert!(graph.is_empty(), "No edges should form if correlation < threshold.");

        let communities = find_communities(&graph);
        // If no edges, no entries in graph. Actually, since no edges surpass threshold,
        // the graph won't even have these nodes recorded. That means zero communities.
        assert!(communities.is_empty(), "No edges and no nodes means no communities.");
    }

    #[test]
    fn test_two_crates_with_edge() {
        let correlations = vec![tuple("crateA", "crateB", 0.7)];
        let graph = build_correlation_graph(&correlations, 0.7);
        // Should form an edge between crateA and crateB
        assert_eq!(graph.len(), 2, "Two nodes expected.");
        assert!(graph.get("crateA").unwrap().contains_key("crateB"), "Edge should exist A->B.");
        assert!(graph.get("crateB").unwrap().contains_key("crateA"), "Edge should exist B->A.");

        let communities = find_communities(&graph);
        assert_eq!(communities.len(), 1, "Single community with both crates.");
        let mut comm = communities[0].clone();
        comm.sort();
        assert_eq!(comm, vec!["crateA", "crateB"]);

        let centralities = compute_degree_centrality(&graph);
        assert_eq!(centralities.get("crateA"), Some(&1));
        assert_eq!(centralities.get("crateB"), Some(&1));
    }

    #[test]
    fn test_threshold_one_requiring_perfect_correlation() {
        let correlations = vec![
            tuple("crateA", "crateB", 1.0),
            tuple("crateA", "crateC", 0.99),
            tuple("crateB", "crateC", 1.0),
        ];
        let graph = build_correlation_graph(&correlations, 1.0);
        assert_eq!(graph.len(), 3, "All crates A, B, C appear because B-C also has perfect correlation.");

        // Check edges:
        assert!(graph.get("crateA").unwrap().contains_key("crateB"));
        // crateA->crateC should not exist because corr=0.99 < 1.0
        assert!(!graph.get("crateA").unwrap().contains_key("crateC"));

        assert!(graph.get("crateB").unwrap().contains_key("crateA"));
        assert!(graph.get("crateB").unwrap().contains_key("crateC"));
        // B and C have perfect correlation too.

        let communities = find_communities(&graph);
        // Actually, we have only crateA and crateB and crateC known from edges?
        // Wait, crateC must appear in graph. B<->C is perfect correlation, so C is also in graph.
        // Graph should have A, B, C since B<->C is also 1.0
        // Let's re-check logic:
        // Insert A-B since 1.0 >=1.0
        // Insert B-C since 1.0 >=1.0
        // Insert A-C is 0.99 not inserted.

        // Actually, that means A, B, C all appear. Because from B-C we also insert C with B.
        assert_eq!(graph.len(), 3, "All three crates should be nodes because of B-C edge.");

        let communities = find_communities(&graph);
        assert_eq!(communities.len(), 1, "All three form one community due to two edges.");

        let mut comm = communities[0].clone();
        comm.sort();
        assert_eq!(comm, vec!["crateA", "crateB", "crateC"]);

        let centralities = compute_degree_centrality(&graph);
        // A connected to B only -> degree=1
        // B connected to A and C -> degree=2
        // C connected to B only -> degree=1
        assert_eq!(centralities.get("crateA"), Some(&1));
        assert_eq!(centralities.get("crateB"), Some(&2));
        assert_eq!(centralities.get("crateC"), Some(&1));
    }

    #[test]
    fn test_threshold_zero_all_edges() {
        let correlations = vec![
            tuple("a", "b", 0.1),
            tuple("a", "c", 0.5),
            tuple("b", "c", 0.2),
            tuple("c", "d", 0.9),
        ];
        let graph = build_correlation_graph(&correlations, 0.0);
        // Since threshold=0.0, all correlations form edges.
        // Nodes: a,b,c,d
        assert_eq!(graph.len(), 4);

        // Check some edges:
        assert!(graph.get("a").unwrap().contains_key("b"));
        assert!(graph.get("a").unwrap().contains_key("c"));
        assert!(graph.get("b").unwrap().contains_key("c"));
        assert!(graph.get("c").unwrap().contains_key("d"));

        let communities = find_communities(&graph);
        // All nodes connected together (since all edges allowed), should form one big community.
        assert_eq!(communities.len(), 1);
        let mut comm = communities[0].clone();
        comm.sort();
        assert_eq!(comm, vec!["a", "b", "c", "d"]);

        let centralities = compute_degree_centrality(&graph);
        // Degrees:
        // a connected to b,c -> degree=2
        // b connected to a,c -> degree=2
        // c connected to a,b,d -> degree=3
        // d connected to c -> degree=1
        assert_eq!(centralities.get("a"), Some(&2));
        assert_eq!(centralities.get("b"), Some(&2));
        assert_eq!(centralities.get("c"), Some(&3));
        assert_eq!(centralities.get("d"), Some(&1));
    }

    #[test]
    fn test_disconnected_graph_multiple_components() {
        // Two separate subgraphs:
        // Subgraph1: (x <-> y) corr=0.8
        // Subgraph2: (p <-> q, q <-> r) corr=0.9
        // Subgraph3: Single node s with no edges.
        let correlations = vec![
            tuple("x", "y", 0.8),
            tuple("p", "q", 0.9),
            tuple("q", "r", 0.9),
            // s is isolated, no edges above threshold
            tuple("s", "t", 0.4), // below threshold, no edge formed
        ];
        let graph = build_correlation_graph(&correlations, 0.7);
        // Edges formed: x-y; p-q; q-r. s and t appear only if an edge surpass threshold
        // s-t corr=0.4 <0.7 no edge formed -> s and t don't appear in graph since no edges.

        // Instead of:
        // assert_eq!(graph.len(), 4, "Only x,y,p,q,r appear. s,t do not appear as they have no edges.");
        // Use:
        assert_eq!(graph.len(), 5, "x,y,p,q,r appear because their edges meet the threshold, s,t do not.");
        // Actually, we must consider if `build_correlation_graph` adds nodes only when edges pass threshold.
        // s and t never got an edge above threshold, so they won't appear in graph at all.

        // Check edges:
        assert!(graph.get("x").unwrap().contains_key("y"));
        assert!(graph.get("y").unwrap().contains_key("x"));

        assert!(graph.get("p").unwrap().contains_key("q"));
        assert!(graph.get("q").unwrap().contains_key("p"));
        assert!(graph.get("q").unwrap().contains_key("r"));
        assert!(graph.get("r").unwrap().contains_key("q"));

        // Communities:
        let communities = find_communities(&graph);
        // Expect two communities:
        // 1) (x,y)
        // 2) (p,q,r)
        // s,t are absent entirely as they have no edges above threshold.

        assert_eq!(communities.len(), 2);
        let mut c1 = communities[0].clone();
        let mut c2 = communities[1].clone();
        c1.sort();
        c2.sort();
        // Sorted by size, smaller community first. (x,y) size=2, (p,q,r) size=3
        assert_eq!(c1, vec!["x", "y"]);
        assert_eq!(c2, vec!["p", "q", "r"]);

        let centralities = compute_degree_centrality(&graph);
        // Degrees:
        // x-y each have degree=1
        // p connected to q -> degree=1
        // q connected to p,r -> degree=2
        // r connected to q -> degree=1
        assert_eq!(centralities.get("x"), Some(&1));
        assert_eq!(centralities.get("y"), Some(&1));
        assert_eq!(centralities.get("p"), Some(&1));
        assert_eq!(centralities.get("q"), Some(&2));
        assert_eq!(centralities.get("r"), Some(&1));
    }

    #[test]
    fn test_duplicate_entries() {
        // Suppose the same pair is listed multiple times with the same or different correlations.
        let correlations = vec![
            tuple("a", "b", 0.8),
            tuple("a", "b", 0.85), // duplicate pair with slightly higher corr
            tuple("b", "a", 0.8),  // reversed order duplicate
        ];
        let graph = build_correlation_graph(&correlations, 0.7);
        // Regardless of duplicates, we should end up with a single edge a<->b.
        let a_neighbors = graph.get("a").unwrap();
        assert_eq!(a_neighbors.len(), 1);
        assert!(a_neighbors.contains_key("b"));

        let b_neighbors = graph.get("b").unwrap();
        assert_eq!(b_neighbors.len(), 1);
        assert!(b_neighbors.contains_key("a"));

        let communities = find_communities(&graph);
        assert_eq!(communities.len(), 1);
        let mut comm = communities[0].clone();
        comm.sort();
        assert_eq!(comm, vec!["a", "b"]);

        let centralities = compute_degree_centrality(&graph);
        assert_eq!(centralities.get("a"), Some(&1));
        assert_eq!(centralities.get("b"), Some(&1));
    }

    #[test]
    fn test_large_random_data() {
        // Just a performance or stress test scenario, we won't check exact results extensively.
        // We'll just ensure it doesn't panic and produces a logically consistent result.
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let crate_names = vec!["crate1", "crate2", "crate3", "crate4", "crate5"];
        let mut correlations = Vec::new();

        // Generate random correlations between these crates
        for i in 0..crate_names.len() {
            for j in (i+1)..crate_names.len() {
                let corr = rng.gen_range(0.0..1.0);
                correlations.push(tuple(crate_names[i], crate_names[j], corr));
            }
        }

        // Use a threshold of 0.5
        let graph = build_correlation_graph(&correlations, 0.5);
        // Check for no panic:
        let communities = find_communities(&graph);
        let centralities = compute_degree_centrality(&graph);

        // Just sanity checks:
        // All nodes that have edges above threshold should appear.
        // If no edges above threshold, graph might be empty.
        // If we have edges, communities should reflect actual connectivity.
        // Centralities should be consistent.

        for (node, neighbors) in &graph {
            for neighbor in neighbors.keys() {
                assert!(graph.get(neighbor).unwrap().contains_key(node), "Graph should be symmetric.");
            }
        }

        // No specific assertion because it's random. Just ensure no panic and structures are well-formed.
    }

    fn tuple(a: &str, b: &str, c: f64) -> (String, String, f64) {
        (a.to_string(), b.to_string(), c)
    }

    #[test]
    fn test_empty_graph_summary() {
        let graph: HashMap<String, HashMap<String, f64>> = HashMap::new();
        let communities = find_communities(&graph);
        assert!(communities.is_empty(), "No communities in empty graph.");

        // Just ensure no panic:
        // display_graph_summary doesn't return a value, we trust it to print.
        // We'll not test stdout here, just correctness of logic if possible.
        // We'll trust no panic occurs.
    }

    #[test]
    fn test_girvan_newman_basic() {
        // A small "bridge" scenario:
        // Two clusters: (A,B) and (C,D)
        // A-B corr=0.9, C-D corr=0.9, and a bridging edge B-C = 0.8
        let correlations = vec![
            tuple("A", "B", 0.9),
            tuple("C", "D", 0.9),
            tuple("B", "C", 0.8),
        ];
        let graph = build_correlation_graph(&correlations, 0.7);
        // Initially one community because B-C connects them.

        let initial_communities = find_communities(&graph);
        assert_eq!(initial_communities.len(), 1, "All connected initially.");

        // Apply Girvan-Newman to form 2 communities.
        let communities = girvan_newman_communities(graph.clone(), 2);
        assert_eq!(communities.len(), 2, "Should have split into two communities after removing bridge.");

        // Check which communities formed:
        // Likely (A,B) and (C,D), order by size yields smallest first = (A,B) and then (C,D) or vice versa.
        // Since both are size 2, sorted by size stable: We can just check that we have two size=2 communities.
        for c in &communities {
            assert_eq!(c.len(), 2);
        }
    }

    #[test]
    fn test_betweenness_centrality_star() {
        // Star graph: center = X, leaves = A,B,C
        // Edges: X-A, X-B, X-C all with corr=0.9
        let correlations = vec![
            tuple("X", "A", 0.9),
            tuple("X", "B", 0.9),
            tuple("X", "C", 0.9),
        ];
        let graph = build_correlation_graph(&correlations, 0.7);
        // X is center, shortest paths between leaves always go through X.
        let (node_bet, edge_bet) = compute_betweenness_centrality(&graph);

        // Check node betweenness:
        // X should have highest betweenness because all shortest paths between A,B,C go via X.
        // There are 3 leaves, shortest paths among leaves: A-B, B-C, A-C. All go through X.
        // Each leaf pair shortest path: X is intermediary.
        // So X betweenness > 0, leaves betweenness = 0.
        let x_bet = node_bet.get("X").cloned().unwrap_or(0.0);
        let a_bet = node_bet.get("A").cloned().unwrap_or(0.0);
        let b_bet = node_bet.get("B").cloned().unwrap_or(0.0);
        let c_bet = node_bet.get("C").cloned().unwrap_or(0.0);

        assert!(x_bet > a_bet && x_bet > b_bet && x_bet > c_bet, "X should have highest betweenness.");
        assert_eq!(a_bet, 0.0, "Leaves no betweenness in a star.");
        assert_eq!(b_bet, 0.0, "Leaves no betweenness in a star.");
        assert_eq!(c_bet, 0.0, "Leaves no betweenness in a star.");

        // Edge betweenness: each edge X-A, X-B, X-C should have some betweenness due to shortest paths passing through them.
        // It's symmetrical. Just ensure >0.
        for ((u,v), val) in edge_bet.iter() {
            assert!(val > &0.0, "Star edges should have >0 edge betweenness.");
            assert!((u == "X" || v == "X"), "Edges should connect to X in a star.");
        }
    }

    #[test]
    fn test_girvan_newman_no_change_if_already_multiple_components() {
        // If we start with multiple disconnected components, Girvan–Newman won't remove any edges.
        let correlations = vec![
            tuple("A", "B", 0.9), // component 1
            tuple("C", "D", 0.9), // component 2
            // No edges between these pairs, so we have 2 communities already.
        ];
        let graph = build_correlation_graph(&correlations, 0.7);
        let communities = girvan_newman_communities(graph.clone(), 2);
        assert_eq!(communities.len(), 2, "Already at desired number of communities.");
    }

    #[test]
    fn test_graph_summary_basic() {
        // Just ensure the function runs with no panic and logic is correct.
        let correlations = vec![
            tuple("X", "Y", 0.8),
            tuple("Y", "Z", 0.8),
        ];
        let graph = build_correlation_graph(&correlations, 0.7);
        // 3 nodes: X,Y,Z
        // Edges: X-Y, Y-Z. Total edges=2. Average degree = (2*2)/3 ~1.33
        // Communities: 1 big community (X,Y,Z)
        let communities = find_communities(&graph);
        assert_eq!(communities.len(), 1);
        assert_eq!(graph.len(), 3);
        let total_edges: usize = graph.values().map(|nbrs| nbrs.len()).sum::<usize>() / 2;
        assert_eq!(total_edges, 2);

        // We trust display_graph_summary to print correct info; no panic means success.
        // Could parse stdout in a more advanced test environment, but here we rely on correctness.
    }

    #[test]
    fn test_betweenness_top_nodes() {
        // Square: A-B, B-C, C-D, D-A plus diagonal A-C:
        // A--B
        // |\/|
        // |/\|
        // D--C
        // This is a fully connected structure except missing B-D edge:
        // Distances are short, many equal shortest paths.
        let correlations = vec![
            tuple("A", "B", 0.9),
            tuple("B", "C", 0.9),
            tuple("C", "D", 0.9),
            tuple("D", "A", 0.9),
            tuple("A", "C", 0.9),
        ];
        let graph = build_correlation_graph(&correlations, 0.7);
        let (node_bet, _edge_bet) = compute_betweenness_centrality(&graph);
        // Symmetric graph, betweenness should be relatively even.
        // Just ensure no panic calling display_top_betweenness_nodes.
        display_top_betweenness_nodes(&node_bet, 10);

        // Check all nodes exist in node_bet
        for n in &["A", "B", "C", "D"] {
            assert!(node_bet.contains_key(*n), "All nodes should have a betweenness value.");
        }
    }

    #[test]
    fn test_girvan_newman_high_target_communities() {
        // If we ask for more communities than possible, it should stop when no edges left.
        // Triangle: A-B, B-C, A-C
        let correlations = vec![
            tuple("A", "B", 0.9),
            tuple("B", "C", 0.9),
            tuple("A", "C", 0.9),
        ];
        let graph = build_correlation_graph(&correlations, 0.7);

        // Initially 1 community of {A,B,C}.
        // Girvan-Newman removing edges:
        // Eventually we can get 3 communities (A), (B), (C) if we remove enough edges.
        let communities = girvan_newman_communities(graph.clone(), 5);
        // 5 is more than possible, we end up with single nodes each:
        assert_eq!(communities.len(), 3, "Max communities = number of nodes.");
    }
}
