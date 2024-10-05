#![allow(dead_code)]
use error_tree::error_tree;

#[test]
fn test_enum_is_clone() {

    // Use the macro to define an enum with #[derive(Clone)]
    error_tree! {
        #[derive(Clone)]
        pub enum MyError {
            SimpleError,
            SimpleStringError(String),
        }
    }

    // Function to assert that a type implements Clone
    fn assert_clone<T: Clone>() {}

    // This will fail to compile if MyError does not implement Clone
    assert_clone::<MyError>();

    // Create an instance of the enum and clone it
    let error = MyError::SimpleError;
    let cloned_error = error.clone();

    // Also test cloning a variant with data
    let io_error = MyError::SimpleStringError("oops".to_string());
    let cloned_io_error = io_error.clone();

    // If the code compiles and runs, then MyError implements Clone
    println!("Cloned errors successfully: {:?}, {:?}", cloned_error, cloned_io_error);
}
