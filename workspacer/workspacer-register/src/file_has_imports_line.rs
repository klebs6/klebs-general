// ---------------- [ File: src/file_has_imports_line.rs ]
crate::ix!();

pub fn file_has_imports_line(parsed_file: &SourceFile) -> bool {
    trace!("Entering file_has_imports_line");
    let has_imports = parsed_file.items().any(|item| is_imports_line(&item));
    debug!("file_has_imports_line={}", has_imports);
    trace!("Exiting file_has_imports_line");
    has_imports
}

#[cfg(test)]
mod test_file_has_imports_line {
    use super::*;
    use ra_ap_syntax::{Edition, SourceFile};
    use tracing::{trace, debug};

    fn parse_source(text: &str) -> SourceFile {
        SourceFile::parse(text, Edition::Edition2021).tree()
    }

    /// 1) Empty file => no imports line
    #[traced_test]
    fn test_empty_file() {
        trace!("test_empty_file for file_has_imports_line");
        let file = parse_source("");
        let result = file_has_imports_line(&file);
        debug!("Result={}", result);
        assert!(!result, "Empty file => false");
    }

    /// 2) Basic mod or fn => no imports
    #[traced_test]
    fn test_no_imports() {
        trace!("test_no_imports for file_has_imports_line");
        let src = r#"
fn foo() {}
pub mod bar;
"#;
        let file = parse_source(src);
        let result = file_has_imports_line(&file);
        debug!("Result={}", result);
        assert!(!result, "No recognized imports => false");
    }

    /// 3) Single recognized imports line => true
    #[traced_test]
    fn test_single_imports_line() {
        trace!("test_single_imports_line for file_has_imports_line");
        let src = r#"
#[macro_use] mod imports; use imports::*;
"#;
        let file = parse_source(src);
        let result = file_has_imports_line(&file);
        debug!("Result={}", result);
        assert!(result, "Should detect the recognized imports line => true");
    }

    /// 4) Multiple items, only one is recognized => true
    #[traced_test]
    fn test_mixed_items_with_import() {
        trace!("test_mixed_items_with_import for file_has_imports_line");
        let src = r#"
fn alpha() {}

#[macro_use] mod imports; use imports::*;

fn beta() {}
"#;
        let file = parse_source(src);
        let result = file_has_imports_line(&file);
        debug!("Result={}", result);
        assert!(result, "We do have an imports line among them => true");
    }

    /// 5) Another 'use something_else;' => not recognized if it doesn't mention imports::
    #[traced_test]
    fn test_non_imports_use() {
        trace!("test_non_imports_use for file_has_imports_line");
        let src = r#"
use something_else::*;

fn main() {}
"#;
        let file = parse_source(src);
        let result = file_has_imports_line(&file);
        debug!("Result={}", result);
        assert!(!result, "A use line that doesn't mention imports::* => not recognized => false");
    }
}
