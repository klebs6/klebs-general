# hydro2-network-wire-derive

This crate provides a procedural macro named `#[derive(NetworkWire)]`, which auto-generates the internal bridging and execution logic for a "wire" type in the Hydro2 operator framework. By combining information about the wire’s generics and a list of available operators (via an `#[available_operators(op="...")]` attribute), it automatically creates:

- An IO enum (e.g., `MyWireIO`) to mirror each referenced operator’s types.
- Operator trait implementations and port handling methods, ensuring that inputs/outputs can be safely mapped into the correct Rust types at runtime.
- Support for reusing or minting new generic parameters when bridging types, helping unify operator-specific and wire-specific type/lifetime/const generics.

### Key Features

1. **`#[derive(NetworkWire)]`**: Attach to a struct to generate:
   - A companion `enum` (like `MyWireIO`) containing zero or more operator-IO variants.
   - Operator trait implementations bridging between that enum and each operator’s internal types.
2. **Operator attribute**: Use `#[available_operators(op="FooOp", op="BarOp<Z>", ...)]` on the wire struct to specify the set of operators the wire can handle.
3. **Generics Unification**: Automatically recognizes matching lifetimes, type parameters, and const parameters in both the wire struct and operator types. It will reuse or mint new parameters (e.g., `OpTy0`, `OPC1`) as needed.
4. **Flexible Where-Clauses**: Merges where-clauses from the wire struct and newly introduced operator parameters.

### Example Overview

When a struct uses:

```rust
#[available_operators(op="AddOp", op="ConstantOp<T>")]
#[derive(NetworkWire)]
pub struct MyWire<T> {
    // ...
}
```

this macro will produce:

- An enum `MyWireIO<T>` with variants like `AddOpIO(AddOpIO<T>)`, `ConstantOpIO(ConstantOpIO<T>)`, and `None`.
- Implementations of the required operator and port-try-into traits, ensuring correct input/output type conversion.

You can then rely on `MyWireIO` for operator graph constructions involving the listed operators.

Be sure to declare this crate in your `Cargo.toml` dependencies as:
```toml
[dependencies]
hydro2-network-wire-derive = "0.1"
```

Then, import and use the macro in your Rust code as needed.

## License

Distributed under the OGPv1 License (see `ogp-license-text` crate for more details).
