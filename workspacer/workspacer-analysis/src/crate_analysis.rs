// ---------------- [ File: workspacer-analysis/src/crate_analysis.rs ]
crate::ix!();

#[derive(Debug,Clone)]
pub struct CrateAnalysis {

    /// Total size of files in bytes
    total_file_size:     u64,       

    /// Total number of lines of code
    total_lines_of_code: usize, 

    /// Total number of source files
    total_source_files:  usize,  

    /// Total number of test files
    total_test_files:    usize,    

    /// Size of the largest file in bytes
    largest_file_size:   u64,     

    /// Size of the smallest file in bytes
    smallest_file_size:  u64,    
}

impl CrateAnalysis {

    /// Constructs a `CrateAnalysis` by analyzing the files in the given `CrateHandle`
    pub async fn new(crate_handle: &(impl HasTestsDirectory + GetTestFiles + GetSourceFilesWithExclusions)) -> Result<Self, WorkspaceError> 
    {

        let mut total_file_size     = 0;
        let mut total_lines_of_code = 0;
        let mut total_source_files  = 0;
        let mut total_test_files    = 0;
        let mut largest_file_size   = 0;
        let mut smallest_file_size  = u64::MAX;

        // Analyze source files in `src/`
        let source_files = crate_handle.source_files_excluding(&[]).await?;

        for file in source_files {

            let file_size     = file.file_size().await?;
            let lines_of_code = count_lines_in_file(&file).await?;

            total_file_size     += file_size;
            total_lines_of_code += lines_of_code;
            total_source_files  += 1;

            largest_file_size  = largest_file_size.max(file_size);
            smallest_file_size = smallest_file_size.min(file_size);
        }

        // Analyze test files if the `tests/` directory exists
        if crate_handle.has_tests_directory() {

            let test_files = crate_handle.test_files().await?;

            for file in test_files {

                let file_size     = file.file_size().await?;
                let lines_of_code = count_lines_in_file(&file).await?;

                total_file_size     += file_size;
                total_lines_of_code += lines_of_code;
                total_test_files    += 1;

                largest_file_size  = largest_file_size.max(file_size);
                smallest_file_size = smallest_file_size.min(file_size);
            }
        }

        Ok(CrateAnalysis {
            total_file_size,
            total_lines_of_code,
            total_source_files,
            total_test_files,
            largest_file_size,
            smallest_file_size,
        })
    }

    // --- Getters ---
    pub fn total_file_size(&self) -> u64 {
        self.total_file_size
    }

    pub fn total_lines_of_code(&self) -> usize {
        self.total_lines_of_code
    }

    pub fn total_source_files(&self) -> usize {
        self.total_source_files
    }

    pub fn total_test_files(&self) -> usize {
        self.total_test_files
    }

    pub fn largest_file_size(&self) -> u64 {
        self.largest_file_size
    }

    pub fn smallest_file_size(&self) -> u64 {
        self.smallest_file_size
    }
}

#[cfg(test)]
mod test_crate_analysis {
    use super::*;
    use tempfile::TempDir;
    use std::path::PathBuf;
    use tokio::fs::{File, create_dir_all};
    use tokio::io::AsyncWriteExt;
    use workspacer_3p::tokio;
    use crate::WorkspaceError;

    // We only implement the 3 traits below:
    //   - HasTestsDirectory
    //   - GetTestFiles
    //   - GetSourceFilesWithExclusions
    //
    // That’s precisely what CrateAnalysis::new requires.
    struct MockCrateHandle {
        root_dir:   TempDir,
        has_tests:  bool,
        src_files:  Vec<PathBuf>,
        test_files: Vec<PathBuf>,
    }

    impl MockCrateHandle {
        fn new() -> Self {
            Self {
                root_dir: tempfile::tempdir().unwrap(),
                has_tests: false,
                src_files: vec![],
                test_files: vec![],
            }
        }

        // Helper to create a file in `src/` and push to src_files
        async fn add_src_file(&mut self, name: &str, contents: &str) {
            let src_dir = self.root_dir.path().join("src");
            create_dir_all(&src_dir).await.unwrap();

            let path = src_dir.join(name);
            let mut file = File::create(&path).await.unwrap();
            file.write_all(contents.as_bytes()).await.unwrap();
            // Force flush
            file.sync_all().await.unwrap();
            drop(file); // ensure the handle is closed

            // Double-check size
            let meta = tokio::fs::metadata(&path).await.unwrap();
            assert_eq!(
                meta.len(),
                contents.len() as u64,
                "File size does not match expected!"
            );

            self.src_files.push(path);
        }


        // Helper to create a file in `tests/` and push to test_files
        async fn add_test_file(&mut self, name: &str, contents: &str) {
            let tests_dir = self.root_dir.path().join("tests");
            create_dir_all(&tests_dir).await.unwrap();

            let path = tests_dir.join(name);
            let mut file = File::create(&path).await.unwrap();
            file.write_all(contents.as_bytes()).await.unwrap();
            // Force flush
            file.sync_all().await.unwrap();
            drop(file); // ensure the handle is closed

            // Make sure the file truly has the expected size
            let meta = tokio::fs::metadata(&path).await.unwrap();
            assert_eq!(meta.len(), contents.len() as u64, "File size does not match expected!");

            self.test_files.push(path);
            self.has_tests = true;
        }
    }

    #[async_trait]
    impl GetSourceFilesWithExclusions for MockCrateHandle {
        async fn source_files_excluding(
            &self,
            _exclude_files: &[&str],
        ) -> Result<Vec<PathBuf>, CrateError> {
            // For mock purposes, we ignore the exclude list.
            Ok(self.src_files.clone())
        }
    }

    #[async_trait]
    impl GetTestFiles for MockCrateHandle {
        async fn test_files(&self) -> Result<Vec<PathBuf>, CrateError> {
            Ok(self.test_files.clone())
        }
    }

    impl HasTestsDirectory for MockCrateHandle {
        fn has_tests_directory(&self) -> bool {
            self.has_tests
        }
    }

    // ------------------------------------------------------------------------
    //  Existing tests
    // ------------------------------------------------------------------------

    #[tokio::test]
    async fn test_no_src_files_no_tests() {
        let handle = MockCrateHandle::new();
        let analysis = CrateAnalysis::new(&handle).await.unwrap();
        assert_eq!(analysis.total_source_files(), 0);
        assert_eq!(analysis.total_test_files(), 0);
        assert_eq!(analysis.total_lines_of_code(), 0);
        assert_eq!(analysis.total_file_size(), 0);
        // largest_file_size() and smallest_file_size() are also tested implicitly.
        // By default, smallest_file_size is u64::MAX if no files exist; but we only
        // store that if at least one file was found. So:
        assert_eq!(analysis.largest_file_size(), 0);
        assert_eq!(analysis.smallest_file_size(), u64::MAX);
    }

    #[tokio::test]
    async fn test_single_src_file() {
        let mut handle = MockCrateHandle::new();
        // Ensure the file truly has 2 lines of code:
        //   (Line 1) fn main() {
        //   (Line 2) println!("hi");}
        let contents = "fn main(){\nprintln!(\"hi\");}";
        handle.add_src_file("main.rs", contents).await;

        let analysis = CrateAnalysis::new(&handle).await.unwrap();
        assert_eq!(analysis.total_source_files(), 1);
        assert_eq!(analysis.total_test_files(), 0);
        assert_eq!(analysis.total_lines_of_code(), 2);
        // Check sizes
        let expected_size = contents.len() as u64;
        assert_eq!(analysis.total_file_size(), expected_size);
        assert_eq!(analysis.largest_file_size(), expected_size);
        assert_eq!(analysis.smallest_file_size(), expected_size);
    }

    #[tokio::test]
    async fn test_has_tests() {
        let mut handle = MockCrateHandle::new();
        let contents = "testline\nanother\n";
        handle.add_test_file("test_something.rs", contents).await;

        // Double check from within the test:
        let test_path = handle.test_files.last().unwrap();
        let meta = tokio::fs::metadata(&test_path).await.unwrap();
        assert_eq!(meta.len(), contents.len() as u64, "File not the size we expect!");

        let analysis = CrateAnalysis::new(&handle).await.unwrap();

        assert_eq!(analysis.total_source_files(), 0);
        assert_eq!(analysis.total_test_files(), 1);
        assert_eq!(analysis.total_lines_of_code(), 2);
        let expected_size = contents.len() as u64;
        assert_eq!(analysis.total_file_size(), expected_size);//this is the assertion that failed. 17 is the expected_size in terms of length units in the input string
        assert_eq!(analysis.largest_file_size(), expected_size);
        assert_eq!(analysis.smallest_file_size(), expected_size);
    }

    // ------------------------------------------------------------------------
    //  Additional tests for thorough coverage
    // ------------------------------------------------------------------------

    #[tokio::test]
    async fn test_multiple_source_files() {
        let mut handle = MockCrateHandle::new();
        // Add 3 files with different line counts and sizes
        let contents_a = "let x = 1;";
        let contents_b = "fn foo() {}\nfn bar() {}";
        let contents_c = "";
        handle.add_src_file("file_a.rs", contents_a).await;
        handle.add_src_file("file_b.rs", contents_b).await;
        handle.add_src_file("file_c.rs", contents_c).await;

        let analysis = CrateAnalysis::new(&handle).await.unwrap();
        assert_eq!(analysis.total_source_files(), 3);
        assert_eq!(analysis.total_test_files(), 0);

        // Lines
        let lines_a = 1; // "let x = 1;"
        let lines_b = 2; // 2 lines
        let lines_c = 0; // empty
        let total_lines = lines_a + lines_b + lines_c;
        assert_eq!(analysis.total_lines_of_code(), total_lines);

        // File sizes
        let size_a = contents_a.len() as u64;
        let size_b = contents_b.len() as u64;
        let size_c = contents_c.len() as u64;
        let total_size = size_a + size_b + size_c;
        assert_eq!(analysis.total_file_size(), total_size);

        // Largest & smallest
        let largest = size_a.max(size_b).max(size_c);
        let smallest = size_a.min(size_b).min(size_c);
        assert_eq!(analysis.largest_file_size(), largest);
        assert_eq!(analysis.smallest_file_size(), smallest);
    }

    #[tokio::test]
    async fn test_multiple_test_files() {
        let mut handle = MockCrateHandle::new();
        // Add 2 test files
        let test_content_1 = "mod test1 {}\nmod test2 {}";
        let test_content_2 = "mod test3 {}";
        handle.add_test_file("test1.rs", test_content_1).await;
        handle.add_test_file("test2.rs", test_content_2).await;

        let analysis = CrateAnalysis::new(&handle).await.unwrap();
        assert_eq!(analysis.total_source_files(), 0);
        assert_eq!(analysis.total_test_files(), 2);

        let lines_1 = 2; // test_content_1 has 2 lines
        let lines_2 = 1; // test_content_2 has 1 line
        assert_eq!(analysis.total_lines_of_code(), lines_1 + lines_2);

        let size_1 = test_content_1.len() as u64;
        let size_2 = test_content_2.len() as u64;
        let total_size = size_1 + size_2;
        assert_eq!(analysis.total_file_size(), total_size);

        // Largest & smallest
        let largest = size_1.max(size_2);
        let smallest = size_1.min(size_2);
        assert_eq!(analysis.largest_file_size(), largest);
        assert_eq!(analysis.smallest_file_size(), smallest);
    }

    #[tokio::test]
    async fn test_mixed_source_and_test_files() {
        let mut handle = MockCrateHandle::new();
        // 2 source files
        let src1 = "src1 line1\nsrc1 line2";
        let src2 = "src2 line1";
        handle.add_src_file("file1.rs", src1).await;
        handle.add_src_file("file2.rs", src2).await;

        // 1 test file
        let test1 = "test line1\ntest line2\ntest line3";
        handle.add_test_file("test_stuff.rs", test1).await;

        let analysis = CrateAnalysis::new(&handle).await.unwrap();

        // Check counts
        assert_eq!(analysis.total_source_files(), 2);
        assert_eq!(analysis.total_test_files(), 1);

        // Lines
        let lines_src1 = 2;
        let lines_src2 = 1;
        let lines_test1 = 3;
        assert_eq!(
            analysis.total_lines_of_code(),
            lines_src1 + lines_src2 + lines_test1
        );

        // File sizes
        let size_src1 = src1.len() as u64;
        let size_src2 = src2.len() as u64;
        let size_test1 = test1.len() as u64;
        assert_eq!(
            analysis.total_file_size(),
            size_src1 + size_src2 + size_test1
        );

        // Largest & smallest
        let largest = size_src1.max(size_src2).max(size_test1);
        let smallest = size_src1.min(size_src2).min(size_test1);
        assert_eq!(analysis.largest_file_size(), largest);
        assert_eq!(analysis.smallest_file_size(), smallest);
    }

    #[tokio::test]
    async fn test_tests_dir_exists_but_empty() {
        // Even if we create an empty `tests/` directory, if there are no files in it,
        // total_test_files remains 0.
        let mut handle = MockCrateHandle::new();
        // Force creation of an empty `tests` directory
        let tests_dir = handle.root_dir.path().join("tests");
        create_dir_all(&tests_dir).await.unwrap();
        // Mark has_tests = true so it sees the directory
        handle.has_tests = true;

        // Add one src file for good measure
        let contents = "fn main() {}";
        handle.add_src_file("main.rs", contents).await;

        let analysis = CrateAnalysis::new(&handle).await.unwrap();
        assert_eq!(analysis.total_source_files(), 1);
        assert_eq!(analysis.total_test_files(), 0);
    }

    #[tokio::test]
    async fn test_zero_sized_file() {
        let mut handle = MockCrateHandle::new();
        // Create an empty src file
        handle.add_src_file("empty.rs", "").await;

        let analysis = CrateAnalysis::new(&handle).await.unwrap();
        assert_eq!(analysis.total_source_files(), 1);
        assert_eq!(analysis.total_test_files(), 0);
        assert_eq!(analysis.total_lines_of_code(), 0);
        // File size is 0
        assert_eq!(analysis.total_file_size(), 0);
        assert_eq!(analysis.largest_file_size(), 0);
        assert_eq!(analysis.smallest_file_size(), 0);
    }

    // This test won't actually show a difference in this mock because
    // we're ignoring excludes. Provided here for completeness of the
    // interface usage.
    #[tokio::test]
    async fn test_excluded_files_are_ignored() {
        let mut handle = MockCrateHandle::new();
        let included = "line1\nline2";
        let excluded = "some content";
        handle.add_src_file("included.rs", included).await;
        handle.add_src_file("excluded.rs", excluded).await;

        // In the real environment, the next call would skip "excluded.rs"
        // if it is passed in `_exclude_files`.
        // Our mock simply returns all files, so the result is the same.
        let files = handle
            .source_files_excluding(&["excluded.rs"])
            .await
            .expect("should get source files");
        assert_eq!(files.len(), 2, "Mock returns all files, ignoring excludes");

        let analysis = CrateAnalysis::new(&handle).await.unwrap();
        assert_eq!(analysis.total_source_files(), 2); 
        assert_eq!(analysis.total_lines_of_code(), 3); // included=2 lines, excluded=1 line
    }
}
