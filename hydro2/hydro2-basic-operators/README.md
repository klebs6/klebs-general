## Overview

This crate supplies a collection of **basic operator implementations** for use with the `hydro2` ecosystem. Each operator is labeled with an opcode and implements a coherent, asynchronous `execute` method. Common tasks include:

- **Arithmetic operators** (addition, multiplication, merging)
- **Fan-out operators** (splitting or distributing values among multiple outputs)
- **Fan-in operators** (merging multiple inputs into a single output)
- **No-op / Pass-through operators** (testing, pipeline placeholders)
- **Failing operators** for simulating error conditions in test networks

They are all defined under a common trait system (via `#[derive(Operator)]` and related macros), and each is thoroughly tested with straightforward Rust tests.

### Feature Highlights

- **Generics**  
  Several operators are templated over type parameters (e.g., `DoubleToTriTwoGenericsOp<T,U>`) allowing flexible usage with integer and floating‐point types.
- **Multiple inputs/outputs**  
  Operators can handle up to 4 inputs and 4 outputs if needed, matching the wide variety of pipeline or DAG shapes.
- **Error Handling**  
  Some operators like `FailingOperator` deliberately produce errors to help test how your scheduling or execution engine deals with operator failures.

### Usage Example

Below is a minimal usage showing how an operator might be instantiated and tested:

```rust
#[tokio::test]
pub async fn test_add_op_in_a_network() -> Result<(), hydro2_3p::NetworkError> {
    use hydro2_basic_operators::AddOp;
    // Suppose we want to create an AddOp that always adds 10
    let add_op = AddOp::new(10);
    
    // Typically, you'd embed this operator into a node in a network, 
    // but we can unit-test it standalone:
    let input = [Some(&AddOpIO::Input0(5)), None, None, None];
    let mut outputs = [None, None, None, None];

    add_op.execute(input, &mut outputs).await?;
    assert_eq!(outputs[0], Some(AddOpIO::Output0(15)));
    
    Ok(())
}
```

### Operators Included

- **Single/Double/Triple/Quad Operators**: Convert single or multiple inputs into multiple outputs (e.g., `SingleToTriOp`, `DoubleOutOp`).
- **Merging Operators**: Combine multiple inputs into one (like `Merge2Op`).
- **Math Operators**: `AddOp`, `MultiplyOp`, `IncrementOperator`, etc.
- **Testing/Utility Operators**:
  - **NoOpOperator**: Does nothing, useful for pipeline placeholders.
  - **FailingOperator**: Always errors out, testing error handling logic.
  - **ConstantOp**: Outputs a constant value each time it is invoked.

### Integration with `hydro2`

These operators are designed to be used in a `hydro2_network` DAG, orchestrated by the `hydro2_async_scheduler` or similar frameworks. They each conform to the `Operator` trait, enabling dynamic scheduling and hooking into upstream/downstream data flows.

---

## Development

- **Logging**: Uses the [`tracing`](https://crates.io/crates/tracing) crate for detailed operator‐level logs.
- **Testing**: Each operator defines unit tests covering expected and edge‐case behaviors.
- **Contribution**: If you wish to add new operators or refine existing ones, open a pull request at the repository listed below.

## License

Distributed under the OGP License (see `ogp-license-text` crate for more details).

## Repository

Developed on GitHub at:  
[https://github.com/klebs6/klebs-general](https://github.com/klebs6/klebs-general)
