#![allow(unused_variables)]
use error_tree::error_tree;

#[derive(Debug)]
struct NonPartialEqType;

error_tree! {
    #[derive(PartialEq)]
    enum TestError {
        SimpleError,
        #[cmp_neq]
        ComplexError(NonPartialEqType),
        DataError {
            data: String,
        },
    }
}

fn main() {
    let e1 = TestError::SimpleError;
    let e2 = TestError::SimpleError;
    assert_eq!(e1, e2); // Should be true

    let e3 = TestError::ComplexError(NonPartialEqType);
    let e4 = TestError::ComplexError(NonPartialEqType);
    assert_ne!(e3, e4); // Should be true due to #[cmp_neq]

    let e5 = TestError::DataError { data: "test".into() };
    let e6 = TestError::DataError { data: "test".into() };
    assert_eq!(e5, e6); // Should be true
}

