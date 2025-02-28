# README

## Overview

The **hydro2-network** crate defines the **core data structures** and utility functions for building operator‐based networks in the `hydro2` ecosystem. A network is a directed acyclic graph (DAG) composed of:

- **Nodes** (each wrapping an operator and its I/O buffers)
- **Edges** (defining how data flows between node outputs and inputs)

This crate includes:

- **`NetworkNode`** and **`Network`** structs to store the topology.  
- **`NetworkEdge`** structs for explicit connections.  
- **Macros** (`node!`, `edge!`, `network!`) that simplify constructing nodes, edges, and entire networks.  
- **`wire_up_network`**: Allocates and connects channel buffers between node outputs/inputs, ensuring each node’s input type matches the corresponding output type.  
- **Validation** (`Network::validate`) for cycle detection, ensuring the network is acyclic.

### Key Features

1. **Operator Agnosticism**  
   Each node references a generic `Operator<NetworkItem>`, enabling you to integrate any operator that implements the `Operator` trait.

2. **Wiring & Allocation**  
   The `wire_up_network` function dynamically allocates shared arcs (`Arc<AsyncRwLock<...>>`) for node outputs. Edges link those arcs to the downstream node’s inputs. This automatic wiring eliminates the need for manual buffer handling.

3. **Cycle Detection**  
   The built‐in `validate` routine checks for DAG correctness. If a cycle is found (or other configuration errors), it returns a `NetworkError`.

4. **Macros**  
   - **`edge!(src_idx:src_port -> dst_idx:dst_port)`**: Clean syntax for building `NetworkEdge`.
   - **`node!(idx => op)`**: Instantiates a `NetworkNode` for a given operator.
   - **`network!(nodes_vec, edges_vec)`**: Validates and wires everything in one shot.

### Usage Example

Below is a minimal example of creating and validating a single‐node network with no edges:

```rust
use hydro2_network::{Network, node, network};
use hydro2_basic_operators::NoOpOperator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build a list of nodes
    let nodes = vec![
        node!(0 => NoOpOperator::default())
    ];
    // Build an empty list of edges
    let edges = vec![];

    // Use the `network!` macro to finalize and validate
    let net = network!(nodes, edges);
    println!("Successfully built a single-node network: {:?}", net);

    Ok(())
}
```

- **Adding Edges**  
  If you have multiple nodes, define edges with `edge!(0:0 -> 1:0)` to connect node 0’s output‐port 0 to node 1’s input‐port 0. The macros check that ports are below `4`.

- **Execution**  
  Once validated and wired, your runtime or scheduler can lock each node’s inputs and outputs and invoke `node.execute()`. This triggers the operator’s async logic, reading from inputs and writing to outputs.

---

## License

Distributed under the OGP License (see `ogp-license-text` crate for more details).

## Repository

Hosted on GitHub:  
[https://github.com/klebs6/klebs-general](https://github.com/klebs6/klebs-general)
