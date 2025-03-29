# hydro2-operator-derive

`hydro2-operator-derive` is a procedural macro crate that auto-generates the boilerplate needed to implement the `Operator` trait from **[hydro2-operator]**. By simply annotating your structs with `#[derive(Operator)]` and an `#[operator(...)]` attribute, you can specify:

- Which function to call at execution time (`execute="..."`).
- The operator's opcode (`opcode="..."`).
- Up to four input types (`input0="...", input1="..."`, etc.).
- Up to four output types (`output0="...", output1="..."`, etc.).

From this specification, the macro generates:

1. An `enum` (named `YourStructIO`) to represent each of the operator's input and output variants.
2. Implementations of all required port traits (`PortTryFromN`, `PortTryIntoN`, etc.) for robust type conversions on each port.
3. An implementation of the `OperatorInterface<YourStructIO>` trait itself, including an asynchronous `execute(...)` method that matches inputs, calls your specified function, and packages outputs into the appropriate enum variants.
4. A hidden operator signature struct that implements `OperatorSignature`, mapping your input/output types to each port index.

### Core Use Cases

- **Define an Operator**: Convert a normal Rust struct into a Hydro2 operator by annotating it:
  ```rust
  use hydro2_operator::Operator;

  #[operator(execute="process_fn", opcode="BasicOpCode::AddOp", input0="i32", output0="i32")]
  #[derive(Operator)]
  pub struct AddOperator {
      // your fields here
  }
  ```
- **Automatically Handle Ports**: This macro ensures type-safe bridging for each of up to four inputs and outputs, even if your operator uses reference types (like `&[T]`), optional inputs (`Option<T>`), or no inputs/outputs at all.
- **Customize Generics**: If your operator needs lifetime or type parameters, the derive mechanism will pick them up, unify them, and generate the correct references in the operator-IO enum.
- **Compile-Time Checks**: Many potential errors (e.g., specifying more than four inputs, reusing a key multiple times in `#[operator(...)]`) are caught at compile time, providing clear diagnostic messages.

### Quick Example

```rust
#[operator(
    execute="run_logic",
    opcode="BasicOpCode::MyCustomOp",
    input0="&[u8]",
    output0="Vec<u8>",
    output1="Option<String>"
)]
#[derive(Operator)]
pub struct MyCustomOperator {
    factor: usize,
}

impl MyCustomOperator {
    async fn run_logic(&self, data: &[u8]) -> hydro2_operator::NetResult<(Vec<u8>, Option<String>)> {
        // do something with `data` and `self.factor`
        Ok((data.to_vec(), Some(format!("Factor is {}", self.factor))))
    }
}
```

This macro invocation generates:

1. An `enum MyCustomOperatorIO` with variants:
   - `Input0(&[u8])`
   - `Output0(Vec<u8>)`
   - `Output1(Option<String>)`
2. Implementations for bridging each port:
   - `PortTryFrom0<&[u8]>` for `MyCustomOperatorIO::Input0(...)`
   - `PortTryInto0<Vec<u8>>` for `MyCustomOperatorIO::Output0(...)`
   - And so on.
3. `OperatorInterface<MyCustomOperatorIO>` trait impl for `MyCustomOperator`, mapping `execute([...])` to call `run_logic()`.

All of these generated artifacts allow Hydro2’s runtime or other tooling to safely execute your operator within a multi-operator dataflow.

Include this in your Cargo dependencies:

```toml
[dependencies]
hydro2-operator-derive = "0.1"
```

You will also want to depend on `hydro2-operator` to use the `Operator` trait and related definitions.

## License

Distributed under the OGPv1 License (see `ogp-license-text` crate for more details).
