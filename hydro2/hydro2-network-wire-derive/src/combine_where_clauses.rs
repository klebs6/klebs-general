// ---------------- [ File: hydro2-network-wire-derive/src/combine_where_clauses.rs ]
crate::ix!();

/// Combine two `Option<&WhereClause>` => a new `quote!{ where ... }`
pub fn combine_where_clauses(
    w: Option<&WhereClause>,
    o: Option<&WhereClause>,
) -> TokenStream
{
    info!("combine_where_clauses: START");
    match (w, o) {
        (None, None) => {
            info!("combine_where_clauses: both None, returning empty");
            quote! {}
        },
        (Some(a), None) => {
            info!("combine_where_clauses: left has where-clause => {}", quote!{#a}.to_string());
            info!("combine_where_clauses: right is None, returning left");
            quote! { #a }
        },
        (None, Some(b)) => {
            info!("combine_where_clauses: left is None");
            info!("combine_where_clauses: right has where-clause => {}", quote!{#b}.to_string());
            quote! { #b }
        },
        (Some(a), Some(b)) => {
            info!("combine_where_clauses: merging both sides");
            let a_preds = &a.predicates;
            let b_preds = &b.predicates;
            let merged = quote! {
                where #a_preds, #b_preds
            };
            info!("combine_where_clauses: result = {}", merged.to_string());
            merged
        }
    }
}

#[cfg(test)]
mod test_combine_where_clauses {
    use super::*;
    use syn::{parse_str, WhereClause};

    /// A small helper that attempts to parse a `where` clause from a string.
    /// Returns `None` on parse failure rather than panicking or unwrapping.
    fn try_parse_where_clause(input: &str) -> Option<WhereClause> {
        info!("try_parse_where_clause: attempting to parse '{}'", input);
        match parse_str::<WhereClause>(input.trim()) {
            Ok(wc) => {
                info!("  parse succeeded => {:?}", wc);
                Some(wc)
            },
            Err(e) => {
                info!("  parse failed => {}", e);
                None
            },
        }
    }

    /// Basic whitespace normalization for test comparisons.
    fn normalize_ts(ts: &TokenStream) -> String {
        ts.to_string()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[test]
    fn none_none_returns_empty() {
        info!("none_none_returns_empty: START");
        let result = combine_where_clauses(None, None);
        let result_str = normalize_ts(&result);
        assert!(result_str.trim().is_empty(), "Expected empty, got: {}", result_str);
    }

    #[test]
    fn some_none_returns_first() {
        info!("some_none_returns_first: START");
        let maybe_wc = try_parse_where_clause("where A: Copy");
        match maybe_wc {
            Some(wc) => {
                let result = combine_where_clauses(Some(&wc), None);
                let result_str = normalize_ts(&result);
                assert!(
                    result_str.contains("where A : Copy"),
                    "Expected `where A : Copy`, got: {}",
                    result_str
                );
            }
            None => {
                panic!("Could not parse `where A: Copy` into a WhereClause");
            }
        }
    }

    #[test]
    fn none_some_returns_second() {
        info!("none_some_returns_second: START");
        let maybe_wc = try_parse_where_clause("where B: Clone + Debug");
        match maybe_wc {
            Some(wc) => {
                let result = combine_where_clauses(None, Some(&wc));
                let result_str = normalize_ts(&result);
                assert!(
                    result_str.contains("where B : Clone + Debug"),
                    "Expected `where B : Clone + Debug`, got: {}",
                    result_str
                );
            }
            None => {
                panic!("Could not parse `where B: Clone + Debug` into a WhereClause");
            }
        }
    }

    #[test]
    fn some_some_merges_both() {
        info!("some_some_merges_both: START");
        let first_wc = match try_parse_where_clause("where T: Clone") {
            Some(w) => w,
            None => panic!("Could not parse `where T: Clone`"),
        };
        let second_wc = match try_parse_where_clause("where U: Send, V: 'static") {
            Some(w) => w,
            None => panic!("Could not parse `where U: Send, V: 'static`"),
        };

        let result = combine_where_clauses(Some(&first_wc), Some(&second_wc));
        let result_str = normalize_ts(&result);
        assert!(
            result_str.contains("T : Clone") &&
            result_str.contains("U : Send") &&
            result_str.contains("V : 'static"),
            "Merged clause missing expected predicates; got: {}",
            result_str
        );
    }

    #[test]
    fn some_some_multiple_predicates_each_side() {
        info!("some_some_multiple_predicates_each_side: START");
        let left_wc = match try_parse_where_clause("where A: Copy + Clone, B: Ord") {
            Some(w) => w,
            None => panic!("Could not parse `where A: Copy + Clone, B: Ord`"),
        };
        let right_wc = match try_parse_where_clause("where C: Default, D: Eq + PartialOrd") {
            Some(w) => w,
            None => panic!("Could not parse `where C: Default, D: Eq + PartialOrd`"),
        };

        let result = combine_where_clauses(Some(&left_wc), Some(&right_wc));
        let result_str = normalize_ts(&result);
        assert!(
            result_str.contains("A : Copy + Clone") &&
            result_str.contains("B : Ord") &&
            result_str.contains("C : Default") &&
            result_str.contains("D : Eq + PartialOrd"),
            "Merged clause missing expected predicates; got: {}",
            result_str
        );
    }

    #[test]
    fn maintains_where_token_in_merged() {
        info!("maintains_where_token_in_merged: START");
        let left_wc = match try_parse_where_clause("where X: Iterator<Item = i32>") {
            Some(w) => w,
            None => panic!("Could not parse left clause"),
        };
        let right_wc = match try_parse_where_clause("where Y: Into<String>") {
            Some(w) => w,
            None => panic!("Could not parse right clause"),
        };

        let result = combine_where_clauses(Some(&left_wc), Some(&right_wc));
        let result_str = normalize_ts(&result);
        assert!(
            result_str.contains("where X : Iterator"),
            "Expected final output to contain `where X : Iterator`, got: {}",
            result_str
        );
    }
}
