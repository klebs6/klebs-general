# hydro2-operator

`hydro2-operator` provides the core interfaces, traits, and error types for building and executing operators in a Hydro2 pipeline. Operators consume up to four input "ports" and produce up to four outputs; by conforming to the provided `Operator` trait, they can be assembled into larger dataflow networks. The crate supports compile-time type-checking of operator inputs/outputs and implements runtime bridging for more flexible wiring.

### Key Components

- **`Operator<NetworkItem>`**: Defines how an operator processes up to four inputs to produce up to four outputs, including information on input/output port count, port type strings, port connection requirements, and an asynchronous `execute(...)` method.
- **`OperatorSignature`**: Associates up to four input types and four output types to a specific operator. Any user-defined operator implementing `Operator` usually also implements or references an `OperatorSignature`.
- **Port Conversions**:
  - **`PortTryIntoN<T>` and `PortTryFromN<T>`**: Type conversion traits that attach a "port index" context to the conversions. Operators declare how data is read from or written to each port via these traits. 
  - **`PortTryIntoNAny`**: Enables an operator to convert data into an "erased" container, facilitating uniform handling of multiple possible types.
- **`OpCode`**: General trait (or interface) for enumerating operator codes (e.g., for logging or identifying distinct transformations in the pipeline).
- **`NetworkError`**: Comprehensive error enumeration for all potential failures in operator creation, graph construction, and execution. This includes resource exhaustion, invalid port assignments, operator task panics, etc.

### Highlights

1. **Multi-Port Handling**: Handles up to 4 inputs and 4 outputs, letting you build operators ranging from simple single-input transforms to multi-channel merges or splits.
2. **`#[async_trait]`**: Allows each operator to define `async fn execute(...)` for asynchronous dataflow, integrating well with async runtimes.
3. **Convert to `Arc<dyn Operator<...>>`**: The `IntoArcOperator` trait easily boxes up your operators into a trait object that can be managed by the Hydro2 runtime.
4. **Flexible Type System**: Operators can declare custom input and output types, and these can be validated at compile time or checked at runtime through the provided port traits.

Add the following to your dependencies:

```toml
[dependencies]
hydro2-operator = "0.1"
```

Then, import and implement the `Operator` trait for your custom transformations, specifying how each input is handled and how outputs are produced.

## License

Distributed under the OGPv1 License (see `ogp-license-text` crate for more details).
