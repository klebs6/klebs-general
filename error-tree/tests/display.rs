#![allow(unused_imports)]
#![allow(unused_variables)]
use error_tree::error_tree;
use std::fmt;

#[test]
fn test_basic_variant_with_display() {
    error_tree! {
        pub enum MyError {
            #[display("A simple error occurred")]
            SimpleError,
        }
    }

    let error = MyError::SimpleError;
    let formatted = format!("{}", error);
    assert_eq!(formatted, "A simple error occurred");
}

#[test]
fn test_basic_variant_without_display() {
    error_tree! {
        pub enum MyError {
            SimpleError,
        }
    }

    let error = MyError::SimpleError;
    let formatted = format!("{}", error);
    assert_eq!(formatted, "SimpleError");
}

#[test]
fn test_wrapped_variant_with_display() {
    error_tree! {
        pub enum MyError {
            #[display("IO error occurred: {inner}")]
            IOError(std::io::Error),
        }
    }

    let io_error = std::io::Error::new(std::io::ErrorKind::Other, "Disk not found");
    let error = MyError::IOError(io_error);
    let formatted = format!("{}", error);
    assert_eq!(formatted, "IO error occurred: Disk not found");
}

#[test]
fn test_wrapped_variant_without_display() {
    error_tree! {
        pub enum MyError {
            IOError(std::io::Error),
        }
    }

    let io_error = std::io::Error::new(std::io::ErrorKind::Other, "Disk not found");
    let error = MyError::IOError(io_error);
    let formatted = format!("{}", error);
    assert_eq!(formatted, "IOError: Custom { kind: Other, error: \"Disk not found\" }");
}

#[test]
fn test_struct_variant_with_display() {
    error_tree! {
        pub enum MyError {
            #[display("Data error occurred: {data}")]
            DataError { data: String },
        }
    }

    let error = MyError::DataError {
        data: "Invalid format".to_string(),
    };
    let formatted = format!("{}", error);
    assert_eq!(formatted, "Data error occurred: Invalid format");
}

#[test]
fn test_struct_variant_without_display() {
    error_tree! {
        pub enum MyError {
            DataError { data: String },
        }
    }

    let error = MyError::DataError {
        data: "Invalid format".to_string(),
    };
    let formatted = format!("{}", error);
    assert_eq!(formatted, "DataError");
}

#[test]
fn test_nested_errors_with_display() {
    error_tree! {
        pub enum OuterError {
            #[display("Outer error occurred: {inner}")]
            OuterError { inner: InnerError },
        }

        pub enum InnerError {
            #[display("Inner error occurred: {message}")]
            InnerError { message: String },
        }
    }

    let inner_error = InnerError::InnerError {
        message: "Something went wrong".to_string(),
    };
    let error = OuterError::OuterError { inner: inner_error };
    let formatted = format!("{}", error);
    assert_eq!(
        formatted,
        "Outer error occurred: Inner error occurred: Something went wrong"
    );
}

#[test]
fn test_variant_with_multiple_fields() {
    error_tree! {
        pub enum MyError {
            #[display("Error code {code}: {message}")]
            ComplexError { code: i32, message: String },
        }
    }

    let error = MyError::ComplexError {
        code: 404,
        message: "Not Found".to_string(),
    };
    let formatted = format!("{}", error);
    assert_eq!(formatted, "Error code 404: Not Found");
}

#[test]
fn test_display_trait_implementation() {
    error_tree! {
        pub enum MyError {
            #[display("An error occurred")]
            SimpleError,
            #[display("IO error occurred: {inner}")]
            IOError(std::io::Error),
            #[display("Data error with data: {data}")]
            DataError { data: String },
            OtherError,
        }
    }

    // Create instances of the errors
    let simple_error = MyError::SimpleError;
    let io_error = MyError::IOError(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Disk not found",
    ));
    let data_error = MyError::DataError {
        data: "Invalid format".to_string(),
    };
    let other_error = MyError::OtherError;

    // Format the errors
    let simple_error_str = format!("{}", simple_error);
    let io_error_str = format!("{}", io_error);
    let data_error_str = format!("{}", data_error);
    let other_error_str = format!("{}", other_error);

    // Check that the formatted strings match expectations
    assert_eq!(simple_error_str, "An error occurred");
    assert_eq!(io_error_str, "IO error occurred: Disk not found");
    assert_eq!(data_error_str, "Data error with data: Invalid format");
    // For the variant without a display attribute, check the default formatting
    assert_eq!(other_error_str, "OtherError");
}
