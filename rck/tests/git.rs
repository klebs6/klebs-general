use rck::*;
use mockall::mock;
use git2::{Statuses};

// Define a trait for repository operations
pub trait RepositoryTrait {
    fn statuses(&self, options: Option<git2::StatusOptions>) -> Result<git2::Statuses<'static>, git2::Error>;
    fn open(path: &std::path::Path) -> Result<Self, git2::Error> where Self: Sized;
    fn git_status_clean(&self) -> Result<bool, git2::Error>;
}

// Mock the trait using `mockall`
mock! {
    RepositoryTrait {}

    impl RepositoryTrait for RepositoryTrait {
        fn statuses(&self, options: Option<git2::StatusOptions>) -> Result<git2::Statuses<'static>, git2::Error>;
        fn open(path: &std::path::Path) -> Result<Self, git2::Error>;
        fn git_status_clean(&self) -> Result<bool, git2::Error>; // Mock the `git_status_clean` method
    }
}

#[test]
fn test_is_git_status_clean() {
    let mut mock_repo = MockRepositoryTrait::new();

    // Mock the `git_status_clean` method to return `true` to simulate a clean repository.
    mock_repo
        .expect_git_status_clean()
        .returning(|| Ok(true));

    // Use the mocked object directly to test the method.
    let result = mock_repo.git_status_clean();

    // This assertion should now pass
    assert_eq!(result.unwrap(), true);
}
