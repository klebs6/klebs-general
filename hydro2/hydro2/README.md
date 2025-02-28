# `hydro2`
This crate encapsulates the `hydro2` system.
See the following crates:

`hydro2-3p`                  - Encapsulates the third party (3p) dependencies for the hydro2 system
`hydro2-async-scheduler`     - An asynchronous DAG-based scheduling framework for parallel operator execution.
`hydro2-basic-operators`     - A collection of fundamental operators for the hydro2 system.
`hydro2-mock`                - Mock utility components for testing within the hydro2 system.
`hydro2-network`             - Core data structures and DAG logic for building operator-based networks in the hydro2 system.
`hydro2-network-performance` - Performance tracking for network execution in the hydro2 ecosystem.
`hydro2-network-wire-derive` - A procedural macro providing #[derive(NetworkWire)] for bridging Hydro2 operator wires and enumerating operator IO variants. It automatically handles generics, type parameters, and attribute parsing to unify wire and operator definitions.
`hydro2-operator`            - Core interfaces, traits, and error handling for creating and running Hydro2 operators, including multi - port type conversion utilities.
`hydro2-operator-derive`     - Procedural macro that derives implementations of hydro2-operator's Operator trait, including port enumeration and bridging code for up to four inputs/outputs.

## License

Distributed under the OGP License (see `ogp-license-text` crate for more details).
