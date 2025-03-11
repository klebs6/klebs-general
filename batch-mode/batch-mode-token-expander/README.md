# batch-mode-token-expander

This crate provides a streamlined workflow for expanding tokens in a batch-oriented fashion, especially useful when working with language models or automated generation systems. It supports defining custom expansion axes, generating system messages, and creating language model API requests tailored to specified coordinates.

## Features

- **SystemMessageGoal**: Describes the overarching goal or context for expansions at the enum or type level.
- **TokenExpansionAxes** and **GetTokenExpansionAxes**: Define one or more dimensions (axes) of token expansion, encapsulating specific instructions or transformations.
- **TokenExpander**: A compositional trait unifying all aspects of token expansion, including naming, default initialization, and synchronous, multi-thread–safe usage.
- **ExpandedToken**: Represents the result of a token expansion process.
- **GetSystemMessageAtAgentCoordinate**: Builds a comprehensive system message string by combining instructions and context from a given coordinate.
- **CreateLanguageModelRequestsAtAgentCoordinate**: Constructs language model API requests in batch form based on a system message and user inputs.

## Example Use Case

1. Define a struct or enum implementing `TokenExpander`:
   ```rust
   use std::borrow::Cow;
   use std::fmt::Debug;
   use std::sync::Arc;

   // Suppose we have a custom token expander enum:
   #[derive(Default, Debug)]
   enum MyTokenExpander {
       #[default]
       VariantA,
       VariantB,
   }

   impl crate::SystemMessageGoal for MyTokenExpander {
       fn system_message_goal(&self) -> Cow<'_, str> {
           match self {
               MyTokenExpander::VariantA => "Goal for VariantA".into(),
               MyTokenExpander::VariantB => "Goal for VariantB".into(),
           }
       }
   }

   impl crate::GetTokenExpansionAxes for MyTokenExpander {
       fn axes(&self) -> crate::TokenExpansionAxes {
           // In practice, you'd return a list of dynamic axes, e.g.:
           Vec::new()
       }
   }

   impl crate::Named for MyTokenExpander {
       fn name(&self) -> &'static str {
           match self {
               MyTokenExpander::VariantA => "VariantA",
               MyTokenExpander::VariantB => "VariantB",
           }
       }
   }

   impl crate::TokenExpander for MyTokenExpander {}
   ```

2. Use the trait methods to build a system message or create batch requests for a language model:
   ```rust
   let expander = MyTokenExpander::default();
   let coordinate = crate::AgentCoordinate::default();
   let system_message = expander.get_system_message_at_agent_coordinate(&coordinate);

   // Prepare queries
   let inputs = vec!["example input 1", "example input 2"];
   let model = crate::LanguageModelType::default(); 
   let requests = expander.create_language_model_requests_at_agent_coordinate(
       &model,
       &coordinate,
       &inputs
   );

   // `requests` can now be submitted to a batch API of your choice
   ```

## Installation

Add this to your `Cargo.toml`:
```toml
[dependencies]
batch-mode-token-expander = "0.1"
```

Then import and use the crate in your Rust code as needed.
