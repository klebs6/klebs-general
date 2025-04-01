crate::ix!();

pub const LEGAL_CATEGORIES: &[&'static str] = &[
    "accessibility",
    "aerospace",
    "algorithms",
    "api-bindings",
    "asynchronous",
    "authentication",
    "caching",
    "command-line-interface",
    "command-line-utilities",
    "compilers",
    "compression",
    "computer-vision",
    "concurrency",
    "config",
    "cryptography",
    "data-structures",
    "database",
    "database-implementations",
    "date-and-time",
    "development-tools",
    "email",
    "embedded",
    "emulators",
    "encoding",
    "external-ffi-bindings",
    "filesystem",
    "finance",
    "game-development",
    "game-engines",
    "games",
    "graphics",
    "gui",
    "hardware-support",
    "internationalization",
    "localization",
    "mathematics",
    "memory-management",
    "multimedia",
    "network-programming",
    "no-std",
    "os",
    "parser-implementations",
    "parsing",
    "rendering",
    "rust-patterns",
    "science",
    "simulation",
    "template-engine",
    "text-editors",
    "text-processing",
    "value-formatting",
    "virtualization",
    "visualization",
    "wasm",
    "web-programming",
];

/// TODO: make sure this is *exactly* what cratesio needs
pub fn clean_cratesio_keyword(x: &str) -> String {

    trace!("clean_keyword: received keyword: {:?}", x);

    // Retain only alphanumeric characters and convert to lowercase.
    let cleaned: String = x
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_lowercase();

    debug!("clean_keyword: cleaned keyword: {:?}", cleaned);
    cleaned
}

#[cfg(test)]
mod test_clean_keyword {
    use super::*;

    #[traced_test]
    fn test_clean_keyword_removes_special_chars() {
        let input = "Hello, World!";
        let output = clean_cratesio_keyword(input);
        // Expected: "Hello, World!" -> "helloworld"
        assert_eq!(output, "helloworld");
    }

    #[traced_test]
    fn test_clean_keyword_already_clean() {
        let input = "rustlang123";
        let output = clean_cratesio_keyword(input);
        // Expected: unchanged except lowercase.
        assert_eq!(output, "rustlang123");
    }

    #[traced_test]
    fn test_clean_keyword_with_spaces() {
        let input = "  Rust   Language ";
        let output = clean_cratesio_keyword(input);
        // Expected: spaces removed, becomes "rustlanguage"
        assert_eq!(output, "rustlanguage");
    }
}
