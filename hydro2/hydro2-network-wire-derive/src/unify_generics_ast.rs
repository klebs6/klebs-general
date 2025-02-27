// ---------------- [ File: src/unify_generics_ast.rs ]
crate::ix!();

/// Merges the `wire_generics` and `op_generics` purely in the AST.
/// Returns a new `syn::Generics` that includes all params + merged where-clauses.
pub fn unify_generics_ast(
    wire_generics: &syn::Generics,
    op_generics:   &syn::Generics,
) -> syn::Generics {
    // Clone the wire generics to start.
    let mut merged = wire_generics.clone();

    // Append each of op_generics' parameters to merged.params.
    for param in &op_generics.params {
        merged.params.push(param.clone());
    }

    // Merge any where-clauses. If both sides have them, combine them; otherwise, use whichever exists.
    match (&mut merged.where_clause, &op_generics.where_clause) {
        (Some(wire_wc), Some(op_wc)) => {
            wire_wc.predicates.extend(op_wc.predicates.clone());
        }
        (None, Some(op_wc)) => {
            merged.where_clause = Some(op_wc.clone());
        }
        // If op has no where-clause, do nothing; if wire had none, it stays None.
        _ => {}
    }

    merged
}

#[cfg(test)]
mod test_unify_generics_ast {
    use super::*; // so we can see unify_generics_ast, merge_generics, etc.
    use quote::ToTokens;
    use syn::{parse_str, DeriveInput, Generics};

    /// A small helper that parses a struct definition like `struct Dummy<T> where ...;`
    /// then returns `syn::Generics`. This lets us write test inputs conveniently.
    fn parse_generics_str(input: &str) -> Generics {
        // For example, if input == "<T> where T: Clone",
        // we'll build `struct Dummy<T> where T: Clone {};`
        let code = format!("struct Dummy{};", input);
        match parse_str::<DeriveInput>(&code) {
            Ok(ast) => ast.generics,
            Err(e) => panic!("Failed parsing generics from {}: {}", code, e),
        }
    }

    /// Converts a `syn::Generics` to a debug string, normalizing whitespace
    /// for simpler substring checks.
    fn normalize_gens(gens: &Generics) -> String {
        gens.into_token_stream().to_string().replace('\n', " ")
    }

    /// Helper to extract just the `.where_clause` from the generics, if any,
    /// and convert that to a single-line string for easy substring checking.
    fn where_clause_string(gens: &Generics) -> String {
        match &gens.where_clause {
            Some(wc) => wc.into_token_stream().to_string().replace('\n', " "),
            None => String::new(),
        }
    }

    /// Merges `wire` and `op` with `unify_generics_ast`, returning the new Generics.
    /// Then you can check `.params` and `.where_clause`.
    fn merge_ast(wire: &Generics, op: &Generics) -> Generics {
        let merged = unify_generics_ast(wire, op);
        // (You could log or debug-print here if you like)
        merged
    }

    // -------------------------------------------------
    //  The actual test cases
    // -------------------------------------------------

    #[test]
    fn unify_no_generics() {
        // No generics on either side => merged should be empty
        let wire = parse_generics_str("");
        let op   = parse_generics_str("");

        let merged = merge_ast(&wire, &op);
        assert!(merged.params.is_empty(), "Expected no params");
        assert!(merged.where_clause.is_none(), "Expected no where-clause");
    }

    #[test]
    fn unify_wire_generics_only() {
        // wire has <T: Clone + Default>, op has none
        let wire = parse_generics_str("<T: Clone + Default>");
        let op   = parse_generics_str("");

        let merged = merge_ast(&wire, &op);
        // We expect exactly T: Clone+Default, no extra param
        assert_eq!(merged.params.len(), 1, "Should have 1 param (T)");
        let merged_str = normalize_gens(&merged);
        assert!(
            merged_str.contains("T : Clone + Default"),
            "Missing T:Clone+Default in merged generics; got: {}",
            merged_str
        );
        // no merged where-clause expected
        assert!(merged.where_clause.is_none());
    }

    #[test]
    fn unify_op_generics_only() {
        // wire has none, op has <U: Send, const N: usize>
        let wire = parse_generics_str("");
        let op   = parse_generics_str("<U: Send, const N: usize>");

        let merged = merge_ast(&wire, &op);
        assert_eq!(merged.params.len(), 2, "Should have U, N");
        let merged_str = normalize_gens(&merged);
        assert!(
            merged_str.contains("U : Send") && merged_str.contains("const N : usize"),
            "Missing U:Send or N in merged generics; got: {}",
            merged_str
        );
        assert!(merged.where_clause.is_none());
    }

    #[test]
    fn unify_type_and_const_generics() {
        // wire has <T, 'a, const M: usize>, op has <U, const N: usize>
        let wire = parse_generics_str("<T, 'a, const M: usize>");
        let op   = parse_generics_str("<U, const N: usize>");

        let merged = merge_ast(&wire, &op);
        // We should end up with T, 'a, M, U, N
        let merged_str = normalize_gens(&merged);
        assert!(merged_str.contains("'a"), "No 'a param found!");
        assert!(merged_str.contains("T"),  "No T param found!");
        assert!(merged_str.contains("M : usize"), "No M param!");
        assert!(merged_str.contains("U"), "No U param found!");
        assert!(merged_str.contains("N : usize"), "No N param!");
        assert!(merged.where_clause.is_none());
    }

    #[test]
    fn unify_where_clauses() {
        // wire => <T: Clone> where T: Default
        // op   => <U: Send>  where U: Sync
        let wire = parse_generics_str("<T: Clone> where T: Default");
        let op   = parse_generics_str("<U: Send> where U: Sync");

        let merged = merge_ast(&wire, &op);
        // We expect T:Clone, U:Send as params
        let merged_str = normalize_gens(&merged);
        assert!(merged_str.contains("T : Clone") && merged_str.contains("U : Send"),
            "Expected T:Clone, U:Send in generics; got {}", merged_str
        );
        // The where clause should have T: Default, U: Sync
        let wc_str = where_clause_string(&merged);
        assert!(wc_str.contains("T : Default") && wc_str.contains("U : Sync"),
            "Expected T:Default, U:Sync in where-clause; got {}", wc_str
        );
    }

    #[test]
    fn unify_complex_generics() {
        // wire => <'a, A, B: 'a + Debug> where A: Iterator<Item=i32>, B: Clone
        let wire_str = r#"
            <'a, A, B: 'a + std::fmt::Debug>
            where
                A: core::iter::Iterator<Item = i32>,
                B: core::clone::Clone
        "#;
        let wire = parse_generics_str(wire_str);

        // op => <C: Eq + Ord, const N: usize> where C: Copy
        let op_str = r#"
            <C: ::core::cmp::Eq + ::core::cmp::Ord, const N: usize>
            where
                C: ::core::marker::Copy
        "#;
        let op = parse_generics_str(op_str);

        let merged = merge_ast(&wire, &op);
        let merged_str = normalize_gens(&merged);
        info!("Merged generics => {}", merged_str);

        // Check that 'a, A, B, C, N appear
        // B has 'a + Debug
        assert!(merged_str.contains("'a"), "No 'a found in merged generics");
        assert!(merged_str.contains("A"),  "No A found");
        assert!(merged_str.contains("B : 'a") && merged_str.contains("Debug"), "B missing 'a or Debug");
        assert!(merged_str.contains("C : ::core::cmp :: Eq + ::core::cmp :: Ord") ||
                merged_str.contains("C : :: core :: cmp :: Eq + :: core :: cmp :: Ord"),
            "C missing Eq+Ord"
        );
        assert!(merged_str.contains("N : usize"), "No N param found");

        // Check where-clause => A: Iterator<Item=i32>, B: Clone, C: Copy
        let wc_str = where_clause_string(&merged);
        info!("Merged where-clause => {}", wc_str);

        assert!(wc_str.contains("A : core :: iter :: Iterator") && wc_str.contains("Item = i32"),
            "Missing A:Iterator<Item=i32> in where-clause"
        );
        assert!(wc_str.contains("B : core :: clone :: Clone"),
            "Missing B:Clone in where-clause"
        );
        assert!(wc_str.contains("C : :: core :: marker :: Copy") || wc_str.contains("C : core :: marker :: Copy"),
            "Missing C:Copy in where-clause"
        );
    }

    #[test]
    fn unify_conflicting_params_not_deduplicated() {
        // Currently, unify_generics_ast doesn't deduplicate params by ident. 
        // If both sides have <T>, it just appends them => <T, T>.
        let wire = parse_generics_str("<T>");
        let op   = parse_generics_str("<T>");

        let merged = merge_ast(&wire, &op);
        assert_eq!(merged.params.len(), 2, "Expected T, T repeated (no dedup).");
    }

    #[test]
    fn parse_error_handling() {
        // For demonstration, show that parse_generics_str fails on malformed input
        let input = "<T where T: Clone"; // missing bracket
        let code = format!("struct Dummy{};", input);
        let parse_res = parse_str::<DeriveInput>(&code);
        assert!(parse_res.is_err(), "Expected parse error for malformed input");
    }
}
