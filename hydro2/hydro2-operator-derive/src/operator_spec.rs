// ---------------- [ File: hydro2-operator-derive/src/operator_spec.rs ]
//! spec.rs — `OperatorSpec` definition and parsing logic.

crate::ix!();

#[derive(Builder, Getters, Debug)]
#[getset(get = "pub")]
#[builder(setter(into))]
pub struct OperatorSpec {
    execute_fn:  Path,
    opcode_expr: Path,
    inputs:      Vec<Type>,
    outputs:     Vec<Type>,
}

impl OperatorSpec {

    /// Returns the input type at the given index. If the index is out of range,
    /// it returns the unit type `()`.
    pub fn get_input(&self, idx: usize) -> Option<Type> {
        match self.inputs.get(idx) {
            Some(t) => Some(t.clone()),
            None => if idx < 4 {
                Some(syn::parse_quote!(()))
            } else {
                None
            },
        }
    }

    /// Returns the output type at the given index. If the index is out of range,
    /// it returns the unit type `()`.
    pub fn get_output(&self, idx: usize) -> Option<Type> {
        match self.outputs.get(idx) {
            Some(t) => Some(t.clone()),
            None => if idx < 4 {
                Some(syn::parse_quote!(()))
            } else {
                None
            },
        }
    }

    /// Parse the first `#[operator(...)]` attribute on the struct, or return an error.
    pub fn parse_operator_attrs(
        attrs:       &[Attribute],
        struct_span: Span,

    ) -> Result<Self, OperatorSpecError> {

        // 1) Find `#[operator(...)]` attribute
        let operator_attr = match attrs.iter().find(|a| a.path().is_ident("operator")) {
            Some(a) => a,
            None => {
                return Err(OperatorSpecError::missing_operator_attribute(&struct_span));
            }
        };

        // 2) Manually parse the token stream inside the (...) of `#[operator(...)]`.
        //    We'll expect zero or more `ident = "string"` pairs, separated by commas.
        //    Example: execute="foo", opcode="BasicOpCode::Bar", input0="&[X]"
        let kv_pairs = match operator_attr.parse_args_with(OperatorKeyValues::parse) {
            Ok(kv) => kv.pairs,
            Err(_syn_err) => {
                // If the entire attribute is malformed, treat as invalid key or something generic
                return Err(OperatorSpecError::invalid_key(&operator_attr.span()));
            }
        };

        let mut execute_fn: Option<Path> = None;
        let mut opcode_expr: Option<Path> = None;
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();

        // 3) Process each key="value" pair
        for (key_ident, lit_val) in kv_pairs {
            // We require the value to be a string literal
            let lit_str = match lit_val {
                Lit::Str(s) => s,
                _ => {
                    return Err(OperatorSpecError::non_string_key_value(&lit_val.span()));
                }
            };

            let key = key_ident.to_string();
            match key.as_str() {
                "execute" => {
                    if execute_fn.is_some() {
                        return Err(OperatorSpecError::duplicate_execute_key(&key_ident.span()));
                    }
                    let path_val = syn::parse_str::<Path>(&lit_str.value())
                        .map_err(|_| OperatorSpecError::could_not_parse_execute_path(&lit_str.span()))?;
                    execute_fn = Some(path_val);
                }
                "opcode" => {
                    if opcode_expr.is_some() {
                        return Err(OperatorSpecError::duplicate_opcode_key(&key_ident.span()));
                    }
                    let path_val = syn::parse_str::<Path>(&lit_str.value())
                        .map_err(|_| OperatorSpecError::could_not_parse_opcode_path(&lit_str.span()))?;
                    opcode_expr = Some(path_val);
                }
                k if k.starts_with("input") => {
                    let idx_str = &k["input".len()..];
                    let idx = idx_str.parse::<usize>()
                        .map_err(|_| OperatorSpecError::invalid_key(&key_ident.span()))?;
                    if idx > 3 {
                        return Err(OperatorSpecError::more_than_four_inputs(&key_ident.span()));
                    }
                    // must match current inputs.len()
                    if idx != inputs.len() {
                        return Err(OperatorSpecError::invalid_input_order(&key_ident.span()));
                    }
                    let ty_val = syn::parse_str::<Type>(&lit_str.value())
                        .map_err(|_| OperatorSpecError::could_not_parse_input_type(&lit_str.span()))?;
                    inputs.push(ty_val);
                }
                k if k.starts_with("output") => {
                    let idx_str = &k["output".len()..];
                    let idx = idx_str.parse::<usize>()
                        .map_err(|_| OperatorSpecError::invalid_key(&key_ident.span()))?;
                    if idx > 3 {
                        return Err(OperatorSpecError::more_than_four_outputs(&key_ident.span()));
                    }
                    // must match outputs.len()
                    if idx != outputs.len() {
                        return Err(OperatorSpecError::invalid_output_order(&key_ident.span()));
                    }
                    let ty_val = syn::parse_str::<Type>(&lit_str.value())
                        .map_err(|_| OperatorSpecError::could_not_parse_output_type(&lit_str.span()))?;
                    outputs.push(ty_val);
                }
                _ => {
                    return Err(OperatorSpecError::invalid_key(&key_ident.span()));
                }
            }
        }

        // 4) Check required fields
        let Some(execute_fn) = execute_fn else {
            return Err(OperatorSpecError::missing_execute_fn(&operator_attr.span()));
        };
        let Some(opcode_expr) = opcode_expr else {
            return Err(OperatorSpecError::missing_opcode(&operator_attr.span()));
        };

        Ok(Self {
            execute_fn,
            opcode_expr,
            inputs,
            outputs,
        })
    }
}

#[cfg(test)]
mod operator_spec_test {
    use super::*;
    use crate::errors::{OperatorSpecErrorKind, OperatorSpecError};
    use syn::parse_quote;
    use proc_macro2::Span;
    use syn::Attribute;

    fn span() -> Span {
        Span::call_site()
    }

    #[test]
    fn parse_valid_operator_attribute_single_input_output() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[operator(execute="foo", opcode="BasicOpCode::Bar", input0="&[MyInput]", output0="Vec<Out>")]
        }];
        let spec = OperatorSpec::parse_operator_attrs(&attrs, span()).expect("Expected success");
        assert_eq!(spec.inputs.len(), 1);
        assert_eq!(spec.outputs.len(), 1);
        assert_eq!(spec.inputs[0].to_token_stream().to_string(), "& [MyInput]");
        assert_eq!(spec.outputs[0].to_token_stream().to_string(), "Vec < Out >");
    }

    #[test]
    fn parse_missing_operator_attribute() {
        let attrs: Vec<Attribute> = vec![];
        let err = OperatorSpec::parse_operator_attrs(&attrs, span()).unwrap_err();
        assert_eq!(*err.kind(), OperatorSpecErrorKind::MissingOperatorAttribute);
    }

    #[test]
    fn parse_duplicate_execute() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[operator(
                execute="foo",
                execute="bar",
                opcode="BasicOpCode::Baz"
            )]
        }];
        let err = OperatorSpec::parse_operator_attrs(&attrs, span()).unwrap_err();
        assert_eq!(*err.kind(), OperatorSpecErrorKind::DuplicateExecuteKey);
    }

    #[test]
    fn parse_duplicate_opcode() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[operator(
                execute="foo",
                opcode="BasicOpCode::Alpha",
                opcode="BasicOpCode::Beta"
            )]
        }];
        let err = OperatorSpec::parse_operator_attrs(&attrs, span()).unwrap_err();
        assert_eq!(*err.kind(), OperatorSpecErrorKind::DuplicateOpcodeKey);
    }

    #[test]
    fn parse_missing_execute() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[operator(opcode="BasicOpCode::Bar")]
        }];
        let err = OperatorSpec::parse_operator_attrs(&attrs, span()).unwrap_err();
        assert_eq!(*err.kind(), OperatorSpecErrorKind::MissingExecuteFn);
    }

    #[test]
    fn parse_missing_opcode() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[operator(execute="something")]
        }];
        let err = OperatorSpec::parse_operator_attrs(&attrs, span()).unwrap_err();
        assert_eq!(*err.kind(), OperatorSpecErrorKind::MissingOpcode);
    }

    #[test]
    fn parse_input_out_of_order() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[operator(
                execute="some_fn",
                opcode="BasicOpCode::Test",
                input0="Foo",
                input2="Bar"
            )]
        }];
        let err = OperatorSpec::parse_operator_attrs(&attrs, span()).unwrap_err();
        assert_eq!(*err.kind(), OperatorSpecErrorKind::InvalidInputOrder);
    }

    #[test]
    fn parse_output_out_of_order() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[operator(
                execute="some_fn",
                opcode="BasicOpCode::Test",
                output0="Foo",
                output2="Bar"
            )]
        }];
        let err = OperatorSpec::parse_operator_attrs(&attrs, span()).unwrap_err();
        assert_eq!(*err.kind(), OperatorSpecErrorKind::InvalidOutputOrder);
    }

    #[test]
    fn parse_more_than_four_inputs() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[operator(
                execute="execute_fn",
                opcode="BasicOpCode::Test",
                input0="A",
                input1="B",
                input2="C",
                input3="D",
                input4="E"
            )]
        }];
        let err = OperatorSpec::parse_operator_attrs(&attrs, span()).unwrap_err();
        assert_eq!(*err.kind(), OperatorSpecErrorKind::MoreThanFourInputs);
    }

    #[test]
    fn parse_more_than_four_outputs() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[operator(
                execute="execute_fn",
                opcode="BasicOpCode::Test",
                output0="A",
                output1="B",
                output2="C",
                output3="D",
                output4="E"
            )]
        }];
        let err = OperatorSpec::parse_operator_attrs(&attrs, span()).unwrap_err();
        assert_eq!(*err.kind(), OperatorSpecErrorKind::MoreThanFourOutputs);
    }

    #[test]
    fn parse_non_string_value() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[operator(execute=123)]
        }];
        let err = OperatorSpec::parse_operator_attrs(&attrs, span()).unwrap_err();
        assert_eq!(*err.kind(), OperatorSpecErrorKind::NonStringKeyValue);
    }

    #[test]
    fn parse_could_not_parse_execute_path() {
        // e.g. `execute="fn("` is invalid syntax for a path
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[operator(execute="fn(",
                       opcode="BasicOpCode::Ok")]
        }];
        let err = OperatorSpec::parse_operator_attrs(&attrs, span()).unwrap_err();
        assert_eq!(*err.kind(), OperatorSpecErrorKind::CouldNotParseExecutePath);
    }

    #[test]
    fn parse_could_not_parse_opcode_path() {
        // e.g. `opcode="BasicOpCode(("` is invalid syntax
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[operator(execute="some_fn",
                       opcode="BasicOpCode((")]
        }];
        let err = OperatorSpec::parse_operator_attrs(&attrs, span()).unwrap_err();
        assert_eq!(*err.kind(), OperatorSpecErrorKind::CouldNotParseOpcodePath);
    }

    #[test]
    fn parse_could_not_parse_input_type() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[operator(
                execute="foo",
                opcode="BasicOpCode::Ok",
                input0="&["
            )]
        }];
        let err = OperatorSpec::parse_operator_attrs(&attrs, span()).unwrap_err();
        assert_eq!(*err.kind(), OperatorSpecErrorKind::CouldNotParseInputType);
    }

    #[test]
    fn parse_could_not_parse_output_type() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[operator(
                execute="foo",
                opcode="BasicOpCode::Ok",
                output0="Vec<"
            )]
        }];
        let err = OperatorSpec::parse_operator_attrs(&attrs, span()).unwrap_err();
        assert_eq!(*err.kind(), OperatorSpecErrorKind::CouldNotParseOutputType);
    }

    #[test]
    fn parse_valid_operator_attribute_many_inputs_outputs() {
        let attrs: Vec<Attribute> = vec![parse_quote! {
            #[operator(
                execute="foo",
                opcode="BasicOpCode::Bar",
                input0="A",
                input1="B",
                input2="C",
                input3="D",
                output0="X",
                output1="Y",
                output2="Z"
            )]
        }];
        let spec = OperatorSpec::parse_operator_attrs(&attrs, span()).unwrap();
        assert_eq!(spec.inputs().len(), 4);
        assert_eq!(spec.outputs().len(), 3);
    }
}
