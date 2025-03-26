// ---------------- [ File: src/check_panic_message.rs ]
crate::ix!();

// Inside the macro-generated code
pub fn check_panic_message(err: Box<dyn std::any::Any + Send>, expected_message: &str) {

    let panic_message = if let Some(s) = err.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = err.downcast_ref::<String>() {
        s.clone()
    } else {
        "Unknown panic message".to_string()
    };

    if panic_message == expected_message {
        // Test passes
    } else {
        panic!("Unexpected panic occurred: {}", panic_message);
    }
}
