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
    pub async fn new(crate_handle: &CrateHandle) -> Result<Self, WorkspaceError> {

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
