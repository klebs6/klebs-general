# language-model-format-instructions

This crate offers straightforward, enumerated instructions for ensuring language model output remains **concrete**, **specific**, and **valid JSON**. By integrating these instructions into system messages or prompt headers, users can guide models toward generating more reliably parsable and unambiguous responses.

## Features

- **LanguageModelOutputFormatInstruction**  
  Enumerates key instructions, each describing a dimension of good practices for structured output.  
  - **AvoidVagueness**: Encourages the model to avoid sweeping generalities and unclear language.  
  - **ProvideOutputAsValidJson**: Directs the model to produce well-formed JSON free of extra text.

- **InstructedLanguageModelAtCoordinate**  
  A builder-based struct holding an optional coordinate (e.g., `AgentCoordinate`) and a vector of format instructions, which can be invoked via:
  ```rust
  InstructedLanguageModelAtCoordinate::emit_detailed_json_objects(...)
  ```
  This builds a set of guidance statements ready to be incorporated into prompts.

- **SystemMessageHeader**  
  Automatically aggregates and serializes any instructions from `InstructedLanguageModelAtCoordinate` into a user-friendly string. Ensures that models receive explicit instructions like “avoid vagueness” or “provide valid JSON” without extraneous text.

## Usage

```rust
use language_model_format_instructions::{
    InstructedLanguageModelAtCoordinate, SystemMessageHeader
};
use agent_coordinate::AgentCoordinate;

// Suppose you have a coordinate:
let coord = AgentCoordinate::default();

// Build an instructed LLM object:
let instructed_llm = InstructedLanguageModelAtCoordinate::emit_detailed_json_objects(&coord);

// Convert instructions into a system message header:
let header = SystemMessageHeader::from(&instructed_llm).get();

println!("System Message:\n{}", header);
```

The resulting text can be added to your system or prompt messages, guiding language models to return more consistent, JSON-based outputs.

## Getting Started

1. Add the following line to your `Cargo.toml`:
   ```toml
   [dependencies]
   language-model-format-instructions = "0.1"
   ```
2. Import the relevant structs and traits in your Rust code.

