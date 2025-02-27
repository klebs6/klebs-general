// ---------------- [ File: hydro2-network-wire-derive/src/merge_generics.rs ]
crate::ix!();

/// Merge two generics: wire + op => combine the params + the where clauses
pub fn merge_generics(wire: &Generics, op: &Generics) -> Generics {
    info!("merge_generics: START");
    info!("  wire generics = {}", wire.into_token_stream());
    info!("  op   generics = {}", op.into_token_stream());

    let mut result = wire.clone();
    info!("  cloned wire generics into result => {}", result.clone().into_token_stream());

    for param in &op.params {
        info!("  appending op param => {}", param.clone().into_token_stream());
        result.params.push(param.clone());
    }

    if let Some(op_wc) = &op.where_clause {
        info!("  op has where-clause => {}", op_wc.into_token_stream());
        if let Some(wire_wc) = &mut result.where_clause {
            info!("  wire also has where-clause => {}", wire_wc.into_token_stream());
            wire_wc.predicates.extend(op_wc.predicates.clone());
            info!("  merged wire+op where-clause => {}", wire_wc.into_token_stream());
        } else {
            info!("  wire has no where-clause, using op's where-clause in result");
            result.where_clause = Some(op_wc.clone());
        }
    } else {
        info!("  op has no where-clause, leaving wire's alone");
    }

    info!("merge_generics: FINISH => {}", result.clone().into_token_stream());
    result
}

#[cfg(test)]
mod test_merge_generics {

    /*!
     * This module exhaustively tests the `merge_generics` function.
     * We parse representative sets of `syn::Generics` (including type, lifetime,
     * and const parameters, as well as various where-clause forms), then merge
     * them via `merge_generics` and verify the correct behavior.
     */
    use super::*;
    use syn::{parse_str, DeriveInput};

    /// Helper: parse a `struct` definition with a given generics string,
    /// then return the `Generics`. E.g. if `input` == "<T, U>", we parse
    /// `struct Dummy<T, U> {};` for test usage.
    fn parse_generics_str(input: &str) -> Result<Generics, syn::Error> {
        info!("parse_generics_str: START => '{}'", input);
        let code = format!("struct Dummy{};", input);
        info!("  constructing dummy struct => {}", code);
        match parse_str::<DeriveInput>(&code) {
            Ok(ast) => {
                let g = &ast.generics;
                info!("  parse succeeded => {}", g.into_token_stream());
                Ok(g.clone())
            }
            Err(err) => {
                info!("  parse failed => {}", err);
                Err(err)
            }
        }
    }

    #[test]
    fn test_merge_empty_generics() {
        info!("test_merge_empty_generics: START");
        let wire = parse_generics_str("").unwrap();
        let op = parse_generics_str("").unwrap();

        let merged = merge_generics(&wire, &op);
        info!("test_merge_empty_generics: merged => {}", merged.clone().into_token_stream());

        assert!(merged.params.is_empty(), "Expected no generic parameters.");
        assert!(merged.where_clause.is_none(), "Expected no where-clause.");
    }

    #[test]
    fn test_merge_type_params() {
        info!("test_merge_type_params: START");
        // wire has <A, B>
        let wire = parse_generics_str("<A, B>").unwrap();
        // op has <C>
        let op = parse_generics_str("<C>").unwrap();

        let merged = merge_generics(&wire, &op);
        let merged_params: Vec<String> = merged
            .params
            .iter()
            .map(|gp| gp.into_token_stream().to_string())
            .collect();

        info!("  merged params = {:?}", merged_params);
        // Expect <A, B, C>
        assert_eq!(merged_params, vec!["A", "B", "C"]);
        assert!(merged.where_clause.is_none(), "Expected no where-clause");
    }

    #[test]
    fn test_merge_lifetime_params() {
        info!("test_merge_lifetime_params: START");
        // wire has <'a, T>
        let wire = parse_generics_str("<'a, T>").unwrap();
        // op has <'b>
        let op = parse_generics_str("<'b>").unwrap();

        let merged = merge_generics(&wire, &op);
        let merged_params: Vec<String> = merged
            .params
            .iter()
            .map(|gp| gp.into_token_stream().to_string())
            .collect();
        info!("  merged params = {:?}", merged_params);

        // Expect <'a, T, 'b>
        assert_eq!(merged_params, vec!["'a", "T", "'b"]);
    }

    #[test]
    fn test_merge_const_params() {
        info!("test_merge_const_params: START");
        // wire has <T, const N: usize>
        let wire = parse_generics_str("<T, const N: usize>").unwrap();
        // op has <const M: u8>
        let op = parse_generics_str("<const M: u8>").unwrap();

        let merged = merge_generics(&wire, &op);
        let merged_params: Vec<String> = merged
            .params
            .iter()
            .map(|gp| gp.into_token_stream().to_string())
            .collect();
        info!("  merged params = {:?}", merged_params);

        // Expect <T, const N: usize, const M: u8>
        assert_eq!(merged_params, vec!["T", "const N : usize", "const M : u8"]);
    }

    #[test]
    fn test_merge_single_where_clause() {
        info!("test_merge_single_where_clause: START");
        // wire has <T> where T: Clone
        let wire = parse_generics_str("<T> where T: Clone").unwrap();
        // op has <U> no where-clause
        let op = parse_generics_str("<U>").unwrap();

        let merged = merge_generics(&wire, &op);
        info!("  merged => {}", merged.clone().into_token_stream());

        assert_eq!(merged.params.len(), 2, "Expected T, U");
        match &merged.where_clause {
            Some(clause) => {
                let clause_str = clause.into_token_stream().to_string();
                assert!(
                    clause_str.contains("T : Clone"),
                    "Expected 'T : Clone' in merged where."
                );
            }
            None => panic!("Expected where-clause with T: Clone"),
        }
    }

    #[test]
    fn test_merge_both_where_clauses() {
        info!("test_merge_both_where_clauses: START");
        // wire has <T> where T: Clone
        let wire = parse_generics_str("<T> where T: Clone").unwrap();
        // op has <U> where U: Copy
        let op = parse_generics_str("<U> where U: Copy").unwrap();

        let merged = merge_generics(&wire, &op);
        info!("  merged => {}", merged.clone().into_token_stream());

        let merged_params: Vec<String> = merged
            .params
            .iter()
            .map(|gp| gp.into_token_stream().to_string())
            .collect();
        assert_eq!(merged_params, vec!["T", "U"]);
        match &merged.where_clause {
            Some(clause) => {
                let clause_str = clause.into_token_stream().to_string();
                assert!(clause_str.contains("T : Clone"));
                assert!(clause_str.contains("U : Copy"));
            }
            None => panic!("Expected merged where-clause with T:Clone, U:Copy"),
        }
    }

    #[test]
    fn test_merge_repeated_params() {
        info!("test_merge_repeated_params: START");
        // wire has <T, U>
        let wire = parse_generics_str("<T, U>").unwrap();
        // op also has <T, U>
        let op = parse_generics_str("<T, U>").unwrap();

        let merged = merge_generics(&wire, &op);
        info!("  merged => {}", merged.clone().into_token_stream());

        let merged_params: Vec<String> = merged
            .params
            .iter()
            .map(|gp| gp.into_token_stream().to_string())
            .collect();

        // Currently, we do not deduplicate, so expect <T, U, T, U>.
        assert_eq!(merged_params, vec!["T", "U", "T", "U"]);
    }

    #[test]
    fn test_merge_mixed_params_and_where() {
        info!("test_merge_mixed_params_and_where: START");
        // wire has <'a, T, const M: u8> where T: PartialEq
        let wire = parse_generics_str("<'a, T, const M: u8> where T: PartialEq").unwrap();

        // op has <'b, U, const N: usize> where 'b: 'static, U: Clone
        let op = parse_generics_str("<'b, U, const N: usize> where 'b: 'static, U: Clone").unwrap();

        let merged = merge_generics(&wire, &op);
        info!("  merged => {}", merged.clone().into_token_stream());

        let merged_params: Vec<String> = merged
            .params
            .iter()
            .map(|gp| gp.into_token_stream().to_string())
            .collect();
        assert_eq!(
            merged_params,
            vec!["'a", "T", "const M : u8", "'b", "U", "const N : usize"],
            "Expected all wire + op parameters in correct order."
        );

        match &merged.where_clause {
            Some(clause) => {
                let clause_str = clause.into_token_stream().to_string();
                info!("  merged where-clause => {}", clause_str);
                assert!(clause_str.contains("T : PartialEq"));
                assert!(clause_str.contains("'b : 'static"));
                assert!(clause_str.contains("U : Clone"));
            }
            None => panic!("Expected a merged where-clause, but found none."),
        }
    }
}
