// ---------------- [ File: src/crate_analysis.rs ]
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

// ---------------- [ File: src/crate_analysis.rs ]
#[cfg(test)]
mod test_crate_analysis {
    use super::*;
    use tempfile::TempDir;
    use std::path::PathBuf;
    use tokio::fs::{File, create_dir_all};
    use tokio::io::AsyncWriteExt;
    use workspacer_3p::tokio; // or however you import tokio
    use crate::WorkspaceError;

    // We only implement these 3 traits:
    //  - HasTestsDirectory
    //  - GetTestFiles
    //  - GetSourceFilesWithExclusions
    //
    // That’s exactly the minimal interface CrateAnalysis::new requires.
    struct MockCrateHandle {
        root_dir:      TempDir,
        has_tests:     bool,
        src_files:     Vec<PathBuf>,
        test_files:    Vec<PathBuf>,
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

        // a helper to create a file in `src/` and push to src_files
        async fn add_src_file(&mut self, name: &str, contents: &str) {
            let src_dir = self.root_dir.path().join("src");
            create_dir_all(&src_dir).await.unwrap();
            
            let path = src_dir.join(name);
            let mut file = File::create(&path).await.unwrap();
            file.write_all(contents.as_bytes()).await.unwrap();
            self.src_files.push(path);
        }

        // a helper to create a file in `tests/` and push to test_files
        async fn add_test_file(&mut self, name: &str, contents: &str) {
            let tests_dir = self.root_dir.path().join("tests");
            create_dir_all(&tests_dir).await.unwrap();
            
            let path = tests_dir.join(name);
            let mut file = File::create(&path).await.unwrap();
            file.write_all(contents.as_bytes()).await.unwrap();
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
            // ignoring excludes for test
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

    // Now we can test CrateAnalysis::new with just these minimal traits.

    #[tokio::test]
    async fn test_no_src_files_no_tests() {
        let handle = MockCrateHandle::new();
        let analysis = CrateAnalysis::new(&handle).await.unwrap();
        assert_eq!(analysis.total_source_files(), 0);
        assert_eq!(analysis.total_test_files(), 0);
    }

    #[tokio::test]
    async fn test_single_src_file() {
        let mut handle = MockCrateHandle::new();
        let contents = "fn main(){\nprintln!(\"hi\");\n}";
        handle.add_src_file("main.rs", contents).await;

        let analysis = CrateAnalysis::new(&handle).await.unwrap();
        assert_eq!(analysis.total_source_files(), 1);
        assert_eq!(analysis.total_lines_of_code(), 2);
        // etc...
    }

    #[tokio::test]
    async fn test_has_tests() {
        let mut handle = MockCrateHandle::new();
        handle.add_test_file("test_something.rs", "testline\nanother\n").await;
        let analysis = CrateAnalysis::new(&handle).await.unwrap();
        assert_eq!(analysis.total_test_files(), 1);
        // etc...
    }
}
