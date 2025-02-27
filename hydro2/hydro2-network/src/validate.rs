// ---------------- [ File: src/validate.rs ]
crate::ix!();

impl<NetworkItem> Network<NetworkItem> 
where NetworkItem: Debug + Send + Sync
{
    pub fn validate(&self) -> NetResult<()> {
        let node_count = self.nodes().len();

        // 1) check each edge's node idx, port idx
        for (edge_idx, edge) in self.edges().iter().enumerate() {

            let src = edge.source_index();
            let so  = edge.source_output_idx();
            let dst = edge.dest_index();
            let di  = edge.dest_input_idx();

            if *src >= node_count {
                return Err(NetworkError::InvalidConfiguration {
                    details: format!("Edge #{} has invalid src node={}", edge_idx, src),
                });
            }
            if *dst >= node_count {
                return Err(NetworkError::InvalidConfiguration {
                    details: format!("Edge #{} has invalid dst node={}", edge_idx, dst),
                });
            }
            if *so >= 4 {
                return Err(NetworkError::InvalidConfiguration {
                    details: format!("Edge #{} has invalid src port={}", edge_idx, so),
                });
            }
            if *di >= 4 {
                return Err(NetworkError::InvalidConfiguration {
                    details: format!("Edge #{} has invalid dst port={}", edge_idx, di),
                });
            }
        }

        // 2) check for cycles => BFS or Kahn's
        let mut in_degree = vec![0; node_count];
        for edge in self.edges() {
            in_degree[*edge.dest_index()] += 1;
        }

        let mut queue = std::collections::VecDeque::new();
        for i in 0..node_count {
            if in_degree[i] == 0 {
                queue.push_back(i);
            }
        }
        let mut processed = 0;
        while let Some(n) = queue.pop_front() {
            processed += 1;
            // decrement children
            for e in self.edges() {
                if *e.source_index() == n {
                    let dst = e.dest_index();
                    in_degree[*dst] -= 1;
                    if in_degree[*dst] == 0 {
                        queue.push_back(*dst);
                    }
                }
            }
        }
        if processed < node_count {
            return Err(NetworkError::InvalidConfiguration {
                details: "Cycle detected in network graph".into()
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod validate_network_tests {
    use super::*;

    #[test]
    fn test_validate_empty_ok() -> Result<(), NetworkError> {
        let net = NetworkBuilder::<TestWireIO<i32>>::default()
            .nodes(vec![])
            .edges(vec![])
            .build()
            .unwrap();
        net.validate()?;
        Ok(())
    }

    #[test]
    fn test_validate_single_node_no_edges_ok() -> Result<(), NetworkError> {
        // single node => no edges => no cycle => valid
        let n0: NetworkNode<TestWireIO<i32>> = node!(0 => NoOpOperator::default());
        let net = NetworkBuilder::<TestWireIO<i32>>::default()
            .nodes(vec![n0])
            .edges(vec![])
            .build()
            .unwrap();
        net.validate()?;
        Ok(())
    }

    #[test]
    fn test_validate_cycle() {
        // Node0 => Node1 => Node0
        let n0: NetworkNode<TestWireIO<i32>> = node!(0 => NoOpOperator::default());
        let n1: NetworkNode<TestWireIO<i32>> = node!(1 => NoOpOperator::default());
        let e0 = edge!(0:0 -> 1:0);
        let e1 = edge!(1:0 -> 0:0); // cycle

        let net = NetworkBuilder::<TestWireIO<i32>>::default()
            .nodes(vec![n0, n1])
            .edges(vec![e0, e1])
            .build()
            .unwrap();

        let res = net.validate();
        assert!(res.is_err());
        if let Err(NetworkError::InvalidConfiguration{details}) = res {
            assert!(details.contains("Cycle detected"), "Expected cycle error, got: {}", details);
        }
    }

    #[test]
    fn test_validate_out_of_range_edge() {
        // Node0 => Node1 => but we have an edge referencing node=99
        let n0: NetworkNode<TestWireIO<i32>> = node!(0 => ConstantOp::new(42));
        let n1: NetworkNode<TestWireIO<i32>> = node!(1 => AddOp::new(5));
        // edge => source=0 => dest=99 => invalid
        let e_bad = edge!(0:0 -> 99:0);
        let net = NetworkBuilder::<TestWireIO<i32>>::default()
            .nodes(vec![n0, n1])
            .edges(vec![e_bad])
            .build()
            .unwrap();

        let res = net.validate();
        assert!(res.is_err());
        if let Err(NetworkError::InvalidConfiguration{details}) = res {
            assert!(details.contains("invalid dst node=99"));
        }
    }

    #[test]
    fn test_validate_disconnected_multiple_nodes() -> Result<(), NetworkError> {
        // 3 nodes => no edges => no cycle => valid
        // It's “disconnected” but no rule says we must connect them.
        let n0: NetworkNode<TestWireIO<i32>> = node!(0 => ConstantOp::new(1));
        let n1: NetworkNode<TestWireIO<i32>> = node!(1 => AddOp::new(5));
        let n2: NetworkNode<TestWireIO<i32>> = node!(2 => MultiplyOp::new(2));

        let net = NetworkBuilder::<TestWireIO<i32>>::default()
            .nodes(vec![n0, n1, n2])
            .edges(vec![])
            .build()
            .unwrap();

        net.validate()?;
        Ok(())
    }

    #[test]
    fn test_validate_ok_chained() -> Result<(), NetworkError> {
        // Node0 => Node1 => Node2 => no cycle
        let n0: NetworkNode<TestWireIO<i32>> = node!(0 => ConstantOp::new(1));
        let n1: NetworkNode<TestWireIO<i32>> = node!(1 => MultiplyOp::new(5));
        let n2: NetworkNode<TestWireIO<i32>> = node!(2 => AddOp::new(100));
        let e0 = edge!(0:0 -> 1:0);
        let e1 = edge!(1:0 -> 2:0);

        let net = NetworkBuilder::<TestWireIO<i32>>::default()
            .nodes(vec![n0, n1, n2])
            .edges(vec![e0, e1])
            .build()
            .unwrap();

        net.validate()?;
        Ok(())
    }
}
